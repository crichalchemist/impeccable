use ratatui::prelude::*;
const STATUS: [&str; 4] = ["🚀 start", "✅ done", "❌ failed", "⚠ retry"]; // flag: canonical set
const PLAIN: [&str; 2] = ["ok", "err"]; // pass
const MARKS: [&str; 8] = ["✓", "✗", "★", "☐", "➜", "☆", "✔", "✘"]; // pass: text-presentation marks are not emoji
