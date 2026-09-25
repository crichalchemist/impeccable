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
use std::thread::sleep;
use std::time::Duration;

use impeccable_core::findings::Finding;
use impeccable_detect::engines::{EngineError, ScanOptions, TmuxEngine};

use capture::{parse_frames, parse_rows, Frame, FrameRole};
use palette::Palette;
use tmux::{PaneInfo, Tmux};

/// The gap between the first capture and the recapture.
pub const RECAPTURE_DELAY_MS: u64 = 1000;
/// `--tmux-settle` when the flag is absent.
pub const DEFAULT_SETTLE_MS: u64 = 300;

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
/// The caller restores the window whatever this returns.
fn capture_sizes(tmux: &Tmux, target: &str, options: &ScanOptions, colorterm: &str, frames: &mut Vec<Frame>) -> Result<(), String> {
    let settle = Duration::from_millis(options.tmux_settle_ms.unwrap_or(DEFAULT_SETTLE_MS));
    for &(w, h) in &options.tmux_sizes {
        tmux.resize(target, w as usize, h as usize)?;
        sleep(settle);
        let info = tmux.pane_info(target)?;
        let text = tmux.capture(target)?;
        frames.push(frame_from(&text, &info, colorterm, FrameRole::Capture));
    }
    Ok(())
}

impl TmuxEngine for TerminalEngine {
    fn detect_pane(&self, target: &str, options: &ScanOptions) -> Result<Vec<Finding>, EngineError> {
        let tmux = Tmux::locate(&self.env).map_err(EngineError::new)?;
        let info = tmux.pane_info(target).map_err(EngineError::new)?;
        let colorterm = self.colorterm();
        let first = tmux.capture(target).map_err(EngineError::new)?;
        let mut frames = vec![frame_from(&first, &info, &colorterm, FrameRole::Capture)];
        sleep(Duration::from_millis(RECAPTURE_DELAY_MS));
        let second = tmux.capture(target).map_err(EngineError::new)?;
        frames.push(frame_from(&second, &info, &colorterm, FrameRole::Recapture));
        if !options.tmux_sizes.is_empty() {
            let captured = capture_sizes(&tmux, target, options, &colorterm, &mut frames);
            let restored = tmux.restore(target, info.window_width, info.window_height);
            match (captured, restored) {
                (Err(c), Err(r)) => return Err(EngineError::new(format!("{c}; window not restored: {r}"))),
                (Ok(()), Err(r)) => return Err(EngineError::new(format!("window not restored: {r}"))),
                (Err(c), Ok(())) => return Err(EngineError::new(c)),
                (Ok(()), Ok(())) => {}
            }
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

    #[cfg(unix)]
    #[test]
    fn restore_failure_is_reported_alongside_a_capture_failure() {
        let script = r#"#!/bin/sh
case "$1" in
  -V) echo "tmux 3.7c" ;;
  list-panes) exit 0 ;;
  display-message) printf '40\t5\t40\t5\t\n' ;;
  capture-pane) echo "hi" ;;
  resize-window) exit 0 ;;
  set-option) echo boom >&2; exit 1 ;;
  *) exit 1 ;;
esac
"#;
        let (dir, fake) = fake_tmux("impeccable-terminal-restore-failure", script);
        let mut env = HashMap::new();
        env.insert("IMPECCABLE_TMUX".to_string(), fake.to_string_lossy().into_owned());
        let engine = TerminalEngine::new(env);
        let options = ScanOptions { tmux_sizes: vec![(30, 5)], tmux_settle_ms: Some(0), ..ScanOptions::default() };
        let err = engine.detect_pane("smoke:0.0", &options).unwrap_err();
        assert_eq!(err.message, "window not restored: tmux set-option: boom");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
