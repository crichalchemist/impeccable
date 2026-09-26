use crossterm::terminal;
use ratatui::layout::{Constraint, Layout, Rect};
fn area() -> Rect {
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    Rect::new(0, 0, cols, rows)
}
fn fallback() -> Rect { Rect::new(0, 0, 80, 24) } // pass: this file measures the size, so 80x24 is the fallback
fn fallback_width() -> u16 { let width = 80; width } // pass: the same fallback
fn rigid() -> Layout { Layout::horizontal([Constraint::Length(4), Constraint::Length(8)]) } // flag: the exemption covers literals, not layouts
