//! The tmux side of the engine: find the executable, check its version, read
//! pane geometry, capture, resize, restore (spec section 5, "Requirements and
//! degradation" and "Multi-size pass").

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const NOT_FOUND_MESSAGE: &str = "tmux 3.2 or newer is required for --tmux and was not found on PATH. Install tmux, or point IMPECCABLE_TMUX at the executable.";
/// `resize-window -x -y` and `client_termfeatures` both arrived in 3.2.
pub const MIN_VERSION: (u32, u32) = (3, 2);

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
}

impl Tmux {
    pub fn locate(env: &HashMap<String, String>) -> Result<Tmux, String> {
        let exe = find_tmux(env)?;
        let tmux = Tmux { exe, env: env.clone() };
        let version = tmux.run(&["-V"])?;
        match parse_version(&version) {
            Some(v) if v >= MIN_VERSION => Ok(tmux),
            _ => Err(format!("tmux 3.2 or newer is required for --tmux; found {}", version.trim())),
        }
    }

    /// Run tmux with `args`: stdout on success, `tmux <verb>: <first stderr line>` otherwise.
    pub fn run(&self, args: &[&str]) -> Result<String, String> {
        let output = Command::new(&self.exe)
            .args(args)
            .env_clear()
            .envs(&self.env)
            .stdin(Stdio::null())
            .output()
            .map_err(|e| format!("could not run {}: {e}", self.exe.display()))?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!(
            "tmux {}: {}",
            args.first().copied().unwrap_or(""),
            stderr.lines().next().unwrap_or("").trim()
        ))
    }

    /// tmux 3.7's `display-message -p -t <target>` does not fail on a
    /// missing session or window (or a target that resolves to the wrong
    /// pane) -- it exits 0 with empty `#{...}` fields -- so a `list-panes`
    /// probe runs first to get a real, target-specific error.
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

    /// Back to the recorded size, then drop the `manual` window-size that
    /// `resize-window` set so the window follows its clients again.
    pub fn restore(&self, target: &str, width: usize, height: usize) -> Result<(), String> {
        self.resize(target, width, height)?;
        self.run(&["set-option", "-w", "-t", target, "-u", "window-size"]).map(|_| ())
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
}
