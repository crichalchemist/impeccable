//! impeccable-terminal: the tmux engine of `impeccable detect` (spec section
//! 5). Captures a running tmux pane, or replays a saved capture, parses it
//! into frames, and runs the `tui-rt-` runtime rules. Wired into the binary
//! through [`impeccable_detect::engines::TmuxEngine`]; the engine never
//! launches or kills a process and always restores a window size it changed.

pub mod capture;
pub mod palette;
