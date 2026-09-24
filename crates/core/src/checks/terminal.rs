//! Terminal source rules (the `tui-` registry rows): pure text scans over TUI
//! and rich-CLI source. `impeccable detect` runs them only on a `terminal`
//! project and only over files that name a terminal framework
//! ([`classify_terminal_source`]); the browser adapters never call this module.
//!
//! Every scan takes comment-blanked text (line count preserved, so line
//! numbers hold) and the [`ProjectSignals`] the walker collected. `None`
//! signals means a single-file scan, and the absence rules then report with
//! [`SIGNALS_NOTE`] instead of deciding.

use crate::findings::{finding, Finding};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

macro_rules! re {
    ($name:ident, $pat:expr) => {
        static $name: Lazy<Regex> = Lazy::new(|| Regex::new(&$pat).expect(stringify!($name)));
    };
}

/// The extensions the walker adds on a terminal project. `.tsx`, `.jsx`,
/// `.ts`, `.js` are already web-scannable and carry Ink components.
pub const TERMINAL_EXTENSIONS: &[&str] = &[".rs", ".go", ".py", ".tcss"];

/// The note an absence rule carries when no project signals were collected.
pub const SIGNALS_NOTE: &str =
    "project signals were not collected; scan the project directory to confirm";

/// The terminal stack a source file belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stack {
    /// Rust: ratatui or crossterm.
    Ratatui,
    /// Go: any charmbracelet import (Bubble Tea, Lip Gloss, Bubbles).
    Charm,
    /// Python: textual or rich.
    Textual,
    /// Textual CSS (`.tcss`), always terminal source.
    TextualCss,
    /// TypeScript or JavaScript that imports ink.
    Ink,
}

re!(PY_TEXTUAL_RE, r"(?m)^[ \t]*(?:from|import)[ \t]+(?:textual|rich)(?:[ \t.]|$)");
re!(INK_IMPORT_RE, r#"(?:from[ \t]+|require\([ \t]*)["']ink(?:-[a-z0-9-]+)?["']"#);

/// Spec section 3 framework table. A file that fails this gets no terminal
/// rules, which keeps Bevy games and Django backends out.
pub fn classify_terminal_source(ext: &str, text: &str) -> Option<Stack> {
    match ext {
        ".rs" if text.contains("ratatui") || text.contains("crossterm") => Some(Stack::Ratatui),
        ".go" if text.contains("charmbracelet") => Some(Stack::Charm),
        ".py" if PY_TEXTUAL_RE.is_match(text) => Some(Stack::Textual),
        ".tcss" => Some(Stack::TextualCss),
        ".tsx" | ".jsx" | ".ts" | ".js" | ".mjs" | ".cjs" if INK_IMPORT_RE.is_match(text) => {
            Some(Stack::Ink)
        }
        _ => None,
    }
}

re!(
    TTY_GUARD_RE,
    r#"isatty|isTTY|is_terminal|IsTerminal|--no-progress|(?:env::var|getenv|Getenv|process\.env\[|os\.environ(?:\.get)?)[ \t]*[\[(]?[ \t]*["']CI["']|process\.env\.CI\b"#
);
re!(ADAPTIVE_COLOR_RE, r"AdaptiveColor|LightDark|HasDarkBackground|ansi_color|NO_COLOR");
re!(
    WIDTH_LIB_RE,
    r"unicode[-_]width|unicode[-_]segmentation|runewidth|uniseg|ansi\.Truncate|string-width|wcwidth|grapheme"
);
re!(
    ICON_TOGGLE_RE,
    r#"--no-icons|["'](?:--)?(?:ascii|nerd|icons)["']|\b(?:ascii|nerd|icons)[ \t]*[:=]"#
);
re!(PLAIN_FLAG_RE, r"--plain|--json|--no-tui|--static|NO_TUI");

/// Spec section 3 "Project signals": booleans collected over every scanned
/// file before the source rules run. Half the rules are "X present and Y
/// absent anywhere in the project"; this is the Y.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProjectSignals {
    pub has_tty_guard: bool,
    pub has_adaptive_color: bool,
    pub has_width_lib: bool,
    pub has_icon_toggle: bool,
    pub has_plain_flag: bool,
}

impl ProjectSignals {
    /// Fold one file's raw text (comments included: a `// NO_COLOR` mention
    /// still tells us the author knows the convention) into the set.
    pub fn absorb(&mut self, text: &str) {
        self.has_tty_guard |= TTY_GUARD_RE.is_match(text);
        self.has_adaptive_color |= ADAPTIVE_COLOR_RE.is_match(text);
        self.has_width_lib |= WIDTH_LIB_RE.is_match(text);
        self.has_icon_toggle |= ICON_TOGGLE_RE.is_match(text);
        self.has_plain_flag |= PLAIN_FLAG_RE.is_match(text);
    }

    pub fn collect<'a>(texts: impl IntoIterator<Item = &'a str>) -> ProjectSignals {
        let mut s = ProjectSignals::default();
        for t in texts {
            s.absorb(t);
        }
        s
    }
}

/// Blank `//` and `/* */` comments in Rust, Go, or Textual CSS, keeping every
/// newline so line numbers hold. String literals (`"..."`, Go backtick raw
/// strings) and Rust char literals are copied verbatim, so `"http://x"` and
/// `'/'` survive.
pub fn strip_c_comments(text: &str) -> String {
    let c: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let n = c.len();
    let mut i = 0;
    while i < n {
        let ch = c[i];
        if ch == '/' && i + 1 < n && c[i + 1] == '/' {
            while i < n && c[i] != '\n' {
                out.push(' ');
                i += 1;
            }
        } else if ch == '/' && i + 1 < n && c[i + 1] == '*' {
            out.push_str("  ");
            i += 2;
            while i < n && !(c[i] == '*' && i + 1 < n && c[i + 1] == '/') {
                out.push(if c[i] == '\n' { '\n' } else { ' ' });
                i += 1;
            }
            if i < n {
                out.push_str("  ");
                i += 2;
            }
        } else if ch == '"' || ch == '`' {
            let q = ch;
            out.push(q);
            i += 1;
            while i < n && c[i] != q {
                if c[i] == '\\' && q == '"' && i + 1 < n {
                    out.push(c[i]);
                    out.push(c[i + 1]);
                    i += 2;
                    continue;
                }
                out.push(c[i]);
                i += 1;
            }
            if i < n {
                out.push(q);
                i += 1;
            }
        } else if ch == '\'' {
            // `'x'` or `'\n'` is a char literal; a lone `'` (a Rust lifetime)
            // is code. Copy the literal through its closing quote.
            let close = if i + 1 < n && c[i + 1] == '\\' { i + 3 } else { i + 2 };
            if close < n && c[close] == '\'' {
                for k in i..=close {
                    out.push(c[k]);
                }
                i = close + 1;
            } else {
                out.push(ch);
                i += 1;
            }
        } else {
            out.push(ch);
            i += 1;
        }
    }
    out
}

/// Blank `#` comments and bare docstrings in Python, keeping every newline.
/// A triple-quoted literal is blanked only when the quote (optionally after a
/// prefix such as `r` or `f`) is the first token on its line: that is a
/// docstring statement. `CSS = """..."""` assignments are kept, because
/// Textual apps carry their stylesheet there and the rules must see it.
pub fn strip_python_comments(text: &str) -> String {
    let c: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let n = c.len();
    let mut i = 0;
    let mut line_start = 0;
    while i < n {
        let ch = c[i];
        if ch == '\n' {
            out.push('\n');
            i += 1;
            line_start = i;
        } else if ch == '#' {
            while i < n && c[i] != '\n' {
                out.push(' ');
                i += 1;
            }
        } else if ch == '"' || ch == '\'' {
            let q = ch;
            let triple = i + 2 < n && c[i + 1] == q && c[i + 2] == q;
            if triple {
                let prefix: String = c[line_start..i].iter().collect();
                let is_docstring = prefix
                    .trim_start()
                    .chars()
                    .all(|p| matches!(p, 'r' | 'R' | 'b' | 'B' | 'f' | 'F' | 'u' | 'U'));
                out.push_str(&q.to_string().repeat(3));
                i += 3;
                while i < n && !(c[i] == q && i + 2 < n && c[i + 1] == q && c[i + 2] == q) {
                    out.push(if is_docstring && c[i] != '\n' { ' ' } else { c[i] });
                    i += 1;
                }
                if i < n {
                    out.push_str(&q.to_string().repeat(3));
                    i += 3;
                }
            } else {
                out.push(q);
                i += 1;
                while i < n && c[i] != q && c[i] != '\n' {
                    if c[i] == '\\' && i + 1 < n {
                        out.push(c[i]);
                        out.push(c[i + 1]);
                        i += 2;
                        continue;
                    }
                    out.push(c[i]);
                    i += 1;
                }
                if i < n && c[i] == q {
                    out.push(q);
                    i += 1;
                }
            }
        } else {
            out.push(ch);
            i += 1;
        }
    }
    out
}

fn hit(id: &str, file_path: &str, lines: &[&str], idx: usize) -> Finding {
    finding(id, file_path, lines[idx].trim(), (idx + 1) as f64)
}

fn with_note(mut f: Finding, note: &str) -> Finding {
    f.extras.insert("note".into(), Value::String(note.into()));
    f
}

fn with_count(mut f: Finding, key: &str, n: usize) -> Finding {
    f.extras.insert(key.into(), Value::from(n as u64));
    f
}

// ─── tui-figlet-banner ──────────────────────────────────────────────────────
re!(
    FIGLET_RE,
    r"(?i)\b(?:pyfiglet|figlet_rs|figlet|go-figure|cfonts|ink-big-text|text2art)\b|from[ \t]+art[ \t]+import"
);
const BLOCK_GLYPHS: &[char] = &['█', '▀', '▄', '╔', '═', '╗', '║', '╚', '╝'];

/// A line that carries a string literal whose visible characters are mostly
/// block glyphs: one row of a hand-drawn banner.
fn is_block_glyph_line(line: &str) -> bool {
    if !(line.contains('"') || line.contains('\'') || line.contains('`')) {
        return false;
    }
    let body: Vec<char> = line
        .chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '"' | '\'' | '`' | ',' | '+' | ';' | '(' | ')' | '[' | ']'))
        .collect();
    if body.len() < 6 {
        return false;
    }
    let glyphs = body.iter().filter(|c| BLOCK_GLYPHS.contains(c)).count();
    glyphs * 10 >= body.len() * 6
}

fn scan_terminal_figlet_banner(lines: &[&str], file_path: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        if FIGLET_RE.is_match(l) {
            out.push(hit("tui-figlet-banner", file_path, lines, i));
        }
    }
    let mut run = 0usize;
    for (i, l) in lines.iter().enumerate() {
        if is_block_glyph_line(l) {
            run += 1;
            if run == 4 {
                out.push(with_count(hit("tui-figlet-banner", file_path, lines, i + 1 - run), "bannerLines", run));
            }
        } else {
            run = 0;
        }
    }
    out.sort_by(|a, b| a.line.partial_cmp(&b.line).unwrap_or(std::cmp::Ordering::Equal));
    out
}

// ─── tui-gradient-title ─────────────────────────────────────────────────────
re!(
    GRADIENT_RE,
    r"gradient-string|ink-gradient|<Gradient\b|rich_gradient|\bGradient\(|\blolcat\b|\bchromacat\b|tui[-_]gradient"
);
re!(BLEND_RE, r"lipgloss\.Blend1D\(|\.Blend\(");
re!(TITLE_WORD_RE, r"(?i)title|header|banner");

fn scan_terminal_gradient_title(lines: &[&str], file_path: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        // A style chain may wrap: `titleStyle := lipgloss.NewStyle().` then
        // `Foreground(lipgloss.Blend1D(...))`. The previous line counts only
        // when it ends mid-expression, so a separate statement after a title
        // line is not a title.
        let continued = i > 0
            && lines[i - 1].trim_end().ends_with(|c: char| matches!(c, '.' | '(' | ',' | '{'));
        let blend_into_title = BLEND_RE.is_match(l)
            && (TITLE_WORD_RE.is_match(l) || (continued && TITLE_WORD_RE.is_match(lines[i - 1])));
        if GRADIENT_RE.is_match(l) || blend_into_title {
            out.push(hit("tui-gradient-title", file_path, lines, i));
        }
    }
    out
}

// ─── tui-blink-attribute ────────────────────────────────────────────────────
re!(
    BLINK_RE,
    r"Modifier::SLOW_BLINK|Modifier::RAPID_BLINK|\.Blink\(|\[blink\]|text-style:[^;\n]*\bblink\b|\\x1b\[5m|\\033\[5m|\\e\[5m|\x1b\[5m"
);

fn scan_terminal_blink_attribute(lines: &[&str], file_path: &str) -> Vec<Finding> {
    lines
        .iter()
        .enumerate()
        .filter(|(_, l)| BLINK_RE.is_match(l))
        .map(|(i, _)| hit("tui-blink-attribute", file_path, lines, i))
        .collect()
}

// ─── tui-emoji-density ──────────────────────────────────────────────────────
const EMOJI_FILE_THRESHOLD: usize = 8;
const CANONICAL_EMOJI_THRESHOLD: usize = 3;
const CANONICAL_EMOJI: &[char] = &['🚀', '✅', '❌', '⚠', '✨', '🎉', '📦', '🔧', '🔥', '💡'];

fn is_emoji(c: char) -> bool {
    matches!(c, '\u{1F300}'..='\u{1FAFF}' | '\u{2600}'..='\u{27BF}')
}

fn scan_terminal_emoji_density(lines: &[&str], file_path: &str) -> Vec<Finding> {
    let mut total = 0usize;
    let mut canonical = 0usize;
    let mut first: Option<usize> = None;
    for (i, l) in lines.iter().enumerate() {
        for c in l.chars().filter(|c| is_emoji(*c)) {
            total += 1;
            if CANONICAL_EMOJI.contains(&c) {
                canonical += 1;
            }
            first.get_or_insert(i);
        }
    }
    match first {
        Some(i) if total >= EMOJI_FILE_THRESHOLD || canonical >= CANONICAL_EMOJI_THRESHOLD => {
            vec![with_count(hit("tui-emoji-density", file_path, lines, i), "emojiCount", total)]
        }
        _ => Vec::new(),
    }
}

// ─── tui-double-border ──────────────────────────────────────────────────────
re!(
    DOUBLE_BORDER_RE,
    r#"BorderType::Double\b|DoubleBorder\(\)|borderStyle=\{?["']double["']|box\.DOUBLE\b|\bborder(?:-(?:top|right|bottom|left))?:[ \t]*double\b"#
);

fn scan_terminal_double_border(lines: &[&str], file_path: &str) -> Vec<Finding> {
    lines
        .iter()
        .enumerate()
        .filter(|(_, l)| DOUBLE_BORDER_RE.is_match(l))
        .map(|(i, _)| hit("tui-double-border", file_path, lines, i))
        .collect()
}

// ─── tui-hardcoded-rgb-no-adapt ─────────────────────────────────────────────
re!(
    RGB_RE,
    r##"Color::Rgb\(|Color::from_u32\(0x|lipgloss\.Color\("#|chalk\.hex\(|\bcolor=["']#|\[#[0-9a-fA-F]{6}\]|\\x1b\[38;2;|\\033\[38;2;|\x1b\[38;2;"##
);
/// The note `tui-hardcoded-rgb-no-adapt` carries when the project does have
/// an adaptive color helper somewhere: the literal may still be themed.
pub const ADAPTIVE_PRESENT_NOTE: &str =
    "an adaptive color helper exists elsewhere in this project; confirm this literal is themed";

fn scan_terminal_hardcoded_rgb(
    lines: &[&str],
    file_path: &str,
    stack: Stack,
    signals: Option<&ProjectSignals>,
) -> Vec<Finding> {
    if stack == Stack::TextualCss {
        // A stylesheet is where theme colors belong; the signal is weak there.
        return Vec::new();
    }
    lines
        .iter()
        .enumerate()
        .filter(|(_, l)| RGB_RE.is_match(l))
        .map(|(i, _)| {
            let f = hit("tui-hardcoded-rgb-no-adapt", file_path, lines, i);
            match signals {
                None => with_note(f, SIGNALS_NOTE),
                Some(s) if s.has_adaptive_color => with_note(f, ADAPTIVE_PRESENT_NOTE),
                Some(_) => f,
            }
        })
        .collect()
}

/// Run every terminal rule over one file. `source` is the file text with
/// comments already blanked (Ink files use the detect crate's JS stripper,
/// Python files [`strip_python_comments`], the rest [`strip_c_comments`]).
pub fn scan_terminal_source(
    source: &str,
    file_path: &str,
    stack: Stack,
    signals: Option<&ProjectSignals>,
) -> Vec<Finding> {
    let lines: Vec<&str> = source.split('\n').collect();
    let mut out = Vec::new();
    out.extend(scan_terminal_figlet_banner(&lines, file_path));
    out.extend(scan_terminal_gradient_title(&lines, file_path));
    out.extend(scan_terminal_blink_attribute(&lines, file_path));
    out.extend(scan_terminal_emoji_density(&lines, file_path));
    out.extend(scan_terminal_double_border(&lines, file_path));
    out.extend(scan_terminal_hardcoded_rgb(&lines, file_path, stack, signals));
    // Quality rules follow in Task 5.
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framework_detection_follows_the_spec_table() {
        assert_eq!(classify_terminal_source(".rs", "use ratatui::prelude::*;"), Some(Stack::Ratatui));
        assert_eq!(classify_terminal_source(".rs", "use crossterm::event;"), Some(Stack::Ratatui));
        assert_eq!(classify_terminal_source(".go", "import tea \"github.com/charmbracelet/bubbletea\""), Some(Stack::Charm));
        assert_eq!(classify_terminal_source(".py", "from textual.app import App"), Some(Stack::Textual));
        assert_eq!(classify_terminal_source(".py", "import rich\n"), Some(Stack::Textual));
        assert_eq!(classify_terminal_source(".tcss", "Screen { }"), Some(Stack::TextualCss));
        assert_eq!(classify_terminal_source(".tsx", "import { Box } from 'ink';"), Some(Stack::Ink));
        assert_eq!(classify_terminal_source(".js", "const { render } = require(\"ink\");"), Some(Stack::Ink));
        assert_eq!(classify_terminal_source(".tsx", "import Spinner from 'ink-spinner';"), Some(Stack::Ink));
    }

    #[test]
    fn bevy_game_and_django_backend_are_not_terminal_source() {
        assert_eq!(classify_terminal_source(".rs", "use bevy::prelude::*; let c = Color::Rgb(1,2,3);"), None);
        assert_eq!(classify_terminal_source(".py", "from django.db import models\nprint('x')"), None);
        assert_eq!(classify_terminal_source(".py", "enrich = richness(3)"), None, "rich as a substring is not an import");
        assert_eq!(classify_terminal_source(".go", "package main\nimport \"fmt\""), None);
        assert_eq!(classify_terminal_source(".tsx", "import React from 'react'; // ink is a word"), None);
        assert_eq!(classify_terminal_source(".html", "<b>ratatui</b>"), None);
    }

    #[test]
    fn project_signals_fold_across_files() {
        let s = ProjectSignals::collect(["fn main() {}", "if std::io::stdout().is_terminal() {}", "use unicode_width::UnicodeWidthStr;"]);
        assert!(s.has_tty_guard);
        assert!(s.has_width_lib);
        assert!(!s.has_adaptive_color);
        assert!(!s.has_icon_toggle);
        assert!(!s.has_plain_flag);
        let s = ProjectSignals::collect(["if os.environ.get(\"CI\"): quiet()", "style = lipgloss.AdaptiveColor{}", "flags.Bool(\"no-icons\", ...)  // --no-icons", "--json"]);
        assert!(s.has_tty_guard && s.has_adaptive_color && s.has_icon_toggle && s.has_plain_flag);
    }

    #[test]
    fn c_comment_blanking_keeps_lines_and_strings() {
        let src = "let url = \"http://x\"; // Color::Rgb(1,2,3)\n/* BorderType::Double\n   spans */ let c = '/';\nlet ok = 1;\n";
        let out = strip_c_comments(src);
        assert_eq!(out.matches('\n').count(), src.matches('\n').count());
        assert!(out.contains("\"http://x\""));
        assert!(out.contains("'/'"));
        assert!(!out.contains("Color::Rgb"));
        assert!(!out.contains("BorderType::Double"));
        assert!(out.contains("let ok = 1;"));
        let go = "s := `raw // not a comment`\n// real\n";
        assert!(strip_c_comments(go).contains("raw // not a comment"));
        assert!(!strip_c_comments(go).contains("real"));
        let lifetime = "fn f<'a>(s: &'a str) -> &'a str { s } // BorderType::Double\n";
        assert!(!strip_c_comments(lifetime).contains("BorderType::Double"), "a lifetime tick is not a char literal");
    }

    #[test]
    fn python_blanking_keeps_the_css_block_but_drops_docstrings_and_hashes() {
        let src = "class A(App):\n    \"\"\"A [blink] docstring.\"\"\"\n    CSS = \"\"\"\n    Screen { border: double; }\n    \"\"\"\n    x = 1  # text-style: blink\n    y = \"# not a comment\"\n";
        let out = strip_python_comments(src);
        assert_eq!(out.matches('\n').count(), src.matches('\n').count());
        assert!(!out.contains("[blink] docstring"));
        assert!(out.contains("border: double"));
        assert!(!out.contains("text-style: blink"));
        assert!(out.contains("\"# not a comment\""));
    }

    fn ids(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|f| f.antipattern.as_str()).collect()
    }
    fn scan(src: &str, stack: Stack, signals: Option<&ProjectSignals>) -> Vec<Finding> {
        scan_terminal_source(src, "app.rs", stack, signals)
    }
    const ALL: ProjectSignals = ProjectSignals {
        has_tty_guard: true,
        has_adaptive_color: true,
        has_width_lib: true,
        has_icon_toggle: true,
        has_plain_flag: true,
    };
    const NONE: ProjectSignals = ProjectSignals {
        has_tty_guard: false,
        has_adaptive_color: false,
        has_width_lib: false,
        has_icon_toggle: false,
        has_plain_flag: false,
    };

    #[test]
    fn figlet_import_and_hand_drawn_banner_are_flagged() {
        let src = "use figlet_rs::FIGfont;\nlet a = \"███╗   ███╗\";\nlet b = \"████╗ ████║\";\nlet c = \"██╔████╔██║\";\nlet d = \"██║╚██╔╝██║\";\nlet e = 1;\n";
        let f = scan(src, Stack::Ratatui, Some(&ALL));
        let lines: Vec<f64> = f.iter().filter(|f| f.antipattern == "tui-figlet-banner").map(|f| f.line).collect();
        assert_eq!(lines, vec![1.0, 2.0], "{f:?}");
        assert_eq!(f[1].extras.get("bannerLines"), Some(&Value::from(4u64)));
    }

    #[test]
    fn one_box_drawing_line_is_not_a_banner() {
        let src = "let top = \"╔══════╗\";\nlet mid = \"║ text ║\";\nlet ok = render();\n";
        assert!(ids(&scan(src, Stack::Ratatui, Some(&ALL))).is_empty());
    }

    #[test]
    fn gradient_title_is_flagged_and_blend_needs_title_context() {
        let src = "import Gradient from 'ink-gradient';\nconst t = <Gradient name=\"rainbow\">Hi</Gradient>;\n";
        assert_eq!(ids(&scan(src, Stack::Ink, Some(&ALL))), vec!["tui-gradient-title", "tui-gradient-title"]);
        let go = "title := lipgloss.NewStyle().Foreground(lipgloss.Blend1D(0.5, a, b))\nbar := lipgloss.Blend1D(0.2, a, b)\nheaderStyle := lipgloss.NewStyle().\n\tForeground(lipgloss.Blend1D(0.5, a, b))\n";
        let f = scan(go, Stack::Charm, Some(&ALL));
        let lines: Vec<f64> = f.iter().map(|f| f.line).collect();
        assert_eq!(ids(&f), vec!["tui-gradient-title", "tui-gradient-title"]);
        assert_eq!(lines, vec![1.0, 4.0], "line 2 follows a title but is its own statement");
        assert_eq!(ids(&scan("use tui_gradient::gradient;\n", Stack::Ratatui, Some(&ALL))), vec!["tui-gradient-title"]);
    }

    #[test]
    fn blink_is_flagged_in_every_stack_spelling() {
        for (src, stack) in [
            ("Style::default().add_modifier(Modifier::SLOW_BLINK)", Stack::Ratatui),
            ("lipgloss.NewStyle().Blink(true)", Stack::Charm),
            ("console.print(\"[blink]go[/blink]\")", Stack::Textual),
            ("Label { text-style: bold blink; }", Stack::TextualCss),
            ("process.stdout.write(\"\\x1b[5mHi\")", Stack::Ink),
        ] {
            assert_eq!(ids(&scan(src, stack, Some(&ALL))), vec!["tui-blink-attribute"], "{src}");
        }
        assert!(scan("Label { text-style: bold; }", Stack::TextualCss, Some(&ALL)).is_empty());
    }

    #[test]
    fn emoji_density_reports_once_per_file_with_a_count() {
        let many = "let s = [\"🚀\", \"✅\", \"❌\", \"🎉\", \"📦\", \"🔧\", \"🔥\", \"💡\", \"✨\"];\nlet t = 1;\n";
        let f = scan(many, Stack::Ratatui, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-emoji-density"]);
        assert_eq!(f[0].extras.get("emojiCount"), Some(&Value::from(9u64)));
        let canonical = "print(\"🚀 start\")\nprint(\"✅ done\")\nprint(\"❌ fail\")\n";
        assert_eq!(ids(&scan(canonical, Stack::Textual, Some(&ALL))), vec!["tui-emoji-density"]);
        let few = "print(\"🚀 start\")\nprint(\"done\")\n";
        assert!(scan(few, Stack::Textual, Some(&ALL)).is_empty());
    }

    #[test]
    fn double_border_is_flagged_and_single_border_passes() {
        for (src, stack) in [
            ("Block::default().border_type(BorderType::Double)", Stack::Ratatui),
            ("lipgloss.NewStyle().Border(lipgloss.DoubleBorder())", Stack::Charm),
            ("Panel(x, box=box.DOUBLE)", Stack::Textual),
            ("#main { border: double $accent; }", Stack::TextualCss),
            ("<Box borderStyle=\"double\">", Stack::Ink),
        ] {
            assert_eq!(ids(&scan(src, stack, Some(&ALL))), vec!["tui-double-border"], "{src}");
        }
        assert!(scan("Block::default().border_type(BorderType::Rounded)", Stack::Ratatui, Some(&ALL)).is_empty());
        assert!(scan("#main { border: round $accent; }", Stack::TextualCss, Some(&ALL)).is_empty());
    }

    #[test]
    fn hardcoded_truecolor_reports_plainly_without_adaptation_and_with_a_note_otherwise() {
        let src = "let c = Color::Rgb(255, 0, 128);\n";
        let plain = scan(src, Stack::Ratatui, Some(&NONE));
        assert_eq!(ids(&plain), vec!["tui-hardcoded-rgb-no-adapt"]);
        assert!(plain[0].extras.get("note").is_none());
        let adapted = scan(src, Stack::Ratatui, Some(&ALL));
        assert_eq!(adapted[0].extras.get("note"), Some(&Value::String(ADAPTIVE_PRESENT_NOTE.into())));
        let single = scan(src, Stack::Ratatui, None);
        assert_eq!(single[0].extras.get("note"), Some(&Value::String(SIGNALS_NOTE.into())));
        assert!(scan("#main { color: #ff0080; }", Stack::TextualCss, Some(&NONE)).is_empty(), "tcss is excluded");
        for (s, stack) in [
            ("lipgloss.Color(\"#ff0080\")", Stack::Charm),
            ("console.print(\"[#ff0080]x[/]\")", Stack::Textual),
            ("<Text color=\"#ff0080\">x</Text>", Stack::Ink),
        ] {
            assert_eq!(ids(&scan(s, stack, Some(&NONE))), vec!["tui-hardcoded-rgb-no-adapt"], "{s}");
        }
        assert!(scan("let c = Color::Red;", Stack::Ratatui, Some(&NONE)).is_empty());
    }
}
