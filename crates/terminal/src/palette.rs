//! SGR colors to sRGB for the contrast rule (spec section 5, "Palette").

use impeccable_core::color::Rgba;

use crate::capture::{Color, Style};

/// The reference palette contrast is measured on. The two differ only in
/// the default foreground and background.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Palette {
    #[default]
    Dark,
    Light,
}

impl Palette {
    pub fn parse(name: &str) -> Option<Palette> {
        match name {
            "dark" => Some(Palette::Dark),
            "light" => Some(Palette::Light),
            _ => None,
        }
    }

    fn default_fg(self) -> (u8, u8, u8) {
        match self {
            Palette::Dark => (0xff, 0xff, 0xff),
            Palette::Light => (0x00, 0x00, 0x00),
        }
    }

    fn default_bg(self) -> (u8, u8, u8) {
        match self {
            Palette::Dark => (0x00, 0x00, 0x00),
            Palette::Light => (0xff, 0xff, 0xff),
        }
    }

    /// The (foreground, background) a cell renders with, after `reverse`.
    pub fn resolve(self, style: &Style) -> (Rgba, Rgba) {
        let fg = self.color(style.fg, true);
        let bg = self.color(style.bg, false);
        if style.reverse {
            (bg, fg)
        } else {
            (fg, bg)
        }
    }

    fn color(self, color: Color, foreground: bool) -> Rgba {
        let (r, g, b) = match color {
            Color::Default => {
                if foreground {
                    self.default_fg()
                } else {
                    self.default_bg()
                }
            }
            Color::Indexed(i) => indexed(i),
            Color::Rgb(r, g, b) => (r, g, b),
        };
        Rgba { r: r as f64, g: g as f64, b: b as f64, a: None }
    }
}

/// xterm's sixteen named colors, the table both palettes share: the rule is
/// about the pairs the app chose, not about a theme.
pub const ANSI16: [(u8, u8, u8); 16] = [
    (0x00, 0x00, 0x00), (0xcd, 0x00, 0x00), (0x00, 0xcd, 0x00), (0xcd, 0xcd, 0x00),
    (0x00, 0x00, 0xee), (0xcd, 0x00, 0xcd), (0x00, 0xcd, 0xcd), (0xe5, 0xe5, 0xe5),
    (0x7f, 0x7f, 0x7f), (0xff, 0x00, 0x00), (0x00, 0xff, 0x00), (0xff, 0xff, 0x00),
    (0x5c, 0x5c, 0xff), (0xff, 0x00, 0xff), (0x00, 0xff, 0xff), (0xff, 0xff, 0xff),
];

/// The 256-color space: the named sixteen, the 6x6x6 cube, the 24 grays.
pub fn indexed(index: u8) -> (u8, u8, u8) {
    match index {
        0..=15 => ANSI16[index as usize],
        16..=231 => {
            let n = index - 16;
            let level = |v: u8| if v == 0 { 0 } else { 55 + 40 * v };
            (level(n / 36), level((n / 6) % 6), level(n % 6))
        }
        _ => {
            let v = 8 + 10 * (index - 232);
            (v, v, v)
        }
    }
}

/// `#rrggbb`, lowercase, for snippets.
pub fn hex(color: &Rgba) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r as u8, color.g as u8, color.b as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexed_cube_and_grays_match_xterm() {
        assert_eq!(indexed(16), (0, 0, 0));
        assert_eq!(indexed(196), (255, 0, 0));
        assert_eq!(indexed(208), (255, 135, 0));
        assert_eq!(indexed(232), (8, 8, 8));
        assert_eq!(indexed(255), (238, 238, 238));
    }

    #[test]
    fn reverse_swaps_foreground_and_background() {
        let style = Style { fg: Color::Indexed(1), bg: Color::Default, reverse: true, ..Style::default() };
        let (fg, bg) = Palette::Dark.resolve(&style);
        assert_eq!(hex(&fg), "#000000");
        assert_eq!(hex(&bg), "#cd0000");
    }

    #[test]
    fn light_palette_inverts_only_the_defaults() {
        let plain = Style::default();
        let (dfg, dbg) = Palette::Dark.resolve(&plain);
        let (lfg, lbg) = Palette::Light.resolve(&plain);
        assert_eq!((hex(&dfg), hex(&dbg)), ("#ffffff".to_string(), "#000000".to_string()));
        assert_eq!((hex(&lfg), hex(&lbg)), ("#000000".to_string(), "#ffffff".to_string()));
        let red = Style { fg: Color::Indexed(9), ..Style::default() };
        assert_eq!(hex(&Palette::Light.resolve(&red).0), hex(&Palette::Dark.resolve(&red).0));
        assert_eq!(Palette::parse("sepia"), None);
        assert_eq!(Palette::parse("light"), Some(Palette::Light));
    }
}
