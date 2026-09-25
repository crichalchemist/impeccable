//! The runtime rules (spec section 4, PR 3 table): pure functions over the
//! frames of one target. Findings name the registry rows in
//! `crates/foundation` (`tui-rt-*`), all advisory.

use impeccable_core::color::contrast_ratio;
use impeccable_core::findings::{derive_advisory_flag, finding, Finding};
use serde_json::Value;

use crate::capture::{Color, Frame, FrameRole, Style};
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
re!(PAGER_PROMPT_RE, r"\(END\)|--More--|\blines \d+-\d+");

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

/// The terminal's default colors and the 16 named ones come from the user's
/// theme, so a ratio measured over them holds only for the `--palette` the
/// scan assumed (spec section 7, PR 4 item 8).
fn palette_relative(style: &Style) -> bool {
    let themed = |c: Color| matches!(c, Color::Default) || matches!(c, Color::Indexed(n) if n < 16);
    themed(style.fg) || themed(style.bg)
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
            // Truncate rather than round: a 4.478 ratio must print as "4.4:1"
            // rather than round up to a misleading "4.5:1" (the floor it just
            // missed).
            let shown = (ratio * 10.0).floor() / 10.0;
            let snippet = format!("\"{run}\" {} on {}: {shown:.1}:1 (need 4.5:1)", key.0, key.1);
            let mut f = rt_finding("tui-rt-low-contrast", file, frame, r, c, snippet);
            f.extras.insert("cellCount".into(), Value::from(count));
            f.extras.insert("paletteRelative".into(), Value::from(palette_relative(&cell.style)));
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

/// A pager's own prompt on the bottom row: exactly `:` once the trailing
/// spaces `capture-pane -J` keeps are trimmed, or a row containing `(END)`,
/// `--More--`, or `lines <n>-<m>`.
fn is_pager_prompt(text: &str) -> bool {
    text.trim_end() == ":" || PAGER_PROMPT_RE.is_match(text)
}

pub fn rt_no_key_hints(frame: &Frame, file: &str) -> Vec<Finding> {
    let non_blank: Vec<usize> = (0..frame.rows.len()).filter(|&r| !frame.is_blank(r)).collect();
    if non_blank.len() < MIN_ROWS_FOR_HINTS {
        return Vec::new();
    }
    // A key named on the first row (a help line, a table header) counts too.
    if KEY_HINT_RE.is_match(&frame.text(non_blank[0])) {
        return Vec::new();
    }
    let r = *non_blank.last().expect("checked above");
    let text = frame.text(r);
    if KEY_HINT_RE.is_match(&text) || is_pager_prompt(&text) {
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

/// A closed box-drawing rectangle, inclusive cell coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub top: usize,
    pub left: usize,
    pub bottom: usize,
    pub right: usize,
}

fn glyph_in(frame: &Frame, r: usize, c: usize, set: &str) -> bool {
    frame.glyph(r, c).map(|g| set.contains(g)).unwrap_or(false)
}

fn is_edge_vertical(frame: &Frame, r: usize, c: usize) -> bool {
    glyph_in(frame, r, c, VERTICAL) || glyph_in(frame, r, c, TEE)
}

/// Every closed rectangle: a top-left corner, horizontals or a title to the
/// first top-right corner, verticals (or tees) down both sides to matching
/// bottom corners.
pub fn find_boxes(frame: &Frame) -> Vec<Rect> {
    let mut out = Vec::new();
    for (top, row) in frame.rows.iter().enumerate() {
        for (left, cell) in row.iter().enumerate() {
            if !cell.glyph.map(|g| TOP_LEFT.contains(g)).unwrap_or(false) {
                continue;
            }
            let Some(right) = (left + 1..row.len()).find(|&c| glyph_in(frame, top, c, TOP_RIGHT)) else { continue };
            let top_edge_broken = (left + 1..right).any(|c| {
                glyph_in(frame, top, c, VERTICAL)
                    || glyph_in(frame, top, c, TOP_LEFT)
                    || glyph_in(frame, top, c, BOTTOM_LEFT)
                    || glyph_in(frame, top, c, BOTTOM_RIGHT)
            });
            if top_edge_broken {
                continue;
            }
            let Some(bottom) = (top + 1..frame.rows.len()).find(|&r| glyph_in(frame, r, left, BOTTOM_LEFT)) else { continue };
            if !glyph_in(frame, bottom, right, BOTTOM_RIGHT) {
                continue;
            }
            let sides_closed = (top + 1..bottom).all(|r| is_edge_vertical(frame, r, left) && is_edge_vertical(frame, r, right));
            if sides_closed {
                out.push(Rect { top, left, bottom, right });
            }
        }
    }
    out
}

pub fn rt_nested_borders(frame: &Frame, file: &str) -> Vec<Finding> {
    let boxes = find_boxes(frame);
    let mut out = Vec::new();
    for inner in &boxes {
        let enclosing = boxes
            .iter()
            .filter(|o| o.top < inner.top && o.left < inner.left && o.right > inner.right && o.bottom > inner.bottom)
            .max_by_key(|o| o.top);
        let Some(outer) = enclosing else { continue };
        // Immediate nesting: the ring between the two borders holds nothing.
        let ring_blank = (outer.top + 1..outer.bottom).all(|r| {
            (outer.left + 1..outer.right).all(|c| {
                let inside_inner = r >= inner.top && r <= inner.bottom && c >= inner.left && c <= inner.right;
                inside_inner || frame.glyph(r, c).map(char::is_whitespace).unwrap_or(true)
            })
        });
        if !ring_blank {
            continue;
        }
        let snippet = format!(
            "{}x{} box drawn directly inside a {}x{} box",
            inner.right - inner.left + 1,
            inner.bottom - inner.top + 1,
            outer.right - outer.left + 1,
            outer.bottom - outer.top + 1
        );
        out.push(rt_finding("tui-rt-nested-borders", file, frame, inner.top, inner.left, snippet));
    }
    out
}

/// The cell index of the row's last vertical border or right corner.
fn last_vertical(frame: &Frame, r: usize) -> Option<usize> {
    frame.rows.get(r)?.iter().rposition(|c| {
        c.glyph.map(|g| VERTICAL.contains(g) || TOP_RIGHT.contains(g) || BOTTOM_RIGHT.contains(g)).unwrap_or(false)
    })
}

/// The most common value, when at least two rows agree; ties go to the larger.
fn mode_of(values: &[usize]) -> Option<usize> {
    let mut best: Option<(usize, usize)> = None;
    for &v in values {
        let count = values.iter().filter(|&&x| x == v).count();
        if best.map(|(bv, bc)| count > bc || (count == bc && v > bv)).unwrap_or(true) {
            best = Some((v, count));
        }
    }
    best.filter(|&(_, count)| count >= 2).map(|(v, _)| v)
}

/// What a row wider than its frame is (spec section 7, PR 4 item 1). Each
/// row gets exactly one verdict, shared by width drift and narrow collapse;
/// `line` and `column` index the joined frame `capture-pane -J` returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overflow {
    /// No wider than the frame.
    Fits,
    /// The overflow is no larger than the row's count of wide glyphs: the
    /// app measured some of them as one cell. Any frame width.
    WidthDrift,
    /// Otherwise, in a frame under 60 columns, a row holding a Box Drawing
    /// glyph: the layout itself overflowed.
    Collapse,
    /// Anything else, such as prose the terminal wrapped and `-J` joined.
    Wrapped,
}

fn is_box_drawing(ch: char) -> bool {
    ('\u{2500}'..='\u{257F}').contains(&ch)
}

pub fn overflow_verdict(frame: &Frame, r: usize) -> Overflow {
    let w = frame.row_width(r);
    if frame.width == 0 || w <= frame.width {
        return Overflow::Fits;
    }
    if w - frame.width <= frame.wide_count(r) {
        return Overflow::WidthDrift;
    }
    let boxed = frame.rows[r].iter().any(|c| c.glyph.map(is_box_drawing).unwrap_or(false));
    if frame.width < NARROW_COLUMNS && boxed {
        Overflow::Collapse
    } else {
        Overflow::Wrapped
    }
}

pub fn rt_width_drift(frame: &Frame, file: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    let ends: Vec<usize> = (0..frame.rows.len()).filter_map(|r| last_vertical(frame, r)).collect();
    let usual_end = mode_of(&ends);
    for r in 0..frame.rows.len() {
        match overflow_verdict(frame, r) {
            Overflow::WidthDrift => {
                let snippet = format!("row measures {} cells on a {}-column pane", frame.row_width(r), frame.width);
                out.push(rt_finding("tui-rt-width-drift", file, frame, r, frame.width, snippet));
                continue;
            }
            Overflow::Fits => {}
            Overflow::Collapse | Overflow::Wrapped => continue,
        }
        if !frame.has_wide(r) {
            continue;
        }
        if let (Some(end), Some(usual)) = (last_vertical(frame, r), usual_end) {
            if end != usual {
                let snippet = format!("border ends at column {} while other rows end at {}", end + 1, usual + 1);
                out.push(rt_finding("tui-rt-width-drift", file, frame, r, end, snippet));
            }
        }
    }
    out
}

pub fn rt_collapse_narrow(frame: &Frame, file: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    for r in 0..frame.rows.len() {
        let row = &frame.rows[r];
        if row.is_empty() {
            continue;
        }
        let w = row.len();
        match overflow_verdict(frame, r) {
            Overflow::Collapse => {
                let snippet = format!("row overflows the {}-column pane by {} cells", frame.width, w - frame.width);
                out.push(rt_finding("tui-rt-collapse-narrow", file, frame, r, frame.width, snippet));
                continue;
            }
            Overflow::Fits => {}
            Overflow::WidthDrift | Overflow::Wrapped => continue,
        }
        let text = frame.text(r);
        let has_left = text.chars().any(|g| TOP_LEFT.contains(g) || BOTTOM_LEFT.contains(g));
        let has_right = text.chars().any(|g| TOP_RIGHT.contains(g) || BOTTOM_RIGHT.contains(g));
        if has_left && !has_right {
            out.push(rt_finding("tui-rt-collapse-narrow", file, frame, r, w - 1, "border loses its right corner".to_string()));
            continue;
        }
        // A word split at the right edge: the row fills the pane, ends in a
        // letter, and the next row starts with one.
        let ends_in_letter = row.last().and_then(|c| c.glyph).map(char::is_alphabetic).unwrap_or(false);
        let next_starts_with_letter = frame
            .rows
            .get(r + 1)
            .and_then(|n| n.first())
            .and_then(|c| c.glyph)
            .map(char::is_alphabetic)
            .unwrap_or(false);
        if w == frame.width && ends_in_letter && next_starts_with_letter {
            let tail: String = text.chars().rev().take(11).collect::<Vec<_>>().into_iter().rev().collect();
            let snippet = format!("word split at the pane edge: \"{}\"", tail.trim());
            out.push(rt_finding("tui-rt-collapse-narrow", file, frame, r, w - 1, snippet));
        }
    }
    out
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::capture::parse_rows;
    use impeccable_core::color::Rgba;

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
    fn a_ratio_just_under_the_floor_never_prints_as_the_floor() {
        // 4.478 must print "4.4:1", not round up to "4.5:1" (the floor it
        // just missed) and read as if it passed.
        let white = Rgba { r: 255.0, g: 255.0, b: 255.0, a: None };
        let gray = (100..140)
            .find(|&g| {
                let c = Rgba { r: g as f64, g: g as f64, b: g as f64, a: None };
                let ratio = contrast_ratio(&c, &white);
                (4.45..4.5).contains(&ratio)
            })
            .expect("a gray in 100..140 with a ratio between 4.45 and 4.5");
        let sgr = format!("\x1b[38;2;{gray};{gray};{gray};48;2;255;255;255m");
        let f = frame(40, &[&format!("{sgr}faint\x1b[0m"), HINTS]);
        let out = rt_low_contrast(&f, Palette::Dark, "x");
        assert_eq!(out.len(), 1, "{:?}", ids(&out));
        assert!(!out[0].snippet.contains("4.5:1 (need"), "{}", out[0].snippet);
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

    #[test]
    fn box_directly_inside_a_box_is_flagged_and_content_between_is_not() {
        let nested = frame(40, &[
            "┌──────────┐",
            "│┌────────┐│",
            "││ inner  ││",
            "│└────────┘│",
            "└──────────┘",
            HINTS,
        ]);
        let out = rt_nested_borders(&nested, "x");
        assert_eq!(out.len(), 1, "{:?}", ids(&out));
        assert_eq!((out[0].line, out[0].extras["column"].clone()), (2.0, Value::from(2)));
        assert_eq!(out[0].snippet, "10x3 box drawn directly inside a 12x5 box");
        let spaced = frame(40, &[
            "┌ Title ────┐",
            "│ label     │",
            "│ ┌───────┐ │",
            "│ │ inner │ │",
            "│ └───────┘ │",
            "└───────────┘",
            HINTS,
        ]);
        assert!(rt_nested_borders(&spaced, "x").is_empty(), "text between the borders is content");
        let siblings = frame(40, &["┌──┐ ┌──┐", "│a │ │b │", "└──┘ └──┘", HINTS]);
        assert!(rt_nested_borders(&siblings, "x").is_empty());
        assert_eq!(find_boxes(&siblings).len(), 2);
        let split = frame(40, &["┌────┬────┐", "│ a  │ b  │", "└────┴────┘", HINTS]);
        assert_eq!(find_boxes(&split).len(), 1, "a divider with tees is one box");
    }

    #[test]
    fn emoji_row_wider_than_the_pane_is_width_drift_and_a_shifted_border_too() {
        let f = frame(12, &[
            "┌──────────┐",
            "│ plain    │",
            "│ 🚀 launch │",   // padded as if the rocket were one cell: 13 cells
            "│ 日本 tok│",      // padded as if 日本 were six cells: 11 cells
            "└──────────┘",
            HINTS,
        ]);
        let out = rt_width_drift(&f, "x");
        assert_eq!(ids(&out), vec!["tui-rt-width-drift", "tui-rt-width-drift"]);
        assert_eq!((out[0].line, out[0].extras["column"].clone()), (3.0, Value::from(13)));
        assert_eq!(out[0].snippet, "row measures 13 cells on a 12-column pane");
        assert_eq!((out[1].line, out[1].extras["column"].clone()), (4.0, Value::from(11)));
        assert_eq!(out[1].snippet, "border ends at column 11 while other rows end at 12");
    }

    #[test]
    fn ascii_rows_never_trip_width_drift() {
        let f = frame(12, &["┌──────────┐", "│ plain    │", "│ overflow row │", "└──────────┘", HINTS]);
        assert!(rt_width_drift(&f, "x").is_empty());
    }

    #[test]
    fn narrow_frame_overflow_lost_corner_and_split_word_are_collapse() {
        let f = frame(40, &[
            &format!("┌{}┐", "─".repeat(48)),
            "┌────────",
            "Processing the selected directory record",
            "s to the index",
            "q quit",
        ]);
        let out = rt_collapse_narrow(&f, "x");
        assert_eq!(ids(&out), vec!["tui-rt-collapse-narrow"; 3]);
        assert_eq!(out[0].snippet, "row overflows the 40-column pane by 10 cells");
        assert_eq!(out[1].snippet, "border loses its right corner");
        assert_eq!(out[2].snippet, "word split at the pane edge: \"tory record\"");
        assert_eq!(out[2].line, 3.0);
    }

    #[test]
    fn geometry_rules_skip_the_recapture_and_size_rules_read_the_first_frame() {
        let first = frame(40, &["┌──┐", "│ab│", "└──┘", HINTS]);
        let mut recap = frame(40, &["┌────┐", "│┌──┐│", "││ab││", "│└──┘│", "└────┘", HINTS]);
        recap.role = FrameRole::Recapture;
        let narrow = frame(20, &["┌──────────────────────┐", "text", HINTS]);
        let out = scan_frames(&[first, recap, narrow], Palette::Dark, "tmux:app:0.0");
        let found = ids(&out);
        assert!(!found.contains(&"tui-rt-nested-borders"), "the recapture is never scanned for geometry: {found:?}");
        assert!(found.contains(&"tui-rt-collapse-narrow"), "{found:?}");
        assert!(
            out.iter().filter(|f| f.antipattern == "tui-rt-collapse-narrow").all(|f| f.extras["frame"] == Value::from("20x3")),
            "collapse only on the narrow frame"
        );
        assert!(out.iter().all(|f| f.advisory == Some(true) && f.file == "tmux:app:0.0"));
        let wide = frame(80, &["┌──────────────────────────────────────────────────────────────────────────────────────┐", "text", HINTS]);
        assert!(!ids(&scan_frames(&[wide], Palette::Dark, "x")).contains(&"tui-rt-collapse-narrow"), "80 columns is not narrow");
    }

    const PROSE: &str = "The quick brown fox jumps over the lazy dog while the index rebuilds in the background.";

    #[test]
    fn joined_prose_row_under_sixty_columns_reports_nothing() {
        let f = frame(40, &[&format!("┌{}┐", "─".repeat(38)), &format!("└{}┘", "─".repeat(38)), PROSE, HINTS]);
        assert_eq!(overflow_verdict(&f, 2), Overflow::Wrapped);
        assert!(rt_collapse_narrow(&f, "x").is_empty(), "a row -J joined is the terminal's wrap");
        assert!(rt_width_drift(&f, "x").is_empty());
    }

    #[test]
    fn emoji_overflow_under_sixty_columns_is_width_drift_only() {
        let top = format!("┌{}┐", "─".repeat(38));
        let rocket = format!("│ 🚀 launch{}│", " ".repeat(29)); // 1 + 1 + 2 + 7 + 29 + 1 = 41 cells
        let bottom = format!("└{}┘", "─".repeat(38));
        let f = frame(40, &[&top, &rocket, &bottom, HINTS]);
        assert_eq!(f.row_width(1), 41);
        assert_eq!(overflow_verdict(&f, 1), Overflow::WidthDrift);
        let out = scan_frames(&[f], Palette::Dark, "x");
        assert_eq!(ids(&out), vec!["tui-rt-width-drift"], "one verdict per row: no collapse beside the drift");
        assert_eq!(out[0].snippet, "row measures 41 cells on a 40-column pane");
    }

    #[test]
    fn heart_with_emoji_presentation_counts_as_one_wide_glyph() {
        let one_over = frame(40, &[&format!("│ ❤\u{FE0F} ok{}│", " ".repeat(33))]); // 1 + 1 + 2 + 3 + 33 + 1 = 41
        assert_eq!((one_over.row_width(0), one_over.wide_count(0)), (41, 1));
        assert_eq!(overflow_verdict(&one_over, 0), Overflow::WidthDrift);
        let two_over = frame(40, &[&format!("│ ❤\u{FE0F} ok{}│", " ".repeat(34))]);
        assert_eq!(overflow_verdict(&two_over, 0), Overflow::Collapse, "42 cells with one wide glyph is more than drift");
    }

    #[test]
    fn boxed_overflow_under_sixty_columns_is_collapse_only() {
        let f = frame(40, &[&format!("┌{}┐", "─".repeat(48)), HINTS]);
        assert_eq!(overflow_verdict(&f, 0), Overflow::Collapse);
        assert_eq!(ids(&rt_collapse_narrow(&f, "x")), vec!["tui-rt-collapse-narrow"]);
        assert!(rt_width_drift(&f, "x").is_empty());
    }

    #[test]
    fn overflow_past_the_wide_glyph_count_at_sixty_columns_or_more_reports_nothing() {
        // A joined row in a wide frame: one emoji plus prose the terminal wrapped.
        let f = frame(80, &[&format!("│ 🚀 {}", "word ".repeat(20)), HINTS]); // 105 cells on 80
        assert_eq!(overflow_verdict(&f, 0), Overflow::Wrapped);
        assert!(rt_width_drift(&f, "x").is_empty());
        assert_eq!(overflow_verdict(&f, 1), Overflow::Fits);
    }

    #[test]
    fn a_key_on_the_first_row_or_a_pager_prompt_at_the_bottom_silences_key_hints() {
        let help_on_top = frame(80, &["q quit  / search  ? help", "NAME        SIZE", "a.txt       12K", "b.txt       40K"]);
        assert!(rt_no_key_hints(&help_on_top, "x").is_empty(), "a screen whose first row names the keys");
        for prompt in [":", ":                    ", "(END)", "--More--(42%)", "lines 1-24/200 12%", "README.md lines 25-48"] {
            let pager = frame(80, &["NAME", "     less - opposite of more", "DESCRIPTION", prompt]);
            assert!(rt_no_key_hints(&pager, "x").is_empty(), "pager prompt {prompt:?}");
        }
        // Rows captured from macOS top on a scratch server while planning.
        let top = frame(100, &[
            "Processes: 714 total, 2 running, 712 sleeping, 4442 threads                                14:22:30",
            "Load Avg: 4.94, 5.73, 7.70  CPU usage: 13.12% user, 22.89% sys, 63.98% idle",
            "87568  installd     0.0  00:00.21 2     1    59    1216K 0B    0B    87568 1     sleeping",
        ]);
        assert_eq!(rt_no_key_hints(&top, "x").len(), 1, "top names no key on its first row, so it still fires");
        let colon_inside = frame(80, &["NAME", "     less - opposite of more", "DESCRIPTION", "Status: idle"]);
        assert_eq!(rt_no_key_hints(&colon_inside, "x").len(), 1, "a colon inside text is not a prompt");
        let guidelines = frame(80, &["NAME", "     less - opposite of more", "DESCRIPTION", "Guidelines 1-5 apply"]);
        assert_eq!(rt_no_key_hints(&guidelines, "x").len(), 1, "\"lines\" inside another word is not a pager prompt");
        let readme_lines = frame(80, &["NAME", "     less - opposite of more", "DESCRIPTION", "README.md lines 25-48"]);
        assert!(rt_no_key_hints(&readme_lines, "x").is_empty(), "a real lines prompt still silences");
    }

    #[test]
    fn a_ratio_over_theme_colors_is_marked_palette_relative() {
        let f = frame(40, &[
            "\x1b[38;2;120;120;120;48;2;100;100;100mfaint\x1b[0m",
            "\x1b[30mblack on default\x1b[0m",
            "\x1b[38;5;236;48;5;235mdim cube\x1b[0m",
            HINTS,
        ]);
        let out = rt_low_contrast(&f, Palette::Dark, "x");
        assert_eq!(out.len(), 3, "{:?}", out.iter().map(|x| &x.snippet).collect::<Vec<_>>());
        assert_eq!(out[0].extras["paletteRelative"], Value::from(false), "truecolor on truecolor is the same on every theme");
        assert_eq!(out[1].extras["paletteRelative"], Value::from(true), "named black on the default background comes from the theme");
        assert_eq!(out[2].extras["paletteRelative"], Value::from(false), "the 256-color cube and grays are fixed");
        let keys: Vec<&String> = out[0].extras.keys().collect();
        assert_eq!(keys, vec!["column", "frame", "cellCount", "paletteRelative"], "the new extra comes last");
    }
}
