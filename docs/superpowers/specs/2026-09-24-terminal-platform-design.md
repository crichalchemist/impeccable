# Terminal platform: design guidance, source rules, and a tmux engine

Date: 2026-09-24
Status: approved in conversation; PR 1 merged (ed600fd0) and PR 2 merged (86679d9d) on the fork; section 4's PR 3 table and section 5 expanded on 2026-09-24 for PR 3 planning; PR 3 merged (81c2d17d); section 7 added on 2026-09-25 for follow-up PRs 4 to 7, with sections 2, 4, and 5 corrected in place where a follow-up changes them
Fork: crichalchemist/impeccable, upstream pbakaus/impeccable

## Summary

Impeccable gains a fifth platform value, `terminal`, for full-screen terminal UIs and rich line-mode CLIs. The value follows the pattern `ios` and `android` set: a bare value in PRODUCT.md, a reference file inlined at boot, platform variants for `audit` and `adapt`, and doctor evidence that catches a mismatch. It differs from the mobile values in one deliberate way: `terminal` keeps the detector and the design hook active, because the engine learns to scan terminal source code and, later, a running terminal pane.

The work ships as three PRs on the fork, each merged to the fork's `main` as it lands:

1. **Guidance.** The platform value, the reference files, routing, init, doctor evidence.
2. **Source rules.** The walker learns `.rs`, `.go`, `.py`, and `.tcss` on terminal projects; eleven regex-class rules land in the registry; the hook watches the new extensions.
3. **tmux engine.** A third detector engine captures a running tmux pane and runs runtime rules over the rendered cell grid.

Four follow-up PRs (section 7, added 2026-09-25) close the gaps the first three left: runtime engine hardening, source rule tightening, init evidence and routing targets, and skill text.

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

`automaticHookMode` returns `none` for `ios`, `android`, and `adaptive`, which also emits `MANUAL_DETECTOR_REQUIRED`. `terminal` is not added to that set. On a terminal project the hook stays active. When no hook is installed, the manual-detector directive says "terminal UI" and points the detector at its source (PR 6).

### Doctor evidence

`staleness.rs` gains a dependency table for terminal stacks:

| manifest | dependency | reason text |
|---|---|---|
| `Cargo.toml` | `ratatui` (PR 6 dropped `crossterm`) | a ratatui dependency |
| `go.mod` | `bubbletea`, `lipgloss`, and (PR 6) `bubbles`, under `github.com/charmbracelet/` or (PR 6) `charm.land/` | a bubbletea dependency, a lipgloss dependency, a bubbles dependency |
| `pyproject.toml`, `requirements.txt` | `textual` (PR 6 dropped `rich`) | a textual dependency |
| `package.json` | `ink` | an ink dependency |

PR 6 also counts only direct `require` entries in go.mod (no `// indirect`, nothing in `replace`, `exclude`, or `retract` blocks).

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
- **Verifying the build.** Capture, do not describe: run the app in tmux and call `impeccable detect --tmux <target>` (PR 3), or use the stack's own snapshot tool. Run the matrix: 80x24, 120x40, 40 columns (PR 7; this line first said about 30, while every capture command passes `40x24`), a live resize, dark and light theme, `NO_COLOR=1`, `TERM=dumb`, piped stdout, no Nerd Font, CJK and emoji rows.

### Router table

The `audit` and `adapt` rows in `SKILL.src.md` gain a third link: `terminal: [reference/audit.terminal.md]` and `terminal: [reference/adapt.terminal.md]`. Setup step 2's phrase "native variant on native platforms" becomes "platform variant when the Commands table lists one for the resolved platform". The root guide's variant rule extends to: `.native.md` for `ios`, `android`, `adaptive`; `.terminal.md` for `terminal`.

### `audit.terminal.md`

Mirrors `audit.md`'s report skeleton. The root guide requires the skeleton to change in every variant together, so a skeleton edit now touches three files. Its checks: color contrast under a dark and a light default palette; behavior at 80x24 and under 60 columns, probed at 40x24 (corrected 2026-09-25: this line first said 40 columns, which neither the shipped file nor the engine used); keyboard-only completion of every task; `NO_COLOR` and `TERM=dumb` degradation; availability of a plain or line mode; time to first frame.

### `adapt.terminal.md`

Adapt on the web means devices; on a terminal it means size classes and capability tiers. It defines four widths (narrow under 60 columns, compact 60 to 79 added by PR 7, standard 80 to 119, wide 120 and above) and three color tiers (16, 256, truecolor) and asks for a layout decision at each.

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
| `tui-emoji-density` | slop | emoji codepoints inside string literals at a per-file count of 8 or more, or 3+ from the canonical set 🚀 ✅ ❌ ⚠️ ✨ 🎉 📦 🔧 🔥 💡; `U+1F300` to `U+1FAFF` always count, `U+2600` to `U+27BF` only with default emoji presentation or a following U+FE0F (PR 5) | all stacks |
| `tui-double-border` | slop | `BorderType::Double`, `DoubleBorder()`, `borderStyle="double"`, `box.DOUBLE`, Textual `border: double` | all stacks |
| `tui-hardcoded-rgb-no-adapt` | slop | `Color::Rgb(`, `Color::from_u32(0x`, `lipgloss.Color("#`, chalk's `.hex(` or `.bgHex(` with a literal `#` string or `.rgb(` or `.bgRgb(` with a number, after any chained styles (PR 5), Ink `color="#`, Rich `[#rrggbb]`, `\x1b[38;2;`; reported when `has_adaptive_color` is false, advisory-with-note otherwise | all stacks; weak for `.tcss`, which is excluded |
| `tui-spinner-no-tty-guard` | quality | `bubbles/spinner` (either module path) with `has_tty_guard` false; PR 5 dropped `ink-spinner`, `ora(`, `rich.spinner`, and `briandowns/spinner`, which stop animating off a TTY on their own | Bubble Tea |
| `tui-hardcoded-size` | quality | `Rect::new(` with four integer literals in non-test code; a `Layout` whose constraints are all `Constraint::Length`; `(width|cols|columns) [:=] 80`, `(height|rows|lines) [:=] 24`; `.Width(80)`; `width={80}`; `size=(80, 24)`; outside paths containing `test`; PR 5 skips an 80 or 24 literal in a file that measures the terminal, counts `Length` per layout call, and skips `Rect::new` in a file that uses ratatui's canvas | all stacks |
| `tui-grapheme-unsafe-truncate` | quality | `&s[..n]`, `.chars().take(n)`, Go `s[:n]`, JS `.slice(0, n)` or `.substring(0,`, Python `[:n]`, each followed within the statement by an ellipsis literal, with `has_width_lib` false | Rust, Go, Ink, raw Python; skipped for Rich and Textual files, which measure for the caller |
| `tui-nerd-glyph-no-fallback` | quality | string literals containing private-use codepoints (`U+E000` to `U+F8FF`, `U+F0000` to `U+FFFFD`) with `has_icon_toggle` false | all stacks |
| `tui-print-in-loop` | quality | `console.log` in an Ink component file that sets `patchConsole: false`; PR 5 dropped the Textual half, because Textual captures `print` while an app runs | Ink |

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
| `tui-rt-low-contrast` | quality | first | a foreground and background pair below 4.5:1 on the palette, over cells whose glyph is alphanumeric; one finding per distinct pair at its first cell, with `cellCount` and `paletteRelative` (PR 4) in extras |
| `tui-rt-nested-borders` | slop | every capture frame | a closed box-drawing rectangle whose nearest enclosing rectangle has only blank cells between the two borders; corners are `┌┐└┘╭╮╰╯╔╗╚╝┏┓┗┛`, edges may carry tees, and the top edge may carry a title |
| `tui-rt-truecolor-on-256` | quality | first | a cell colored by `38;2` or `48;2` while `termfeatures` is known, lacks `RGB`, and `colorterm` is neither `truecolor` nor `24bit`; one finding per frame |
| `tui-rt-width-drift` | quality | every capture frame | a row holding a wide (two-cell) glyph that measures wider than the pane by no more than its count of wide glyphs (PR 4), or whose last vertical border sits on a different column than the column most rows end on |
| `tui-rt-no-key-hints` | quality | first | the last non-blank row matches no key token (`q`, `Esc`, `Enter`, `Tab`, `Space`, `Ctrl`, `Alt`, `Shift`, `F1` to `F12`, `h j k l`, `?`, arrows, `<x>`, `[x]`, `^X`); silent when the frame has fewer than three non-blank rows, when the first non-blank row names a key, or when the last is a pager prompt (`:`, `(END)`, `--More--`, `lines n-m`) (PR 4) |
| `tui-rt-collapse-narrow` | quality | capture frames under 60 columns | a row wider than the pane that holds a Box Drawing glyph (U+2500 to U+257F) and is not width drift (PR 4); a row with a left corner and no right corner; a row that fills the pane, ends in a letter, and is followed by a row starting with one |
| `tui-rt-spinner-never-rests` | quality | first against each recapture (PR 4) | a cell whose glyph is in the spinner set (braille U+2800 to U+28FF, `|/-\`, quarter circles, block bars) in frame 0 and a recapture and differs between them; one finding |

All seven ship at `advisory` with `platforms: Some(&["terminal"])`, appended after `tui-print-in-loop`, so `antipatterns.json` grows from 72 to 79 rows and web output stays byte-identical. They never run in the hook or the text engine; only `--tmux` and `--tmux-capture` reach them, on any resolved platform, because the flag itself is the request.

## Section 5: the tmux engine

Expanded on 2026-09-24 while planning PR 3, after PR 1 and PR 2 landed. The original text named the shape; this version fixes the vocabulary, the file format, the flags, the messages, and where every piece lives, so the plan can be checked against it.

### Vocabulary

- `frame` is one parsed capture: a cell grid plus width, height, `termfeatures`, `colorterm`, and a role.
- `capture frame` is a frame taken at a size the run asked for: the pane's own size first, then each `--tmux-sizes` entry.
- `recapture` is a frame of the pane's own size taken after the first with no input in between. PR 4 takes two, 700 ms and 1000 ms after frame 0 (PR 3 took one at 1000 ms). Only `tui-rt-spinner-never-rests` reads them.
- `header` is the `#!capture` line that introduces a frame in a saved capture file.
- `palette` is the reference mapping from SGR colors to sRGB used for contrast: `dark` (default) or `light`.

### Shape

A new crate `crates/terminal` (`impeccable-terminal`) mirroring `crates/browser`: it depends on `impeccable-core` (findings, registry, color) and `impeccable-detect` (the engine seam) and adds one external crate, `unicode-width`, for cell widths. It holds four modules: `capture` (the SGR parser and the frame model), `palette` (ANSI 16, the 256-color cube and grays, truecolor, the two reference palettes), `rules` (one function per runtime rule over frames), and `tmux` (locating the executable, the version check, pane geometry, capture, resize, restore). The runtime rules live in this crate, not in `crates/core`: nothing else consumes them, and the wasm bundle must not carry tmux code. The registry rows still live in `crates/foundation`, so `cargo xtask bundle` lists them in `antipatterns.json`.

`crates/detect` gains the seam: `trait TmuxEngine { detect_pane(target, &ScanOptions); detect_capture(path, &ScanOptions) }`, `Engines.tmux: Option<&dyn TmuxEngine>`, a `MissingTmuxEngine` for builds without the crate, and three `ScanOptions` fields (`tmux_sizes: Vec<(u32, u32)>`, `tmux_settle_ms: Option<u64>`, `palette: Option<String>`). `crates/cli` wires `impeccable_terminal::TerminalEngine::from_process_env()` the way it wires the browser engine.

### Input is a pane or a saved capture

`impeccable detect --tmux <session:window.pane>` captures an already running app. The engine never launches or kills a process, and it never leaves a window resized; the agent starts the app in tmux following `terminal.md`'s verification section. The engine talks to whatever server the `tmux` client resolves: the socket in `$TMUX` inside a session, else the default socket. `IMPECCABLE_TMUX` names the executable when it is not on `PATH`, the override pattern `IMPECCABLE_BROWSER` set.

`impeccable detect --tmux-capture <file>` replays a saved capture without tmux. It exists for the oracle, for CI, and for the review flow: an agent that saved `tmux capture-pane -p -e -J` output while following `terminal.md` can scan the file later. Both flags are repeatable and combine with file targets; when either is present and no file target is given, the run does not fall back to scanning the working directory.

### Capture and the frame model

`tmux capture-pane -p -e -J -t <target>` returns the visible rows with SGR sequences preserved. `-J` keeps trailing spaces and joins rows the terminal wrapped, so line and column index the joined frame, not the screen (corrected by PR 4; this sentence first said trailing blanks were trimmed). The parser walks each row: an SGR sequence updates the current style (parameters 0, 1, 2, 5, 7, 22, 25, 27, 30 to 37, 39, 40 to 47, 49, 90 to 97, 100 to 107, `38;5;n`, `48;5;n`, `38;2;r;g;b`, `48;2;r;g;b`; anything else is ignored); every other character becomes a cell carrying the style. A character `unicode-width` measures at two takes two cells, the second marked as a wide tail; a zero-width character (combining marks, ZWJ, variation selectors) takes none. PR 4 matches tmux 3.7c on three sequences: U+FE0F after a one-cell glyph makes it two cells, an emoji modifier (U+1F3FB to U+1F3FF) takes no cell, and a character after U+200D takes no cell. Row width is the cell count. The parser is a pure function over text, so goldens replay from fixture files.

A saved capture file holds one or more frames. Each frame begins with a header:

    #!capture width=80 height=24 termfeatures=256,RGB colorterm=truecolor role=capture

Every key is optional (`role` is `capture` or `recapture`; an unknown key, an unknown role, or a non-numeric size is an error). A file without a header is one capture frame whose width is its widest row and whose height is its row count, with unknown `termfeatures`. Pane scans fill the same fields from `tmux display-message -p -t <target>` (`#{pane_width}`, `#{pane_height}`, `#{window_width}`, `#{window_height}`, `#{client_termfeatures}`) and from `COLORTERM` in the engine's environment. `termfeatures` is empty when no client is attached to the session; the rule that reads it stays silent then.

### Multi-size pass

A pane scan takes frames in this order: the pane's own size; the recaptures at 700 ms and 1000 ms (PR 4); then, for each `--tmux-sizes WxH`, `tmux resize-window -t <target> -x W -y H`, a settle of `--tmux-settle` milliseconds (default 300), and a capture with the pane geometry read again. After the sizes the engine runs `resize-window` back to the recorded window size and `set-option -w -t <target> -u window-size`, so the window follows its clients again instead of keeping the `manual` size `resize-window` set. The restore runs even when a capture failed. PR 4 reads `show-options -w -v -t <target> window-size` before the first resize and restores that value, unsetting only when it was unset, because `resize-window` overwrites a user-set value with `manual`. PR 4 also restores after SIGINT or SIGTERM and gives every tmux call a 5-second timeout (section 7). `adapt.terminal.md` gets evidence for its three width classes in one command.

### Rules over frames

Size-invariant rules read the first frame only: `tui-rt-low-contrast`, `tui-rt-truecolor-on-256`, `tui-rt-no-key-hints`. `tui-rt-spinner-never-rests` compares the first frame with each recapture (PR 4). Geometry rules run on every capture frame and never on the recapture: `tui-rt-nested-borders`, `tui-rt-width-drift`, and, on frames narrower than 60 columns (the Narrow class in `adapt.terminal.md`), `tui-rt-collapse-narrow`. Section 4's PR 3 table carries the exact signals.

### Palette

`--palette dark` (default) and `--palette light` differ only in the default foreground and background: white on black, black on white. Named colors 0 to 15 use xterm's table in both, the 256-color cube and grays use their standard formula, and truecolor is taken as written. `reverse` swaps the pair; `bold` and `dim` do not change the color. Contrast is WCAG 2.x through `impeccable_core::color::contrast_ratio`, floor 4.5:1, over cells whose glyph is alphanumeric, so borders and decorations never count as text.

### Output

Findings use the existing `Finding` shape: `file` is `tmux:<target>` for a pane and the resolved path for a saved capture; `line` is the 1-based row of the joined frame; `extras` carry `column` (1-based cell) and `frame` (`WxH`), and `tui-rt-low-contrast` adds `cellCount` and, from PR 4, `paletteRelative`. `--json` prints the same array `detect` prints for files and URLs; text mode prints `line N: [id] snippet`. Every runtime rule ships at `advisory`, so a pane scan exits 0 unless a target failed. `ignoreRules`, `--scope`, and `--no-advisory` apply as for every other finding.

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

## Section 7: follow-up PRs 4 to 7

Added 2026-09-25, after PR 3 merged. Four PRs close the gaps recorded in the two known-gaps sections below. Each lands on the fork's `main` the way PRs 1 to 3 did, in the order 4, 5, 6, 7; PR 7 goes last so its skill-behavior run sees the finished engine. The owner's four decisions for this round: keep `capture-pane -J` and gate collapse-narrow's overflow check on box-drawing glyphs; keep narrow under 60 columns and add a compact class for 60 to 79; stop counting `rich` or `crossterm` alone as terminal evidence; spec and plan all four PRs now.

Facts this section rests on were probed on tmux 3.7c with a scratch server (`-L`), not taken from the manual:

- Without `-J`, no captured row is wider than the pane. With `-J`, rows the terminal wrapped are joined; a row that fills the pane exactly and ends in a newline is not joined.
- `show-options -w -v -t <target> window-size` prints nothing and exits 0 when the window has no value of its own; `resize-window` replaces a user-set value (such as `largest`) with `manual`.
- `#{cursor_x}` puts each of ❤️ ✔️ ☀️ ⚠️ 👍🏽 🧑‍💻 at two cells. The PR 3 parser gives ❤️ one cell and 👍🏽 and 🧑‍💻 four each.

### PR 4, runtime engine hardening

Branch `terminal-platform-pr4`. Scope: `crates/terminal`, `crates/common/src/proc.rs`, `crates/foundation/src/registry.rs` (one description), capture fixtures, goldens, `docs/CLI-CONTRACT.md`.

1. **One overflow verdict per row.** A row wider than its frame gets exactly one classification. The overflow (row width minus pane width) is width drift when it is no larger than the row's count of wide glyphs, at any frame width. A wide glyph is a cluster the parser measures at two cells after item 6's rules, so ❤️ counts as one. Otherwise, in a frame under 60 columns, it is collapse when the row holds a Box Drawing glyph (U+2500 to U+257F). Otherwise it reports nothing: a joined prose row is the terminal's wrap, not a layout defect. Collapse-narrow's corner and split-word checks are unchanged, and width drift's border-column check is unchanged. `line` and `column` index the joined frame.
2. **Two recaptures.** The engine takes recaptures 700 ms and 1000 ms after frame 0, and `tui-rt-spinner-never-rests` compares frame 0 with each, still reporting one finding. A 100 ms animation aliases at 1000 ms and not at 700 ms. The snippet becomes `spinner glyph ⠋ became ⠙ between captures with no input`, and the registry description says "across captures taken within one second" in place of "two captures a second apart", which moves `antipatterns.json`. A `#!capture` file may carry one or two `recapture` frames; a PR 3 file with one still replays.
3. **Timeout.** Every tmux call runs with a 5-second deadline, following `git_run` in `crates/context/src/signals.rs` (spawn, reader thread, `try_wait` polled every 5 ms, kill and wait on expiry). An expired call fails with `tmux <verb>: timed out after 5 seconds`. A capture that times out still triggers the restore, which has its own deadline.
4. **Interrupts.** The order is fixed: read `window-size` (item 5), register a flag with `impeccable_common::proc::on_interrupt`, then run the first `resize-window`. An interrupt before the registration kills the process with the window untouched, as today. After it, every settle wait sleeps in 50 ms steps and checks the flag. On SIGINT or SIGTERM the scan stops, restores, and fails with `interrupted; window restored` or `interrupted; window not restored: <message>`, an operational failure like any tmux error. A new `impeccable_common::proc::clear_interrupt()` returns both signals to their default disposition when the scan ends, so an interrupt after it kills the process as before.
5. **Restore the prior window-size.** Before the first resize the engine reads `show-options -w -v -t <target> window-size`. Restore runs `resize-window` back as before, then `set-option -w -t <target> window-size <value>` when a value was read, or `set-option -w -t <target> -u window-size` when the output was empty.
6. **Emoji widths match tmux.** The parser applies three sequence rules on top of `unicode-width`: U+FE0F after a one-cell glyph makes that glyph two cells; an emoji modifier (U+1F3FB to U+1F3FF) takes no cell; a character right after U+200D takes no cell. The six probed sequences then measure two cells each.
7. **Key-hint false positives.** `tui-rt-no-key-hints` stays silent when the first non-blank row matches the key-hint pattern or when the last non-blank row is a pager prompt: exactly `:`, or containing `(END)`, `--More--`, or `lines <n>-<m>`. `top` still fires, since its first row names no key.
8. **Palette-relative contrast.** `tui-rt-low-contrast` adds the boolean extra `paletteRelative`, true when the foreground or the background is the terminal default or one of the 16 named colors. Those colors come from the user's theme, so the ratio holds only for the `--palette` the scan assumed.
9. **Docs.** `docs/CLI-CONTRACT.md` says that `-J` keeps trailing spaces and joins wrapped rows, and that `tui-rt-truecolor-on-256` needs a client attached to the scanned session or a capture header that names the features. Section 5's matching sentences are corrected in place above.

Tests: `tests/fixtures/terminal-captures/` gains `collapse-narrow-prose.txt` (a 40-column frame whose only overflow row is joined prose; golden `[]`), `width-drift-narrow.txt` (a 40-column frame whose overflow is one emoji; only width drift fires), and `no-key-hints-pager.txt` (a pager frame ending in `:`; golden `[]`). The spinner fixture gains a second recapture that matches frame 0 at 1000 ms and differs at 700 ms, so the finding depends on the new frame. Each new fixture gets a `detect-tmux-capture-<name>` golden, and the spinner golden is re-recorded for its snippet. Crate tests cover the overflow verdict, the three width rules, the pager prompts, `paletteRelative`, and the restore command sequence for both an empty and a set window-size. `tests/tmux-engine.test.mjs` sets `window-size largest` on the scratch window before the scan and asserts it is still `largest` afterward.

### PR 5, source rules

Branch `terminal-platform-pr5`. Scope: `crates/core/src/checks/terminal.rs`, `crates/foundation/src/registry.rs` (one description), `tests/fixtures/antipatterns/terminal/`, goldens.

- **`tui-hardcoded-size`**, three narrowings. (a) An `80` or `24` literal is skipped in a file that measures the terminal, one matching `GetSize\(|terminal::size\(|terminal_size\(|get_terminal_size\(|stdout\.columns|stdout\.rows|console\.(?:width|size)\b`; there the literal is the fallback. (b) `Constraint::Length` is counted per layout call rather than per file: from `Layout::vertical(`, `Layout::horizontal(`, `Layout::new(`, or `.constraints(` to the matching close paren. A call with two or more `Length` and no flexible constraint fires once. (c) `Rect::new` with literal arguments is skipped in a file that uses `widgets::canvas`, where the rectangle is a drawing coordinate. The canonical `fn area() -> Rect { Rect::new(0, 0, 80, 24) }` keeps firing, because it neither measures nor draws on a canvas.
- **`tui-emoji-density`.** U+1F300 to U+1FAFF always count. A codepoint in U+2600 to U+27BF counts only when it has default emoji presentation (the Unicode Emoji_Presentation property), is followed by U+FE0F, or is in the canonical set. ✓ ✗ ★ ☐ ➜ stop counting. The threshold stays `>= 8`, and the section 4 row now says so.
- **`tui-hardcoded-rgb-no-adapt`, chalk.** The pattern matches `.hex(` or `.bgHex(` whose argument is a string literal starting with `#`, and `.rgb(` or `.bgRgb(` whose first argument is a number, after `chalk` and any chained styles (`chalk.bold.hex("#ff00aa")`). A computed argument (`chalk.hex(theme.primary)`) no longer matches.
- **`tui-spinner-no-tty-guard`.** Only `bubbles/spinner` (both module paths) triggers it. `ora`, Rich, `briandowns/spinner`, and Ink stop animating off a TTY on their own. The `.py` and `.tsx` fixtures become pass files, and the oracle case `detect-terminal-single-file-advisory` moves to the `.go` fixture, which still flags.
- **`tui-print-in-loop`.** The Textual half is removed: Textual captures `print` while an app runs (Textualize/textual#2952, `App._print`). The rule is Ink-only. Its registry description becomes "`console.log` in an Ink app that sets `patchConsole: false` writes over the rendered frame. Leave `patchConsole` on, or log to a file.", which moves `antipatterns.json`. The `.py` fixture becomes a pass file.

Tests: each narrowing gets a unit test named for its outcome. The file-scoped exemptions (a) and (c) get their own pass fixture files, because a `GetSize(` or `widgets::canvas` line added to an existing fixture would silence that file's flag lines. The other narrowings add lines to their rule's fixture: two layouts that each hold one `Length`, eight dingbats, a computed `chalk.hex`, and a chained literal `chalk.bold.hex`. The goldens for every touched rule and `detect-terminal-text-all` are re-recorded and reviewed by hand.

### PR 6, init evidence and routing targets

Branch `terminal-platform-pr6`. Scope: `crates/context/src/staleness.rs`, `staleness_deep.rs`, `signals.rs`, oracle cases.

- **go.mod.** Only `require` entries count: a single-line `require <path> <version>` or a line inside a `require (` block. A line ending in `// indirect` does not count, and nothing inside `replace (`, `exclude (`, or `retract (` blocks counts. The unit test that asserts an indirect lipgloss line counts is inverted to assert that it does not.
- **Charm paths.** `github.com/charmbracelet/bubbles` joins the list, and each Charm module also matches under `charm.land/` (`charm.land/bubbletea/v2`, `charm.land/lipgloss/v2`, `charm.land/bubbles/v2`).
- **Python manifests.** Package names are compared after PEP 503 normalization (lowercase, runs of `-`, `_`, `.` folded to `-`), and every quoted string on a line is checked, so `dependencies = ["Textual>=0.80", "rich"]` matches.
- **Cargo.** Dotted keys (`ratatui.workspace = true`, `ratatui.version = "0.29"`) and table headers ending in `dependencies.ratatui]` match. Member crates stay unread: Tier 1 reads only the root manifest.
- **Weak evidence.** `rich` and `crossterm` are removed from the evidence list, so neither suggests `terminal` alone. The tests that pin `rich` alone are inverted.
- **Doctor wording.** The deep finding says a workspace "carries terminal dependencies" when the evidence is terminal, and keeps "carries native build files" for mobile.
- **Detector directive.** On `terminal`, the boot's `MANUAL_DETECTOR_REQUIRED` directive (`append_detector_fallback` in `crates/context/src/context_cli.rs`) says "Once the changed terminal UI is finished, run the mechanical detector over its source:" where it said "Once the changed web UI is finished, run the mechanical detector over it:". Every other platform keeps its wording; native platforms still get no directive. Only the `context-terminal` golden moves.
- **Routing targets.** `scan_targets` takes the platform that `gather_signals` already extracts one line earlier. On `terminal` it also counts `.rs`, `.go`, `.py`, and `.tcss` files, adds `cmd`, `internal`, and `pkg` to the source directories, and targets the root when it holds `Cargo.toml`, `go.mod`, or `pyproject.toml` beside top-level source files. A Go `cmd/` layout or a flat Python package then gets the detect run `routing.md` promises.

Tests: unit tests per matcher change, a unit test per `scan_targets` layout, and two oracle cases recorded with `--bin`: `doctor-terminal-evidence-charm-v2` (a `charm.land` go.mod) and `signals-terminal-go-layout` (a `cmd/` tree), plus `doctor-monorepo-terminal-evidence` for the workspace wording. `doctor-terminal-evidence-text` and `-json` report the root finding, whose wording does not change, so they replay unchanged.

### PR 7, skill text

Branch `terminal-platform-pr7`. Scope: `skill/SKILL.src.md`, `skill/reference/*.md`, `skill/agents/`, `tests/skill-reference.test.mjs`, the `context-terminal` golden.

- **Terminal branches.** Every place that splits web from native gets a terminal case: `SKILL.src.md` (verify rounds, platform guidance), `polish.md` (representative sizes, size classes, runtime checks), `new-work.md` (comp aspect, representative sizes, the detect step, the reviewer packet), `layout.md` and `typeset.md` (a **Terminal:** bullet beside **Native:**, as `animate.md` has), `doctor.md` (the evidence finding names `terminal.md`), `init.md` (live config skip, the reference sentence), and the agents `impeccable-finish-reviewer.md` and `impeccable-asset-producer.md`. The terminal case verifies with the tmux capture matrix (80x24, 120x40, 40x24) plus one run with `NO_COLOR=1` and one piped through `cat`; says `impeccable detect` runs on terminal source; and treats a comp as a landscape frame on an 80x24 grid.
- **Live guard.** `live.md` gains the one-line web-only guard that `generate.md` and `routing.md` carry.
- **Widths.** `adapt.terminal.md` gains a Compact row (60 to 79 columns) and says "At 120 columns and above" where it said "Above 120 columns". `terminal.md` says "40 columns" where it said "about 30". `NARROW_COLUMNS` stays 60, so the engine needs no change.
- **Left alone.** `skill/reference/degraded/finish-reviewer.md` has no platform awareness for any platform; adding terminal alone would be a partial fix.

Tests: `tests/skill-reference.test.mjs` pins the Compact row, the live guard, and one terminal branch per edited file. `context-terminal` is re-recorded because it inlines `terminal.md`. One `bun run test:skill-behavior` run is compared against the baseline table in `tests/skill-behavior/README.md`.

### Left out of PRs 4 to 7

- **concept-seed forwarding `platform=terminal`:** the roll API is in the private repo; forwarding a value it may not handle is a change to both sides.
- **A CLI driver for the skill-behavior suite:** its own brainstorm.
- **The OpenRouter provider branch** (`skill-behavior-openrouter`): unrelated to the terminal platform.
- **Promoting any rule out of `advisory`:** each promotion waits for a manual pass after its tightening ships.
- **The `ENGINE_VERSION` bump:** a release step.
- **Reading Cargo member manifests:** a directory walk Tier 1 cannot afford.
- **Populating `client_termfeatures` on a detached server:** tmux does not; PR 4 documents it instead.

### Gap disposition

| Gap | Where |
|---|---|
| Native-only branches in `SKILL.src.md`, `polish.md`, `new-work.md` | PR 7 |
| `live.md` has no terminal guard | PR 7 |
| `audit.terminal.md` 60 vs 40 columns | Section 2 corrected in place; the shipped 60 is right |
| `adapt.terminal.md` 60 to 79 gap | PR 7 |
| concept-seed does not forward `platform=terminal` | Left out |
| `// indirect` go.mod lines | PR 6 |
| pyproject arrays, Cargo dotted keys and headers, capitalized PyPI names | PR 6 |
| `rich` or `crossterm` alone | PR 6 |
| Charm v2 `charm.land` paths | PR 6 |
| `tui-print-in-loop` Textual half | PR 5 |
| `tui-spinner-no-tty-guard` self-guarding libraries | PR 5 |
| Emoji range and `>= 8` | PR 5 (range); row corrected (threshold) |
| `chalk.hex(` computed argument | PR 5 |
| `tui-hardcoded-size` tightening | PR 5 |
| `scan_targets` not platform-aware | PR 6 |
| CLI driver for skill-behavior | Left out |
| Width drift and collapse co-fire | PR 4, item 1 |
| `-J` joins wrapped rows | PR 4, item 1 (kept, gated on box glyphs) |
| Spinner aliasing at one second | PR 4, item 2 |
| No tmux timeout; interrupt leaves the window resized | PR 4, items 3 and 4 |
| Restore drops a user-set `window-size` | PR 4, item 5 |
| Trailing-blank sentence | Section 5 corrected; PR 4, item 9 |
| Emoji width mismatch | PR 4, item 6 |
| `client_termfeatures` empty when detached | PR 4, item 9 (documented) |
| `tui-rt-no-key-hints` on tables and pager prompts | PR 4, item 7 |
| `MANUAL_DETECTOR_REQUIRED` says "web UI" on terminal (found while planning) | PR 6 |
| Four runtime rules unmeasured on real programs | Left out: needs a manual pass after PR 4 |

### Delivery gates

- **PR 4:** `cargo test --workspace`; `cargo xtask bundle` and `cargo xtask bundle --check`; the new and re-recorded goldens reviewed by hand; `bun run build`; `bun run test`; `node --test tests/tmux-engine.test.mjs` with tmux installed; one manual `--tmux` pass over `less` and a 40-column frame of the ratatui demo, recording the collapse-narrow count against PR 3's 15.
- **PR 5:** `cargo test --workspace`; `cargo xtask bundle` and `--check`; the re-recorded goldens; `bun run build`; `bun run test`.
- **PR 6:** `cargo test --workspace`; the three new oracle cases, the re-recorded `context-terminal` golden, and the doctor goldens replaying unchanged; `bun run test`.
- **PR 7:** `bun run build`; `bun run test`; one `bun run test:skill-behavior` run against the baseline.

PRs 4, 5, and 6 change engine behavior and add to the `ENGINE_VERSION` bump already owed at release. PR 7 changes skill content, which the skill version bump at release covers. None of the four bumps a manifest or adds a changelog entry.

## Open questions

PR 2's two calibration questions were settled by its manual pass (`docs/superpowers/research/2026-09-24-terminal-rules-manual-pass.md`): the emoji thresholds stay at 8 and 3, and `tui-hardcoded-size` is marked tighten-before-promotion rather than exempting `examples/`.

Two questions are deferred to PR 3's manual pass. Whether one second is the right recapture window for `tui-rt-spinner-never-rests`: a spinner over real pending work is legitimate and will fire, which is why the rule is advisory. And whether `client_termfeatures` is populated when the scan runs from outside the session; if it is not, `tui-rt-truecolor-on-256` fires only on saved captures whose header names the features. PR 3's manual pass answered both; see the known gaps after PR 3 and section 7.

## Known gaps after PR 1 and PR 2

Recorded here so they are not lost; none is in PR 3's scope. Section 7's gap disposition assigns each to PR 5, 6, or 7 or leaves it out with the reason.

- Skill text: native-only branches in `SKILL.src.md`, `polish.md`, and `new-work.md` that `terminal` falls through; `live.md` has no terminal guard; `audit.terminal.md` scores 60 columns where this spec said 40; `adapt.terminal.md` leaves a 60 to 79 column gap; concept-seed does not forward `platform=terminal` to the private roll API.
- Init evidence: `// indirect` go.mod lines count as evidence; the matchers miss pyproject inline arrays, Cargo dotted keys and table headers, and capitalized PyPI names; `rich` or `crossterm` alone is weak evidence; `staleness.rs` misses Charm v2 `charm.land/...` module paths.
- Source rules: `tui-print-in-loop`'s Ink premise was inverted in the original section 4 row (Ink's default is `patchConsole: true`, so PR 2 flags only `patchConsole: false`), and its Textual half is unverified; `ora` and Rich `Live` and `Status` self-guard on a non-TTY, so `tui-spinner-no-tty-guard` likely over-reports them; the emoji range U+2600 to U+27BF catches dingbats and the threshold is `>= 8` where the row says "above 8"; `chalk.hex(` with a computed argument is matched (this line first said missed; the pattern takes any argument, so it over-matches); `tui-hardcoded-size` needs tightening before promotion.
- Routing: `crates/context/src/signals.rs` `scan_targets` is not platform-aware, so routing's detect run skips terminal source.
- Process: a CLI driver (headless agents with stream-json) for the skill-behavior suite is its own brainstorm.

## Known gaps after PR 3

Recorded from the PR 3 final review and the manual pass (`docs/superpowers/research/2026-09-24-terminal-runtime-rules-manual-pass.md`). None is fixed in PR 3. Section 7's gap disposition assigns each to PR 4 or leaves it out with the reason.

- `tui-rt-collapse-narrow` and `tui-rt-width-drift` both fire on a row wider than a pane under 60 columns, so a 40-column frame reports every overflow row twice.
- `capture-pane -J` joins soft-wrapped screen rows into one logical row before the parser sees them; the manual pass classed 13 of 15 collapse-narrow findings as this join. Capturing without `-J` keeps screen rows but turns an app's own soft wrap into an overflow at every boundary. The rule stays advisory until one of the two is chosen.
- `tui-rt-spinner-never-rests` needs the glyph to differ between frame 0 and the recapture one second later. An animation whose period divides one second (ten glyphs at 100 ms) aliases and is missed; a second recapture at an offset such as 700 ms would close the gap.
- tmux calls have no timeout, so a wedged server hangs `--tmux`. A SIGINT or SIGTERM during the recapture or the size pass leaves the window at the last requested size in `manual` mode.
- Restore runs `set-option -w -u window-size`, which also drops a `window-size` the user had set on that window before the scan. Restoring the earlier value needs `show-options -w -v window-size` first.
- Section 5 says trailing blanks are trimmed; the tmux manual says `-J` preserves trailing spaces. The rules tolerate padded rows, so only the sentence needs correcting.
- `unicode-width` 0.2 and tmux can disagree on the width of some emoji (VS16 sequences, recent additions), which makes a correct row measure one cell off and can trip `tui-rt-width-drift`.
- `client_termfeatures` is empty on a detached server, so `tui-rt-truecolor-on-256` is silent unless a client is attached to the scanned session.
- Four rules (`tui-rt-width-drift`, `tui-rt-nested-borders`, `tui-rt-truecolor-on-256`, `tui-rt-spinner-never-rests`) are unmeasured on real programs; `tui-rt-no-key-hints` assumes the bottom row is a footer and misfires on scrolling tables and bare pager prompts.

## Known gaps after PR 4

Recorded from the PR 4 final review. Neither is fixed in PR 4.

- The width-drift test (overflow no larger than the row's count of two-cell glyphs) also passes for wrapped CJK or emoji prose: a joined row of n two-cell glyphs overflows by 2n minus the pane width, which is at most n whenever n is at most the pane width, so a CJK sentence the terminal wrapped once reads as width drift. A follow-up could also require a vertical border glyph at the row's end.
- `tui-rt-no-key-hints`'s first-row exemption reuses the whole key-hint pattern, including bracketed and angle-bracketed tokens and a bare `?`, so a log viewer whose first row starts `[INFO]`, or any first row containing `?`, silences the rule. The rule is advisory, so this is a false negative only.

## Known gaps after PR 5

Recorded from the PR 5 final review; neither is fixed in PR 5.

- `tui-hardcoded-size`'s canvas exemption matches the literal `widgets::canvas`, so a grouped import such as `use ratatui::widgets::{canvas::Canvas, Block}` does not exempt the file. A missed exemption leaves the rule firing as it did before PR 5.
- Counting `Constraint::Length` per layout call found four new findings in the ratatui examples (color-explorer, demo, inline), all fixed-height sub-layouts. They match the rule as specified, but fixed-height sub-layouts are common in ratatui, so they count against promoting the rule out of `advisory`.

## Appendix A: research report

The sourced research this spec distills is `docs/superpowers/research/2026-09-24-terminal-design-research.md` (72 sources). Its distillation section is the origin of `terminal.md`'s bullets and the rule table.
