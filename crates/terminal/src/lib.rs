//! impeccable-terminal: the tmux engine of `impeccable detect` (spec section
//! 5). Captures a running tmux pane, or replays a saved capture, parses it
//! into frames, and runs the `tui-rt-` runtime rules. Wired into the binary
//! through [`impeccable_detect::engines::TmuxEngine`]; the engine never
//! launches or kills a process and always restores a window size it changed.

pub mod capture;
pub mod palette;
pub mod rules;
pub mod tmux;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};

use impeccable_core::findings::Finding;
use impeccable_detect::engines::{EngineError, ScanOptions, TmuxEngine};

use capture::{parse_frames, parse_rows, Frame, FrameRole};
use palette::Palette;
use tmux::{PaneInfo, Tmux};

/// When the recaptures are taken, in milliseconds after frame 0 (spec
/// section 7, PR 4 item 2). A 100 ms animation aliases at 1000 ms and not
/// at 700 ms.
pub const RECAPTURE_DELAYS_MS: [u64; 2] = [700, 1000];
/// `--tmux-settle` when the flag is absent.
pub const DEFAULT_SETTLE_MS: u64 = 300;
/// Settle waits sleep in steps this long and check for an interrupt between them.
pub const INTERRUPT_POLL_MS: u64 = 50;

/// Set by SIGINT or SIGTERM while a size pass runs; registered with
/// `impeccable_common::proc::on_interrupt` only around that pass.
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// Wait `total` in 50 ms steps; false as soon as `interrupted` is set.
fn settle(total: Duration, interrupted: &AtomicBool) -> bool {
    let start = Instant::now();
    loop {
        if interrupted.load(Ordering::SeqCst) {
            return false;
        }
        let elapsed = start.elapsed();
        if elapsed >= total {
            return true;
        }
        sleep((total - elapsed).min(Duration::from_millis(INTERRUPT_POLL_MS)));
    }
}

/// The tmux engine, holding the process environment it reads
/// (`IMPECCABLE_TMUX`, `PATH`, `TMUX`, `COLORTERM`).
pub struct TerminalEngine {
    env: HashMap<String, String>,
}

impl TerminalEngine {
    pub fn new(env: HashMap<String, String>) -> Self {
        TerminalEngine { env }
    }

    pub fn from_process_env() -> Self {
        TerminalEngine::new(std::env::vars().collect())
    }

    fn colorterm(&self) -> String {
        self.env.get("COLORTERM").cloned().unwrap_or_default()
    }
}

fn palette_for(options: &ScanOptions) -> Palette {
    options.palette.as_deref().and_then(Palette::parse).unwrap_or_default()
}

/// A frame from a live capture, sized by the pane rather than by a header.
fn frame_from(text: &str, info: &PaneInfo, colorterm: &str, role: FrameRole) -> Frame {
    Frame {
        width: info.width,
        height: info.height,
        termfeatures: info.termfeatures.clone(),
        colorterm: colorterm.to_string(),
        role,
        rows: parse_rows(text),
    }
}

/// The `--tmux-sizes` pass: resize, settle, re-read the geometry, capture.
/// Stops at the next check once `interrupted` is set. The caller restores
/// the window whatever this returns.
fn capture_sizes(
    tmux: &Tmux,
    target: &str,
    options: &ScanOptions,
    colorterm: &str,
    frames: &mut Vec<Frame>,
    interrupted: &AtomicBool,
) -> Result<(), String> {
    let settle_for = Duration::from_millis(options.tmux_settle_ms.unwrap_or(DEFAULT_SETTLE_MS));
    for &(w, h) in &options.tmux_sizes {
        if interrupted.load(Ordering::SeqCst) {
            return Err("interrupted".to_string());
        }
        tmux.resize(target, w as usize, h as usize)?;
        if !settle(settle_for, interrupted) {
            return Err("interrupted".to_string());
        }
        let info = tmux.pane_info(target)?;
        let text = tmux.capture(target)?;
        frames.push(frame_from(&text, &info, colorterm, FrameRole::Capture));
    }
    Ok(())
}

/// The size pass and the restore that always follows it. When `interrupted`
/// is set by the end, the interrupt names the failure whatever the pass
/// returned (on Windows, where the tmux child shares the terminal's process
/// group, a Ctrl-C can also fail the tmux call in flight). The restore never
/// checks the flag.
#[allow(clippy::too_many_arguments)]
fn size_pass(
    tmux: &Tmux,
    target: &str,
    options: &ScanOptions,
    info: &PaneInfo,
    window_size: &str,
    colorterm: &str,
    frames: &mut Vec<Frame>,
    interrupted: &AtomicBool,
) -> Result<(), String> {
    let captured = capture_sizes(tmux, target, options, colorterm, frames, interrupted);
    let restored = tmux.restore(target, info.window_width, info.window_height, window_size);
    if interrupted.load(Ordering::SeqCst) {
        return Err(match restored {
            Ok(()) => "interrupted; window restored".to_string(),
            Err(r) => format!("interrupted; window not restored: {r}"),
        });
    }
    match (captured, restored) {
        (Err(c), Err(r)) => Err(format!("{c}; window not restored: {r}")),
        (Ok(()), Err(r)) => Err(format!("window not restored: {r}")),
        (Err(c), Ok(())) => Err(c),
        (Ok(()), Ok(())) => Ok(()),
    }
}

/// A signal that lands after `size_pass` last read the flag and before the
/// handlers are cleared still reports as an interrupt. An Ok pass means the
/// restore succeeded; an error already says what happened to the window.
fn late_interrupt(result: Result<(), String>, interrupted: &AtomicBool) -> Result<(), String> {
    match result {
        Ok(()) if interrupted.load(Ordering::SeqCst) => Err("interrupted; window restored".to_string()),
        other => other,
    }
}

impl TmuxEngine for TerminalEngine {
    fn detect_pane(&self, target: &str, options: &ScanOptions) -> Result<Vec<Finding>, EngineError> {
        let tmux = Tmux::locate(&self.env).map_err(EngineError::new)?;
        let info = tmux.pane_info(target).map_err(EngineError::new)?;
        let colorterm = self.colorterm();
        let first = tmux.capture(target).map_err(EngineError::new)?;
        let mut frames = vec![frame_from(&first, &info, &colorterm, FrameRole::Capture)];
        // Both delays count from the end of frame 0's capture.
        let taken = Instant::now();
        for delay in RECAPTURE_DELAYS_MS {
            sleep(Duration::from_millis(delay).saturating_sub(taken.elapsed()));
            let again = tmux.capture(target).map_err(EngineError::new)?;
            frames.push(frame_from(&again, &info, &colorterm, FrameRole::Recapture));
        }
        if !options.tmux_sizes.is_empty() {
            let window_size = tmux.window_size(target).map_err(EngineError::new)?;
            // From here to the end of the restore, SIGINT or SIGTERM sets a
            // flag instead of killing the process (spec section 7, item 4).
            INTERRUPTED.store(false, Ordering::SeqCst);
            impeccable_common::proc::on_interrupt(&INTERRUPTED);
            let result = size_pass(&tmux, target, options, &info, &window_size, &colorterm, &mut frames, &INTERRUPTED);
            impeccable_common::proc::clear_interrupt();
            late_interrupt(result, &INTERRUPTED).map_err(EngineError::new)?;
        }
        Ok(rules::scan_frames(&frames, palette_for(options), &format!("tmux:{target}")))
    }

    fn detect_capture(&self, path: &str, options: &ScanOptions) -> Result<Vec<Finding>, EngineError> {
        let text = std::fs::read_to_string(path).map_err(|e| {
            EngineError::new(match e.kind() {
                std::io::ErrorKind::NotFound => format!("ENOENT: no such file or directory, open '{path}'"),
                std::io::ErrorKind::PermissionDenied => format!("EACCES: permission denied, open '{path}'"),
                _ => format!("{e}"),
            })
        })?;
        let frames = parse_frames(&text).map_err(EngineError::new)?;
        Ok(rules::scan_frames(&frames, palette_for(options), path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_frames_carry_the_pane_geometry_and_client_features() {
        let info = PaneInfo { width: 60, height: 8, window_width: 60, window_height: 8, termfeatures: "256,RGB".into() };
        let f = frame_from("\x1b[31mhi\x1b[0m\n\n", &info, "truecolor", FrameRole::Recapture);
        assert_eq!((f.width, f.height, f.rows.len()), (60, 8, 2));
        assert_eq!(f.termfeatures, "256,RGB");
        assert_eq!(f.colorterm, "truecolor");
        assert_eq!(f.role, FrameRole::Recapture);
    }

    #[test]
    fn palette_flag_falls_back_to_dark() {
        assert_eq!(palette_for(&ScanOptions::default()), Palette::Dark);
        assert_eq!(palette_for(&ScanOptions { palette: Some("light".into()), ..ScanOptions::default() }), Palette::Light);
    }

    #[test]
    fn saved_capture_replays_through_the_engine() {
        let dir = std::env::temp_dir().join(format!("impeccable-terminal-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("bare.txt");
        std::fs::write(&file, "#!capture width=40 height=4\n┌──┐\n│ab│\n└──┘\nLoading complete.\n").unwrap();
        let engine = TerminalEngine::new(HashMap::new());
        let out = engine.detect_capture(&file.to_string_lossy(), &ScanOptions::default()).unwrap();
        assert_eq!(out.iter().map(|f| f.antipattern.as_str()).collect::<Vec<_>>(), vec!["tui-rt-no-key-hints"]);
        assert_eq!(out[0].file, file.to_string_lossy());
        let missing = engine.detect_capture("/nonexistent/x.txt", &ScanOptions::default()).unwrap_err();
        assert_eq!(missing.message, "ENOENT: no such file or directory, open '/nonexistent/x.txt'");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    fn fake_tmux(dir_name: &str, script: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("{dir_name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let fake = dir.join("tmux");
        std::fs::write(&fake, script).unwrap();
        std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        (dir, fake)
    }

    #[cfg(unix)]
    #[test]
    fn missing_target_is_reported_with_tmux_own_line() {
        let script = r#"#!/bin/sh
case "$1" in
  -V) echo "tmux 3.7c" ;;
  list-panes) echo "can't find session: nope" >&2; exit 1 ;;
  *) exit 1 ;;
esac
"#;
        let (dir, fake) = fake_tmux("impeccable-terminal-missing-target", script);
        let mut env = HashMap::new();
        env.insert("IMPECCABLE_TMUX".to_string(), fake.to_string_lossy().into_owned());
        let engine = TerminalEngine::new(env);
        let err = engine.detect_pane("nope:0.0", &ScanOptions::default()).unwrap_err();
        assert_eq!(err.message, "tmux list-panes: can't find session: nope");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A fake tmux that logs every subcommand to `calls.log` and fails
    /// `set-option` when `set_option_fails`. When `capture_fails_after_resize`
    /// is set, `capture-pane` fails once `resize-window` has run at least
    /// once (marked by a file it writes); the resize/restore pass in
    /// `detect_pane` always calls `resize-window` before its own
    /// `capture-pane`, so the size-pass capture is the first one to see the
    /// marker. The two initial captures at the original size run before any
    /// resize and always succeed either way.
    #[cfg(unix)]
    fn fake_tmux_for_restore_scenarios(capture_fails_after_resize: bool, set_option_fails: bool) -> String {
        let set_option = if set_option_fails { "echo boom >&2; exit 1" } else { "exit 0" };
        let capture_pane = if capture_fails_after_resize {
            r#"if [ -f "$DIR/resized" ]; then
      echo nope >&2
      exit 1
    fi
    echo hi"#
        } else {
            "echo hi"
        };
        format!(
            r#"#!/bin/sh
DIR="$(dirname "$0")"
echo "$1" >> "$DIR/calls.log"
case "$1" in
  -V) echo "tmux 3.7c" ;;
  list-panes) exit 0 ;;
  show-options) exit 0 ;;
  display-message) printf '40\t5\t40\t5\t\n' ;;
  capture-pane)
    {capture_pane}
    ;;
  resize-window) touch "$DIR/resized" ;;
  set-option) {set_option} ;;
  *) exit 1 ;;
esac
"#
        )
    }

    /// A fake tmux that logs every call's full argument list to `calls.log`
    /// and prints `window_size` for `show-options`, as tmux prints a window
    /// option (nothing at all when the window has no value of its own).
    #[cfg(unix)]
    fn fake_tmux_logging_args(window_size: &str) -> String {
        format!(
            r#"#!/bin/sh
DIR="$(dirname "$0")"
echo "$*" >> "$DIR/calls.log"
case "$1" in
  -V) echo "tmux 3.7c" ;;
  list-panes) exit 0 ;;
  display-message) printf '40\t5\t40\t5\t\n' ;;
  capture-pane) echo hi ;;
  show-options) printf '{window_size}' ;;
  resize-window|set-option) exit 0 ;;
  *) exit 1 ;;
esac
"#
        )
    }

    /// The calls that read or change the window, in order.
    #[cfg(unix)]
    fn window_calls(dir: &std::path::Path) -> Vec<String> {
        std::fs::read_to_string(dir.join("calls.log"))
            .unwrap()
            .lines()
            .filter(|l| ["show-options", "resize-window", "set-option"].iter().any(|v| l.starts_with(v)))
            .map(String::from)
            .collect()
    }

    /// A fake tmux located the way the engine locates it, for driving
    /// `size_pass` directly with a flag the test controls.
    #[cfg(unix)]
    fn located(dir_name: &str, script: &str) -> (std::path::PathBuf, Tmux) {
        let (dir, fake) = fake_tmux(dir_name, script);
        let mut env = HashMap::new();
        env.insert("IMPECCABLE_TMUX".to_string(), fake.to_string_lossy().into_owned());
        (dir, Tmux::locate(&env).unwrap())
    }

    fn pane_40x5() -> PaneInfo {
        PaneInfo { width: 40, height: 5, window_width: 40, window_height: 5, termfeatures: String::new() }
    }

    #[cfg(unix)]
    #[test]
    fn an_interrupt_mid_settle_stops_the_pass_and_restores_the_window() {
        let (dir, tmux) = located("impeccable-terminal-interrupt", &fake_tmux_logging_args(""));
        let flag = AtomicBool::new(false);
        let options = ScanOptions { tmux_sizes: vec![(30, 5), (20, 5)], tmux_settle_ms: Some(5_000), ..ScanOptions::default() };
        let mut frames = Vec::new();
        let start = Instant::now();
        let result = std::thread::scope(|s| {
            s.spawn(|| {
                sleep(Duration::from_millis(200));
                flag.store(true, Ordering::SeqCst);
            });
            size_pass(&tmux, "smoke:0.0", &options, &pane_40x5(), "", "", &mut frames, &flag)
        });
        assert_eq!(result.unwrap_err(), "interrupted; window restored");
        assert!(start.elapsed() < Duration::from_secs(2), "the 5 s settle stopped within a poll step: {:?}", start.elapsed());
        assert!(frames.is_empty(), "nothing was captured at the interrupted size");
        assert_eq!(
            window_calls(&dir),
            vec![
                "resize-window -t smoke:0.0 -x 30 -y 5",
                "resize-window -t smoke:0.0 -x 40 -y 5",
                "set-option -w -t smoke:0.0 -u window-size",
            ],
            "the second size never ran"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn an_interrupt_with_a_failed_restore_says_the_window_was_not_restored() {
        let (dir, tmux) = located("impeccable-terminal-interrupt-restore-fails", &fake_tmux_for_restore_scenarios(false, true));
        let flag = AtomicBool::new(true); // the signal landed before the first size
        let options = ScanOptions { tmux_sizes: vec![(30, 5)], tmux_settle_ms: Some(0), ..ScanOptions::default() };
        let err = size_pass(&tmux, "smoke:0.0", &options, &pane_40x5(), "", "", &mut Vec::new(), &flag).unwrap_err();
        assert_eq!(err, "interrupted; window not restored: tmux set-option: boom");
        let log = std::fs::read_to_string(dir.join("calls.log")).unwrap();
        assert_eq!(log.lines().filter(|&c| c == "resize-window").count(), 1, "no size was applied; only the restore resized: {log}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn a_capture_that_times_out_still_restores_the_window() {
        let script = r#"#!/bin/sh
DIR="$(dirname "$0")"
echo "$1" >> "$DIR/calls.log"
case "$1" in
  -V) echo "tmux 3.7c" ;;
  list-panes) exit 0 ;;
  display-message) printf '30\t5\t30\t5\t\n' ;;
  capture-pane) exec sleep 30 ;;
  resize-window|set-option) exit 0 ;;
  *) exit 1 ;;
esac
"#;
        let (dir, mut tmux) = located("impeccable-terminal-capture-timeout", script);
        tmux.timeout = Duration::from_secs(1);
        let options = ScanOptions { tmux_sizes: vec![(30, 5)], tmux_settle_ms: Some(0), ..ScanOptions::default() };
        let err = size_pass(&tmux, "smoke:0.0", &options, &pane_40x5(), "", "", &mut Vec::new(), &AtomicBool::new(false)).unwrap_err();
        assert_eq!(err, crate::tmux::timed_out("capture-pane", tmux.timeout));
        let log = std::fs::read_to_string(dir.join("calls.log")).unwrap();
        let calls: Vec<&str> = log.lines().collect();
        assert_eq!(calls[calls.len() - 2..], ["resize-window", "set-option"], "the restore ran after the hung capture: {calls:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_signal_after_the_last_check_still_reports_the_interrupt() {
        let flag = AtomicBool::new(true);
        assert_eq!(late_interrupt(Ok(()), &flag).unwrap_err(), "interrupted; window restored");
        assert_eq!(late_interrupt(Ok(()), &AtomicBool::new(false)), Ok(()));
        // An error already names what went wrong with the window; it passes through.
        assert_eq!(late_interrupt(Err("window not restored: boom".to_string()), &flag).unwrap_err(), "window not restored: boom");
    }

    #[cfg(unix)]
    #[test]
    fn restore_puts_back_a_window_size_the_user_had_set() {
        let (dir, fake) = fake_tmux("impeccable-terminal-restore-set", &fake_tmux_logging_args("largest\\n"));
        let mut env = HashMap::new();
        env.insert("IMPECCABLE_TMUX".to_string(), fake.to_string_lossy().into_owned());
        let options = ScanOptions { tmux_sizes: vec![(30, 5)], tmux_settle_ms: Some(0), ..ScanOptions::default() };
        TerminalEngine::new(env).detect_pane("smoke:0.0", &options).unwrap();
        assert_eq!(
            window_calls(&dir),
            vec![
                "show-options -w -v -t smoke:0.0 window-size",
                "resize-window -t smoke:0.0 -x 30 -y 5",
                "resize-window -t smoke:0.0 -x 40 -y 5",
                "set-option -w -t smoke:0.0 window-size largest",
            ]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn restore_unsets_window_size_when_the_window_had_none() {
        let (dir, fake) = fake_tmux("impeccable-terminal-restore-unset", &fake_tmux_logging_args(""));
        let mut env = HashMap::new();
        env.insert("IMPECCABLE_TMUX".to_string(), fake.to_string_lossy().into_owned());
        let options = ScanOptions { tmux_sizes: vec![(30, 5)], tmux_settle_ms: Some(0), ..ScanOptions::default() };
        TerminalEngine::new(env).detect_pane("smoke:0.0", &options).unwrap();
        assert_eq!(
            window_calls(&dir),
            vec![
                "show-options -w -v -t smoke:0.0 window-size",
                "resize-window -t smoke:0.0 -x 30 -y 5",
                "resize-window -t smoke:0.0 -x 40 -y 5",
                "set-option -w -t smoke:0.0 -u window-size",
            ]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn a_failed_capture_and_a_failed_restore_are_both_reported() {
        let script = fake_tmux_for_restore_scenarios(true, true);
        let (dir, fake) = fake_tmux("impeccable-terminal-restore-failure", &script);
        let mut env = HashMap::new();
        env.insert("IMPECCABLE_TMUX".to_string(), fake.to_string_lossy().into_owned());
        let engine = TerminalEngine::new(env);
        let options = ScanOptions { tmux_sizes: vec![(30, 5)], tmux_settle_ms: Some(0), ..ScanOptions::default() };
        let err = engine.detect_pane("smoke:0.0", &options).unwrap_err();
        assert_eq!(err.message, "tmux capture-pane: nope; window not restored: tmux set-option: boom");
        let log = std::fs::read_to_string(dir.join("calls.log")).unwrap();
        let calls: Vec<&str> = log.lines().collect();
        assert_eq!(calls.iter().filter(|&&c| c == "resize-window").count(), 2, "the size pass and the restore: {calls:?}");
        assert!(calls.contains(&"set-option"), "{calls:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn a_failed_restore_is_reported_when_capture_succeeded() {
        let script = fake_tmux_for_restore_scenarios(false, true);
        let (dir, fake) = fake_tmux("impeccable-terminal-restore-only-failure", &script);
        let mut env = HashMap::new();
        env.insert("IMPECCABLE_TMUX".to_string(), fake.to_string_lossy().into_owned());
        let engine = TerminalEngine::new(env);
        let options = ScanOptions { tmux_sizes: vec![(30, 5)], tmux_settle_ms: Some(0), ..ScanOptions::default() };
        let err = engine.detect_pane("smoke:0.0", &options).unwrap_err();
        assert_eq!(err.message, "window not restored: tmux set-option: boom");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn a_pane_scan_recaptures_twice_and_the_last_waits_a_full_second() {
        let (dir, fake) = fake_tmux("impeccable-terminal-recaptures", &fake_tmux_for_restore_scenarios(false, false));
        let mut env = HashMap::new();
        env.insert("IMPECCABLE_TMUX".to_string(), fake.to_string_lossy().into_owned());
        let start = std::time::Instant::now();
        TerminalEngine::new(env).detect_pane("smoke:0.0", &ScanOptions::default()).unwrap();
        let elapsed = start.elapsed();
        let log = std::fs::read_to_string(dir.join("calls.log")).unwrap();
        assert_eq!(log.lines().filter(|&c| c == "capture-pane").count(), 3, "frame 0 and two recaptures: {log}");
        assert!(!log.lines().any(|c| c == "resize-window"), "no sizes, no resize");
        assert!(elapsed >= std::time::Duration::from_millis(1000), "the last recapture waits 1000 ms: {elapsed:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn a_failed_capture_still_restores_the_window() {
        let script = fake_tmux_for_restore_scenarios(true, false);
        let (dir, fake) = fake_tmux("impeccable-terminal-capture-failure", &script);
        let mut env = HashMap::new();
        env.insert("IMPECCABLE_TMUX".to_string(), fake.to_string_lossy().into_owned());
        let engine = TerminalEngine::new(env);
        let options = ScanOptions { tmux_sizes: vec![(30, 5)], tmux_settle_ms: Some(0), ..ScanOptions::default() };
        let err = engine.detect_pane("smoke:0.0", &options).unwrap_err();
        assert_eq!(err.message, "tmux capture-pane: nope");
        let log = std::fs::read_to_string(dir.join("calls.log")).unwrap();
        let calls: Vec<&str> = log.lines().collect();
        let last_capture = calls.iter().rposition(|&c| c == "capture-pane").expect("a capture-pane call");
        let set_option = calls.iter().position(|&c| c == "set-option").expect("a set-option call");
        assert!(set_option > last_capture, "set-option must run after the failing capture-pane: {calls:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
