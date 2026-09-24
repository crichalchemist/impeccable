use ratatui::widgets::{Block, Paragraph};

fn main() {
    let _ = Paragraph::new("fixture").block(Block::bordered());
}
