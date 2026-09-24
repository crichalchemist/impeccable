use ratatui::style::{Modifier, Style};
fn hot() -> Style { Style::default().add_modifier(Modifier::SLOW_BLINK) } // flag
fn calm() -> Style { Style::default().add_modifier(Modifier::BOLD) } // pass
