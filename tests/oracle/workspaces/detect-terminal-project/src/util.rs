// Names no terminal framework: the per-file gate keeps this out even though
// the token below would match the rule.
pub struct Color;
impl Color {
    pub fn rgb(_: u8, _: u8, _: u8) -> Color { Color }
}
pub const DECOY: &str = "Color::Rgb(1, 2, 3)";
