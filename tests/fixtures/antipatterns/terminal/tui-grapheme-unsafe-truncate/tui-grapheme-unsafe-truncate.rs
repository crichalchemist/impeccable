use ratatui::prelude::*;
fn short(title: &str) -> String { format!("{}...", &title[..20]) } // flag: no width library in this directory
fn head(buf: &[u8]) -> &[u8] { &buf[..4] } // pass: no ellipsis
