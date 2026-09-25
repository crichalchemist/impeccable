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
    /// The pane's own size again, one second after the first capture.
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
}

/// The line that introduces a frame in a saved capture file.
pub const HEADER: &str = "#!capture";

/// Rows of captured text into cells. A trailing newline does not add a row.
pub fn parse_rows(text: &str) -> Vec<Vec<Cell>> {
    let mut lines: Vec<&str> = text.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    lines.into_iter().map(parse_row).collect()
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
        frame.rows = lines.into_iter().map(parse_row).collect();
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

/// One captured row. SGR sequences update the style; every other character
/// is a cell (zero-width characters take none, two-column glyphs take two).
pub fn parse_row(line: &str) -> Vec<Cell> {
    let mut cells = Vec::new();
    let mut style = Style::default();
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                let mut params = String::new();
                let mut terminator = None;
                for c in chars.by_ref() {
                    if c.is_ascii_digit() || c == ';' || c == ':' {
                        params.push(c);
                    } else {
                        terminator = Some(c);
                        break;
                    }
                }
                if terminator == Some('m') {
                    apply_sgr(&mut style, &params);
                }
            }
            continue;
        }
        if ch == '\r' {
            continue;
        }
        match ch.width().unwrap_or(0) {
            0 => {}
            1 => cells.push(Cell { glyph: Some(ch), wide_tail: false, style }),
            _ => {
                cells.push(Cell { glyph: Some(ch), wide_tail: false, style });
                cells.push(Cell { glyph: None, wide_tail: true, style });
            }
        }
    }
    cells
}

/// The SGR parameters tmux emits: reset, bold, dim, blink, reverse and their
/// offs, the 16 named colors, `38;5;n` / `48;5;n`, `38;2;r;g;b` / `48;2;r;g;b`.
/// Anything else is ignored.
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
            38 | 48 => {
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
                    if p == 38 {
                        style.fg = c
                    } else {
                        style.bg = c
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
        assert_eq!(cells.len(), 9);
        assert!(cells[2].wide_tail && cells[2].glyph.is_none());
        let frame = Frame { rows: vec![cells], ..Frame::default() };
        assert_eq!(frame.text(0), "a😀b日本⚠");
        assert!(frame.has_wide(0));
        assert_eq!(frame.row_width(0), 9);
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
}
