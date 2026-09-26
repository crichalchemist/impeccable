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
    /// Go: any charmbracelet import (Bubble Tea, Lip Gloss, Bubbles), including
    /// the v2 `charm.land/...` module paths.
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
        ".go" if text.contains("charmbracelet") || text.contains("charm.land/") => Some(Stack::Charm),
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
    r#"--(?:no-)?icons\b|--ascii\b|["'](?:--)?(?:no-)?(?:icons|ascii)["']|NERD_FONT|nerd_font|nerdFont|\bascii[ \t]*:[ \t]*bool\b"#
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

/// A Rust raw string opener (`r"`, `r#"`, `br##"`, ...) at `i`, not preceded
/// by an identifier character. Returns the hash count and the opener length.
fn rust_raw_string_opener(c: &[char], i: usize) -> Option<(usize, usize)> {
    if i > 0 && (c[i - 1].is_alphanumeric() || c[i - 1] == '_') {
        return None;
    }
    let mut j = i;
    if c.get(j) == Some(&'b') {
        j += 1;
    }
    if c.get(j) != Some(&'r') {
        return None;
    }
    j += 1;
    let hashes = c[j..].iter().take_while(|ch| **ch == '#').count();
    j += hashes;
    (c.get(j) == Some(&'"')).then_some((hashes, j + 1 - i))
}

/// Blank `//` and `/* */` comments in Rust, Go, or Textual CSS, keeping every
/// newline so line numbers hold. String literals (`"..."`, Rust raw strings,
/// Go backtick raw strings) and Rust char literals are copied verbatim, so
/// `"http://x"` and `'/'` survive.
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
        } else if let Some((hashes, opener)) = rust_raw_string_opener(&c, i) {
            // `r#"..."#` and `br"..."`: no escapes, and the literal ends only
            // at a quote followed by the same number of hashes.
            out.extend(&c[i..i + opener]);
            i += opener;
            while i < n {
                if c[i] == '"' && (1..=hashes).all(|h| i + h < n && c[i + h] == '#') {
                    out.extend(&c[i..=i + hashes]);
                    i += hashes + 1;
                    break;
                }
                out.push(c[i]);
                i += 1;
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

/// U+2600 to U+27BF codepoints with the Unicode Emoji_Presentation property
/// (emoji-data.txt, Unicode 16.0.0; the subset is unchanged in 17.0.0).
/// The rest of that range (✓ ✗ ★ ☐ ➜) is a one-cell text glyph by default.
fn has_emoji_presentation(c: char) -> bool {
    matches!(
        c,
        '\u{2614}'..='\u{2615}'
            | '\u{2648}'..='\u{2653}'
            | '\u{267F}'
            | '\u{2693}'
            | '\u{26A1}'
            | '\u{26AA}'..='\u{26AB}'
            | '\u{26BD}'..='\u{26BE}'
            | '\u{26C4}'..='\u{26C5}'
            | '\u{26CE}'
            | '\u{26D4}'
            | '\u{26EA}'
            | '\u{26F2}'..='\u{26F3}'
            | '\u{26F5}'
            | '\u{26FA}'
            | '\u{26FD}'
            | '\u{2705}'
            | '\u{270A}'..='\u{270B}'
            | '\u{2728}'
            | '\u{274C}'
            | '\u{274E}'
            | '\u{2753}'..='\u{2755}'
            | '\u{2757}'
            | '\u{2795}'..='\u{2797}'
            | '\u{27B0}'
            | '\u{27BF}'
    )
}

/// Spec section 7, PR 5: U+1F300 to U+1FAFF always count. A codepoint in
/// U+2600 to U+27BF counts only with default emoji presentation, a following
/// U+FE0F, or membership in the canonical set.
fn counts_as_emoji(c: char, next: Option<char>) -> bool {
    match c {
        '\u{1F300}'..='\u{1FAFF}' => true,
        '\u{2600}'..='\u{27BF}' => {
            has_emoji_presentation(c) || next == Some('\u{FE0F}') || CANONICAL_EMOJI.contains(&c)
        }
        _ => false,
    }
}

fn scan_terminal_emoji_density(lines: &[&str], file_path: &str) -> Vec<Finding> {
    let mut total = 0usize;
    let mut canonical = 0usize;
    let mut first: Option<usize> = None;
    for (i, l) in lines.iter().enumerate() {
        let chars: Vec<char> = l.chars().collect();
        for (k, &c) in chars.iter().enumerate() {
            if !counts_as_emoji(c, chars.get(k + 1).copied()) {
                continue;
            }
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
    r##"Color::Rgb\((?:[ \t]*(?:0x[0-9a-fA-F]+|\d+)[ \t]*,){2}[ \t]*(?:0x[0-9a-fA-F]+|\d+)[ \t]*\)|Color::from_u32\(0x|lipgloss\.Color\("#|\bchalk(?:\.[A-Za-z]+)*\.(?:hex|bgHex)\([ \t]*["']#|\bchalk(?:\.[A-Za-z]+)*\.(?:rgb|bgRgb)\([ \t]*\d|\bcolor=["']#|\[#[0-9a-fA-F]{6}\]|\\x1b\[38;2;|\\033\[38;2;|\x1b\[38;2;"##
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

// ─── tui-spinner-no-tty-guard ───────────────────────────────────────────────
// Only Bubbles' spinner: ora, Rich, briandowns/spinner, and Ink's renderer
// all stop animating off a TTY on their own. Matches the github.com and the
// charm.land v2 module paths.
re!(SPINNER_RE, r"bubbles/(?:v\d+/)?spinner");

fn scan_terminal_spinner(lines: &[&str], file_path: &str, signals: Option<&ProjectSignals>) -> Vec<Finding> {
    if matches!(signals, Some(s) if s.has_tty_guard) {
        return Vec::new();
    }
    lines
        .iter()
        .enumerate()
        .filter(|(_, l)| SPINNER_RE.is_match(l))
        .map(|(i, _)| {
            let f = hit("tui-spinner-no-tty-guard", file_path, lines, i);
            if signals.is_none() { with_note(f, SIGNALS_NOTE) } else { f }
        })
        .collect()
}

// ─── tui-hardcoded-size ─────────────────────────────────────────────────────
re!(
    RECT_LITERAL_RE,
    r"Rect::new\([ \t]*\d+[ \t]*,[ \t]*\d+[ \t]*,[ \t]*\d+[ \t]*,[ \t]*\d+[ \t]*\)"
);
re!(SIZE_80_RE, r"(?i)\b(?:width|cols|columns)[ \t]*[:=][ \t]*80\b");
re!(SIZE_24_RE, r"(?i)\b(?:height|rows|lines)[ \t]*[:=][ \t]*24\b");
re!(SIZE_CALL_RE, r"\.Width\(80\)|width=\{80\}|size=\(80,[ \t]*24\)");
re!(CONSTRAINT_LENGTH_RE, r"Constraint::Length\(");
re!(CONSTRAINT_FLEX_RE, r"Constraint::(?:Min|Max|Percentage|Ratio|Fill)\(");
// Spec section 7, PR 5 (a): a file that measures the terminal holds its 80
// or 24 as the fallback for when the measurement fails.
re!(
    MEASURES_TERMINAL_RE,
    r"GetSize\(|terminal::size\(|terminal_size\(|get_terminal_size\(|stdout\.columns|stdout\.rows|console\.(?:width|size)\b"
);
// (c): in a file that uses ratatui's canvas, a literal Rect is a drawing
// coordinate, not the terminal.
re!(CANVAS_RE, r"widgets::canvas\b");
re!(SIZE_LITERAL_RE, r"\b(?:80|24)\b");
re!(LAYOUT_CALL_RE, r"Layout::(?:vertical|horizontal|new)\(|\.constraints\(");

/// The byte span of each layout call's argument list, from the opener's `(`
/// to just past its matching `)`, or to the end of the text when the call is
/// unbalanced. Only parens are counted; comments are already blanked and a
/// paren inside a string literal is not special-cased. An opener that starts
/// inside an earlier span belongs to that call.
fn layout_call_spans(text: &str) -> Vec<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut spans = Vec::new();
    let mut covered = 0usize;
    for m in LAYOUT_CALL_RE.find_iter(text) {
        if m.start() < covered {
            continue;
        }
        let open = m.end() - 1; // every alternative ends on the `(`
        let mut depth = 0usize;
        let mut end = text.len();
        for (k, &b) in bytes.iter().enumerate().skip(open) {
            match b {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = k + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        covered = end;
        spans.push((open, end));
    }
    spans
}

/// Test code pins sizes on purpose (spec: "outside paths containing test").
/// Only the file name and its parent directory are consulted, so a fixture
/// tree under `tests/fixtures/` still scans (planning ruling 7). Both are
/// matched whole, not as substrings, so `src/latest.rs` still scans.
fn in_test_path(file_path: &str) -> bool {
    let lower = file_path.to_lowercase().replace('\\', "/");
    let mut segs = lower.rsplit('/');
    let name = segs.next().unwrap_or("");
    let parent = segs.next().unwrap_or("");
    let stem = name.rsplit_once('.').map_or(name, |(s, _)| s);
    matches!(parent, "test" | "tests" | "__tests__" | "spec")
        || name.starts_with("test_")
        || stem.ends_with("_test")
        || name.contains(".test.")
        || name.contains(".spec.")
}

fn scan_terminal_hardcoded_size(lines: &[&str], file_path: &str) -> Vec<Finding> {
    if in_test_path(file_path) {
        return Vec::new();
    }
    let text = lines.join("\n");
    let measures = MEASURES_TERMINAL_RE.is_match(&text);
    let canvas = CANVAS_RE.is_match(&text);
    let mut out: Vec<Finding> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| {
            let rect = RECT_LITERAL_RE
                .find(l)
                .is_some_and(|m| !canvas && !(measures && SIZE_LITERAL_RE.is_match(m.as_str())));
            let literal =
                !measures && (SIZE_80_RE.is_match(l) || SIZE_24_RE.is_match(l) || SIZE_CALL_RE.is_match(l));
            rect || literal
        })
        .map(|(i, _)| hit("tui-hardcoded-size", file_path, lines, i))
        .collect();
    // (b) Constraints are counted per layout call, so the two Lengths of a
    // centered box or two one-Length layouts are not an all-Length layout.
    for (start, end) in layout_call_spans(&text) {
        let call = &text[start..end];
        let lengths: Vec<usize> = CONSTRAINT_LENGTH_RE.find_iter(call).map(|m| start + m.start()).collect();
        if lengths.len() >= 2 && !CONSTRAINT_FLEX_RE.is_match(call) {
            let line = text[..lengths[0]].matches('\n').count();
            out.push(with_count(hit("tui-hardcoded-size", file_path, lines, line), "lengthConstraints", lengths.len()));
        }
    }
    out.sort_by(|a, b| a.line.partial_cmp(&b.line).unwrap_or(std::cmp::Ordering::Equal));
    out
}

// ─── tui-grapheme-unsafe-truncate ───────────────────────────────────────────
re!(RUST_TRUNC_RE, r"&\w+\[\.\.[ \t]*\w+\]|\.chars\(\)\.take\(");
re!(GO_TRUNC_RE, r"\b\w+\[:\w+\]");
re!(JS_TRUNC_RE, r"\.slice\(0,|\.substring\(0,");
re!(ELLIPSIS_RE, r#""[^"\n]*\.\.\.[^"\n]*"|'[^'\n]*\.\.\.[^'\n]*'|`[^`\n]*\.\.\.[^`\n]*`|…|\\u\{2026\}|\\u2026"#);

/// Python is not matched: every `.py` file this module sees imports rich or
/// textual (framework detection), and both measure width for the caller.
fn scan_terminal_grapheme_truncate(
    lines: &[&str],
    file_path: &str,
    stack: Stack,
    signals: Option<&ProjectSignals>,
) -> Vec<Finding> {
    if matches!(signals, Some(s) if s.has_width_lib) {
        return Vec::new();
    }
    let re: &Regex = match stack {
        Stack::Ratatui => &RUST_TRUNC_RE,
        Stack::Charm => &GO_TRUNC_RE,
        Stack::Ink => &JS_TRUNC_RE,
        Stack::Textual | Stack::TextualCss => return Vec::new(),
    };
    let n = lines.len();
    lines
        .iter()
        .enumerate()
        .filter(|(i, l)| re.is_match(l) && (*i..(*i + 3).min(n)).any(|k| ELLIPSIS_RE.is_match(lines[k])))
        .map(|(i, _)| {
            let f = hit("tui-grapheme-unsafe-truncate", file_path, lines, i);
            if signals.is_none() { with_note(f, SIGNALS_NOTE) } else { f }
        })
        .collect()
}

// ─── tui-nerd-glyph-no-fallback ─────────────────────────────────────────────
fn is_private_use(c: char) -> bool {
    matches!(c, '\u{E000}'..='\u{F8FF}' | '\u{F0000}'..='\u{FFFFD}')
}

fn scan_terminal_nerd_glyph(lines: &[&str], file_path: &str, signals: Option<&ProjectSignals>) -> Vec<Finding> {
    if matches!(signals, Some(s) if s.has_icon_toggle) {
        return Vec::new();
    }
    let mut count = 0usize;
    let mut first: Option<usize> = None;
    for (i, l) in lines.iter().enumerate() {
        let here = l.chars().filter(|c| is_private_use(*c)).count();
        if here > 0 {
            count += here;
            first.get_or_insert(i);
        }
    }
    match first {
        Some(i) => {
            let f = with_count(hit("tui-nerd-glyph-no-fallback", file_path, lines, i), "glyphCount", count);
            vec![if signals.is_none() { with_note(f, SIGNALS_NOTE) } else { f }]
        }
        None => Vec::new(),
    }
}

// ─── tui-print-in-loop ──────────────────────────────────────────────────────
re!(INK_CONSOLE_LOG_RE, r"\bconsole\.log\(");
re!(PATCH_CONSOLE_OFF_RE, r"\bpatchConsole[ \t]*:[ \t]*false\b");

/// Ink only. Textual captures `print` while an app runs (Textualize/textual#2952,
/// `App._print`), so a Python file is never reported.
fn scan_terminal_print_in_loop(lines: &[&str], file_path: &str, stack: Stack) -> Vec<Finding> {
    // Ink defaults to `patchConsole: true`, which captures console output and
    // re-renders it above the frame. Only a file that turns that off can
    // corrupt the frame with `console.log`.
    if stack != Stack::Ink || !lines.iter().any(|l| PATCH_CONSOLE_OFF_RE.is_match(l)) {
        return Vec::new();
    }
    lines
        .iter()
        .enumerate()
        .filter(|(_, l)| INK_CONSOLE_LOG_RE.is_match(l))
        .map(|(i, _)| hit("tui-print-in-loop", file_path, lines, i))
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
    out.extend(scan_terminal_spinner(&lines, file_path, signals));
    out.extend(scan_terminal_hardcoded_size(&lines, file_path));
    out.extend(scan_terminal_grapheme_truncate(&lines, file_path, stack, signals));
    out.extend(scan_terminal_nerd_glyph(&lines, file_path, signals));
    out.extend(scan_terminal_print_in_loop(&lines, file_path, stack));
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
    fn a_glyph_table_named_icons_is_not_an_icon_fallback() {
        let table = "var icons = map[string]string{\n\t\"branch\": \"\u{e0a0}\",\n}\n";
        let signals = ProjectSignals::collect([table]);
        assert!(!signals.has_icon_toggle, "the glyph table itself is not a fallback");
        assert_eq!(ids(&scan(table, Stack::Charm, Some(&signals))), vec!["tui-nerd-glyph-no-fallback"]);
        for bare in ["icons = {\"x\": \"\u{e0a0}\"}", "icons := []string{}", "const icons = {};"] {
            assert!(!ProjectSignals::collect([bare]).has_icon_toggle, "{bare}");
        }
        let flag = "flag.Bool(\"no-icons\", false, \"--no-icons: plain ASCII glyphs\")\n";
        let signals = ProjectSignals::collect([table, flag]);
        assert!(signals.has_icon_toggle, "a --no-icons flag definition is a fallback");
        assert!(scan(table, Stack::Charm, Some(&signals)).is_empty());
        for option in ["--icons", "{\"icons\": true}", "cfg.get('icons')", "NERD_FONT=1", "nerd_font = True", "nerdFont: false", "--ascii", "ascii: bool,"] {
            assert!(ProjectSignals::collect([option]).has_icon_toggle, "{option}");
        }
    }

    #[test]
    fn rust_raw_strings_do_not_flip_comment_blanking() {
        let src = "let s = r#\"a \"quote\"#;\n// let c = BorderType::Double;\n";
        let out = strip_c_comments(src);
        assert!(out.contains("r#\"a \"quote\"#"), "{out}");
        assert!(!out.contains("BorderType::Double"), "{out}");
        assert!(scan(&out, Stack::Ratatui, Some(&ALL)).is_empty(), "the commented-out border is not reported");
        let bytes = "let b = br##\"x\"# \"##;\n// Color::Rgb(1, 2, 3)\n";
        assert!(!strip_c_comments(bytes).contains("Color::Rgb"));
        let plain = "let p = r\"C:\\\";\n// BorderType::Double\n";
        assert!(!strip_c_comments(plain).contains("BorderType::Double"), "a raw string's backslash escapes nothing");
    }

    #[test]
    fn test_paths_are_whole_segments_or_affixes() {
        for exempt in ["src/tests/layout.rs", "src/layout_test.rs", "tests/ui.py", "ui.test.tsx", "src/__tests__/ui.tsx", "spec/ui.go", "src/test_ui.py", "ui.spec.ts", "test/ui.rs"] {
            assert!(in_test_path(exempt), "{exempt}");
        }
        for scanned in ["src/latest.rs", "src/contest/ui.rs", "src/attestation.go", "src/testing_ui.rs", "tests/fixtures/antipatterns/terminal/x/x.rs"] {
            assert!(!in_test_path(scanned), "{scanned}");
        }
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
    fn text_presentation_dingbats_do_not_count_as_emoji() {
        // Eight glyphs from U+2600 to U+27BF without Emoji_Presentation:
        // ordinary one-cell TUI marks, not emoji.
        let marks = "let m = [\"✓\", \"✗\", \"★\", \"☐\", \"➜\", \"☆\", \"✔\", \"✘\"];\n";
        assert!(scan(marks, Stack::Ratatui, Some(&ALL)).is_empty());
        // Four with Emoji_Presentation, and four made emoji by U+FE0F.
        let shown = "let m = [\"☔\", \"☕\", \"⚡\", \"⛔\", \"❤\u{FE0F}\", \"☀\u{FE0F}\", \"✔\u{FE0F}\", \"✓\u{FE0F}\"];\n";
        let f = scan(shown, Stack::Ratatui, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-emoji-density"]);
        assert_eq!(f[0].extras.get("emojiCount"), Some(&Value::from(8u64)));
    }

    #[test]
    fn canonical_warning_sign_counts_without_a_variation_selector() {
        // ⚠ (U+26A0) lacks Emoji_Presentation but is in the canonical set.
        let src = "print(\"⚠ retry\")\nprint(\"✅ done\")\nprint(\"❌ failed\")\n";
        assert_eq!(ids(&scan(src, Stack::Textual, Some(&ALL))), vec!["tui-emoji-density"]);
    }

    #[test]
    fn eight_emoji_reach_the_threshold_and_seven_do_not() {
        let seven = "let s = \"🌀 🌁 🌂 🌃 🌄 🌅 🌆\";\n";
        assert!(scan(seven, Stack::Ratatui, Some(&ALL)).is_empty());
        let eight = "let s = \"🌀 🌁 🌂 🌃 🌄 🌅 🌆 🌇\";\n";
        let f = scan(eight, Stack::Ratatui, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-emoji-density"]);
        assert_eq!(f[0].extras.get("emojiCount"), Some(&Value::from(8u64)));
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

    #[test]
    fn chalk_is_flagged_only_for_a_literal_color() {
        for src in [
            "chalk.hex('#ff00aa')('x')",
            "chalk.bold.hex(\"#ff00aa\")('x')",
            "chalk.bgHex('#101010')('x')",
            "chalk.rgb(255, 0, 170)('x')",
            "chalk.underline.bgRgb(16, 16, 16)('x')",
        ] {
            assert_eq!(ids(&scan(src, Stack::Ink, Some(&NONE))), vec!["tui-hardcoded-rgb-no-adapt"], "{src}");
        }
        for src in [
            "chalk.hex(theme.primary)('x')",
            "chalk.bgHex(color)('x')",
            "chalk.rgb(r, g, b)('x')",
            "chalk.hex(`#${hex}`)('x')",
        ] {
            assert!(scan(src, Stack::Ink, Some(&NONE)).is_empty(), "a computed argument is not a hardcoded color: {src}");
        }
    }

    #[test]
    fn spinner_without_tty_guard_is_flagged_and_a_guard_silences_it() {
        let src = "import \"github.com/charmbracelet/bubbles/spinner\"\n";
        assert_eq!(ids(&scan(src, Stack::Charm, Some(&NONE))), vec!["tui-spinner-no-tty-guard"]);
        assert!(scan(src, Stack::Charm, Some(&ALL)).is_empty());
        assert!(scan("s := spinner.New()\n", Stack::Charm, Some(&NONE)).is_empty(), "the import path is what counts");
        let v2 = "import \"github.com/charmbracelet/bubbles/v2/spinner\"\n";
        assert_eq!(ids(&scan(v2, Stack::Charm, Some(&NONE))), vec!["tui-spinner-no-tty-guard"]);
    }

    #[test]
    fn self_guarding_spinner_libraries_are_not_flagged() {
        // Each of these stops animating when output is not a terminal:
        // ora (isEnabled), Rich (Live refreshes only on a terminal console),
        // briandowns/spinner (Start is a no-op off a terminal), and Ink's
        // renderer (non-interactive in CI or without a TTY).
        for (src, stack) in [
            ("import Spinner from 'ink-spinner';\n", Stack::Ink),
            ("const s = ora('Loading').start();\n", Stack::Ink),
            ("from rich.spinner import Spinner\n", Stack::Textual),
            ("import \"github.com/briandowns/spinner\"\n", Stack::Charm),
        ] {
            assert!(scan(src, stack, Some(&NONE)).is_empty(), "{src}");
        }
    }

    #[test]
    fn hardcoded_terminal_size_is_flagged_outside_test_paths() {
        let src = "let area = Rect::new(0, 0, 80, 24);\nlet width = 80;\nlet rows = 24;\nlet w = term.Width(80)\n";
        let f = scan_terminal_source(src, "src/ui.rs", Stack::Ratatui, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-hardcoded-size"; 4]);
        assert!(scan_terminal_source(src, "tests/ui_test.rs", Stack::Ratatui, Some(&ALL)).is_empty());
        assert!(scan_terminal_source(src, "src/ui_test.rs", Stack::Ratatui, Some(&ALL)).is_empty());
        assert_eq!(scan_terminal_source(src, "tests/fixtures/antipatterns/terminal/x/x.rs", Stack::Ratatui, Some(&ALL)).len(), 4, "a fixture tree under tests/ still scans");
        assert!(scan("let width = 800;\nlet rows = 240;\n", Stack::Ratatui, Some(&ALL)).is_empty());
    }

    #[test]
    fn all_length_layout_is_flagged_once_and_a_flexible_constraint_passes() {
        let rigid = "Layout::vertical([\n    Constraint::Length(3),\n    Constraint::Length(10),\n    Constraint::Length(1),\n])\n";
        let f = scan(rigid, Stack::Ratatui, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-hardcoded-size"]);
        assert_eq!(f[0].line, 2.0);
        assert_eq!(f[0].extras.get("lengthConstraints"), Some(&Value::from(3u64)));
        let flexible = "Layout::vertical([\n    Constraint::Length(3),\n    Constraint::Min(0),\n    Constraint::Length(1),\n])\n";
        assert!(scan(flexible, Stack::Ratatui, Some(&ALL)).is_empty());
    }

    #[test]
    fn two_length_constraints_on_one_line_still_count_as_two() {
        let src = "fn rigid() -> Layout { Layout::vertical([Constraint::Length(3), Constraint::Length(10)]) }\n";
        let f = scan(src, Stack::Ratatui, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-hardcoded-size"]);
        assert_eq!(f[0].line, 1.0);
        assert_eq!(f[0].extras.get("lengthConstraints"), Some(&Value::from(2u64)));
    }

    #[test]
    fn one_length_per_layout_call_is_not_an_all_length_layout() {
        let src = "let top = Layout::vertical([Constraint::Length(1)]);\nlet side = Layout::horizontal([Constraint::Length(20)]);\n";
        assert!(scan(src, Stack::Ratatui, Some(&ALL)).is_empty());
        // The manual-pass false positive: two Lengths that size a centered box.
        let centered = "let r = area.centered(Constraint::Length(w), Constraint::Length(h));\n";
        assert!(scan(centered, Stack::Ratatui, Some(&ALL)).is_empty());
    }

    #[test]
    fn a_rigid_layout_fires_even_when_another_call_is_flexible() {
        let src = "let a = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]);\nlet b = Layout::default()\n    .direction(Direction::Horizontal)\n    .constraints([\n        Constraint::Length(10),\n        Constraint::Length(30),\n    ])\n    .split(area);\n";
        let f = scan(src, Stack::Ratatui, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-hardcoded-size"]);
        assert_eq!(f[0].line, 5.0);
        assert_eq!(f[0].extras.get("lengthConstraints"), Some(&Value::from(2u64)));
    }

    #[test]
    fn every_layout_opener_is_walked_to_its_close_paren() {
        let new = "let l = Layout::new(Direction::Vertical, [Constraint::Length(1), Constraint::Length(2)]);\n";
        assert_eq!(ids(&scan(new, Stack::Ratatui, Some(&ALL))), vec!["tui-hardcoded-size"]);
        // A nested .constraints( inside a Layout call is the same call, reported once.
        let nested = "let l = Layout::new(d, base.constraints([Constraint::Length(1), Constraint::Length(2)]));\n";
        assert_eq!(ids(&scan(nested, Stack::Ratatui, Some(&ALL))), vec!["tui-hardcoded-size"]);
        // An unclosed call runs to the end of the file.
        let open = "let l = Layout::vertical([\n    Constraint::Length(1),\n    Constraint::Length(2),\n";
        let f = scan(open, Stack::Ratatui, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-hardcoded-size"]);
        assert_eq!(f[0].line, 2.0);
    }

    #[test]
    fn a_rigid_layout_call_still_fires_in_a_file_that_measures_the_terminal() {
        // Exemption (a) silences a bare 80/24 literal, not the Length count.
        let src = "let (cols, _) = crossterm::terminal::size().unwrap_or((80, 24));\nlet l = Layout::vertical([Constraint::Length(3), Constraint::Length(10)]);\n";
        let f = scan(src, Stack::Ratatui, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-hardcoded-size"]);
        assert_eq!(f[0].extras.get("lengthConstraints"), Some(&Value::from(2u64)));
    }

    #[test]
    fn a_fallback_size_in_a_file_that_measures_the_terminal_is_not_flagged() {
        // glow's idiom: the literal is what the program uses when GetSize fails.
        let go = "w, _, err := term.GetSize(int(os.Stdout.Fd()))\nif err != nil {\n\twidth = 80\n}\nmsg := tea.WindowSizeMsg{Width: 80, Height: 24}\n";
        assert!(scan_terminal_source(go, "main.go", Stack::Charm, Some(&ALL)).is_empty());
        let rs = "let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));\nlet fallback = Rect::new(0, 0, 80, 24);\nlet width = 80;\n";
        assert!(scan(rs, Stack::Ratatui, Some(&ALL)).is_empty());
        let py = "cols = shutil.get_terminal_size().columns\nsize=(80, 24)\n";
        assert!(scan(py, Stack::Textual, Some(&ALL)).is_empty());
        let rich = "if console.width < 100:\n    columns = 80\n";
        assert!(scan(rich, Stack::Textual, Some(&ALL)).is_empty());
        let ink = "const cols = process.stdout.columns ?? 80;\nexport const A = () => <Box width={80} />;\n";
        assert!(scan(ink, Stack::Ink, Some(&ALL)).is_empty());
        // A rectangle that is not an 80x24 fallback still fires in a measuring file.
        let other = "let (c, r) = crossterm::terminal::size()?;\nlet r = Rect::new(10, 10, 200, 100);\n";
        assert_eq!(ids(&scan(other, Stack::Ratatui, Some(&ALL))), vec!["tui-hardcoded-size"]);
        // A measurement that is commented out exempts nothing: the scan sees blanked text.
        let blanked = strip_c_comments("// let (w, h) = crossterm::terminal::size()?;\nlet width = 80;\n");
        assert_eq!(ids(&scan(&blanked, Stack::Ratatui, Some(&ALL))), vec!["tui-hardcoded-size"]);
        // The canonical case neither measures nor draws on a canvas.
        assert_eq!(ids(&scan("fn area() -> Rect { Rect::new(0, 0, 80, 24) }\n", Stack::Ratatui, Some(&ALL))), vec!["tui-hardcoded-size"]);
    }

    #[test]
    fn a_literal_rect_in_a_canvas_file_is_a_drawing_coordinate() {
        let src = "use ratatui::widgets::canvas::{Canvas, Rectangle};\nlet r = Rect::new(10, 10, 200, 100);\nlet width = 80;\n";
        let f = scan(src, Stack::Ratatui, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-hardcoded-size"], "the canvas exemption covers Rect::new only");
        assert_eq!(f[0].line, 3.0);
    }

    #[test]
    fn truncation_before_an_ellipsis_is_flagged_without_a_width_library() {
        for (src, stack) in [
            ("let short = format!(\"{}...\", &title[..20]);\n", Stack::Ratatui),
            ("let s = title.chars().take(20).collect::<String>();\nlet t = format!(\"{s}…\");\n", Stack::Ratatui),
            ("short := title[:20] + \"...\"\n", Stack::Charm),
            ("const short = title.slice(0, 20) + '...';\n", Stack::Ink),
        ] {
            assert_eq!(ids(&scan(src, stack, Some(&NONE))), vec!["tui-grapheme-unsafe-truncate"], "{src}");
            assert!(scan(src, stack, Some(&ALL)).is_empty(), "a width library silences it: {src}");
        }
        assert!(scan("let head = &buf[..4];\n", Stack::Ratatui, Some(&NONE)).is_empty(), "no ellipsis, no truncation for display");
        assert!(scan("short = title[:20] + '...'\n", Stack::Textual, Some(&NONE)).is_empty(), "Rich and Textual measure for the caller");
    }

    #[test]
    fn nerd_glyph_without_an_icon_toggle_reports_once() {
        let src = "const ICON = \"\u{e0a0}\";\nconst FOLDER = \"\u{f07b}\";\n";
        let f = scan(src, Stack::Ink, Some(&NONE));
        assert_eq!(ids(&f), vec!["tui-nerd-glyph-no-fallback"]);
        assert_eq!(f[0].line, 1.0);
        assert_eq!(f[0].extras.get("glyphCount"), Some(&Value::from(2u64)));
        assert!(scan(src, Stack::Ink, Some(&ALL)).is_empty());
    }

    #[test]
    fn print_in_a_textual_app_is_not_flagged_because_textual_captures_it() {
        // Textual routes `print` to devtools or nowhere while an app runs
        // (Textualize/textual#2952, `App._print`), so it cannot corrupt the frame.
        let src = "from textual.app import App\nprint(\"module level\")\n\nclass Demo(App):\n    def on_mount(self) -> None:\n        print(\"captured by Textual\")\n\n    @network_retry\n    def fetch(self) -> None:\n        def inner() -> None:\n            print(\"nested\")\n        print(\"outer\")\n";
        assert!(scan(src, Stack::Textual, Some(&ALL)).is_empty());
    }

    #[test]
    fn console_log_in_an_ink_component_is_flagged_only_when_patch_console_is_off() {
        // Ink's default is `patchConsole: true`: console output is captured
        // and re-rendered above the frame, so it is safe unless turned off.
        let unpatched = "import { render, Text } from 'ink';\nconst App = () => {\n  console.log('debug');\n  return <Text>hi</Text>;\n};\nrender(<App/>, {patchConsole: false});\n";
        let f = scan(unpatched, Stack::Ink, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-print-in-loop"]);
        assert_eq!(f[0].line, 3.0);
        let default = "import { render, Text } from 'ink';\nconst App = () => {\n  console.log('debug');\n  return <Text>hi</Text>;\n};\nrender(<App/>);\n";
        assert!(scan(default, Stack::Ink, Some(&ALL)).is_empty());
    }

    #[test]
    fn single_file_scan_downgrades_absence_rules() {
        let src = "const c = <Text color=\"#ff0080\">{title.slice(0, 8) + '...'}</Text>;\nconst i = \"\u{e0a0}\";\n";
        let mut f = scan(src, Stack::Ink, None);
        f.extend(scan("import \"github.com/charmbracelet/bubbles/spinner\"\n", Stack::Charm, None));
        let noted: Vec<&str> = f.iter().filter(|f| f.extras.get("note") == Some(&Value::String(SIGNALS_NOTE.into()))).map(|f| f.antipattern.as_str()).collect();
        for id in ["tui-spinner-no-tty-guard", "tui-hardcoded-rgb-no-adapt", "tui-grapheme-unsafe-truncate", "tui-nerd-glyph-no-fallback"] {
            assert!(noted.contains(&id), "{id} should carry the note: {f:?}");
        }
        for f in &f {
            assert_eq!(f.severity, "advisory", "{}", f.antipattern);
        }
    }

    #[test]
    fn charm_v2_vanity_import_is_terminal_source() {
        assert_eq!(classify_terminal_source(".go", "import tea \"charm.land/bubbletea/v2\""), Some(Stack::Charm));
        assert_eq!(classify_terminal_source(".go", "import \"charm.land/lipgloss/v2\""), Some(Stack::Charm));
    }

    #[test]
    fn a_v2_bubbles_spinner_import_is_seen() {
        let src = "import \"charm.land/bubbles/v2/spinner\"\n";
        assert_eq!(ids(&scan(src, Stack::Charm, Some(&NONE))), vec!["tui-spinner-no-tty-guard"]);
    }

    #[test]
    fn computed_and_destructured_rgb_is_not_a_hardcoded_color() {
        for src in [
            "let Color::Rgb(r, g, b) = color else { return; };\n",
            "let c = Color::Rgb(color.red, color.green, color.blue);\n",
            "Color::Rgb(rand::random(), rand::random(), rand::random())\n",
            "let c = Color::Rgb(255, green, 0);\n",
        ] {
            assert!(scan(src, Stack::Ratatui, Some(&NONE)).is_empty(), "{src}");
        }
        assert_eq!(ids(&scan("const BG: Color = Color::Rgb( 20, 20, 50 );\n", Stack::Ratatui, Some(&NONE))), vec!["tui-hardcoded-rgb-no-adapt"]);
        assert_eq!(ids(&scan("const BG: Color = Color::Rgb(0x20, 0x30, 0x60);\n", Stack::Ratatui, Some(&NONE))), vec!["tui-hardcoded-rgb-no-adapt"], "hex channels are literals too");
    }

    #[test]
    fn go_variadic_and_js_spread_are_not_ellipses() {
        assert!(scan("ids := list[:5]\nres := append(dst, more...)\n", Stack::Charm, Some(&NONE)).is_empty());
        assert!(scan("const a = title.slice(0, 5);\nconst b = {...config};\n", Stack::Ink, Some(&NONE)).is_empty());
    }
}
