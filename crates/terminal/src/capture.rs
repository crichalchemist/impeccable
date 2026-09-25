//! Capture parsing: `tmux capture-pane -p -e -J` text into a cell grid
//! (spec section 5, "Capture and the frame model"). Pure functions over
//! text, so goldens replay from saved files without tmux.

use unicode_width::UnicodeWidthChar;

/// A color as the SGR stream named it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Color {
    #[default]
    Default,
    /// `30..37` and `40..47` as 0 to 7, `90..97` and `100..107` as 8 to 15,
    /// `38;5;n` / `48;5;n` as 0 to 255.
    Indexed(u8),
    /// `38;2;r;g;b` / `48;2;r;g;b`.
    Rgb(u8, u8, u8),
}

/// The SGR state a cell was written with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub dim: bool,
    pub blink: bool,
    pub reverse: bool,
}

/// One column of one row. A two-column glyph owns its cell and the next,
/// which carries `glyph: None` and `wide_tail: true`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub glyph: Option<char>,
    pub wide_tail: bool,
    pub style: Style,
}

/// Which capture of the run a frame is (spec section 5 vocabulary).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FrameRole {
    /// A capture at a size the run asked for.
    #[default]
    Capture,
    /// The pane's own size again, 700 ms or 1000 ms after the first capture.
    Recapture,
}

/// One parsed capture.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Frame {
    pub width: usize,
    pub height: usize,
    /// `#{client_termfeatures}` (for example `256,RGB`); empty when unknown.
    pub termfeatures: String,
    /// `COLORTERM` in the engine's environment; empty when unset.
    pub colorterm: String,
    pub role: FrameRole,
    pub rows: Vec<Vec<Cell>>,
}

impl Frame {
    pub fn glyph(&self, row: usize, col: usize) -> Option<char> {
        self.rows.get(row)?.get(col)?.glyph
    }

    pub fn row_width(&self, row: usize) -> usize {
        self.rows.get(row).map(Vec::len).unwrap_or(0)
    }

    /// The row as text, one character per glyph (wide tails skipped).
    pub fn text(&self, row: usize) -> String {
        self.rows
            .get(row)
            .map(|r| r.iter().filter(|c| !c.wide_tail).map(|c| c.glyph.unwrap_or(' ')).collect())
            .unwrap_or_default()
    }

    pub fn is_blank(&self, row: usize) -> bool {
        self.rows
            .get(row)
            .map(|r| r.iter().all(|c| c.glyph.map(char::is_whitespace).unwrap_or(true)))
            .unwrap_or(true)
    }

    /// Does the row hold a two-column glyph (emoji, CJK)?
    pub fn has_wide(&self, row: usize) -> bool {
        self.rows.get(row).map(|r| r.iter().any(|c| c.wide_tail)).unwrap_or(false)
    }

    /// How many two-cell clusters the row holds: one per wide tail.
    pub fn wide_count(&self, row: usize) -> usize {
        self.rows.get(row).map(|r| r.iter().filter(|c| c.wide_tail).count()).unwrap_or(0)
    }
}

/// The line that introduces a frame in a saved capture file.
pub const HEADER: &str = "#!capture";

/// Rows of captured text into cells. A trailing newline does not add a row.
/// tmux's `capture-pane -e` diffs SGR against one running cell state for the
/// whole capture, so a style set on one row and unchanged on the next carries
/// forward; the style is not reset between rows.
pub fn parse_rows(text: &str) -> Vec<Vec<Cell>> {
    let mut lines: Vec<&str> = text.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let mut style = Style::default();
    lines.into_iter().map(|line| parse_row(line, &mut style)).collect()
}

/// A saved capture file into frames: every `#!capture` line starts a frame;
/// a file without one is a single frame measured from its rows.
pub fn parse_frames(text: &str) -> Result<Vec<Frame>, String> {
    let mut groups: Vec<(Option<&str>, Vec<&str>)> = Vec::new();
    for line in text.split('\n') {
        if let Some(rest) = line.strip_prefix(HEADER) {
            groups.push((Some(rest), Vec::new()));
        } else if let Some(last) = groups.last_mut() {
            last.1.push(line);
        } else {
            groups.push((None, vec![line]));
        }
    }
    let mut frames = Vec::new();
    for (header, mut lines) in groups {
        if lines.last() == Some(&"") {
            lines.pop();
        }
        let mut frame = Frame::default();
        if let Some(h) = header {
            apply_header(&mut frame, h)?;
        }
        // Every `#!capture` header starts a new frame, so the carried style
        // resets here rather than continuing from the previous frame.
        let mut style = Style::default();
        frame.rows = lines.into_iter().map(|line| parse_row(line, &mut style)).collect();
        if frame.width == 0 {
            frame.width = frame.rows.iter().map(Vec::len).max().unwrap_or(0);
        }
        if frame.height == 0 {
            frame.height = frame.rows.len();
        }
        frames.push(frame);
    }
    Ok(frames)
}

fn apply_header(frame: &mut Frame, header: &str) -> Result<(), String> {
    for pair in header.split_whitespace() {
        let Some((key, value)) = pair.split_once('=') else {
            return Err(format!("capture header: expected key=value, got {pair:?}"));
        };
        match key {
            "width" => {
                frame.width = value
                    .parse()
                    .map_err(|_| format!("capture header: width {value:?} is not a number"))?
            }
            "height" => {
                frame.height = value
                    .parse()
                    .map_err(|_| format!("capture header: height {value:?} is not a number"))?
            }
            "termfeatures" => frame.termfeatures = value.to_string(),
            "colorterm" => frame.colorterm = value.to_string(),
            "role" => {
                frame.role = match value {
                    "capture" => FrameRole::Capture,
                    "recapture" => FrameRole::Recapture,
                    _ => return Err(format!("capture header: unknown role {value:?}")),
                }
            }
            _ => return Err(format!("capture header: unknown key {key:?}")),
        }
    }
    Ok(())
}

/// One captured row. SGR sequences update `style` (carried in from, and left
/// for, the caller); every other character is a cell (zero-width characters
/// take none, two-column glyphs take two), with tmux's rules for U+FE0F,
/// emoji modifiers, and U+200D on top of `unicode-width`. Non-CSI escapes
/// (OSC hyperlinks, DCS/SOS/PM/APC strings, and anything else two-byte) are
/// skipped rather than leaked as cells, and a CSI with a private parameter
/// byte is ignored.
fn parse_row(line: &str, style: &mut Style) -> Vec<Cell> {
    let mut cells: Vec<Cell> = Vec::new();
    let mut chars = line.chars().peekable();
    let mut after_zwj = false;
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            match chars.peek() {
                Some('[') => {
                    chars.next();
                    let mut params = String::new();
                    let mut private = false;
                    // Parameter bytes: 0x30..=0x3F (digits, `:`, `;`, and the
                    // private markers `<`, `=`, `>`, `?`).
                    while let Some(&c) = chars.peek() {
                        if !('\x30'..='\x3f').contains(&c) {
                            break;
                        }
                        chars.next();
                        if matches!(c, '<' | '=' | '>' | '?') {
                            private = true;
                        } else {
                            params.push(c);
                        }
                    }
                    // Intermediate bytes: 0x20..=0x2F.
                    while let Some(&c) = chars.peek() {
                        if !('\x20'..='\x2f').contains(&c) {
                            break;
                        }
                        chars.next();
                    }
                    // Final byte: 0x40..=0x7E.
                    let final_byte = chars.next();
                    if final_byte == Some('m') && !private {
                        apply_sgr(style, &params);
                    }
                }
                Some(']') => {
                    chars.next();
                    skip_string(&mut chars, true);
                }
                Some('P') | Some('X') | Some('^') | Some('_') => {
                    chars.next();
                    skip_string(&mut chars, false);
                }
                Some(_) => {
                    chars.next();
                }
                None => {}
            }
            continue;
        }
        if ch == '\r' {
            continue;
        }
        // tmux 3.7c's cluster widths (spec section 7, PR 4 item 6): a
        // character joined by U+200D is drawn inside the cluster before it,
        // an emoji modifier recolors the glyph before it, and U+FE0F asks
        // for emoji presentation, which tmux draws two cells wide.
        if after_zwj {
            after_zwj = false;
            continue;
        }
        if ch == '\u{200D}' {
            after_zwj = true;
            continue;
        }
        if ('\u{1F3FB}'..='\u{1F3FF}').contains(&ch) {
            continue;
        }
        if ch == '\u{FE0F}' {
            // Only a one-cell glyph widens: a wide glyph's last cell is its tail.
            if let Some(base) = cells.last().filter(|c| c.glyph.is_some()).map(|c| c.style) {
                cells.push(Cell { glyph: None, wide_tail: true, style: base });
            }
            continue;
        }
        match ch.width().unwrap_or(0) {
            0 => {}
            1 => cells.push(Cell { glyph: Some(ch), wide_tail: false, style: *style }),
            _ => {
                cells.push(Cell { glyph: Some(ch), wide_tail: false, style: *style });
                cells.push(Cell { glyph: None, wide_tail: true, style: *style });
            }
        }
    }
    cells
}

/// Skip an OSC (`allow_bel`) or DCS/SOS/PM/APC string to its terminator: BEL
/// (OSC only) or the two-byte string terminator `ESC \`. Neither appearing
/// skips to the end of the row.
fn skip_string(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, allow_bel: bool) {
    while let Some(c) = chars.next() {
        if allow_bel && c == '\x07' {
            return;
        }
        if c == '\x1b' && chars.peek() == Some(&'\\') {
            chars.next();
            return;
        }
    }
}

/// The SGR parameters tmux emits: reset, bold, dim, blink, reverse and their
/// offs, the 16 named colors, `38;5;n` / `48;5;n`, `38;2;r;g;b` / `48;2;r;g;b`,
/// and `58;5;n` / `58;2;r;g;b` (underline color: consumed so its parameters
/// are never misread as ordinary codes, but not applied to `style`, which has
/// no underline-color slot). Anything else is ignored.
fn apply_sgr(style: &mut Style, params: &str) {
    let parts: Vec<&str> = if params.is_empty() { vec!["0"] } else { params.split([';', ':']).collect() };
    let num = |i: usize| parts.get(i).and_then(|p| p.parse::<u16>().ok());
    let mut i = 0;
    while i < parts.len() {
        let Some(p) = num(i) else {
            i += 1;
            continue;
        };
        match p {
            0 => *style = Style::default(),
            1 => style.bold = true,
            2 => style.dim = true,
            5 | 6 => style.blink = true,
            7 => style.reverse = true,
            22 => {
                style.bold = false;
                style.dim = false;
            }
            25 => style.blink = false,
            27 => style.reverse = false,
            30..=37 => style.fg = Color::Indexed((p - 30) as u8),
            90..=97 => style.fg = Color::Indexed((p - 90 + 8) as u8),
            39 => style.fg = Color::Default,
            40..=47 => style.bg = Color::Indexed((p - 40) as u8),
            100..=107 => style.bg = Color::Indexed((p - 100 + 8) as u8),
            49 => style.bg = Color::Default,
            // 58 (underline color) takes the same `;5;n` / `;2;r;g;b` shape
            // as 38/48 and must consume the same parameters, but tmux has no
            // underline-color slot on `Style` to store it in: the value is
            // read and discarded so it never falls through to `_` and gets
            // misread as ordinary SGR codes (issue: undercurl colors from
            // Neovim/Helix were resetting fg/bg).
            38 | 48 | 58 => {
                let color = match num(i + 1) {
                    Some(5) => {
                        let index = num(i + 2).unwrap_or(0).min(255) as u8;
                        i += 2;
                        Some(Color::Indexed(index))
                    }
                    Some(2) => {
                        // `38;2;r;g;b`; the colon form may carry an empty
                        // color-space id (`38:2::r:g:b`).
                        let skip = usize::from(parts.get(i + 2) == Some(&""));
                        let channel = |k: usize| num(i + 2 + skip + k).unwrap_or(0).min(255) as u8;
                        let rgb = Color::Rgb(channel(0), channel(1), channel(2));
                        i += 4 + skip;
                        Some(rgb)
                    }
                    _ => None,
                };
                if let Some(c) = color {
                    match p {
                        38 => style.fg = c,
                        48 => style.bg = c,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A row parsed on its own, starting (and ending) with a default style.
    fn parse_row(line: &str) -> Vec<Cell> {
        super::parse_row(line, &mut Style::default())
    }

    #[test]
    fn sgr_colors_and_attributes_land_on_the_cells_they_precede() {
        let cells = parse_row("\x1b[1m\x1b[38;2;120;120;120m\x1b[48;2;100;100;100mhi\x1b[0m \x1b[31mred\x1b[39m!");
        assert_eq!(cells.len(), 7);
        assert_eq!(
            cells[0].style,
            Style { fg: Color::Rgb(120, 120, 120), bg: Color::Rgb(100, 100, 100), bold: true, ..Style::default() }
        );
        assert_eq!(cells[2].style, Style::default(), "0 resets everything");
        assert_eq!(cells[3].style.fg, Color::Indexed(1));
        assert_eq!(cells[6].style.fg, Color::Default, "39 resets the foreground only");
        assert_eq!(cells[6].glyph, Some('!'));
    }

    #[test]
    fn bright_indexed_and_combined_parameters_map_into_the_256_index_space() {
        let cells = parse_row("\x1b[97;104ma\x1b[38;5;208mb\x1b[7mc\x1b[38;2;1;2;3;48;2;4;5;6md");
        assert_eq!((cells[0].style.fg, cells[0].style.bg), (Color::Indexed(15), Color::Indexed(12)));
        assert_eq!(cells[1].style.fg, Color::Indexed(208));
        assert!(cells[2].style.reverse);
        assert_eq!((cells[3].style.fg, cells[3].style.bg), (Color::Rgb(1, 2, 3), Color::Rgb(4, 5, 6)));
    }

    #[test]
    fn wide_glyphs_take_two_cells_and_zero_width_marks_take_none() {
        let cells = parse_row("a😀b日本⚠\u{FE0F}");
        assert_eq!(cells.len(), 10, "⚠ with U+FE0F is two cells since PR 4");
        assert!(cells[2].wide_tail && cells[2].glyph.is_none());
        let frame = Frame { rows: vec![cells], ..Frame::default() };
        assert_eq!(frame.text(0), "a😀b日本⚠");
        assert!(frame.has_wide(0));
        assert_eq!(frame.row_width(0), 10);
    }

    #[test]
    fn capture_parser_replays_without_tmux() {
        let text = "#!capture width=40 height=3 termfeatures=256 colorterm=truecolor\nrow one\n\n\n#!capture width=40 height=3 role=recapture\nrow two\n";
        let frames = parse_frames(text).unwrap();
        assert_eq!(frames.len(), 2);
        assert_eq!((frames[0].width, frames[0].height, frames[0].rows.len()), (40, 3, 2));
        assert!(frames[0].is_blank(1), "an empty row inside a frame is kept");
        assert_eq!(frames[0].termfeatures, "256");
        assert_eq!(frames[0].colorterm, "truecolor");
        assert_eq!(frames[0].role, FrameRole::Capture);
        assert_eq!(frames[1].role, FrameRole::Recapture);
        assert_eq!(frames[1].text(0), "row two");
    }

    #[test]
    fn headerless_capture_measures_itself_and_knows_no_client() {
        let frames = parse_frames("ab\nabcd\n").unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!((frames[0].width, frames[0].height), (4, 2));
        assert_eq!(frames[0].termfeatures, "");
        assert_eq!(frames[0].role, FrameRole::Capture);
    }

    #[test]
    fn bad_headers_are_errors_not_guesses() {
        assert_eq!(parse_frames("#!capture width=wide\n").unwrap_err(), "capture header: width \"wide\" is not a number");
        assert_eq!(parse_frames("#!capture role=third\n").unwrap_err(), "capture header: unknown role \"third\"");
        assert_eq!(parse_frames("#!capture depth=8\n").unwrap_err(), "capture header: unknown key \"depth\"");
        assert_eq!(parse_frames("#!capture width\n").unwrap_err(), "capture header: expected key=value, got \"width\"");
    }

    #[test]
    fn background_set_on_one_row_still_applies_on_the_next() {
        // tmux's `capture-pane -e` diffs SGR against one running cell state
        // for the whole capture: a background set on row N and unchanged on
        // row N+1 gets no SGR on row N+1, so the parser must carry it.
        let rows = parse_rows("\x1b[48;5;15mA\nB");
        assert_eq!(rows[0][0].style.bg, Color::Indexed(15));
        assert_eq!(rows[1][0].style.bg, rows[0][0].style.bg, "row 2 inherits row 1's background");
    }

    #[test]
    fn a_new_frame_header_resets_the_carried_style() {
        let text = "#!capture width=1 height=1\n\x1b[48;5;15mA\n#!capture width=1 height=1\nB\n";
        let frames = parse_frames(text).unwrap();
        assert_eq!(frames[0].rows[0][0].style.bg, Color::Indexed(15));
        assert_eq!(frames[1].rows[0][0].style, Style::default(), "a new #!capture header starts clean");
    }

    #[test]
    fn an_osc_8_hyperlink_adds_no_cells() {
        let cells = super::parse_row("\x1b]8;;https://example.com\x1b\\link\x1b]8;;\x1b\\ tail", &mut Style::default());
        assert_eq!(cells.len(), 9);
        let frame = Frame { rows: vec![cells], ..Frame::default() };
        assert_eq!(frame.row_width(0), 9);
        assert_eq!(frame.text(0), "link tail");
    }

    #[test]
    fn a_private_csi_adds_no_cells() {
        let cells = super::parse_row("\x1b[?25l hidden \x1b[?25h", &mut Style::default());
        let frame = Frame { rows: vec![cells], ..Frame::default() };
        assert_eq!(frame.row_width(0), 8);
        assert_eq!(frame.text(0), " hidden ");
    }

    #[test]
    fn an_osc_terminated_by_bel_adds_no_cells() {
        let cells = super::parse_row("\x1b]0;title\x07x", &mut Style::default());
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].glyph, Some('x'));
    }

    #[test]
    fn underline_color_parameters_do_not_reset_the_cell_colors() {
        // SGR 58 (underline color, `58;5;n` / `58;2;r;g;b`) shares the
        // sub-parameter shape of 38/48 but tmux has no slot to store it in;
        // its parameters must be consumed, not read as ordinary SGR codes.
        let cells = parse_row("\x1b[38;5;15m\x1b[48;5;4m\x1b[58;2;255;0;0mX");
        assert_eq!(cells[0].style.fg, Color::Indexed(15));
        assert_eq!(cells[0].style.bg, Color::Indexed(4));
        assert!(!cells[0].style.dim, "the underline-color RGB channels must not be read as SGR 2 (dim)");
    }

    #[test]
    fn emoji_sequences_measure_two_cells_like_tmux() {
        // The six sequences probed on tmux 3.7c with #{cursor_x} (spec section 7).
        for s in ["❤\u{FE0F}", "✔\u{FE0F}", "☀\u{FE0F}", "⚠\u{FE0F}", "👍\u{1F3FD}", "🧑\u{200D}💻"] {
            let cells = parse_row(s);
            assert_eq!(cells.len(), 2, "{s:?} measures two cells");
            assert!(cells[0].glyph.is_some() && cells[1].wide_tail, "{s:?} is one glyph and its wide tail");
        }
        assert_eq!(parse_row("❤").len(), 1, "without U+FE0F the heart stays one cell, as tmux draws it");
        assert_eq!(parse_row("日\u{FE0F}").len(), 2, "U+FE0F never widens a glyph that is already two cells");
        assert_eq!(parse_row("a\u{200D}b c").len(), 3, "the character after U+200D takes no cell");
    }

    #[test]
    fn a_row_counts_one_wide_glyph_per_two_cell_cluster() {
        let frame = Frame { rows: vec![parse_row("❤\u{FE0F} 🚀 日 ok")], ..Frame::default() };
        assert_eq!(frame.wide_count(0), 3);
        assert_eq!(frame.wide_count(1), 0, "a missing row holds none");
    }
}
