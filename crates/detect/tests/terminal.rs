//! The platform gate around the terminal source rules: a web project never
//! sees a `tui-` finding, a terminal project scans terminal source per file,
//! and the walker adds the terminal extensions only on a terminal project.

use impeccable_core::checks::terminal::{ProjectSignals, ADAPTIVE_PRESENT_NOTE};
use impeccable_detect::detect_text::{detect_text, TextOptions};
use impeccable_detect::file_system::{walk_dir_reporting, walk_dir_reporting_for, TERMINAL_SKIP_DIRS};

const RATATUI_DOUBLE: &str = "use ratatui::widgets::{Block, BorderType};\n// BorderType::Double in a comment does not count\nfn ui() -> Block<'static> {\n    Block::default().border_type(BorderType::Double)\n}\n";

fn text(content: &str, path: &str, platform: Option<&str>, signals: Option<&ProjectSignals>) -> Vec<String> {
    detect_text(
        content,
        path,
        &TextOptions { inline_ignores: true, platform, signals, ..Default::default() },
    )
    .into_iter()
    .map(|f| format!("{}@{}", f.antipattern, f.line))
    .collect()
}

#[test]
fn web_project_with_rust_backend_gets_no_terminal_rules() {
    for platform in [None, Some("web"), Some("ios"), Some("adaptive")] {
        let ids = text(RATATUI_DOUBLE, "/app/src/ui.rs", platform, None);
        assert!(ids.iter().all(|id| !id.starts_with("tui-")), "{platform:?}: {ids:?}");
    }
}

#[test]
fn explicit_python_file_on_web_project_still_gets_web_findings() {
    let src = "CSS = \"\"\".card { border-left: 4px solid #6366f1; border-radius: 12px; }\"\"\"\n";
    let ids = text(src, "/app/app.py", None, None);
    assert!(ids.iter().any(|id| id.starts_with("side-tab@")), "{ids:?}");
}

#[test]
fn terminal_project_flags_rust_ui_once_per_line() {
    assert_eq!(text(RATATUI_DOUBLE, "/app/src/ui.rs", Some("terminal"), None), vec!["tui-double-border@4"]);
}

#[test]
fn hardcoded_truecolor_flagged_only_in_tui_files() {
    let bevy = "use bevy::prelude::*;\nlet c = Color::Rgb(1.0, 0.5, 0.2);\n";
    assert!(text(bevy, "/game/src/main.rs", Some("terminal"), None).is_empty());
    let tui = "use ratatui::style::Color;\nlet c = Color::Rgb(255, 0, 128);\n";
    assert_eq!(text(tui, "/app/src/theme.rs", Some("terminal"), None), vec!["tui-hardcoded-rgb-no-adapt@2"]);
}

#[test]
fn ink_component_gets_terminal_rules_only_on_a_terminal_project() {
    let src = "import { Box, Text } from 'ink';\nexport const Card = () => <Box borderStyle=\"double\"><Text>hi</Text></Box>;\n";
    let ids = text(src, "/cli/src/Card.tsx", Some("terminal"), None);
    assert!(ids.contains(&"tui-double-border@2".to_string()), "{ids:?}");
    assert!(text(src, "/cli/src/Card.tsx", None, None).iter().all(|id| !id.starts_with("tui-")));
}

#[test]
fn terminal_findings_are_waivable_inline() {
    let src = "use ratatui::widgets::BorderType;\n// impeccable-disable-next-line tui-double-border\nlet b = BorderType::Double;\n";
    assert!(text(src, "/app/src/ui.rs", Some("terminal"), None).is_empty());
}

#[test]
fn textual_css_is_always_terminal_source() {
    assert_eq!(text("Screen { border: double $primary; }\n", "/app/app.tcss", Some("terminal"), None), vec!["tui-double-border@1"]);
    assert!(text("Screen { border: double $primary; }\n", "/app/app.tcss", None, None).is_empty());
}

#[test]
fn signals_reach_the_rules_through_text_options() {
    let src = "use ratatui::style::Color;\nlet c = Color::Rgb(1, 2, 3);\n";
    let adaptive = ProjectSignals { has_adaptive_color: true, ..Default::default() };
    let f = detect_text(
        src,
        "/app/src/theme.rs",
        &TextOptions { inline_ignores: true, platform: Some("terminal"), signals: Some(&adaptive), ..Default::default() },
    );
    assert_eq!(f[0].extras.get("note").and_then(|v| v.as_str()), Some(ADAPTIVE_PRESENT_NOTE));
}

fn scratch(name: &str) -> String {
    let dir = std::env::temp_dir().join(format!("impeccable-terminal-walk-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for rel in [
        "src/ui.rs", "src/app.py", "src/main.go", "src/app.tcss", "src/Card.tsx", "src/page.html",
        "target/debug/x.rs", "vendor/lib.go", "node_modules/m/index.js",
    ] {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "").unwrap();
    }
    dir.to_string_lossy().into_owned()
}

fn names(files: Vec<String>, root: &str) -> Vec<String> {
    let mut v: Vec<String> = files.iter().map(|f| f[root.len() + 1..].replace('\\', "/")).collect();
    v.sort();
    v
}

#[test]
fn web_walk_is_unchanged() {
    let root = scratch("web");
    assert_eq!(names(walk_dir_reporting(&root, &mut |_, _| {}), &root), vec!["src/Card.tsx", "src/page.html"]);
    assert_eq!(names(walk_dir_reporting_for(&root, Some("web"), &mut |_, _| {}), &root), vec!["src/Card.tsx", "src/page.html"]);
}

#[test]
fn terminal_walk_adds_the_terminal_extensions_and_skips_build_dirs() {
    let root = scratch("tui");
    assert_eq!(
        names(walk_dir_reporting_for(&root, Some("terminal"), &mut |_, _| {}), &root),
        vec!["src/Card.tsx", "src/app.py", "src/app.tcss", "src/main.go", "src/page.html", "src/ui.rs"]
    );
    assert_eq!(TERMINAL_SKIP_DIRS, &["target", ".venv", "vendor"]);
}
