use figlet_rs::FIGfont; // flag: import
use ratatui::prelude::*;
const BANNER: [&str; 4] = [ // flag: four block-glyph rows
    "███╗   ███╗██╗   ██╗",
    "████╗ ████║██║   ██║",
    "██╔████╔██║██║   ██║",
    "██║╚██╔╝██║╚██████╔╝",
];
const TITLE: &str = "mu"; // pass: plain text title
