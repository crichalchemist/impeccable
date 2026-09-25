use ratatui::layout::Rect;
use ratatui::widgets::canvas::{Canvas, Rectangle};
fn bounds() -> Rect { Rect::new(10, 10, 200, 100) } // pass: a canvas drawing coordinate
fn panel() -> u16 { let width = 80; width } // flag: the canvas exemption covers Rect::new only
