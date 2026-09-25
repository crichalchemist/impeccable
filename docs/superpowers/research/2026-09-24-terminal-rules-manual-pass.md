# Terminal source rules: manual false-positive pass

Date: 2026-09-24. Branch `terminal-platform-pr2`. Binary: `target/release/impeccable` built from this branch.

This is the calibration record the spec requires before any `tui-*` rule is promoted (section 6, PR 2 gates). It also settles the two open questions from the spec: the emoji threshold and whether `tui-hardcoded-size` should exempt `examples/`. All eleven rules stay `advisory` in this PR.

## What was scanned

Each project was scanned with `impeccable detect --no-config --json --platform terminal <dir>`.

| Project | Stack | Commit | Target | Candidate files | Classified as terminal source | Scan time |
|---|---|---|---|---|---|---|
| ratatui/ratatui | Rust (Ratatui) | `21324fe` | `examples/` | 64 | 61 `.rs` | 0.03 s |
| charmbracelet/glow | Go (Bubble Tea v2, Lip Gloss v2) | `6b365ee` | whole repo | 33 | 10 before the fix, 17 after | 0.01 s |
| Textualize/toolong | Python (Textual) | `5aa22ee` | `src/` | 19 | 13 `.py` | 0.01 s |
| vadimdemedes/ink-ui | TypeScript (Ink) | `14b1145` | `source/` | 64 | 35 | 0.05 s |
| musicviz (the fork owner's app) | Python (Textual) | `b85b081` | `src/` | 70 | 2 `.py` (`tui/app.py`, `tui/artifact.py`) | 0.01 s |

No clone was substituted. Every scan took well under a second, so the terminal walk costs nothing worth noting. musicviz declares `web` in PRODUCT.md today, so it was scanned with the flag.

## Hits per project

These are the counts from the fixed binary. The figures in parentheses are from the binary before the fixes described below.

| Project | Total | Findings by rule |
|---|---|---|
| ratatui/examples | 43 (52) | hardcoded-rgb-no-adapt 36 (45), hardcoded-size 4, blink-attribute 2, double-border 1 |
| glow | 7 (2) | hardcoded-rgb-no-adapt 4 (0), hardcoded-size 3 (2) |
| toolong | 0 | none |
| ink-ui | 0 | none |
| musicviz | 0 | none |

## Classification

Each finding was read in its file. TP means true positive, FP means false positive.

| Rule | Project | Hits | Sampled | False positives | Verdict |
|---|---|---|---|---|---|
| tui-hardcoded-rgb-no-adapt | ratatui/examples | 45 before, 36 after | 45 | 9 before (all fixed), 0 after | fixed; promotion candidate |
| tui-hardcoded-rgb-no-adapt | glow | 4 (after the classifier fix) | 4 | 0 | promotion candidate |
| tui-hardcoded-size | ratatui/examples | 4 | 4 | 2, plus 2 arguable | tighten before promotion |
| tui-hardcoded-size | glow | 3 (after the classifier fix) | 3 | 3 | tighten before promotion |
| tui-blink-attribute | ratatui/examples | 2 | 2 | 0 | sample too small |
| tui-double-border | ratatui/examples | 1 | 1 | 0 | sample too small |
| tui-spinner-no-tty-guard | all | 0 | n/a | n/a | unmeasured |
| tui-grapheme-unsafe-truncate | all | 0 | n/a | n/a | unmeasured |
| tui-nerd-glyph-no-fallback | all | 0 | n/a | n/a | unmeasured |
| tui-emoji-density | all | 0 | n/a | n/a | unmeasured |
| tui-figlet-banner | all | 0 | n/a | n/a | unmeasured |
| tui-gradient-title | all | 0 | n/a | n/a | unmeasured |
| tui-print-in-loop | all | 0 | n/a | n/a | unmeasured |

### tui-hardcoded-rgb-no-adapt

Every ratatui hit carries the "adaptive color helper exists elsewhere" note. The gauge and flex demos pick between a truecolor value and an `ansi_color` fallback, which sets the project signal for the whole `examples/` directory.

| Finding | Class | Reason |
|---|---|---|
| calendar-explorer `main.rs:156` to `:176` (6 hits) | TP | a literal truecolor background with no fallback in that app |
| custom-widget `main.rs:60` to `:77` (12 hits) | TP | literal theme constants with no fallback |
| demo2 `theme.rs:122` to `:132` (11 hits) | TP | a literal palette; demo2 has no truecolor check |
| demo2 `destroy.rs:91`, weather `main.rs:112`, release-header `main.rs:41` to `:43` | TP | literal colors painted directly |
| demo2 `destroy.rs:106`, `:107` | arguable | default values in `unwrap_or`, fed into a blend rather than painted as they are |
| colors-rgb `main.rs:233`, demo2 `colors.rs:37`, release-header `main.rs:146`, volatility-surface `palette.rs:56`, mouse-drawing `main.rs:66`, weather `main.rs:66`, demo2 `destroy.rs:130` | FP, fixed | the color is computed from variables and is not hardcoded |
| demo2 `destroy.rs:116`, `:119` | FP, fixed | `let Color::Rgb(r, g, b) = c else` is a pattern that reads a color and sets nothing |
| glow `style.go:7`, `ui/styles.go:72`, `:134`, `:152` | TP | literals that bypass glow's own `s.adaptive(light, dark)` helper, which the rest of the file uses; the note is correct |

The glow result is the best evidence for this rule. It skipped the dozens of `s.adaptive("#..", "#..")` calls and flagged exactly the four literals that bypass the helper.

### tui-hardcoded-size

| Finding | Class | Reason |
|---|---|---|
| ratatui canvas `main.rs:65` `Rect::new(10, 10, 200, 100)` | FP | a rectangle in canvas coordinate space, not the terminal |
| ratatui demo2 `main.rs:44` `Viewport::Fixed(Rect::new(0, 0, 81, 18))` | arguable | the viewport is fixed on purpose to match a VHS recording, and a comment says so |
| ratatui release-header `main.rs:57` `Viewport::Fixed(Rect::new(0, 0, 68, 16))` | arguable | the same deliberate recording viewport |
| ratatui release-header `main.rs:85` `area.centered(Constraint::Length(w), Constraint::Length(h))` | FP | two `Length`s that size a centered box inside the measured area; this is not an all-`Length` layout |
| glow `config_cmd.go:23` `width: 80` | FP | the word-wrap default inside a YAML template held in a Go raw string |
| glow `main.go:206` `width = 80` | FP | the idiomatic fallback when `term.GetSize` fails |
| glow `ui/rawpaint.go:40` `tea.WindowSizeMsg{Width: 80, Height: 24}` | FP | the same fallback after a failed `term.GetSize` |

That is 5 false positives and 2 arguable findings out of 7, far above the one-third line. None is a plain bug of the kind this task fixes. Each FP comes from how the rule is designed, so the rule needs tightening before promotion, not a patch. Candidate tightenings for a later PR:

1. Do not report an 80 or 24 literal in a file that also measures the terminal (`term.GetSize`, `crossterm::terminal::size`, `shutil.get_terminal_size`, `process.stdout.columns`). That is the fallback idiom, and it accounts for 2 of glow's 3 FPs.
2. Count `Constraint::Length` per `Layout` call instead of per file, so the two arguments to `centered()` are not read as a layout.
3. Report a `Rect::new` literal only where it reaches a viewport or a frame area, not where it defines a canvas.

The glow YAML case would stay; if the rule were made stricter it could skip string literals, but Textual CSS lives in strings, so that needs care.

### tui-blink-attribute

| Finding | Class | Reason |
|---|---|---|
| ratatui demo `ui.rs:283` `RAPID_BLINK` on a failed server row | TP | blink used as an alert, the exact case the rule questions |
| ratatui user-input `main.rs:174` `RAPID_BLINK` on the help line | TP | decorative blink on ordinary text |

### tui-double-border

| Finding | Class | Reason |
|---|---|---|
| ratatui table `main.rs:291` `BorderType::Double` on the footer | TP | a decorative double border, the choice the rule exists to question |

### The rules with no hits

- **tui-spinner-no-tty-guard**: silenced, not blind. Glow imports `charm.land/bubbles/v2/spinner` (`ui/stash.go:12`). A single-file scan of it, which collects no signals, reports the spinner. The directory scan correctly stays quiet because glow checks `term.IsTerminal` in `main.go:186` before starting the TUI. No other project has a spinner. Before the fix below, the v2 import was not recognized at all.
- **tui-grapheme-unsafe-truncate**: never matched. Both ratatui (`unicode-width`) and glow (`go-runewidth`) turn on the width-library signal. A per-file scan without signals over every classified file still found no Rust or Go slice followed by an ellipsis.
- **tui-nerd-glyph-no-fallback**: never matched. None of the five projects contains a private-use code point.
- **tui-emoji-density**: never matched. The most in any file was 4 (ratatui `todo-list/src/main.rs`, which uses `✓` and `☐`), then 1 (toolong `log_lines.py`, `👉`).
- **tui-figlet-banner**, **tui-gradient-title**: never matched.
- **tui-print-in-loop**: never matched. musicviz's `CockpitApp(App[None])`, `_Steps(Static)` and `ArtifactScreen(ModalScreen[None])` all match the class pattern, so the rule ran and found no `print(`. toolong's only `print(` calls are in a `__main__` block of a module that does not subclass a widget. ink-ui has no `console.log`.

### musicviz: clean, not blind

The owner's app gets zero findings because it uses the idiomatic constructs. Its hex colors live in a Textual `Theme(primary="#5B7CFA", ...)` registration, which the truecolor rule does not match and should not match. Its stylesheet uses `border-top: solid`. Its widget classes contain no `print(`. It has no emoji and no private-use glyphs. The one gap: the Textual arm of the truecolor rule matches `[#hex]` markup and `color="#`, not `color: #hex` inside a `CSS = """..."""` block. musicviz has no such line, so nothing was missed here.

## Rule fixes made

All three fixes are in `crates/core/src/checks/terminal.rs`. Each has a unit test written first and seen failing.

1. **Charm v2 files were not classified as terminal source.** Bubble Tea v2, Lip Gloss v2 and Bubbles v2 import from `charm.land/...`, not `github.com/charmbracelet/...`. Before the fix, 7 of glow's 17 Charm files got no rules. That included `ui/styles.go`, where the real truecolor findings are, and the `utils/` files. The `.go` arm now also accepts `charm.land/`. This was a classification bug, not a rule bug. Test: `charm_v2_vanity_import_is_terminal_source`.
2. **The v2 spinner import was not seen.** `bubbles/spinner` became `bubbles/(?:v\d+/)?spinner`, so `charm.land/bubbles/v2/spinner` matches. Test: `a_v2_bubbles_spinner_import_is_seen`.
3. **A computed or destructured `Color::Rgb` was reported as hardcoded.** The Rust arm now requires three integer literals, decimal or hex: `Color::Rgb(<int>, <int>, <int>)` or `Color::Rgb(0x20, 0x30, 0x60)`. This removes 9 ratatui FPs and keeps every fully literal color. A partly computed color such as weather `Color::Rgb(255, green, 0)` is no longer reported. Test: `computed_and_destructured_rgb_is_not_a_hardcoded_color`.

No golden changed. Every `detect-terminal-*` fixture uses `github.com/charmbracelet` imports and a literal `Color::Rgb(255, 0, 128)`, and the full oracle passes unchanged against the rebuilt binary. `cargo xtask bundle --check` reports both tracked live assets up to date, so the wasm bundle did not change either.

Not changed, same bug class, not observed: `chalk.hex(` in the truecolor pattern would also match a computed argument.

## Decision 1: the emoji thresholds

**Kept:** `EMOJI_FILE_THRESHOLD = 8` and `CANONICAL_EMOJI_THRESHOLD = 3`. No file in the five projects came near either threshold, so there is no evidence to move them.

One latent risk is recorded for the next sample. `is_emoji` counts the whole U+2600 to U+27BF block, which includes `✓`, `✗`, `☐` and `★`. Those are ordinary checkbox and status glyphs in TUIs, not emoji. The ratatui todo-list reaches 4 of them. A checklist renderer with a few more states would reach 8 and be reported as emoji-dense. If a later sample shows that, narrow the range to code points with default emoji presentation instead of changing the threshold.

## Decision 2: should `in_test_path` exempt `examples`?

**Kept:** no `examples` exemption. Three reasons:

1. `in_test_path` looks only at the file name and its parent directory. Every ratatui hit sits at `examples/apps/<app>/src/main.rs`, whose parent is `src`. Adding `examples` to the existing two-segment check would exempt none of them. Only a match on any path segment would, and that is broader than the planning ruling that set the two-segment scope.
2. The size false positives come from the pattern, not the path. The canvas rectangle and the `centered()` call would be FPs in real app code too, and glow's three FPs are not under any examples directory. An exemption would hide the symptom in demos and leave it in apps.
3. The two arguable ratatui hits, the fixed recording viewports, are the rule's real subject. A reader of the examples is better served by the advisory than by silence.

## The isatty-comment risk

**Not observed.** The risk: a comment anywhere that mentions `isatty` (or another TTY-guard trigger) silences `tui-spinner-no-tty-guard` for the whole project, because signals fold raw text including comments. Every TTY trigger in the five projects is a real call:

- glow: `term.IsTerminal` in `main.go:186` and `utils/textsize.go:44`
- toolong: `sys.__stdin__.isatty()` in `cli.py:25`
- musicviz: `sys.stdin.isatty()` in `cli/_output.py:35`

The only spinner is glow's, and glow really does guard its TUI. The pass gives no reason to move signal collection to comment-blanked text, so the design is left as it is.

Two related signal weaknesses were observed. Neither changed a verdict:

- The `ansi_color` closure parameter in ratatui's gauge and flex demos turns on `has_adaptive_color` for the whole `examples/` directory. The signal spans all 32 demo apps, so every one of the 36 truecolor findings carries the "helper exists elsewhere" note, though only 2 of those apps adapt. This is an artifact of scanning a directory of independent apps as one project. In a single app the note is right.
- toolong's `self.icons: dict[int, str]` type annotation matches the icon-toggle pattern and turns on `has_icon_toggle`. toolong has no private-use glyphs, so nothing was suppressed.

## Promotion recommendations

Every rule stays `advisory` in this PR.

| Rule | Recommendation |
|---|---|
| tui-hardcoded-rgb-no-adapt | Promotion candidate after the fix: 0 FP in 40 findings across two stacks. Consider whether a project-wide adaptive signal should soften findings in unrelated files. |
| tui-hardcoded-size | Tighten before promotion: 5 FP and 2 arguable in 7. Apply the three tightenings above first. |
| tui-blink-attribute | 0 FP, but only 2 findings. Needs a second sample. |
| tui-double-border | 0 FP, but only 1 finding. Needs a second sample. |
| tui-spinner-no-tty-guard | Unmeasured (one raw hit, correctly silenced). Needs a project with a spinner and no guard. |
| tui-grapheme-unsafe-truncate | Unmeasured. Needs a project without a width library. |
| tui-nerd-glyph-no-fallback | Unmeasured. Needs a Nerd Font app (a file manager or git TUI). |
| tui-emoji-density | Unmeasured. Watch the dingbat range noted in Decision 1. |
| tui-figlet-banner | Unmeasured. |
| tui-gradient-title | Unmeasured. |
| tui-print-in-loop | Unmeasured. Clean on two Textual apps and one Ink library. |

## Out of scope, noted for follow-up

`crates/context/src/staleness.rs` finds a terminal Go project through `github.com/charmbracelet/bubbletea` and `github.com/charmbracelet/lipgloss` in `go.mod`. A Charm v2 project declares `charm.land/bubbletea/v2` and `charm.land/lipgloss/v2` instead, so that platform evidence misses it. This task did not change it.
