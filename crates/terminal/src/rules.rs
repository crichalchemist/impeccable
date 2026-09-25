//! The runtime rules (spec section 4, PR 3 table): pure functions over the
//! frames of one target. Findings name the registry rows in
//! `crates/foundation` (`tui-rt-*`), all advisory.

use impeccable_core::color::contrast_ratio;
use impeccable_core::findings::{derive_advisory_flag, finding, Finding};
use serde_json::Value;

use crate::capture::{Color, Frame, FrameRole};
use crate::palette::{hex, Palette};

macro_rules! re {
    ($name:ident, $pat:expr) => {
        static $name: once_cell::sync::Lazy<regex::Regex> =
            once_cell::sync::Lazy::new(|| regex::Regex::new($pat).expect(stringify!($name)));
    };
}

/// WCAG AA for body text; the terminal has no large-text tier.
pub const CONTRAST_FLOOR: f64 = 4.5;
/// `adapt.terminal.md`'s Narrow class starts under 60 columns.
pub const NARROW_COLUMNS: usize = 60;
/// Fewer non-blank rows than this is a prompt, not a screen.
pub const MIN_ROWS_FOR_HINTS: usize = 3;

pub(crate) const TOP_LEFT: &str = "┌╭╔┏";
pub(crate) const TOP_RIGHT: &str = "┐╮╗┓";
pub(crate) const BOTTOM_LEFT: &str = "└╰╚┗";
pub(crate) const BOTTOM_RIGHT: &str = "┘╯╝┛";
pub(crate) const VERTICAL: &str = "│║┃┆┇┊┋╎╏";
pub(crate) const TEE: &str = "├┤┬┴┼╠╣╦╩╬┣┫┳┻╋╟╢╤╧╪";
/// Glyphs a spinner cycles through, beyond the braille block U+2800 to U+28FF.
const SPINNER_GLYPHS: &str = "|/-\\◐◓◑◒◴◵◶◷◜◝◞◟▁▂▃▄▅▆▇█▏▎▍▌▋▊▉";

re!(
    KEY_HINT_RE,
    r"(?i)(?:^|[\s:|/,(\[<])(?:q|esc|enter|tab|space|ctrl|alt|shift|f\d{1,2}|[hjkl])(?:$|[\s:|/,)\]>])|[?↑↓←→⏎⌃⌘]|<[^>\s]{1,12}>|\[[^\]\s]{1,12}\]|\^[A-Z]"
);

/// Every runtime rule over the frames of one target, in the frame policy
/// spec section 5 fixes: size-invariant rules on the first frame, the spinner
/// against the recapture, geometry on every capture frame.
pub fn scan_frames(frames: &[Frame], palette: Palette, file: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    let Some(first) = frames.first() else { return out };
    out.extend(rt_low_contrast(first, palette, file));
    out.extend(rt_truecolor_on_256(first, file));
    out.extend(rt_no_key_hints(first, file));
    if let Some(second) = frames.iter().find(|f| f.role == FrameRole::Recapture) {
        out.extend(rt_spinner_never_rests(first, second, file));
    }
    for frame in frames.iter().filter(|f| f.role == FrameRole::Capture) {
        out.extend(rt_nested_borders(frame, file));
        out.extend(rt_width_drift(frame, file));
        if frame.width < NARROW_COLUMNS {
            out.extend(rt_collapse_narrow(frame, file));
        }
    }
    for f in &mut out {
        derive_advisory_flag(f);
    }
    out
}

/// A finding at a cell: `line` is the 1-based row, `column` the 1-based
/// cell, `frame` the size it was seen at.
pub(crate) fn rt_finding(id: &str, file: &str, frame: &Frame, row: usize, col: usize, snippet: String) -> Finding {
    let mut f = finding(id, file, &snippet, (row + 1) as f64);
    f.extras.insert("column".into(), Value::from(col + 1));
    f.extras.insert("frame".into(), Value::String(format!("{}x{}", frame.width, frame.height)));
    f
}

fn is_text(ch: char) -> bool {
    ch.is_alphanumeric()
}

fn is_spinner(ch: char) -> bool {
    SPINNER_GLYPHS.contains(ch) || ('\u{2800}'..='\u{28FF}').contains(&ch)
}

pub fn rt_low_contrast(frame: &Frame, palette: Palette, file: &str) -> Vec<Finding> {
    // One finding per distinct (fg, bg) pair, at the first text cell that fails.
    let mut seen: Vec<(String, String)> = Vec::new();
    let mut out = Vec::new();
    for (r, row) in frame.rows.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            let Some(ch) = cell.glyph else { continue };
            if !is_text(ch) {
                continue;
            }
            let (fg, bg) = palette.resolve(&cell.style);
            let ratio = contrast_ratio(&fg, &bg);
            if ratio >= CONTRAST_FLOOR {
                continue;
            }
            let key = (hex(&fg), hex(&bg));
            if seen.contains(&key) {
                continue;
            }
            seen.push(key.clone());
            let run: String = row[c..]
                .iter()
                .take_while(|x| x.style == cell.style)
                .filter_map(|x| x.glyph)
                .take(24)
                .collect();
            let count = frame
                .rows
                .iter()
                .flatten()
                .filter(|x| x.glyph.map(is_text).unwrap_or(false) && palette.resolve(&x.style) == (fg, bg))
                .count();
            let snippet = format!("\"{run}\" {} on {}: {ratio:.1}:1 (need 4.5:1)", key.0, key.1);
            let mut f = rt_finding("tui-rt-low-contrast", file, frame, r, c, snippet);
            f.extras.insert("cellCount".into(), Value::from(count));
            out.push(f);
        }
    }
    out
}

pub fn rt_truecolor_on_256(frame: &Frame, file: &str) -> Vec<Finding> {
    if frame.termfeatures.is_empty() {
        return Vec::new(); // no client reported its features
    }
    let client_has_rgb = frame.termfeatures.split(',').any(|f| f.trim().eq_ignore_ascii_case("RGB"))
        || matches!(frame.colorterm.as_str(), "truecolor" | "24bit");
    if client_has_rgb {
        return Vec::new();
    }
    for (r, row) in frame.rows.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            let sgr = match (cell.style.fg, cell.style.bg) {
                (Color::Rgb(red, green, blue), _) => format!("38;2;{red};{green};{blue}"),
                (_, Color::Rgb(red, green, blue)) => format!("48;2;{red};{green};{blue}"),
                _ => continue,
            };
            let snippet = format!("truecolor SGR {sgr} while the client reports {}", frame.termfeatures);
            return vec![rt_finding("tui-rt-truecolor-on-256", file, frame, r, c, snippet)];
        }
    }
    Vec::new()
}

pub fn rt_no_key_hints(frame: &Frame, file: &str) -> Vec<Finding> {
    let non_blank: Vec<usize> = (0..frame.rows.len()).filter(|&r| !frame.is_blank(r)).collect();
    if non_blank.len() < MIN_ROWS_FOR_HINTS {
        return Vec::new();
    }
    let r = *non_blank.last().expect("checked above");
    let text = frame.text(r);
    if KEY_HINT_RE.is_match(&text) {
        return Vec::new();
    }
    let shown: String = text.trim().chars().take(60).collect();
    vec![rt_finding("tui-rt-no-key-hints", file, frame, r, 0, format!("bottom row \"{shown}\" names no key"))]
}

pub fn rt_spinner_never_rests(first: &Frame, second: &Frame, file: &str) -> Vec<Finding> {
    for (r, row) in first.rows.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            let (Some(a), Some(b)) = (cell.glyph, second.glyph(r, c)) else { continue };
            if a != b && is_spinner(a) && is_spinner(b) {
                let snippet = format!("spinner glyph {a} became {b} after one second with no input");
                return vec![rt_finding("tui-rt-spinner-never-rests", file, first, r, c, snippet)];
            }
        }
    }
    Vec::new()
}

// Task 6 replaces these three bodies.
pub fn rt_nested_borders(_frame: &Frame, _file: &str) -> Vec<Finding> {
    Vec::new()
}

pub fn rt_width_drift(_frame: &Frame, _file: &str) -> Vec<Finding> {
    Vec::new()
}

pub fn rt_collapse_narrow(_frame: &Frame, _file: &str) -> Vec<Finding> {
    Vec::new()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::capture::parse_rows;

    pub(crate) fn frame(width: usize, rows: &[&str]) -> Frame {
        Frame { width, height: rows.len(), rows: parse_rows(&rows.join("\n")), ..Frame::default() }
    }

    pub(crate) fn ids(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|f| f.antipattern.as_str()).collect()
    }

    pub(crate) const HINTS: &str = " q quit  ? help";

    #[test]
    fn gray_on_gray_text_is_low_contrast_and_borders_never_count() {
        let gray = "\x1b[38;2;120;120;120;48;2;100;100;100m";
        let f = frame(40, &[
            "┌──────┐",
            &format!("│ {gray}faint\x1b[0m │"),
            &format!("{gray}└──────┘\x1b[0m"),
            HINTS,
        ]);
        let out = rt_low_contrast(&f, Palette::Dark, "x");
        assert_eq!(out.len(), 1, "{:?}", ids(&out));
        assert_eq!(out[0].line, 2.0);
        assert_eq!(out[0].extras["column"], Value::from(3));
        assert_eq!(out[0].extras["cellCount"], Value::from(5));
        assert_eq!(out[0].extras["frame"], Value::from("40x4"));
        assert_eq!(out[0].snippet, "\"faint\" #787878 on #646464: 1.3:1 (need 4.5:1)");
    }

    #[test]
    fn black_text_fails_on_the_dark_default_and_passes_on_light() {
        let f = frame(40, &["title", "\x1b[30mblack on default\x1b[0m", HINTS]);
        let dark = rt_low_contrast(&f, Palette::Dark, "x");
        assert_eq!(dark.len(), 1);
        assert_eq!(dark[0].snippet, "\"black on default\" #000000 on #000000: 1.0:1 (need 4.5:1)");
        assert!(rt_low_contrast(&f, Palette::Light, "x").is_empty());
    }

    #[test]
    fn each_failing_pair_reports_once() {
        let f = frame(40, &["\x1b[30ma\x1b[0m \x1b[30mb\x1b[0m", "\x1b[30mc\x1b[0m", HINTS]);
        let out = rt_low_contrast(&f, Palette::Dark, "x");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].extras["cellCount"], Value::from(3));
    }

    #[test]
    fn truecolor_fires_only_when_the_client_is_known_to_lack_rgb() {
        let mut f = frame(40, &["\x1b[38;2;200;100;50mwarm\x1b[0m", "body", HINTS]);
        assert!(rt_truecolor_on_256(&f, "x").is_empty(), "unknown client stays silent");
        f.termfeatures = "256".into();
        let out = rt_truecolor_on_256(&f, "x");
        assert_eq!(out.len(), 1);
        assert_eq!((out[0].line, out[0].extras["column"].clone()), (1.0, Value::from(1)));
        assert_eq!(out[0].snippet, "truecolor SGR 38;2;200;100;50 while the client reports 256");
        f.termfeatures = "256,RGB".into();
        assert!(rt_truecolor_on_256(&f, "x").is_empty());
        f.termfeatures = "256".into();
        f.colorterm = "truecolor".into();
        assert!(rt_truecolor_on_256(&f, "x").is_empty(), "COLORTERM vouches for the outer terminal");
    }

    #[test]
    fn bottom_row_without_a_key_is_flagged_and_hints_or_prompts_pass() {
        let bare = frame(40, &["┌──┐", "│ab│", "└──┘", "Loading complete."]);
        let out = rt_no_key_hints(&bare, "x");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].line, 4.0);
        assert_eq!(out[0].snippet, "bottom row \"Loading complete.\" names no key");
        for hints in [" q quit  ? help", "[Enter] open  <Esc> back", "^C exit", "j/k move", "Press F1 for help"] {
            assert!(rt_no_key_hints(&frame(40, &["┌──┐", "│ab│", hints]), "x").is_empty(), "{hints}");
        }
        assert!(rt_no_key_hints(&frame(40, &["$ ", "Loading complete."]), "x").is_empty(), "a prompt is not a screen");
        assert!(rt_no_key_hints(&frame(40, &["┌──┐", "│ab│", "└──┘", HINTS, "", ""]), "x").is_empty(), "trailing blank rows are skipped");
    }

    #[test]
    fn spinner_cycling_across_the_recapture_is_flagged_once() {
        let a = frame(40, &["working ⠋  ⠙", "│ x │", HINTS]);
        let mut b = frame(40, &["working ⠙  ⠹", "│ x │", HINTS]);
        b.role = FrameRole::Recapture;
        let out = rt_spinner_never_rests(&a, &b, "x");
        assert_eq!(out.len(), 1, "one finding per pair, at the first cycling cell");
        assert_eq!((out[0].line, out[0].extras["column"].clone()), (1.0, Value::from(9)));
        assert_eq!(out[0].snippet, "spinner glyph ⠋ became ⠙ after one second with no input");
        assert!(rt_spinner_never_rests(&a, &a, "x").is_empty(), "a resting glyph is fine");
        let c = frame(40, &["working ✓  ✓", "│ x │", HINTS]);
        assert!(rt_spinner_never_rests(&a, &c, "x").is_empty(), "a spinner that resolved is fine");
    }
}
