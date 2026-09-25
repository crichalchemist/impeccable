//! The tmux side of the engine: find the executable, check its version, read
//! pane geometry, capture, resize, restore (spec section 5, "Requirements and
//! degradation" and "Multi-size pass").

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub const NOT_FOUND_MESSAGE: &str = "tmux 3.2 or newer is required for --tmux and was not found on PATH. Install tmux, or point IMPECCABLE_TMUX at the executable.";
/// `resize-window -x -y` and `client_termfeatures` both arrived in 3.2.
pub const MIN_VERSION: (u32, u32) = (3, 2);

/// Every tmux call's deadline (spec section 7, PR 4 item 3).
pub const TIMEOUT: Duration = Duration::from_secs(5);
/// How often a running tmux call is polled for exit.
const POLL: Duration = Duration::from_millis(5);

/// What an expired call reports: `tmux <verb>: timed out after 5 seconds`.
pub fn timed_out(verb: &str, timeout: Duration) -> String {
    format!("tmux {verb}: timed out after {} seconds", timeout.as_secs())
}

/// Drain a child's pipe on its own thread so a full pipe never stalls it.
fn read_all<R: Read + Send + 'static>(pipe: Option<R>) -> JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut p) = pipe {
            let _ = p.read_to_end(&mut buf);
        }
        buf
    })
}

/// `IMPECCABLE_TMUX` (a path, reported when it does not exist), else the first
/// `tmux` on `PATH`.
pub fn find_tmux(env: &HashMap<String, String>) -> Result<PathBuf, String> {
    if let Some(raw) = env.get("IMPECCABLE_TMUX").map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let path = PathBuf::from(raw);
        return if path.is_file() {
            Ok(path)
        } else {
            Err(format!("tmux was not found at the configured path ({raw}) from IMPECCABLE_TMUX"))
        };
    }
    let path_var = env.get("PATH").cloned().unwrap_or_default();
    let separator = if cfg!(windows) { ';' } else { ':' };
    let name = if cfg!(windows) { "tmux.exe" } else { "tmux" };
    for dir in path_var.split(separator).filter(|d| !d.is_empty()) {
        let candidate = Path::new(dir).join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(NOT_FOUND_MESSAGE.to_string())
}

/// `tmux -V` into `(major, minor)`: "tmux 3.7c" is (3, 7), "tmux next-3.8" is (3, 8).
pub fn parse_version(output: &str) -> Option<(u32, u32)> {
    let digits: String = output
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let mut parts = digits.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().ok()?;
    Some((major, minor))
}

/// What `display-message` reports about the target before the first capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneInfo {
    pub width: usize,
    pub height: usize,
    pub window_width: usize,
    pub window_height: usize,
    /// `#{client_termfeatures}`; empty when no client is attached.
    pub termfeatures: String,
}

/// A located, version-checked tmux executable and the environment it runs
/// with (so `$TMUX` reaches it and a `PATH` override is honored).
#[derive(Debug)]
pub struct Tmux {
    exe: PathBuf,
    env: HashMap<String, String>,
    /// Each call's deadline: [`TIMEOUT`], lowered only by tests.
    pub(crate) timeout: Duration,
}

impl Tmux {
    pub fn locate(env: &HashMap<String, String>) -> Result<Tmux, String> {
        let exe = find_tmux(env)?;
        let tmux = Tmux { exe, env: env.clone(), timeout: TIMEOUT };
        let version = tmux.run(&["-V"])?;
        match parse_version(&version) {
            Some(v) if v >= MIN_VERSION => Ok(tmux),
            _ => Err(format!("tmux 3.2 or newer is required for --tmux; found {}", version.trim())),
        }
    }

    /// Run tmux with `args` under the deadline: stdout on success,
    /// `tmux <verb>: <first stderr line>` on failure, and
    /// `tmux <verb>: timed out after 5 seconds` once the deadline passes (the
    /// child is killed). On unix the child gets its own process group, so a
    /// Ctrl-C at the terminal reaches the engine and not the call in flight.
    pub fn run(&self, args: &[&str]) -> Result<String, String> {
        let verb = args.first().copied().unwrap_or("");
        let mut cmd = Command::new(&self.exe);
        cmd.args(args)
            .env_clear()
            .envs(&self.env)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        let mut child = cmd.spawn().map_err(|e| format!("could not run {}: {e}", self.exe.display()))?;
        let stdout = read_all(child.stdout.take());
        let stderr = read_all(child.stderr.take());
        let start = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if start.elapsed() >= self.timeout => {
                    let _ = child.kill();
                    let _ = child.wait();
                    // The readers are not joined: a grandchild may still hold a pipe.
                    return Err(timed_out(verb, self.timeout));
                }
                Ok(None) => thread::sleep(POLL),
                Err(e) => return Err(format!("could not run {}: {e}", self.exe.display())),
            }
        };
        let out = stdout.join().unwrap_or_default();
        let err = stderr.join().unwrap_or_default();
        if status.success() {
            return Ok(String::from_utf8_lossy(&out).into_owned());
        }
        let err = String::from_utf8_lossy(&err);
        Err(format!("tmux {verb}: {}", err.lines().next().unwrap_or("").trim()))
    }

    /// tmux 3.7's `display-message -p -t <target>` does not fail on a
    /// missing session or window, or on a target that resolves to the wrong
    /// pane. It exits 0 with empty `#{...}` fields. A `list-panes` probe
    /// therefore runs first to get a real, target-specific error.
    pub fn pane_info(&self, target: &str) -> Result<PaneInfo, String> {
        self.run(&["list-panes", "-t", target, "-F", "#{pane_id}"])?;
        let out = self.run(&[
            "display-message",
            "-p",
            "-t",
            target,
            "#{pane_width}\t#{pane_height}\t#{window_width}\t#{window_height}\t#{client_termfeatures}",
        ])?;
        let line = out.trim_end_matches('\n');
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 5 {
            return Err(format!("tmux display-message: unexpected output {line:?}"));
        }
        let number = |s: &str| s.trim().parse::<usize>().map_err(|_| format!("tmux display-message: {s:?} is not a number"));
        Ok(PaneInfo {
            width: number(parts[0])?,
            height: number(parts[1])?,
            window_width: number(parts[2])?,
            window_height: number(parts[3])?,
            termfeatures: parts[4].trim().to_string(),
        })
    }

    /// The visible grid with SGR sequences kept and wrapped lines joined.
    pub fn capture(&self, target: &str) -> Result<String, String> {
        self.run(&["capture-pane", "-p", "-e", "-J", "-t", target])
    }

    pub fn resize(&self, target: &str, width: usize, height: usize) -> Result<(), String> {
        self.run(&["resize-window", "-t", target, "-x", &width.to_string(), "-y", &height.to_string()]).map(|_| ())
    }

    /// The window's own `window-size` option, trimmed: empty when the window
    /// has none and follows the global value. Read before the first resize,
    /// because `resize-window` replaces any value with `manual`.
    pub fn window_size(&self, target: &str) -> Result<String, String> {
        Ok(self.run(&["show-options", "-w", "-v", "-t", target, "window-size"])?.trim().to_string())
    }

    /// Back to the recorded size, then put back the `window-size` read before
    /// the first resize, or unset the `manual` value `resize-window` left
    /// when there was none, so the window follows its clients again.
    pub fn restore(&self, target: &str, width: usize, height: usize, window_size: &str) -> Result<(), String> {
        self.resize(target, width, height)?;
        if window_size.is_empty() {
            self.run(&["set-option", "-w", "-t", target, "-u", "window-size"]).map(|_| ())
        } else {
            self.run(&["set-option", "-w", "-t", target, "window-size", window_size]).map(|_| ())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version_reads_release_and_next_builds() {
        assert_eq!(parse_version("tmux 3.7c\n"), Some((3, 7)));
        assert_eq!(parse_version("tmux 3.2a"), Some((3, 2)));
        assert_eq!(parse_version("tmux next-3.8"), Some((3, 8)));
        assert_eq!(parse_version("tmux master"), None);
        assert!((3, 1) < MIN_VERSION && (3, 2) >= MIN_VERSION);
    }

    #[cfg(unix)]
    #[test]
    fn missing_tmux_names_the_requirement_and_the_override_wins() {
        let mut env = HashMap::new();
        env.insert("PATH".to_string(), "/nonexistent".to_string());
        assert_eq!(find_tmux(&env).unwrap_err(), NOT_FOUND_MESSAGE);
        env.insert("IMPECCABLE_TMUX".to_string(), "/nonexistent/tmux".to_string());
        assert_eq!(
            find_tmux(&env).unwrap_err(),
            "tmux was not found at the configured path (/nonexistent/tmux) from IMPECCABLE_TMUX"
        );
        let dir = std::env::temp_dir().join(format!("impeccable-tmux-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let fake = dir.join("tmux");
        std::fs::write(&fake, "#!/bin/sh\necho tmux 3.7c\n").unwrap();
        env.remove("IMPECCABLE_TMUX");
        env.insert("PATH".to_string(), dir.to_string_lossy().into_owned());
        assert_eq!(find_tmux(&env).unwrap(), fake);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn an_old_tmux_is_refused_with_its_version_in_the_message() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("impeccable-tmux-old-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let fake = dir.join("tmux");
        std::fs::write(&fake, "#!/bin/sh\necho tmux 3.1b\n").unwrap();
        std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut env = HashMap::new();
        env.insert("IMPECCABLE_TMUX".to_string(), fake.to_string_lossy().into_owned());
        env.insert("PATH".to_string(), "/usr/bin:/bin".to_string());
        assert_eq!(Tmux::locate(&env).unwrap_err(), "tmux 3.2 or newer is required for --tmux; found tmux 3.1b");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_expired_call_names_the_verb_and_the_five_second_deadline() {
        assert_eq!(TIMEOUT, Duration::from_secs(5));
        assert_eq!(timed_out("capture-pane", TIMEOUT), "tmux capture-pane: timed out after 5 seconds");
    }

    #[cfg(unix)]
    #[test]
    fn a_hung_tmux_call_is_killed_at_the_deadline() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("impeccable-tmux-hang-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let fake = dir.join("tmux");
        std::fs::write(&fake, "#!/bin/sh\ncase \"$1\" in\n  -V) echo tmux 3.7c ;;\n  *) exec sleep 30 ;;\nesac\n").unwrap();
        std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut env = HashMap::new();
        env.insert("IMPECCABLE_TMUX".to_string(), fake.to_string_lossy().into_owned());
        env.insert("PATH".to_string(), "/usr/bin:/bin".to_string());
        let mut tmux = Tmux::locate(&env).unwrap();
        assert_eq!(tmux.timeout, TIMEOUT, "locate sets the spec's deadline");
        tmux.timeout = Duration::from_secs(1);
        let start = Instant::now();
        assert_eq!(tmux.run(&["capture-pane", "-p"]).unwrap_err(), timed_out("capture-pane", tmux.timeout));
        assert!(start.elapsed() < Duration::from_secs(3), "killed at the deadline, not after sleep 30: {:?}", start.elapsed());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
