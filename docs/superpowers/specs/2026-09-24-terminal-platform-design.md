# Terminal platform: design guidance, source rules, and a tmux engine

Date: 2026-09-24
Status: approved in conversation, awaiting written review
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

Each runtime rule is the capture-side cousin of a source rule, so a reader learns one vocabulary. Ids carry `tui-rt-`.

| id | signal over the capture |
|---|---|
| `tui-rt-low-contrast` | a rendered foreground and background pair below 4.5:1 on the reported palette; with `--palette light`, evaluated on a reference light palette for named ANSI colors |
| `tui-rt-nested-borders` | a closed box-drawing rectangle fully inside another with no focusable content difference |
| `tui-rt-truecolor-on-256` | `38;2;` or `48;2;` SGR present while the client reports 256 colors or fewer |
| `tui-rt-width-drift` | a row whose cell count differs from the pane width after width-aware measurement, on rows containing emoji or CJK |
| `tui-rt-no-key-hints` | the bottom row contains no key-like token (`q`, `Esc`, `Ctrl`, `?`, `Enter`, angle-bracketed or bracketed keys) |
| `tui-rt-collapse-narrow` | at 40 columns: broken border runs, words split mid-cell, or content overflowing the grid |
| `tui-rt-spinner-never-rests` | across two captures one second apart with no input, a cell cycling through a spinner glyph set |

## Section 5: the tmux engine

### Shape

A new crate `crates/terminal` mirroring `crates/browser`: a `TmuxEngine` built from the process environment, a capture parser, and rule adapters. `Engines` gains `tmux: Option<&dyn TmuxEngine>`. `crates/cli` wires it the way it wires the URL engine.

### Input is a pane, not a command

`impeccable detect --tmux <session:window.pane>` captures an already running app. The engine never launches or kills a process. The agent starts the app in tmux following `terminal.md`'s verification section. This keeps the engine deterministic and avoids a second orphaned-process problem of the kind the live-server reaper exists to solve.

### Capture

`tmux capture-pane -p -e -J -t <target>` returns the visible grid with SGR sequences preserved. The parser produces a `TerminalSnapshot`: a grid of cells with glyph, foreground, background, and attributes, plus pane width and height, the client's color depth from `tmux display -p '#{client_termfeatures}'`, and `COLORTERM` from the environment. The parser is a pure function over captured text, so goldens replay from fixture files without tmux.

### Multi-size pass

`--tmux-sizes 80x24,120x40,40x24` resizes with `tmux resize-window -x -y`, waits a configurable settle time (default 300 ms), captures, and restores the original size. `adapt.terminal.md` gets evidence for its width classes in one command.

### Output

Findings use the existing `Finding` shape with `line` set to the row and `column` to the cell. `--json` output is the same envelope `detect` prints for URLs.

### Requirements and degradation

tmux 3.2 or newer. Without tmux on PATH, `--tmux` prints one line naming the requirement and exits 1, matching the URL engine's message when no browser is found.

### Oracle and tests

Goldens recorded from capture fixtures under `tests/fixtures/terminal-captures/<case>.txt`. One integration test, `tests/tmux-engine.test.mjs`, boots a fixture script in a scratch tmux server (`tmux -L impeccable-test`), captures, and asserts findings; it skips when `which tmux` fails. The scratch server is killed in `after()` and by the test runner's process-group guard.

## Section 6: testing, documentation, delivery

### PR 1, guidance

Files: `crates/context` (parser, boot inlining, staleness evidence), `skill/SKILL.src.md`, `skill/reference/terminal.md`, `audit.terminal.md`, `adapt.terminal.md`, `init.md`, `routing.md`, the concept-seed flag, `docs/CLI-CONTRACT.md`, the root `CLAUDE.md` Platform section.

Gates: `cargo test --workspace`; the section 1 oracle cases recorded with `node tests/oracle/record.mjs --bin` and reviewed by hand; `bun run build`; `bun run test`; one `bun run test:skill-behavior` run.

### PR 2, source rules

Files: `crates/foundation/src/registry.rs` (field and rows), `crates/core/src/checks/terminal.rs`, `crates/detect` (walker list, platform gate, project signals, `--platform`), `crates/hook` (platform-aware extensions), `crates/cli` (resolver wiring), `tests/fixtures/antipatterns/terminal/`, the two tracked assets from `cargo xtask bundle`, `README.md` and `README.npm.md` rule counts, `docs/CLI-CONTRACT.md`.

Gates: `cargo test --workspace`; `cargo xtask bundle --check`; the section 3 oracle cases; `bun run build` with corrected counts; `bun run test`; a manual `impeccable detect --platform terminal` over one real project per stack, with the false-positive rate recorded in the PR before any rule is promoted from `advisory`.

### PR 3, tmux engine

Files: `crates/terminal`, `crates/detect` (`Engines` slot, `--tmux`, `--tmux-sizes`, `--palette`), `crates/cli`, registry rows for runtime rules, `tests/fixtures/terminal-captures/`, `tests/tmux-engine.test.mjs`, `terminal.md`'s verification section, `docs/CLI-CONTRACT.md`, `docs/ENGINE.md` layout table.

Gates: as PR 2 plus the integration test.

### Test naming

Tests name outcomes: `web_project_with_rust_backend_gets_no_terminal_rules`, `terminal_platform_keeps_hook_active`, `hardcoded_truecolor_flagged_only_in_tui_files`, `single_file_scan_downgrades_absence_rules`, `capture_parser_replays_without_tmux`.

### Fork and upstream process

Each PR lands on the fork first: branch, review, merge to `crichalchemist/impeccable` `main`. The fork owner plans to test the feature in real use once complete, so `main` carries it as soon as each PR merges.

Upstream is issue-first. Before the first upstream PR, the fork owner opens an issue on pbakaus/impeccable proposing the `terminal` platform and the three-PR shape. PRs go upstream only after a maintainer approves in that issue. AI assistance is disclosed in every commit and PR body. Claude drafts issue and PR text but never posts to the upstream repository, per the root guide's contribution rules.

### Versioning

No manifest bumps and no changelog entries in any PR. PR 2 and PR 3 change engine behavior and will need an `ENGINE_VERSION` bump at release, which follows the engine-first release order.

## Open questions

None blocking. Two calibration decisions are deferred to PR 2's manual pass: the emoji-density threshold (proposed 8 per file) and whether `tui-hardcoded-size` should exempt files under `examples/`.

## Appendix A: research report

The sourced research this spec distills is `docs/superpowers/research/2026-09-24-terminal-design-research.md` (72 sources). Its distillation section is the origin of `terminal.md`'s bullets and the rule table.
