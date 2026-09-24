use ratatui::layout::{Constraint, Layout, Rect};
fn area() -> Rect { Rect::new(0, 0, 80, 24) } // flag
fn rigid() -> Layout { Layout::vertical([Constraint::Length(3), Constraint::Length(10)]) } // flag: all Length
fn live(size: Rect) -> Rect { size } // pass
