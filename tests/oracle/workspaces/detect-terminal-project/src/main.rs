use ratatui::widgets::{Block, BorderType};

fn frame() -> Block<'static> {
    Block::default().border_type(BorderType::Double)
}

fn main() {
    let _ = frame();
}
