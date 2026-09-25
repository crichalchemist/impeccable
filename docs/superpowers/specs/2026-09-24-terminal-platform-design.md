# Terminal platform: design guidance, source rules, and a tmux engine

Date: 2026-09-24
Status: approved in conversation; PR 1 merged (ed600fd0) and PR 2 merged (86679d9d) on the fork; section 4's PR 3 table and section 5 expanded on 2026-09-24 for PR 3 planning
Fork: crichalchemist/impeccable, upstream pbakaus/impeccable

## Summary

Impeccable gains a fifth platform value, `terminal`, for full-screen terminal UIs and rich line-mode CLIs. The value follows the pattern `ios` and `android` set: a bare value in PRODUCT.md, a reference file inlined at boot, platform variants for `audit` and `adapt`, and doctor evidence that catches a mismatch. It differs from the mobile values in one deliberate way: `terminal` keeps the detector and the design hook active, because the engine learns to scan terminal source code and, later, a running terminal pane.

The work ships as three PRs on the fork, each merged to the fork's `main` as it lands:

1. **Guidance.** The platform value, the reference files, routing, init, doctor evidence.
2. **Source rules.** The walker learns `.rs`, `.go`, `.py`, and `.tcss` on terminal projects; eleven regex-class rules land in the registry; the hook watches the new extensions.
3. **tmux engine.** A third detector engine captures a running tmux pane and runs runtime rules over the rendered cell grid.

Every piece follows a pattern the repo already has, which is what keeps the fork mergeable upstream.

## Goals

- A terminal project gets terminal design guidance at boot, not web guidance.
- `impeccable detect` finds the anti-patterns AI-generated TUIs converge on, in Ratatui, Bubble Tea with Lip Gloss, Textual with Rich, and Ink.
- The review flow gets evidence from a real terminal, the way iOS gets Simulator screenshots.
- Built-in web output stays byte-identical, which the oracle enforces.

## Non-goals

- AST parsing of any language. Rules are regex families over comment-stripped text.
- Launching or killing the user's app. The tmux engine reads a pane the agent already started.
- Live mode for terminals. Live stays web-only.
- Extension or site changes. The extension runs rules in a browser and never sees terminal code.
- Design-system token matching for terminal palettes.
- Version bumps or changelog entries. The maintainer cuts releases.

## Vocabulary

The spec uses a closed set of terms so later plans can be checked against it.

- `platform` is the bare value in PRODUCT.md's `## Platform` section: `web`, `ios`, `android`, `adaptive`, or `terminal`.
- `terminal project` is a project whose resolved platform is `terminal`.
- `terminal extensions` are `.rs`, `.go`, `.py`, `.tcss`.
- `stack` is one of Ratatui (Rust), Bubble Tea and Lip Gloss (Go), Textual and Rich (Python), Ink (TypeScript and JavaScript).
- `source rule` is a check over file text. `runtime rule` is a check over a captured pane.
- `project signals` is the boolean set the walker collects before source rules run.
- `capture` is the output of `tmux capture-pane -p -e -J` parsed into a cell grid.

## Section 1: the platform value and boot

### Parsing

`extract_platform` in `crates/context/src/context.rs` accepts `terminal` as a fifth exact value. The token-list fallback stays limited to `ios` and `android`. A line such as `terminal, web` is unrecognized and produces the existing WARNING, whose valid-values sentence now names `terminal`.

### Boot output

`load_native_platform_references` in `crates/context/src/context_cli.rs` maps `terminal` to `reference/terminal.md`. The file is inlined as `# NATIVE PLATFORM REFERENCE: TERMINAL (reference/terminal.md)`, matching the iOS header shape. `RESOLVED_CONTEXT.platform` is `"terminal"`.

### Hook mode at boot

`automaticHookMode` returns `none` for `ios`, `android`, and `adaptive`, which also emits `MANUAL_DETECTOR_REQUIRED`. `terminal` is not added to that set. On a terminal project the hook stays active and the manual-detector directive reads as it does on web.

### Doctor evidence

`staleness.rs` gains a dependency table for terminal stacks:

| manifest | dependency | reason text |
|---|---|---|
| `Cargo.toml` | `ratatui`, `crossterm` | a ratatui dependency, a crossterm dependency |
| `go.mod` | `github.com/charmbracelet/bubbletea`, `github.com/charmbracelet/lipgloss` | a bubbletea dependency, a lipgloss dependency |
| `pyproject.toml`, `requirements.txt` | `textual`, `rich` | a textual dependency, a rich dependency |
| `package.json` | `ink` | an ink dependency |

A project resolving to `web` with that evidence gets the existing `platform-native-evidence` mention with suggested value `terminal`. A project declaring `terminal` skips the check, as `ios` does today. Workspace sweeps in `checkWorkspaces` use the same table.

### Init

`init.md` step 3's platform hypothesis lists `terminal`, with this disambiguation: an app whose primary surface renders in a terminal is `terminal`; a web app with a CLI installer is `web`. Step 4's bare-value list and the post-init reference load both name `terminal.md`.

### concept-seed

`--platform` accepts `terminal` next to `web`, `ios`, `android`. The validation message lists it. Compositions without a `platforms` entry already match every platform, so no catalog change is owed.

### Unchanged

Mode stays orthogonal. `terminal` does not force a mode, though the reference says most terminal surfaces resolve to Operate or Read.

### Oracle cases

`context-terminal`, `context-bad-platform` (updated valid-values text), `doctor-terminal-evidence-json`, `doctor-terminal-evidence-text`, `signals-terminal`, `seed-platform-terminal`.

## Section 2: skill routing and reference files

### `reference/terminal.md`

One file of roughly 50 to 60 lines, shaped like `ios.md`: an intro naming the four stacks, a slop-test paragraph, then sections with bold-lead bullets, each carrying a `<!-- rule:terminal-... -->` marker. The bullets come from the research report (appendix A) with source URLs removed from the shipped file.

**The terminal slop test.** A terminal surface reads as generated when it opens with a figlet banner in a gradient, wraps every region in a double-line box nested inside another box, puts an emoji in front of every line, hard-codes RGB colors that vanish on a light theme, spins a braille spinner that never stops and keeps spinning into CI logs, assumes 80 by 24 and centers everything in it, and gives the user no clue which key quits. The tell is decoration with no information behind it.

Sections and their bullets:

- **Layout & structure.** Design at 80 columns and lay out by constraint. Pick the screen mode the task deserves (alternate screen for a resident app, inline for a tool the user returns from). Handle resize as a first-class event. One archetype per app. Borders are structure, not decoration. Leave the terminal as you found it on every exit path, including panic.
- **Keyboard & input.** Keyboard first, mouse optional. The footer shows the 3 to 5 keys that work now. Make escape obvious. Prompt only on a TTY, and never require the prompt.
- **Typography.** Monospace is the only face; weight, dim, and reverse are the whole type scale. Measure in cells, not characters. Truncate on grapheme boundaries with an ellipsis. No figlet banners. Glyphs need a fallback tier. Emoji only as an indicator, never as a bullet.
- **Color.** The user's 16 ANSI colors are the palette; truecolor is an enhancement. Every RGB choice needs a light-background twin. Honor the disable contract (`NO_COLOR`, `TERM=dumb`, `--no-color`, non-TTY). One accent, semantic red, yellow, and green, dim for secondary. Name colors by role. Green on black is a costume.
- **Components & controls.** Use the framework's primitives. stdout is data, stderr is chrome. Status bar reflects state, help reflects context.
- **Motion.** Spinners tick on work and stop on completion. No animation off-TTY, and a static mode on-TTY. Print something within 100 ms; never blink. Offer a line mode for screen readers.
- **Verifying the build.** Capture, do not describe: run the app in tmux and call `impeccable detect --tmux <target>` (PR 3), or use the stack's own snapshot tool. Run the matrix: 80x24, 120x40, about 30 columns, a live resize, dark and light theme, `NO_COLOR=1`, `TERM=dumb`, piped stdout, no Nerd Font, CJK and emoji rows.

### Router table

The `audit` and `adapt` rows in `SKILL.src.md` gain a third link: `terminal: [reference/audit.terminal.md]` and `terminal: [reference/adapt.terminal.md]`. Setup step 2's phrase "native variant on native platforms" becomes "platform variant when the Commands table lists one for the resolved platform". The root guide's variant rule extends to: `.native.md` for `ios`, `android`, `adaptive`; `.terminal.md` for `terminal`.

### `audit.terminal.md`

Mirrors `audit.md`'s report skeleton. The root guide requires the skeleton to change in every variant together, so a skeleton edit now touches three files. Its checks: color contrast under a dark and a light default palette; behavior at 80x24 and at 40 columns; keyboard-only completion of every task; `NO_COLOR` and `TERM=dumb` degradation; availability of a plain or line mode; time to first frame.

### `adapt.terminal.md`

Adapt on the web means devices; on a terminal it means size classes and capability tiers. It defines three widths (narrow under 60 columns, standard 80 to 119, wide 120 and above) and three color tiers (16, 256, truecolor) and asks for a layout decision at each.

### Guards and routing

- `audit.md` and `adapt.md` carry a one-line web-only guard that redirects native readers. The line names the terminal variant too.
- `animate` and `layout` get no variant, matching the native decision.
- `routing.md`'s rule that skips live and `impeccable detect` for native platforms splits: live stays skipped for `terminal`; detect runs.
- `SKILL.src.md`'s `## Modes` section gains one sentence: terminal surfaces usually resolve to Operate or Read.
- The skill description frontmatter adds "terminal UI" and "TUI" so auto-trigger matches.

### Checks owed

`bun run build` (prose gates, no em dashes), `tests/skill-reference.test.mjs`, and one `bun run test:skill-behavior` run because Setup-adjacent text changes.

## Section 3: the detector engine

### Where the platform comes from

`crates/detect` does not depend on `crates/context` and must not start to. `crates/cli` depends on both. `Engines`, the struct the binary hands to `run_detect`, gains a `platform` resolver: a closure the binary builds from `impeccable_context`'s PRODUCT.md lookup, the same resolver the hook uses. A new `--platform <value>` flag on `detect` overrides it, for CI, tests, and the oracle. `--platform terminal` on a directory with no PRODUCT.md is how fixtures run.

### What the walker scans

`SCANNABLE_EXTENSIONS` is unchanged for web. A second list, `TERMINAL_EXTENSIONS`, holds the terminal extensions and is consulted only on a terminal project, where the union is scanned: Ink projects are `.tsx`, and a Textual project may ship a docs site. `SKIP_DIRS` gains `target`, `.venv`, and `vendor`. Explicit file arguments bypass the extension gate today and keep doing so.

### Which rules run

Registry rows gain a field `platforms: Option<&'static [&'static str]>`. `None` means web, as every existing row is. Terminal rules declare `Some(&["terminal"])`. The text engine filters by resolved platform before running: a web project never runs a `tui-` rule; a terminal project never runs `overused-font` over its `.rs` files. Web rules still run over a terminal project's `.html` files.

Rules keep `category: "slop"` or `"quality"`. There is no new category; the `platforms` field alone marks terminal rules, so the site's counts do not split.

### Project signals

Half the reliable rules are of the form "X present and Y absent anywhere in the project". Before source rules run, the walker collects a boolean set over every scanned file:

| signal | any file matches |
|---|---|
| `has_tty_guard` | `isatty`, `isTTY`, `is_terminal`, `IsTerminal`, `--no-progress`, `CI` env read |
| `has_adaptive_color` | `AdaptiveColor`, `LightDark`, `HasDarkBackground`, `ansi_color`, `NO_COLOR` |
| `has_width_lib` | `unicode-width`, `unicode-segmentation`, `runewidth`, `uniseg`, `ansi.Truncate`, `string-width`, `wcwidth`, `grapheme` |
| `has_icon_toggle` | `ascii`, `nerd`, `icons`, `--no-icons` as an option name |
| `has_plain_flag` | `--plain`, `--json`, `--no-tui`, `--static`, `NO_TUI` |

Absence rules consult the set. In a single-file scan the set is empty, and those rules report at `advisory` with the note "project signals were not collected; scan the project directory to confirm".

### Where the checks live

A new module `crates/core/src/checks/terminal.rs` holds one `scan_terminal_<rule>` function per rule. Each is a pure function over file text and the project signals, returning the existing `Finding` shape with line and column. They follow `css_scan.rs`: regex families keyed by extension, built with the `re!` macro. Comment stripping runs per language before matching (`//` and `/* */` for Rust and Go, `#` and docstrings for Python, the existing JS stripper for Ink) so a commented-out `Color::Rgb` is not a hit.

Framework detection is per file, and a file that fails it gets no terminal rules:

| extension | treated as terminal source when the file names |
|---|---|
| `.rs` | `ratatui` or `crossterm` |
| `.go` | a `charmbracelet` import |
| `.py` | `textual` or `rich` |
| `.tcss` | always (Textual CSS only exists in Textual apps) |
| `.tsx`, `.jsx`, `.ts`, `.js` | `ink` |

This keeps Bevy games and Django backends out.

### Registry and bundle

New rows go at the end of `ANTIPATTERNS` so built-in web output stays byte-identical. `cargo xtask bundle` regenerates `crates/live/assets/antipatterns.json` with the new rows. The wasm bundle grows by the module but the browser adapters never call it. `generateCounts` fails the README rule count until it is updated.

### Hook

`hook_lib.rs`'s native skip does not gain `terminal`. Its watched-extension list becomes platform-aware: the terminal extensions are added when the resolved platform is `terminal`. The per-edit immediate tier runs the cheap regex rules (figlet, gradient, blink, double border, raw RGB); the Stop deep pass runs all of them with project signals. Shape is otherwise unchanged.

### Inline ignores

`impeccable-disable` comments work per line already. The comment-syntax table gains `//` for Rust and Go and `#` for Python.

### Oracle cases

One `detect-terminal-<rule>` golden per rule against `tests/fixtures/antipatterns/terminal/<rule>.<ext>`, one fixture per stack the rule covers; `detect-terminal-gate-web` proving a web project containing `.rs` files scans nothing new; `detect-terminal-single-file-advisory` proving the absence-rule downgrade; `hook-terminal-platform` proving the hook runs on a terminal project.

## Section 4: the rule catalog

### PR 2 source rules

All ship at severity `advisory`. Promotion to `warning` follows a manual pass over one real project per stack (section 5).

| id | category | signal | applies to |
|---|---|---|---|
| `tui-figlet-banner` | slop | imports or uses of `figlet`, `pyfiglet`, `figlet_rs`, `go-figure`, `cfonts`, `ink-big-text`, `text2art`, `from art import`; or 4+ consecutive string literals mostly block glyphs `[█▀▄╔═╗║╚╝]` | all stacks |
| `tui-gradient-title` | slop | `gradient-string`, `ink-gradient`, `<Gradient`, `rich_gradient`, `Gradient(`, `lolcat`, `chromacat`, `tui-gradient`, `lipgloss.Blend1D(` or `Blend(` fed into a title style | all stacks |
| `tui-blink-attribute` | slop | `Modifier::SLOW_BLINK`, `RAPID_BLINK`, `.Blink(`, Rich `[blink]`, Textual `text-style: blink`, `\x1b[5m` | all stacks |
| `tui-emoji-density` | slop | emoji codepoints (`U+1F300` to `U+1FAFF`, `U+2600` to `U+27BF`) inside string literals above a per-file threshold of 8, or 3+ from the canonical set 🚀 ✅ ❌ ⚠️ ✨ 🎉 📦 🔧 🔥 💡 | all stacks |
| `tui-double-border` | slop | `BorderType::Double`, `DoubleBorder()`, `borderStyle="double"`, `box.DOUBLE`, Textual `border: double` | all stacks |
| `tui-hardcoded-rgb-no-adapt` | slop | `Color::Rgb(`, `Color::from_u32(0x`, `lipgloss.Color("#`, `chalk.hex(`, Ink `color="#`, Rich `[#rrggbb]`, `\x1b[38;2;`; reported when `has_adaptive_color` is false, advisory-with-note otherwise | all stacks; weak for `.tcss`, which is excluded |
| `tui-spinner-no-tty-guard` | quality | `ink-spinner`, `ora(`, `rich.spinner`, `bubbles/spinner`, `briandowns/spinner` with `has_tty_guard` false | all stacks |
| `tui-hardcoded-size` | quality | `Rect::new(` with four integer literals in non-test code; a `Layout` whose constraints are all `Constraint::Length`; `(width|cols|columns) [:=] 80`, `(height|rows|lines) [:=] 24`; `.Width(80)`; `width={80}`; `size=(80, 24)`; outside paths containing `test` | all stacks |
| `tui-grapheme-unsafe-truncate` | quality | `&s[..n]`, `.chars().take(n)`, Go `s[:n]`, JS `.slice(0, n)` or `.substring(0,`, Python `[:n]`, each followed within the statement by an ellipsis literal, with `has_width_lib` false | Rust, Go, Ink, raw Python; skipped for Rich and Textual files, which measure for the caller |
| `tui-nerd-glyph-no-fallback` | quality | string literals containing private-use codepoints (`U+E000` to `U+F8FF`, `U+F0000` to `U+FFFFD`) with `has_icon_toggle` false | all stacks |
| `tui-print-in-loop` | quality | `print(` inside a Textual `App` or `Widget` method that is not a worker; `console.log` in an Ink component file that does not import or call `patchConsole` | Textual, Ink |

Deferred, with the reason:

- `tui-nested-borders`: needs structural analysis of render functions. The runtime rule in PR 3 covers it from geometry.
- `tui-no-key-hints`, `tui-no-plain-mode`, `tui-mouse-only`: absence-based with a wide legitimate space; revisit after project signals prove out.
- `tui-center-everything`, `tui-green-on-black`, `tui-rich-color-sprawl`, `tui-decorative-corners`, `tui-triple-encoded-status`: low or medium confidence as slop signals.
- `tui-interval-never-cleared`, `tui-clear-or-altscreen-oneshot`: medium confidence, need a program-level view of stdout use.
- `tui-typed-command-not-selectable`: a web rule about fake terminal components on marketing pages. Out of this spec.

### PR 3 runtime rules

Each runtime rule is the capture-side cousin of a source rule, so a reader learns one vocabulary. Ids carry `tui-rt-`. The `frames` column uses section 5's vocabulary: which captured frames a rule reads.

| id | category | frames | signal over the capture |
|---|---|---|---|
| `tui-rt-low-contrast` | quality | first | a foreground and background pair below 4.5:1 on the palette, over cells whose glyph is alphanumeric; one finding per distinct pair at its first cell, with `cellCount` in extras |
| `tui-rt-nested-borders` | slop | every capture frame | a closed box-drawing rectangle whose nearest enclosing rectangle has only blank cells between the two borders; corners are `┌┐└┘╭╮╰╯╔╗╚╝┏┓┗┛`, edges may carry tees, and the top edge may carry a title |
| `tui-rt-truecolor-on-256` | quality | first | a cell colored by `38;2` or `48;2` while `termfeatures` is known, lacks `RGB`, and `colorterm` is neither `truecolor` nor `24bit`; one finding per frame |
| `tui-rt-width-drift` | quality | every capture frame | a row holding a wide (two-cell) glyph that measures wider than the pane, or whose last vertical border sits on a different column than the column most rows end on |
| `tui-rt-no-key-hints` | quality | first | the last non-blank row matches no key token (`q`, `Esc`, `Enter`, `Tab`, `Space`, `Ctrl`, `Alt`, `Shift`, `F1` to `F12`, `h j k l`, `?`, arrows, `<x>`, `[x]`, `^X`); silent when the frame has fewer than three non-blank rows |
| `tui-rt-collapse-narrow` | quality | capture frames under 60 columns | a row wider than the pane; a row with a left corner and no right corner; a row that fills the pane, ends in a letter, and is followed by a row starting with one |
| `tui-rt-spinner-never-rests` | quality | first against the recapture | a cell whose glyph is in the spinner set (braille U+2800 to U+28FF, `|/-\`, quarter circles, block bars) in both frames and differs between them; one finding |

All seven ship at `advisory` with `platforms: Some(&["terminal"])`, appended after `tui-print-in-loop`, so `antipatterns.json` grows from 72 to 79 rows and web output stays byte-identical. They never run in the hook or the text engine; only `--tmux` and `--tmux-capture` reach them, on any resolved platform, because the flag itself is the request.

## Section 5: the tmux engine

Expanded on 2026-09-24 while planning PR 3, after PR 1 and PR 2 landed. The original text named the shape; this version fixes the vocabulary, the file format, the flags, the messages, and where every piece lives, so the plan can be checked against it.

### Vocabulary

- `frame` is one parsed capture: a cell grid plus width, height, `termfeatures`, `colorterm`, and a role.
- `capture frame` is a frame taken at a size the run asked for: the pane's own size first, then each `--tmux-sizes` entry.
- `recapture` is the second frame of the pane's own size, taken one second after the first with no input in between. Only `tui-rt-spinner-never-rests` reads it.
- `header` is the `#!capture` line that introduces a frame in a saved capture file.
- `palette` is the reference mapping from SGR colors to sRGB used for contrast: `dark` (default) or `light`.

### Shape

A new crate `crates/terminal` (`impeccable-terminal`) mirroring `crates/browser`: it depends on `impeccable-core` (findings, registry, color) and `impeccable-detect` (the engine seam) and adds one external crate, `unicode-width`, for cell widths. It holds four modules: `capture` (the SGR parser and the frame model), `palette` (ANSI 16, the 256-color cube and grays, truecolor, the two reference palettes), `rules` (one function per runtime rule over frames), and `tmux` (locating the executable, the version check, pane geometry, capture, resize, restore). The runtime rules live in this crate, not in `crates/core`: nothing else consumes them, and the wasm bundle must not carry tmux code. The registry rows still live in `crates/foundation`, so `cargo xtask bundle` lists them in `antipatterns.json`.

`crates/detect` gains the seam: `trait TmuxEngine { detect_pane(target, &ScanOptions); detect_capture(path, &ScanOptions) }`, `Engines.tmux: Option<&dyn TmuxEngine>`, a `MissingTmuxEngine` for builds without the crate, and three `ScanOptions` fields (`tmux_sizes: Vec<(u32, u32)>`, `tmux_settle_ms: Option<u64>`, `palette: Option<String>`). `crates/cli` wires `impeccable_terminal::TerminalEngine::from_process_env()` the way it wires the browser engine.

### Input is a pane or a saved capture

`impeccable detect --tmux <session:window.pane>` captures an already running app. The engine never launches or kills a process, and it never leaves a window resized; the agent starts the app in tmux following `terminal.md`'s verification section. The engine talks to whatever server the `tmux` client resolves: the socket in `$TMUX` inside a session, else the default socket. `IMPECCABLE_TMUX` names the executable when it is not on `PATH`, the override pattern `IMPECCABLE_BROWSER` set.

`impeccable detect --tmux-capture <file>` replays a saved capture without tmux. It exists for the oracle, for CI, and for the review flow: an agent that saved `tmux capture-pane -p -e -J` output while following `terminal.md` can scan the file later. Both flags are repeatable and combine with file targets; when either is present and no file target is given, the run does not fall back to scanning the working directory.

### Capture and the frame model

`tmux capture-pane -p -e -J -t <target>` returns the visible rows with SGR sequences preserved and trailing blanks trimmed (tmux's default). The parser walks each row: an SGR sequence updates the current style (parameters 0, 1, 2, 5, 7, 22, 25, 27, 30 to 37, 39, 40 to 47, 49, 90 to 97, 100 to 107, `38;5;n`, `48;5;n`, `38;2;r;g;b`, `48;2;r;g;b`; anything else is ignored); every other character becomes a cell carrying the style. A character `unicode-width` measures at two takes two cells, the second marked as a wide tail; a zero-width character (combining marks, ZWJ, variation selectors) takes none. Row width is the cell count. The parser is a pure function over text, so goldens replay from fixture files.

A saved capture file holds one or more frames. Each frame begins with a header:

    #!capture width=80 height=24 termfeatures=256,RGB colorterm=truecolor role=capture

Every key is optional (`role` is `capture` or `recapture`; an unknown key, an unknown role, or a non-numeric size is an error). A file without a header is one capture frame whose width is its widest row and whose height is its row count, with unknown `termfeatures`. Pane scans fill the same fields from `tmux display-message -p -t <target>` (`#{pane_width}`, `#{pane_height}`, `#{window_width}`, `#{window_height}`, `#{client_termfeatures}`) and from `COLORTERM` in the engine's environment. `termfeatures` is empty when no client is attached to the session; the rule that reads it stays silent then.

### Multi-size pass

A pane scan takes frames in this order: the pane's own size; the recapture one second later; then, for each `--tmux-sizes WxH`, `tmux resize-window -t <target> -x W -y H`, a settle of `--tmux-settle` milliseconds (default 300), and a capture with the pane geometry read again. After the sizes the engine runs `resize-window` back to the recorded window size and `set-option -w -t <target> -u window-size`, so the window follows its clients again instead of keeping the `manual` size `resize-window` set. The restore runs even when a capture failed. `adapt.terminal.md` gets evidence for its three width classes in one command.

### Rules over frames

Size-invariant rules read the first frame only: `tui-rt-low-contrast`, `tui-rt-truecolor-on-256`, `tui-rt-no-key-hints`. `tui-rt-spinner-never-rests` compares the first frame with the recapture. Geometry rules run on every capture frame and never on the recapture: `tui-rt-nested-borders`, `tui-rt-width-drift`, and, on frames narrower than 60 columns (the Narrow class in `adapt.terminal.md`), `tui-rt-collapse-narrow`. Section 4's PR 3 table carries the exact signals.

### Palette

`--palette dark` (default) and `--palette light` differ only in the default foreground and background: white on black, black on white. Named colors 0 to 15 use xterm's table in both, the 256-color cube and grays use their standard formula, and truecolor is taken as written. `reverse` swaps the pair; `bold` and `dim` do not change the color. Contrast is WCAG 2.x through `impeccable_core::color::contrast_ratio`, floor 4.5:1, over cells whose glyph is alphanumeric, so borders and decorations never count as text.

### Output

Findings use the existing `Finding` shape: `file` is `tmux:<target>` for a pane and the resolved path for a saved capture; `line` is the 1-based row; `extras` carry `column` (1-based cell) and `frame` (`WxH`), and `tui-rt-low-contrast` adds `cellCount`. `--json` prints the same array `detect` prints for files and URLs; text mode prints `line N: [id] snippet`. Every runtime rule ships at `advisory`, so a pane scan exits 0 unless a target failed. `ignoreRules`, `--scope`, and `--no-advisory` apply as for every other finding.

### Flags and messages

| flag | value | error (stderr, exit 1) |
|---|---|---|
| `--tmux <target>` | a tmux target | `Error: --tmux requires a tmux target, e.g. --tmux app:0.0` |
| `--tmux-capture <file>` | a saved capture | `Error: --tmux-capture requires a path to a saved capture` |
| `--tmux-sizes <list>` | comma-separated `WxH`, each side one to four digits | `Error: --tmux-sizes requires comma-separated WxH values, e.g. --tmux-sizes 80x24,120x40,40x24` |
| `--tmux-settle <ms>` | whole milliseconds | `Error: --tmux-settle requires a whole number of milliseconds` |
| `--palette <name>` | `dark` or `light` | `Error: --palette requires dark or light` |

Values are spliced out of the argument list like `--viewport`'s, so they never become targets; the `--flag=value` form works for all five.

### Requirements and degradation

tmux 3.2 or newer (`resize-window -x -y` and `client_termfeatures` both need it). The engine locates tmux through `IMPECCABLE_TMUX`, else `PATH`, and checks `tmux -V`. Without tmux, `--tmux` prints one line, `Error: tmux 3.2 or newer is required for --tmux and was not found on PATH. Install tmux, or point IMPECCABLE_TMUX at the executable.`, sets the operational failure, and the run exits 1 the way a URL scan with no browser does. An older tmux prints `Error: tmux 3.2 or newer is required for --tmux; found <tmux -V output>`. A target tmux cannot find surfaces tmux's own first stderr line as `Error: tmux list-panes: <line>` (the engine checks the target with `list-panes -t <target>` before reading geometry, because `display-message -t` on tmux 3.7 silently resolves a missing target to another pane). A `--tmux-capture` file that cannot be read is reported as `Error: cannot scan <file>: <ENOENT or EACCES message>`, like any local file. `--tmux-capture` needs no tmux at all.

### Oracle and tests

Capture fixtures under `tests/fixtures/terminal-captures/<rule>.txt`, one per runtime rule plus `sizes-pass.txt` (three well-behaved frames that produce `[]`), are inputs written by hand with headers; the goldens `detect-tmux-capture-<name>` are recorded from the binary with `--tmux-capture`. Further cases pin the text renderer, `--palette light` on the contrast fixture, a missing capture file, a missing tmux binary (the case runs with `PATH` pointing at a directory that does not exist and no `IMPECCABLE_TMUX`), and each flag's error. `detect-help` is re-recorded for the new usage lines; `context-terminal` moves because `terminal.md` gains a bullet.

One integration test, `tests/tmux-engine.test.mjs`, starts a scratch server (`tmux -L impeccable-test-<pid>`) running `tests/fixtures/terminal-apps/nested-boxes.sh`, points the engine at it through an `IMPECCABLE_TMUX` wrapper script that adds `-L`, runs `--tmux app:0.0 --tmux-sizes 40x24`, asserts the expected rule ids and that the window is back at 80x24, and kills the server in `after()`. It skips without a binary or without tmux. It is registered in the `oracle` suite of `scripts/test-suites.mjs` (a second command, triggered by `crates/terminal/` and the two fixture directories) and runs in CI's oracle job after the golden replay.

## Section 6: testing, documentation, delivery

### PR 1, guidance

Files: `crates/context` (parser, boot inlining, staleness evidence), `skill/SKILL.src.md`, `skill/reference/terminal.md`, `audit.terminal.md`, `adapt.terminal.md`, `init.md`, `routing.md`, the concept-seed flag, `docs/CLI-CONTRACT.md`, the root `CLAUDE.md` Platform section.

Gates: `cargo test --workspace`; the section 1 oracle cases recorded with `node tests/oracle/record.mjs --bin` and reviewed by hand; `bun run build`; `bun run test`; one `bun run test:skill-behavior` run.

### PR 2, source rules

Files: `crates/foundation/src/registry.rs` (field and rows), `crates/core/src/checks/terminal.rs`, `crates/detect` (walker list, platform gate, project signals, `--platform`), `crates/hook` (platform-aware extensions), `crates/cli` (resolver wiring), `tests/fixtures/antipatterns/terminal/`, the two tracked assets from `cargo xtask bundle`, `README.md` and `README.npm.md` rule counts, `docs/CLI-CONTRACT.md`.

Gates: `cargo test --workspace`; `cargo xtask bundle --check`; the section 3 oracle cases; `bun run build` with corrected counts; `bun run test`; a manual `impeccable detect --platform terminal` over one real project per stack, with the false-positive rate recorded in the PR before any rule is promoted from `advisory`.

### PR 3, tmux engine

Files: `crates/terminal` (new crate, four modules), the root `Cargo.toml` (workspace dependency), `crates/detect` (`TmuxEngine`, `MissingTmuxEngine`, `Engines.tmux`, three `ScanOptions` fields, the five flags, `scan_tmux_targets`, USAGE), `crates/cli` (wiring and dependency), `crates/foundation/src/registry.rs` (seven rows), the two tracked assets from `cargo xtask bundle`, `tests/fixtures/terminal-captures/`, `tests/fixtures/terminal-apps/nested-boxes.sh`, `tests/tmux-engine.test.mjs`, `scripts/test-suites.mjs`, `.github/workflows/ci.yml` (one step in the oracle job), `tests/oracle/cases/detect.mjs`, goldens and `DELTAS.md`, `skill/reference/terminal.md` (a verification bullet), `audit.terminal.md`, `adapt.terminal.md`, `README.md` and `README.npm.md` counts (72 to 79), `docs/CLI-CONTRACT.md`, `docs/ENGINE.md`, the root `CLAUDE.md`.

Gates: as PR 2 plus `node --test tests/tmux-engine.test.mjs` with tmux installed, and a manual `--tmux` pass over real terminal programs (a Ratatui example, `vim`, `top`, `less`, plus any Bubble Tea or Textual app that is installed) recorded before any runtime rule leaves `advisory`.

### Test naming

Tests name outcomes: `web_project_with_rust_backend_gets_no_terminal_rules`, `terminal_platform_keeps_hook_active`, `hardcoded_truecolor_flagged_only_in_tui_files`, `single_file_scan_downgrades_absence_rules`, `capture_parser_replays_without_tmux`.

### Fork and upstream process

Each PR lands on the fork first: branch, review, merge to `crichalchemist/impeccable` `main`. The fork owner plans to test the feature in real use once complete, so `main` carries it as soon as each PR merges.

Upstream is issue-first. Before the first upstream PR, the fork owner opens an issue on pbakaus/impeccable proposing the `terminal` platform and the three-PR shape. PRs go upstream only after a maintainer approves in that issue. AI assistance is disclosed in every commit and PR body. Claude drafts issue and PR text but never posts to the upstream repository, per the root guide's contribution rules.

### Versioning

No manifest bumps and no changelog entries in any PR. PR 2 and PR 3 change engine behavior and will need an `ENGINE_VERSION` bump at release, which follows the engine-first release order.

## Open questions

PR 2's two calibration questions were settled by its manual pass (`docs/superpowers/research/2026-09-24-terminal-rules-manual-pass.md`): the emoji thresholds stay at 8 and 3, and `tui-hardcoded-size` is marked tighten-before-promotion rather than exempting `examples/`.

Two questions are deferred to PR 3's manual pass. Whether one second is the right recapture window for `tui-rt-spinner-never-rests`: a spinner over real pending work is legitimate and will fire, which is why the rule is advisory. And whether `client_termfeatures` is populated when the scan runs from outside the session; if it is not, `tui-rt-truecolor-on-256` fires only on saved captures whose header names the features.

## Known gaps after PR 1 and PR 2

Recorded here so they are not lost; none is in PR 3's scope.

- Skill text: native-only branches in `SKILL.src.md`, `polish.md`, and `new-work.md` that `terminal` falls through; `live.md` has no terminal guard; `audit.terminal.md` scores 60 columns where this spec said 40; `adapt.terminal.md` leaves a 60 to 79 column gap; concept-seed does not forward `platform=terminal` to the private roll API.
- Init evidence: `// indirect` go.mod lines count as evidence; the matchers miss pyproject inline arrays, Cargo dotted keys and table headers, and capitalized PyPI names; `rich` or `crossterm` alone is weak evidence; `staleness.rs` misses Charm v2 `charm.land/...` module paths.
- Source rules: `tui-print-in-loop`'s Ink premise was inverted in the original section 4 row (Ink's default is `patchConsole: true`, so PR 2 flags only `patchConsole: false`), and its Textual half is unverified; `ora` and Rich `Live` and `Status` self-guard on a non-TTY, so `tui-spinner-no-tty-guard` likely over-reports them; the emoji range U+2600 to U+27BF catches dingbats and the threshold is `>= 8` where the row says "above 8"; `chalk.hex(` with a computed argument is missed; `tui-hardcoded-size` needs tightening before promotion.
- Routing: `crates/context/src/signals.rs` `scan_targets` is not platform-aware, so routing's detect run skips terminal source.
- Process: a CLI driver (headless agents with stream-json) for the skill-behavior suite is its own brainstorm.

## Known gaps after PR 3

Recorded from the PR 3 final review and the manual pass (`docs/superpowers/research/2026-09-24-terminal-runtime-rules-manual-pass.md`). None is fixed in PR 3; each is a candidate for the next spec amendment.

- `tui-rt-collapse-narrow` and `tui-rt-width-drift` both fire on a row wider than a pane under 60 columns, so a 40-column frame reports every overflow row twice.
- `capture-pane -J` joins soft-wrapped screen rows into one logical row before the parser sees them; the manual pass classed 13 of 15 collapse-narrow findings as this join. Capturing without `-J` keeps screen rows but turns an app's own soft wrap into an overflow at every boundary. The rule stays advisory until one of the two is chosen.
- `tui-rt-spinner-never-rests` needs the glyph to differ between frame 0 and the recapture one second later. An animation whose period divides one second (ten glyphs at 100 ms) aliases and is missed; a second recapture at an offset such as 700 ms would close the gap.
- tmux calls have no timeout, so a wedged server hangs `--tmux`. A SIGINT or SIGTERM during the recapture or the size pass leaves the window at the last requested size in `manual` mode.
- Restore runs `set-option -w -u window-size`, which also drops a `window-size` the user had set on that window before the scan. Restoring the earlier value needs `show-options -w -v window-size` first.
- Section 5 says trailing blanks are trimmed; the tmux manual says `-J` preserves trailing spaces. The rules tolerate padded rows, so only the sentence needs correcting.
- `unicode-width` 0.2 and tmux can disagree on the width of some emoji (VS16 sequences, recent additions), which makes a correct row measure one cell off and can trip `tui-rt-width-drift`.
- `client_termfeatures` is empty on a detached server, so `tui-rt-truecolor-on-256` is silent unless a client is attached to the scanned session.
- Four rules (`tui-rt-width-drift`, `tui-rt-nested-borders`, `tui-rt-truecolor-on-256`, `tui-rt-spinner-never-rests`) are unmeasured on real programs; `tui-rt-no-key-hints` assumes the bottom row is a footer and misfires on scrolling tables and bare pager prompts.

## Appendix A: research report

The sourced research this spec distills is `docs/superpowers/research/2026-09-24-terminal-design-research.md` (72 sources). Its distillation section is the origin of `terminal.md`'s bullets and the rule table.
