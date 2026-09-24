use ratatui::widgets::{Block, BorderType};
fn framed() -> Block<'static> { Block::default().border_type(BorderType::Double) } // flag
fn quiet() -> Block<'static> { Block::default().border_type(BorderType::Plain) } // pass
