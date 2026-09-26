# Terminal platform

For apps whose primary surface renders in a terminal: full-screen TUIs and rich line-mode CLIs built with Ratatui, Bubble Tea and Lip Gloss, Textual and Rich, Ink, or their neighbors.

In a terminal the visitor mode narrows to Operate or Read almost every time. The grid, the user's theme, and the keyboard govern structure; brand expresses through the layer the terminal leaves open: one accent, weight and dim, spacing, copy.

## The terminal slop test

A terminal surface reads as generated when it opens with a figlet banner in a gradient, wraps every region in a double-line box nested inside another box, puts an emoji in front of every line, hard-codes RGB colors that vanish on a light theme, spins a braille spinner that never stops and keeps spinning into CI logs, assumes 80 by 24 and centers everything in it, and gives the user no clue which key quits. The tell is decoration with no information behind it. A designed terminal uses the user's 16 colors first, earns each glyph, keeps the footer honest about the keys that work right now, and looks the same in a pipe as on screen, minus the color.

## Layout & structure

- **Design at 80 columns, lay out by constraint.** Compute every region from the current size (`Frame::area()`, `WindowSizeMsg`, `useWindowSize`, Textual CSS) with percentage, minimum, and fill constraints. A fixed length or a literal rectangle is an absolute size that does not respond to the window.
- **Pick the screen mode the task deserves.** Alternate screen for a resident app (editor, monitor); inline under the prompt for a tool the user runs and returns from. The alternate screen throws away scrollback and history.
- **Handle resize as a first-class event.** The first size arrives at boot and resizes arrive in batches; rerun layout on each and never cache a width.
- **One archetype per app.** Miller columns for drill-down, a widget grid for monitoring, three panels (navigate, work, inspect) for editing. Do not mix them on one screen.
- **Borders are structure, not decoration.** One border weight per level, single-line by default, no box inside a box unless the inner one is focusable. The ASCII border set is the fallback, not an afterthought.
- **Leave the terminal as you found it.** Raw mode and the alternate screen are released on every exit path, including panic.

## Keyboard & input

- **Keyboard first, mouse optional.** Every action has a key. Mouse tracking is an opt-in that some connections cannot deliver, and there is no hover unless the mouse is bound.
- **The footer shows the 3 to 5 keys that work now.** The full reference sits behind `?`; the footer changes with focus.
- **Make escape obvious.** `q`, `Esc`, or `Ctrl-C` always works, and the footer says which.
- **Prompt only on a TTY, and never require the prompt.** Off a TTY, fail with the flag to pass instead.

## Typography

- **Monospace is the only face; weight, dim, and reverse are the whole type scale.** Hierarchy comes from bold, dim, spacing, and position, not from size.
- **Measure in cells, not characters.** Width comes from a width library (`unicode-width`, `go-runewidth`, `wcwidth`, `string-width`). CJK is two cells, combining marks zero, emoji sequences two, and terminals disagree at the edges.
- **Truncate on grapheme boundaries with an ellipsis, never on bytes.** A cut inside a joiner sequence or a surrogate pair corrupts the line and everything after it.
- **No figlet banners, no block-letter titles.** Concise help when run bare, the tool's name once in the header, nothing that scrolls the first screen away.
- **Glyphs need a fallback tier.** Box drawing and braille assume a capable font; Nerd Font icons are an opt-in set with an ASCII or emoji tier behind them.
- **Emoji only as an indicator, never as a bullet.** Emoji widths vary by terminal, color emoji ignore the theme, and a row of them hides the text it decorates.

## Color

- **The user's 16 ANSI colors are the palette; truecolor is an enhancement.** Named colors follow the terminal theme. RGB literals do not, and degrade to default text where 24-bit color is missing.
- **Every RGB choice needs a light-background twin.** Detect the background and pick (`AdaptiveColor`, `LightDark`, `HasDarkBackground`, a light theme), or compute contrast the way Textual does.
- **Honor the disable contract.** A non-empty `NO_COLOR`, `TERM=dumb`, `--no-color`, and a non-TTY stream each strip color. `FORCE_COLOR` re-enables it, and `NO_COLOR` wins over `FORCE_COLOR`.
- **Color is a resource.** One accent, semantic red, yellow, and green, dim for secondary. If everything is a color, nothing is.
- **Name colors by role, not by hex.** `accent`, `muted`, `error`, `focused-border` resolved per theme, so a theme swap is one table.
- **Green on black is a costume.** It fails on light themes and at small sizes; use the theme's foreground.

## Components & controls

- **Use the framework's primitives, not hand-rolled ones.** List, table, input, text area, file picker, pager, spinner, progress, and help exist in Bubbles, Textual, Rich, Ink, and gum. A custom one usually breaks on width or resize.
- **stdout is data, stderr is chrome.** Results go to stdout in a form `grep` can read; status, spinners, and prompts go to stderr. Offer `--plain` or `--json` when the pretty form would break a pipe.
- **Status bar reflects state, help reflects context.** Mode, path, and counts in one line; shortcuts listed only if they work here.

## Motion

- **Spinners tick on work and stop on completion.** Tick per unit processed so a hang is visible; replace the spinner with a past-tense line when done.
- **No animation off a TTY, and a static mode on one.** A spinner in a pipe becomes hundreds of log lines; a repainting status row re-homes a screen reader's cursor ten times a second. Offer `--no-progress` or a static indicator.
- **Print something within 100 ms; never blink.** Show the first frame before the network call. The blink attribute has no legitimate use.
- **Offer a line mode.** The append-only stream is what screen readers handle; a full-screen TUI ships a `--plain` twin or a static-chrome flag.

## Verifying the build

- **Capture, do not describe.** Run the app in a tmux pane and capture it with `tmux capture-pane -p -e -J -t <target>` (the `-e` keeps colors), or use the stack's own snapshot: Textual `--screenshot` and `snap_compare`, Ratatui `TestBackend` with insta, Ink `lastFrame()`, a VHS tape with `Set Columns` and `Set Rows`. Write the captures where the review flow expects them.
- **Run the matrix.** 80x24, 120x40, and 40 columns; a live resize; a dark and a light theme; `NO_COLOR=1`; `TERM=dumb`; stdout piped through `cat`; no Nerd Font; rows with CJK and emoji. Say which configurations produced the evidence.
- **Scan the pane.** With the app running in tmux, run `.qoder/skills/impeccable/scripts/impeccable detect --json --tmux <target> --tmux-sizes 80x24,120x40,40x24` (add `--palette light` for the light-theme pass). It captures the pane at each size, restores it, and reports the `tui-rt-` runtime rules: contrast, nested borders, truecolor on a 256-color client, width drift on emoji or CJK, missing key hints, narrow collapse, a spinner that never rests. A saved `tmux capture-pane -p -e -J` file replays with `--tmux-capture <file>`. Every `tui-rt-` finding is advisory.

Every `tui-` rule is advisory. The design hook drops advisory findings unless `.impeccable/config.json` sets `{"hook":{"advisoryRules":"include"}}`, so on a terminal project it shows nothing by default. Run `.qoder/skills/impeccable/scripts/impeccable detect --json <source directory>` to see them without that setting.
