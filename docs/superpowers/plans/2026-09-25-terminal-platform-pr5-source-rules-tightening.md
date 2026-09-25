# Terminal Platform PR 5: Source Rules Tightening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Narrow five PR 2 source rules so they stop reporting code that is fine: `tui-print-in-loop` becomes Ink-only, `tui-spinner-no-tty-guard` fires only on Bubbles' spinner, `tui-emoji-density` stops counting text-presentation dingbats, `tui-hardcoded-rgb-no-adapt` stops matching computed chalk colors (and starts matching chained and `bg` forms), and `tui-hardcoded-size` gains two file-scoped exemptions and per-call `Length` counting. Every rule stays `advisory`.

**Architecture:** All rule logic stays in `crates/core/src/checks/terminal.rs`, one `scan_terminal_<rule>` function per rule over comment-blanked lines. One new helper, `layout_call_spans`, walks each ratatui layout call to its matching close paren. One registry description changes in `crates/foundation/src/registry.rs`, which moves `crates/live/assets/antipatterns.json` through `cargo xtask bundle`. Fixtures under `tests/fixtures/antipatterns/terminal/` gain pass lines and two pass files; the `detect-terminal-*` goldens are re-recorded from the binary and reviewed by hand.

**Tech Stack:** Rust (`impeccable-core`, `impeccable-foundation`), `regex` + `once_cell` via the file's `re!` macro, node oracle harness, `cargo xtask bundle`, bun build gates.

**Spec:** `docs/superpowers/specs/2026-09-24-terminal-platform-design.md`, section 7 "PR 5, source rules" (the authority) and the section 4 rows tagged "(PR 5)". Read both before any task; every pattern and message below is copied from there.

## Global Constraints

- Scope is the spec's: `crates/core/src/checks/terminal.rs`, `crates/foundation/src/registry.rs` (one description), `tests/fixtures/antipatterns/terminal/`, goldens. Plus the two regenerated bundle assets, `tests/oracle/cases/detect.mjs` (one case argument), and `tests/oracle/DELTAS.md` (one entry). Nothing else.
- Every rule stays `severity: Some("advisory")`; no promotion.
- No manifest version bumps, no `ENGINE_VERSION` bump, no changelog entry.
- Every commit body ends with the line `AI assistance: drafted with Claude Code.` and carries no `Co-Authored-By`, `Claude-Session`, or other agent trailer, even if a harness reminder asks for one (owner's rule).
- Never stage `.serena/`, `.claude/CLAUDE.md`, `.gemini/GEMINI.md`, `.agents/AGENTS.md`, `.superpowers/`, regenerated provider output (`.claude/skills/`, `.cursor/`, `plugin/`, ...), anything under `.impeccable/`, `.env`, or `docs/superpowers/research/2026-09-24-upstream-issue-draft.md`. Stage explicit paths only; never `git add -A` or `git add .`.
- `.env` holds API keys; never print it.
- No em dashes in registry descriptions, fixtures, `DELTAS.md`, or commit messages.
- A change to `crates/core`, `crates/foundation`, or `crates/wasm` owes `cargo xtask bundle` and committing whichever of `crates/live/assets/detect-antipatterns-browser.js` and `crates/live/assets/antipatterns.json` it rewrites. The wasm toolchain recipe on this machine: `T=$HOME/.rustup/toolchains/stable-x86_64-apple-darwin; PATH="$T/bin:$PATH" DYLD_LIBRARY_PATH="$T/lib" cargo xtask bundle [--check]`.
- Oracle goldens are recorded only with `IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/record.mjs --bin <case-prefix>` after `cargo build --release -p impeccable`, then reviewed by hand. Never hand-edit a golden. Never touch `tests/oracle/vectors/calls/`.
- Fixture comments are read raw by `ProjectSignals::absorb` (comments count). A fixture comment must not contain `isatty`, `isTTY`, `is_terminal`, `IsTerminal`, `--no-progress`, `NO_COLOR`, `AdaptiveColor`, `unicode-width`, `wcwidth`, `grapheme`, `--icons`, `--ascii`, `--plain`, or `--json`: DELTAS.md line 184 records a fixture comment that poisoned a whole directory's signals.
- Test names state outcomes (e.g. `one_length_per_layout_call_is_not_an_all_length_layout`), not function names.
- Known environment baseline, recorded in Task 1 and not fixed here: `tests/release.test.mjs` fails on this machine (signed tags forced); a local llama-server on port 8080 breaks oracle case `detect-fixture-text-framework-vite`; bun tests can flake on the 5 s timeout under load (re-run the file in isolation before calling it a failure).

## Decisions fixed while planning

1. **Emoji_Presentation list.** The codepoints in U+2600 to U+27BF with the Emoji_Presentation property, from `emoji-data.txt` for Unicode 16.0.0 (dated 2024-05-01), fetched and diffed against Unicode 17.0.0 (dated 2025-07-25): the subset is identical, 46 codepoints, hardcoded as the ranges `2614..2615 2648..2653 267F 2693 26A1 26AA..26AB 26BD..26BE 26C4..26C5 26CE 26D4 26EA 26F2..26F3 26F5 26FA 26FD 2705 270A..270B 2728 274C 274E 2753..2755 2757 2795..2797 27B0 27BF` (☔ ☕ ♈..♓ ♿ ⚓ ⚡ ⚪ ⚫ ⚽ ⚾ ⛄ ⛅ ⛎ ⛔ ⛪ ⛲ ⛳ ⛵ ⛺ ⛽ ✅ ✊ ✋ ✨ ❌ ❎ ❓ ❔ ❕ ❗ ➕ ➖ ➗ ➰ ➿). No new crate. The spec's example glyphs ✓ (2713) ✗ (2717) ★ (2605) ☐ (2610) ➜ (279C) are not in it. The canonical set keeps ⚠ (U+26A0), which is not Emoji_Presentation, counting, so the `.rs` fixture still flags on its canonical three.
2. **Exemption (a) covers `Rect::new` too.** The spec says (a) skips "an `80` or `24` literal" and then explains the canonical `Rect::new(0, 0, 80, 24)` keeps firing "because it neither measures nor draws on a canvas", which implies measuring would exempt it. Ruling: in a measuring file, a line matched by `SIZE_80_RE`, `SIZE_24_RE`, or `SIZE_CALL_RE` is skipped, and a `RECT_LITERAL_RE` match is skipped when its text contains a whole-word `80` or `24`. A `Rect::new(10, 10, 200, 100)` in a measuring file still fires. `Constraint::Length` counting is untouched by (a). Pinned by `a_fallback_size_in_a_file_that_measures_the_terminal_is_not_flagged`.
3. **Both file-scoped exemptions read the comment-blanked text** that `scan_terminal_source` receives, so a commented-out `terminal::size()` or `widgets::canvas` exempts nothing. Pinned in the (a) test.
4. **The canvas pattern is the spec's literal, `widgets::canvas\b`.** Checked against the PR 2 manual-pass checkout (scratchpad `manual-pass/ratatui`): every ratatui example that uses the canvas imports it as `use ratatui::widgets::canvas::{...}`, including `examples/apps/canvas/src/main.rs:26`, the false positive the manual pass recorded. The nested form `widgets::{canvas::...}` and the `ratatui_widgets::canvas` crate path are not matched; no sample uses them, so no speculative pattern is added. Glow's fallback lines (`glow/main.go:196`, `glow/ui/rawpaint.go:36`) both call `term.GetSize(`, which (a)'s pattern matches.
5. **The paren walker (`layout_call_spans`).** Joins the lines with `\n`, finds each `LAYOUT_CALL_RE` match (`Layout::(?:vertical|horizontal|new)\(|\.constraints\(`), and walks bytes from the opener's `(` counting `(` and `)` only, to the byte after the matching `)`; an unbalanced call ends at the end of the text. A match that starts inside an earlier span belongs to that span, so `Layout::new(d, x.constraints(...))` is one call. `Length` and flexible constraints are counted inside the span only; a span with two or more `Length` and no flexible constraint yields one finding, reported at the line of its first `Length` with `lengthConstraints` = the span's count. Parens inside string literals are not special-cased (comments are already blanked; a string with a paren inside a layout call is not a real pattern). `Length(` without the `Constraint::` prefix (after `use Constraint::*`) is not counted, as today. This keeps the existing unit tests (line 2 count 3, line 1 count 2) and the `.rs` golden (line 3 count 2) unchanged, and `area.centered(Constraint::Length(w), Constraint::Length(h))`, the manual-pass false positive, reports nothing.
6. **Pass fixtures leave a trace in the golden.** A file with no findings is invisible in a golden, so each new pass file also carries one line that still fires: `tui-hardcoded-size.measured.rs` (measures with `terminal::size()`; its `Rect::new(0, 0, 80, 24)` and `width = 80` are skipped; an all-`Length` layout still fires) and `tui-hardcoded-size.canvas.rs` (imports `widgets::canvas`; its `Rect::new(10, 10, 200, 100)` is skipped; a `width = 80` still fires). Naming: `tests/oracle/cases/detect.mjs` scans each `tui-*` directory with the walker, which takes every `.rs`/`.go`/`.py`/`.tcss`/web-extension file in it, so `<rule>.<variant>.<ext>` needs no loader change. No Rust test reads these fixtures (`crates/html/tests/robustness.rs` and `oracle_html.rs` read only top-level `.html` fixtures and `detect-fixture-json-*` goldens). Both new files are `.rs` and import `ratatui` (and `crossterm` for the measured one), so `classify_terminal_source` admits them.
7. **Spinner.** `SPINNER_RE` becomes `bubbles/(?:v\d+/)?spinner`, matching `github.com/charmbracelet/bubbles/spinner`, `github.com/charmbracelet/bubbles/v2/spinner`, and `charm.land/bubbles/v2/spinner`. The registry description ("A spinner that runs when stdout is not a TTY floods logs and CI output with frames. Check isatty (or a CI / --no-progress flag) and print a static line instead.") names no dropped library and stays as it is. The `.py` and `.tsx` fixtures become pass files by comment only; `detect-terminal-single-file-advisory` moves to the `.go` fixture.
8. **print-in-loop description keeps the spec's backticks.** The brief asks for the description "exactly as the spec gives", and the spec's text contains code spans. It becomes the first registry description with backtick characters; the text renderer prints them literally. Reported to the controller as a judgment call (the alternative is the same sentence without backticks). The three Textual unit tests collapse into one pass test; `TEXTUAL_CLASS_RE`, `PY_DEF_RE`, `PY_PRINT_RE`, and `indent_of` are deleted (nothing else in the workspace uses them).
9. **chalk.** A literal is a `'` or `"` string starting with `#`. A template literal (`` chalk.hex(`#${h}`) ``) is computed and does not match.
10. **Goldens that move:** `detect-terminal-hardcoded-rgb-no-adapt`, `detect-terminal-spinner-no-tty-guard`, `detect-terminal-print-in-loop`, `detect-terminal-hardcoded-size`, `detect-terminal-single-file-advisory`, `detect-terminal-text-all`. `detect-terminal-emoji-density` is re-recorded and must not move (the dingbats line leaves `emojiCount` at 4 on the `.rs` file; the old code would give 12). Verified-unchanged without re-recording: the three `detect-dir-*-all-fixtures`, the two `detect-no-advisory-*`, the three `detect-scope-*`, `detect-fixture-{json,text}-terminal`, `detect-terminal-gate-web`, the three `detect-terminal-product-*`, and the `hook-terminal-platform-*` goldens (the `detect-terminal-project` workspace holds none of the changed triggers). A `tests/oracle/DELTAS.md` entry records the moves, as PR 2 and PR 3 did. Probed while planning on a scratch copy of the fixture tree with Task 8's edits and the pre-PR-5 binary: the web walk (no `--platform`) still prints `[]`, so the new `.tsx` lines trip no web rule; on `--platform terminal` the old code gives `emojiCount` 12 on the emoji `.rs`, `lengthConstraints` 4 on `tui-hardcoded-size.rs` line 3, flags `measured.rs` lines 7 to 9 and `canvas.rs` lines 3 and 4, and matches chalk line 6 but not line 5. Each of those is what the new code must change, which makes every narrowing observable in a golden.

## File structure

| File | Responsibility |
|---|---|
| `crates/core/src/checks/terminal.rs` | the five narrowings; `has_emoji_presentation`, `counts_as_emoji`, `layout_call_spans`, four new regexes; unit tests |
| `crates/foundation/src/registry.rs` | `tui-print-in-loop` description; one pinning test |
| `crates/live/assets/antipatterns.json` (+ `detect-antipatterns-browser.js` if it moves) | regenerated by `cargo xtask bundle` |
| `tests/fixtures/antipatterns/terminal/tui-hardcoded-size/` | two lines appended to `.rs`; new `tui-hardcoded-size.measured.rs`, `tui-hardcoded-size.canvas.rs` |
| `tests/fixtures/antipatterns/terminal/tui-emoji-density/tui-emoji-density.rs` | one dingbats line appended |
| `tests/fixtures/antipatterns/terminal/tui-hardcoded-rgb-no-adapt/tui-hardcoded-rgb-no-adapt.tsx` | three chalk lines appended |
| `tests/fixtures/antipatterns/terminal/tui-spinner-no-tty-guard/*.py`, `*.tsx` | comments become `pass` |
| `tests/fixtures/antipatterns/terminal/tui-print-in-loop/tui-print-in-loop.py` | comment becomes `pass` |
| `tests/oracle/cases/detect.mjs` | `detect-terminal-single-file-advisory` targets the `.go` fixture |
| `tests/oracle/golden/detect-terminal-*.json` | re-recorded, reviewed |
| `tests/oracle/DELTAS.md` | one entry |

---

### Task 1: Branch and baseline

**Files:** none changed.

- [ ] **Step 1: Branch from the fork's main**

```bash
cd /Volumes/Containers/impeccable
git checkout main && git pull --ff-only origin main
git log --oneline --merges -5 | grep -F "terminal-platform-pr4" || echo "PR 4 NOT MERGED"
git checkout -b terminal-platform-pr5
git log --oneline -1
```

Spec section 7 orders the PRs 4, 5, 6, 7. If the grep prints `PR 4 NOT MERGED`, stop and ask the controller before going on: PR 4 also regenerates `crates/live/assets/antipatterns.json` and appends to `tests/oracle/DELTAS.md`, and branching before it lands means resolving both later.

If this plan file is still untracked on `main` (`git status --short docs/superpowers/plans`), commit it alone as the branch's first commit: `git add docs/superpowers/plans/2026-09-25-terminal-platform-pr5-source-rules-tightening.md` and a message `Add the PR 5 source rules implementation plan` with the trailer line. The other PR plans are not this branch's to commit.

- [ ] **Step 2: Build and confirm the tooling**

```bash
cargo build --release -p impeccable 2>&1 | tail -n 2
which wasm-pack || cargo install wasm-pack --locked
bun --version
```

- [ ] **Step 3: Record the baseline**

```bash
cargo test --workspace 2>&1 | grep -E "^test result|FAILED" | sort | uniq -c
IMPECCABLE_BIN="$PWD/target/release/impeccable" bun run test 2>&1 | tail -n 15
T=$HOME/.rustup/toolchains/stable-x86_64-apple-darwin; PATH="$T/bin:$PATH" DYLD_LIBRARY_PATH="$T/lib" cargo xtask bundle --check && echo BUNDLE_FRESH
git status --short   # only untracked local files: .agents/AGENTS.md, .claude/CLAUDE.md, .gemini/GEMINI.md, the upstream issue draft, and other PRs' plan files
```

Known pre-existing failures on this machine, not to be fixed here: `tests/release.test.mjs` (signed tags forced), oracle case `detect-fixture-text-framework-vite` when a llama-server holds port 8080 (`lsof -iTCP:8080 -sTCP:LISTEN` tells you), bun 5 s timeouts under load (pass in isolation). The RTK proxy summarizes `cargo test` output; read the `test result` lines, never grep counts out of the summary.

- [ ] **Step 4: No commit.** Record `BASE=$(git rev-parse HEAD)`, the cargo pass counts, the bun suite results, and which known failures appeared, in the ledger.

---
### Task 2: `tui-print-in-loop` becomes Ink-only, with the new description

**Files:**
- Modify: `crates/core/src/checks/terminal.rs` (the `tui-print-in-loop` block, currently lines 590 to 672; tests at 982 to 988, 1015 to 1030)
- Modify: `crates/foundation/src/registry.rs` (`tui-print-in-loop` row, line 830; a test at the end of `mod tests`, which opens at line 1126)
- Regenerate: `crates/live/assets/antipatterns.json` (and `detect-antipatterns-browser.js` if `cargo xtask bundle` rewrites it)

**Interfaces:**
- `fn scan_terminal_print_in_loop(lines: &[&str], file_path: &str, stack: Stack) -> Vec<Finding>`: signature unchanged; returns nothing for every stack but `Stack::Ink`.
- Removed: `TEXTUAL_CLASS_RE`, `PY_DEF_RE`, `PY_PRINT_RE`, `fn indent_of`.

- [ ] **Step 1: Write the failing tests**

In `crates/core/src/checks/terminal.rs`, delete the three Textual tests `print_inside_a_textual_widget_is_flagged_but_workers_and_module_level_pass` (lines 982 to 988), `non_worker_decorator_does_not_exempt_print` (1015 to 1021), and `print_after_a_nested_helper_is_still_flagged` (1023 to 1030), and put this one where the first of them stood:

```rust
    #[test]
    fn print_in_a_textual_app_is_not_flagged_because_textual_captures_it() {
        // Textual routes `print` to devtools or nowhere while an app runs
        // (Textualize/textual#2952, `App._print`), so it cannot corrupt the frame.
        let src = "from textual.app import App\nprint(\"module level\")\n\nclass Demo(App):\n    def on_mount(self) -> None:\n        print(\"captured by Textual\")\n\n    @network_retry\n    def fetch(self) -> None:\n        def inner() -> None:\n            print(\"nested\")\n        print(\"outer\")\n";
        assert!(scan(src, Stack::Textual, Some(&ALL)).is_empty());
    }
```

`console_log_in_an_ink_component_is_flagged_only_when_patch_console_is_off` (lines 990 to 1000) stays as it is: it pins the half that remains.

In `crates/foundation/src/registry.rs`, append inside `mod tests`, after its last `#[test]` function and before the module's closing brace:

```rust
    #[test]
    fn print_in_loop_description_names_only_ink() {
        let row = ANTIPATTERNS.iter().find(|a| a.id == "tui-print-in-loop").expect("row");
        assert_eq!(
            row.description,
            "`console.log` in an Ink app that sets `patchConsole: false` writes over the rendered frame. Leave `patchConsole` on, or log to a file."
        );
    }
```

- [ ] **Step 2: Run them and watch them fail**

```bash
cargo test -p impeccable-core print_in_a_textual_app_is_not_flagged 2>&1 | tail -n 5
cargo test -p impeccable-foundation print_in_loop_description_names_only_ink 2>&1 | tail -n 5
```

Expected: both FAIL (the Textual arm reports lines 6, 11, and 12; the description is the old sentence).

- [ ] **Step 3: Implement**

In `crates/core/src/checks/terminal.rs`, replace the whole block from the `// ─── tui-print-in-loop ───` banner through the closing brace of `scan_terminal_print_in_loop` (lines 590 to 672) with:

```rust
// ─── tui-print-in-loop ──────────────────────────────────────────────────────
re!(INK_CONSOLE_LOG_RE, r"\bconsole\.log\(");
re!(PATCH_CONSOLE_OFF_RE, r"\bpatchConsole[ \t]*:[ \t]*false\b");

/// Ink only. Textual captures `print` while an app runs (Textualize/textual#2952,
/// `App._print`), so a Python file is never reported.
fn scan_terminal_print_in_loop(lines: &[&str], file_path: &str, stack: Stack) -> Vec<Finding> {
    // Ink defaults to `patchConsole: true`, which captures console output and
    // re-renders it above the frame. Only a file that turns that off can
    // corrupt the frame with `console.log`.
    if stack != Stack::Ink || !lines.iter().any(|l| PATCH_CONSOLE_OFF_RE.is_match(l)) {
        return Vec::new();
    }
    lines
        .iter()
        .enumerate()
        .filter(|(_, l)| INK_CONSOLE_LOG_RE.is_match(l))
        .map(|(i, _)| hit("tui-print-in-loop", file_path, lines, i))
        .collect()
}
```

In `crates/foundation/src/registry.rs`, the `tui-print-in-loop` row's `description` (line 830) becomes:

```rust
        description: "`console.log` in an Ink app that sets `patchConsole: false` writes over the rendered frame. Leave `patchConsole` on, or log to a file.",
```

`name` ("Console output inside a TUI render loop") stays.

- [ ] **Step 4: Run the tests and the crates**

```bash
cargo test -p impeccable-core terminal 2>&1 | tail -n 5
cargo test -p impeccable-foundation 2>&1 | tail -n 5
cargo build --workspace 2>&1 | grep -E "^(warning|error)" | head
```

Expected: all pass; no `unused` warning (the deleted helpers had no other callers: `git grep -n "indent_of\|PY_PRINT_RE\|PY_DEF_RE\|TEXTUAL_CLASS_RE" crates` prints nothing).

- [ ] **Step 5: Regenerate the bundle assets**

```bash
T=$HOME/.rustup/toolchains/stable-x86_64-apple-darwin; PATH="$T/bin:$PATH" DYLD_LIBRARY_PATH="$T/lib" cargo xtask bundle
git status --short crates/live/assets
git diff crates/live/assets/antipatterns.json
```

Expected: `antipatterns.json` changes in exactly one line, the `tui-print-in-loop` description (JSON-escaped, backticks literal). If `detect-antipatterns-browser.js` also changed, stage it too; the ledger records which of the two moved.

- [ ] **Step 6: Commit**

```bash
git add crates/core/src/checks/terminal.rs crates/foundation/src/registry.rs crates/live/assets/antipatterns.json
git status --short crates/live/assets   # add detect-antipatterns-browser.js explicitly if it is listed as modified
git commit -m "$(printf 'Make tui-print-in-loop Ink-only\n\nTextual captures print while an app runs (Textualize/textual#2952,\nApp._print), so the Textual half reported code that cannot corrupt the\nframe. The rule now fires only on console.log in an Ink file that sets\npatchConsole: false, and the registry description says so.\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 3: `tui-spinner-no-tty-guard` fires only on Bubbles' spinner

**Files:**
- Modify: `crates/core/src/checks/terminal.rs` (`SPINNER_RE`, line 459; tests at 917 to 924 and 1002 to 1013)

**Interfaces:**
- `SPINNER_RE` now matches only `bubbles/(?:v\d+/)?spinner`. `scan_terminal_spinner` is unchanged.

- [ ] **Step 1: Write the failing tests**

Replace the body of `spinner_without_tty_guard_is_flagged_and_a_guard_silences_it` (lines 917 to 924) so its positive case is Bubbles:

```rust
    #[test]
    fn spinner_without_tty_guard_is_flagged_and_a_guard_silences_it() {
        let src = "import \"github.com/charmbracelet/bubbles/spinner\"\n";
        assert_eq!(ids(&scan(src, Stack::Charm, Some(&NONE))), vec!["tui-spinner-no-tty-guard"]);
        assert!(scan(src, Stack::Charm, Some(&ALL)).is_empty());
        assert!(scan("s := spinner.New()\n", Stack::Charm, Some(&NONE)).is_empty(), "the import path is what counts");
        let v2 = "import \"github.com/charmbracelet/bubbles/v2/spinner\"\n";
        assert_eq!(ids(&scan(v2, Stack::Charm, Some(&NONE))), vec!["tui-spinner-no-tty-guard"]);
    }

    #[test]
    fn self_guarding_spinner_libraries_are_not_flagged() {
        // Each of these stops animating when output is not a terminal:
        // ora (isEnabled), Rich (Live refreshes only on a terminal console),
        // briandowns/spinner (Start is a no-op off a terminal), and Ink's
        // renderer (non-interactive in CI or without a TTY).
        for (src, stack) in [
            ("import Spinner from 'ink-spinner';\n", Stack::Ink),
            ("const s = ora('Loading').start();\n", Stack::Ink),
            ("from rich.spinner import Spinner\n", Stack::Textual),
            ("import \"github.com/briandowns/spinner\"\n", Stack::Charm),
        ] {
            assert!(scan(src, stack, Some(&NONE)).is_empty(), "{src}");
        }
    }
```

`single_file_scan_downgrades_absence_rules` (lines 1002 to 1013) used an Ink spinner; its body becomes:

```rust
    #[test]
    fn single_file_scan_downgrades_absence_rules() {
        let src = "const c = <Text color=\"#ff0080\">{title.slice(0, 8) + '...'}</Text>;\nconst i = \"\u{e0a0}\";\n";
        let mut f = scan(src, Stack::Ink, None);
        f.extend(scan("import \"github.com/charmbracelet/bubbles/spinner\"\n", Stack::Charm, None));
        let noted: Vec<&str> = f.iter().filter(|f| f.extras.get("note") == Some(&Value::String(SIGNALS_NOTE.into()))).map(|f| f.antipattern.as_str()).collect();
        for id in ["tui-spinner-no-tty-guard", "tui-hardcoded-rgb-no-adapt", "tui-grapheme-unsafe-truncate", "tui-nerd-glyph-no-fallback"] {
            assert!(noted.contains(&id), "{id} should carry the note: {f:?}");
        }
        for f in &f {
            assert_eq!(f.severity, "advisory", "{}", f.antipattern);
        }
    }
```

`a_v2_bubbles_spinner_import_is_seen` (lines 1038 to 1042) stays.

- [ ] **Step 2: Run and watch the new test fail**

```bash
cargo test -p impeccable-core spinner 2>&1 | tail -n 8
```

Expected: `self_guarding_spinner_libraries_are_not_flagged` FAILS on `ink-spinner`; the other spinner tests and `single_file_scan_downgrades_absence_rules` pass already.

- [ ] **Step 3: Implement** (line 458 to 459):

```rust
// ─── tui-spinner-no-tty-guard ───────────────────────────────────────────────
// Only Bubbles' spinner: ora, Rich, briandowns/spinner, and Ink's renderer
// all stop animating off a TTY on their own. Matches the github.com and the
// charm.land v2 module paths.
re!(SPINNER_RE, r"bubbles/(?:v\d+/)?spinner");
```

- [ ] **Step 4: Run**

```bash
cargo test -p impeccable-core terminal 2>&1 | tail -n 5
```

Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/checks/terminal.rs
git commit -m "$(printf 'Limit tui-spinner-no-tty-guard to the Bubbles spinner\n\nora, Rich, briandowns/spinner, and the Ink renderer stop animating off a\nTTY on their own; only Bubbles needs the project-level guard.\n\nAI assistance: drafted with Claude Code.')"
```

---
### Task 4: `tui-emoji-density` counts dingbats only with emoji presentation

**Files:**
- Modify: `crates/core/src/checks/terminal.rs` (`is_emoji` and `scan_terminal_emoji_density`, lines 378 to 406; tests after `emoji_density_reports_once_per_file_with_a_count`, line 869)

**Interfaces:**
- New: `fn has_emoji_presentation(c: char) -> bool` (the 46 codepoints of decision 1).
- New: `fn counts_as_emoji(c: char, next: Option<char>) -> bool`, replacing `fn is_emoji(c: char) -> bool` (its only caller is `scan_terminal_emoji_density`).
- `EMOJI_FILE_THRESHOLD` stays 8 and the comparison stays `>=`; `CANONICAL_EMOJI` and its threshold of 3 are unchanged.

- [ ] **Step 1: Write the failing tests** after `emoji_density_reports_once_per_file_with_a_count`:

```rust
    #[test]
    fn text_presentation_dingbats_do_not_count_as_emoji() {
        // Eight glyphs from U+2600 to U+27BF without Emoji_Presentation:
        // ordinary one-cell TUI marks, not emoji.
        let marks = "let m = [\"✓\", \"✗\", \"★\", \"☐\", \"➜\", \"☆\", \"✔\", \"✘\"];\n";
        assert!(scan(marks, Stack::Ratatui, Some(&ALL)).is_empty());
        // Four with Emoji_Presentation, and four made emoji by U+FE0F.
        let shown = "let m = [\"☔\", \"☕\", \"⚡\", \"⛔\", \"❤\u{FE0F}\", \"☀\u{FE0F}\", \"✔\u{FE0F}\", \"✓\u{FE0F}\"];\n";
        let f = scan(shown, Stack::Ratatui, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-emoji-density"]);
        assert_eq!(f[0].extras.get("emojiCount"), Some(&Value::from(8u64)));
    }

    #[test]
    fn canonical_warning_sign_counts_without_a_variation_selector() {
        // ⚠ (U+26A0) lacks Emoji_Presentation but is in the canonical set.
        let src = "print(\"⚠ retry\")\nprint(\"✅ done\")\nprint(\"❌ failed\")\n";
        assert_eq!(ids(&scan(src, Stack::Textual, Some(&ALL))), vec!["tui-emoji-density"]);
    }

    #[test]
    fn eight_emoji_reach_the_threshold_and_seven_do_not() {
        let seven = "let s = \"🌀 🌁 🌂 🌃 🌄 🌅 🌆\";\n";
        assert!(scan(seven, Stack::Ratatui, Some(&ALL)).is_empty());
        let eight = "let s = \"🌀 🌁 🌂 🌃 🌄 🌅 🌆 🌇\";\n";
        let f = scan(eight, Stack::Ratatui, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-emoji-density"]);
        assert_eq!(f[0].extras.get("emojiCount"), Some(&Value::from(8u64)));
    }
```

- [ ] **Step 2: Run and watch the first fail**

```bash
cargo test -p impeccable-core emoji 2>&1 | tail -n 8
```

Expected: `text_presentation_dingbats_do_not_count_as_emoji` FAILS (the eight marks count today); the other two pass already and pin the behavior that must survive (threshold `>= 8`, canonical ⚠).

- [ ] **Step 3: Implement.** Replace `fn is_emoji` (lines 383 to 385) with:

```rust
/// U+2600 to U+27BF codepoints with the Unicode Emoji_Presentation property
/// (emoji-data.txt, Unicode 16.0.0; the subset is unchanged in 17.0.0).
/// The rest of that range (✓ ✗ ★ ☐ ➜) is a one-cell text glyph by default.
fn has_emoji_presentation(c: char) -> bool {
    matches!(
        c,
        '\u{2614}'..='\u{2615}'
            | '\u{2648}'..='\u{2653}'
            | '\u{267F}'
            | '\u{2693}'
            | '\u{26A1}'
            | '\u{26AA}'..='\u{26AB}'
            | '\u{26BD}'..='\u{26BE}'
            | '\u{26C4}'..='\u{26C5}'
            | '\u{26CE}'
            | '\u{26D4}'
            | '\u{26EA}'
            | '\u{26F2}'..='\u{26F3}'
            | '\u{26F5}'
            | '\u{26FA}'
            | '\u{26FD}'
            | '\u{2705}'
            | '\u{270A}'..='\u{270B}'
            | '\u{2728}'
            | '\u{274C}'
            | '\u{274E}'
            | '\u{2753}'..='\u{2755}'
            | '\u{2757}'
            | '\u{2795}'..='\u{2797}'
            | '\u{27B0}'
            | '\u{27BF}'
    )
}

/// Spec section 7, PR 5: U+1F300 to U+1FAFF always count. A codepoint in
/// U+2600 to U+27BF counts only with default emoji presentation, a following
/// U+FE0F, or membership in the canonical set.
fn counts_as_emoji(c: char, next: Option<char>) -> bool {
    match c {
        '\u{1F300}'..='\u{1FAFF}' => true,
        '\u{2600}'..='\u{27BF}' => {
            has_emoji_presentation(c) || next == Some('\u{FE0F}') || CANONICAL_EMOJI.contains(&c)
        }
        _ => false,
    }
}
```

In `scan_terminal_emoji_density`, the loop (lines 391 to 399) becomes:

```rust
    for (i, l) in lines.iter().enumerate() {
        let chars: Vec<char> = l.chars().collect();
        for (k, &c) in chars.iter().enumerate() {
            if !counts_as_emoji(c, chars.get(k + 1).copied()) {
                continue;
            }
            total += 1;
            if CANONICAL_EMOJI.contains(&c) {
                canonical += 1;
            }
            first.get_or_insert(i);
        }
    }
```

The `match first { ... }` after it is unchanged.

- [ ] **Step 4: Run**

```bash
cargo test -p impeccable-core terminal 2>&1 | tail -n 5
```

Expected: all pass, including `emoji_density_reports_once_per_file_with_a_count` (its nine emoji are all in U+1F300 to U+1FAFF or Emoji_Presentation: ✅ 2705, ❌ 274C, ✨ 2728).

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/checks/terminal.rs
git commit -m "$(printf 'Count U+2600 to U+27BF as emoji only with emoji presentation\n\nA dingbat counts toward tui-emoji-density only when it has the Unicode\nEmoji_Presentation property (Unicode 16.0 emoji-data, unchanged in\n17.0), is followed by U+FE0F, or is in the canonical set. Text marks\nsuch as the check mark, ballot X, and star stop counting.\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 5: `tui-hardcoded-rgb-no-adapt` matches chalk only with a literal color

**Files:**
- Modify: `crates/core/src/checks/terminal.rs` (`RGB_RE`, lines 424 to 427; a test after `hardcoded_truecolor_reports_plainly_without_adaptation_and_with_a_note_otherwise`, line 896)

**Interfaces:**
- `RGB_RE`'s `chalk\.hex\(` alternative is replaced by two alternatives; every other alternative is byte-identical.

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn chalk_is_flagged_only_for_a_literal_color() {
        for src in [
            "chalk.hex('#ff00aa')('x')",
            "chalk.bold.hex(\"#ff00aa\")('x')",
            "chalk.bgHex('#101010')('x')",
            "chalk.rgb(255, 0, 170)('x')",
            "chalk.underline.bgRgb(16, 16, 16)('x')",
        ] {
            assert_eq!(ids(&scan(src, Stack::Ink, Some(&NONE))), vec!["tui-hardcoded-rgb-no-adapt"], "{src}");
        }
        for src in [
            "chalk.hex(theme.primary)('x')",
            "chalk.bgHex(color)('x')",
            "chalk.rgb(r, g, b)('x')",
            "chalk.hex(`#${hex}`)('x')",
        ] {
            assert!(scan(src, Stack::Ink, Some(&NONE)).is_empty(), "a computed argument is not a hardcoded color: {src}");
        }
    }
```

- [ ] **Step 2: Run and watch it fail**

```bash
cargo test -p impeccable-core chalk_is_flagged_only_for_a_literal_color 2>&1 | tail -n 5
```

Expected: FAIL. Today `chalk.bold.hex`, `bgHex`, `rgb`, and `bgRgb` literals are missed, and `chalk.hex(theme.primary)` is matched.

- [ ] **Step 3: Implement.** `RGB_RE` (lines 424 to 427) becomes:

```rust
re!(
    RGB_RE,
    r##"Color::Rgb\((?:[ \t]*(?:0x[0-9a-fA-F]+|\d+)[ \t]*,){2}[ \t]*(?:0x[0-9a-fA-F]+|\d+)[ \t]*\)|Color::from_u32\(0x|lipgloss\.Color\("#|\bchalk(?:\.[A-Za-z]+)*\.(?:hex|bgHex)\([ \t]*["']#|\bchalk(?:\.[A-Za-z]+)*\.(?:rgb|bgRgb)\([ \t]*\d|\bcolor=["']#|\[#[0-9a-fA-F]{6}\]|\\x1b\[38;2;|\\033\[38;2;|\x1b\[38;2;"##
);
```

- [ ] **Step 4: Run**

```bash
cargo test -p impeccable-core terminal 2>&1 | tail -n 5
```

Expected: all pass, including `hardcoded_truecolor_reports_plainly_without_adaptation_and_with_a_note_otherwise` and `computed_and_destructured_rgb_is_not_a_hardcoded_color`.

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/checks/terminal.rs
git commit -m "$(printf 'Match chalk colors only when the argument is a literal\n\ntui-hardcoded-rgb-no-adapt now matches .hex( or .bgHex( with a string\nstarting with #, and .rgb( or .bgRgb( with a number, after chalk and any\nchained styles. A computed argument such as chalk.hex(theme.primary) no\nlonger matches.\n\nAI assistance: drafted with Claude Code.')"
```

---
### Task 6: `tui-hardcoded-size` skips fallbacks in measuring files and canvas rectangles

**Files:**
- Modify: `crates/core/src/checks/terminal.rs` (regexes at lines 476 to 485; `scan_terminal_hardcoded_size`, lines 504 to 526; tests after `two_length_constraints_on_one_line_still_count_as_two`, line 955)

**Interfaces:**
- New regexes: `MEASURES_TERMINAL_RE` (spec (a), verbatim), `CANVAS_RE` (`widgets::canvas\b`, decision 4), `SIZE_LITERAL_RE` (`\b(?:80|24)\b`).
- `scan_terminal_hardcoded_size(lines: &[&str], file_path: &str) -> Vec<Finding>`: signature unchanged.

- [ ] **Step 1: Write the failing tests**

```rust
    #[test]
    fn a_fallback_size_in_a_file_that_measures_the_terminal_is_not_flagged() {
        // glow's idiom: the literal is what the program uses when GetSize fails.
        let go = "w, _, err := term.GetSize(int(os.Stdout.Fd()))\nif err != nil {\n\twidth = 80\n}\nmsg := tea.WindowSizeMsg{Width: 80, Height: 24}\n";
        assert!(scan_terminal_source(go, "main.go", Stack::Charm, Some(&ALL)).is_empty());
        let rs = "let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));\nlet fallback = Rect::new(0, 0, 80, 24);\nlet width = 80;\n";
        assert!(scan(rs, Stack::Ratatui, Some(&ALL)).is_empty());
        let py = "cols = shutil.get_terminal_size().columns\nsize=(80, 24)\n";
        assert!(scan(py, Stack::Textual, Some(&ALL)).is_empty());
        let rich = "if console.width < 100:\n    columns = 80\n";
        assert!(scan(rich, Stack::Textual, Some(&ALL)).is_empty());
        let ink = "const cols = process.stdout.columns ?? 80;\nexport const A = () => <Box width={80} />;\n";
        assert!(scan(ink, Stack::Ink, Some(&ALL)).is_empty());
        // A rectangle that is not an 80x24 fallback still fires in a measuring file.
        let other = "let (c, r) = crossterm::terminal::size()?;\nlet r = Rect::new(10, 10, 200, 100);\n";
        assert_eq!(ids(&scan(other, Stack::Ratatui, Some(&ALL))), vec!["tui-hardcoded-size"]);
        // A measurement that is commented out exempts nothing: the scan sees blanked text.
        let blanked = strip_c_comments("// let (w, h) = crossterm::terminal::size()?;\nlet width = 80;\n");
        assert_eq!(ids(&scan(&blanked, Stack::Ratatui, Some(&ALL))), vec!["tui-hardcoded-size"]);
        // The canonical case neither measures nor draws on a canvas.
        assert_eq!(ids(&scan("fn area() -> Rect { Rect::new(0, 0, 80, 24) }\n", Stack::Ratatui, Some(&ALL))), vec!["tui-hardcoded-size"]);
    }

    #[test]
    fn a_literal_rect_in_a_canvas_file_is_a_drawing_coordinate() {
        let src = "use ratatui::widgets::canvas::{Canvas, Rectangle};\nlet r = Rect::new(10, 10, 200, 100);\nlet width = 80;\n";
        let f = scan(src, Stack::Ratatui, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-hardcoded-size"], "the canvas exemption covers Rect::new only");
        assert_eq!(f[0].line, 3.0);
    }
```

- [ ] **Step 2: Run and watch them fail**

```bash
cargo test -p impeccable-core -- a_fallback_size_in_a_file a_literal_rect_in_a_canvas 2>&1 | tail -n 8
```

Expected: both FAIL (every literal fires today).

- [ ] **Step 3: Implement.** After `re!(CONSTRAINT_FLEX_RE, ...)` (line 485) add:

```rust
// Spec section 7, PR 5 (a): a file that measures the terminal holds its 80
// or 24 as the fallback for when the measurement fails.
re!(
    MEASURES_TERMINAL_RE,
    r"GetSize\(|terminal::size\(|terminal_size\(|get_terminal_size\(|stdout\.columns|stdout\.rows|console\.(?:width|size)\b"
);
// (c): in a file that uses ratatui's canvas, a literal Rect is a drawing
// coordinate, not the terminal.
re!(CANVAS_RE, r"widgets::canvas\b");
re!(SIZE_LITERAL_RE, r"\b(?:80|24)\b");
```

In `scan_terminal_hardcoded_size`, replace the first statement after the `in_test_path` guard (the `let mut out: Vec<Finding> = lines ... .collect();` at lines 508 to 515) with:

```rust
    let text = lines.join("\n");
    let measures = MEASURES_TERMINAL_RE.is_match(&text);
    let canvas = CANVAS_RE.is_match(&text);
    let mut out: Vec<Finding> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| {
            let rect = RECT_LITERAL_RE
                .find(l)
                .is_some_and(|m| !canvas && !(measures && SIZE_LITERAL_RE.is_match(m.as_str())));
            let literal =
                !measures && (SIZE_80_RE.is_match(l) || SIZE_24_RE.is_match(l) || SIZE_CALL_RE.is_match(l));
            rect || literal
        })
        .map(|(i, _)| hit("tui-hardcoded-size", file_path, lines, i))
        .collect();
```

The `lengths` block and the sort that follow are unchanged in this task (Task 7 replaces the `lengths` block and reuses `text`).

- [ ] **Step 4: Run**

```bash
cargo test -p impeccable-core terminal 2>&1 | tail -n 5
```

Expected: all pass, including `hardcoded_terminal_size_is_flagged_outside_test_paths` (its source has no measurement: `term.Width(80)` is not `GetSize(`) and `test_paths_are_whole_segments_or_affixes`.

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/checks/terminal.rs
git commit -m "$(printf 'Skip terminal-size fallbacks and canvas rectangles\n\ntui-hardcoded-size no longer reports an 80 or 24 literal in a file that\nmeasures the terminal, where it is the fallback, or a literal Rect::new\nin a file that uses widgets::canvas, where it is a drawing coordinate.\nThe canonical fn area() -> Rect { Rect::new(0, 0, 80, 24) } still fires.\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 7: `tui-hardcoded-size` counts `Length` per layout call

**Files:**
- Modify: `crates/core/src/checks/terminal.rs` (a regex and a helper before `scan_terminal_hardcoded_size`; the `lengths` block inside it; tests after Task 6's)

**Interfaces:**
- New: `re!(LAYOUT_CALL_RE, r"Layout::(?:vertical|horizontal|new)\(|\.constraints\(")`.
- New: `fn layout_call_spans(text: &str) -> Vec<(usize, usize)>`: byte ranges `[open_paren, after_close_paren)` of each top-level layout call (decision 5).

- [ ] **Step 1: Write the failing tests**

```rust
    #[test]
    fn one_length_per_layout_call_is_not_an_all_length_layout() {
        let src = "let top = Layout::vertical([Constraint::Length(1)]);\nlet side = Layout::horizontal([Constraint::Length(20)]);\n";
        assert!(scan(src, Stack::Ratatui, Some(&ALL)).is_empty());
        // The manual-pass false positive: two Lengths that size a centered box.
        let centered = "let r = area.centered(Constraint::Length(w), Constraint::Length(h));\n";
        assert!(scan(centered, Stack::Ratatui, Some(&ALL)).is_empty());
    }

    #[test]
    fn a_rigid_layout_fires_even_when_another_call_is_flexible() {
        let src = "let a = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]);\nlet b = Layout::default()\n    .direction(Direction::Horizontal)\n    .constraints([\n        Constraint::Length(10),\n        Constraint::Length(30),\n    ])\n    .split(area);\n";
        let f = scan(src, Stack::Ratatui, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-hardcoded-size"]);
        assert_eq!(f[0].line, 5.0);
        assert_eq!(f[0].extras.get("lengthConstraints"), Some(&Value::from(2u64)));
    }

    #[test]
    fn every_layout_opener_is_walked_to_its_close_paren() {
        let new = "let l = Layout::new(Direction::Vertical, [Constraint::Length(1), Constraint::Length(2)]);\n";
        assert_eq!(ids(&scan(new, Stack::Ratatui, Some(&ALL))), vec!["tui-hardcoded-size"]);
        // A nested .constraints( inside a Layout call is the same call, reported once.
        let nested = "let l = Layout::new(d, base.constraints([Constraint::Length(1), Constraint::Length(2)]));\n";
        assert_eq!(ids(&scan(nested, Stack::Ratatui, Some(&ALL))), vec!["tui-hardcoded-size"]);
        // An unclosed call runs to the end of the file.
        let open = "let l = Layout::vertical([\n    Constraint::Length(1),\n    Constraint::Length(2),\n";
        let f = scan(open, Stack::Ratatui, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-hardcoded-size"]);
        assert_eq!(f[0].line, 2.0);
    }
```

- [ ] **Step 2: Run and watch them fail**

```bash
cargo test -p impeccable-core -- one_length_per_layout_call a_rigid_layout_fires every_layout_opener 2>&1 | tail -n 8
```

Expected: `one_length_per_layout_call_is_not_an_all_length_layout` FAILS (file-wide count is 2 today) and `a_rigid_layout_fires_even_when_another_call_is_flexible` FAILS (a flexible constraint anywhere silences the file today). `every_layout_opener_is_walked_to_its_close_paren` may pass already; it pins the walker.

- [ ] **Step 3: Implement.** After Task 6's `SIZE_LITERAL_RE` add:

```rust
re!(LAYOUT_CALL_RE, r"Layout::(?:vertical|horizontal|new)\(|\.constraints\(");

/// The byte span of each layout call's argument list, from the opener's `(`
/// to just past its matching `)`, or to the end of the text when the call is
/// unbalanced. Only parens are counted; comments are already blanked and a
/// paren inside a string literal is not special-cased. An opener that starts
/// inside an earlier span belongs to that call.
fn layout_call_spans(text: &str) -> Vec<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut spans = Vec::new();
    let mut covered = 0usize;
    for m in LAYOUT_CALL_RE.find_iter(text) {
        if m.start() < covered {
            continue;
        }
        let open = m.end() - 1; // every alternative ends on the `(`
        let mut depth = 0usize;
        let mut end = text.len();
        for (k, &b) in bytes.iter().enumerate().skip(open) {
            match b {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = k + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        covered = end;
        spans.push((open, end));
    }
    spans
}
```

In `scan_terminal_hardcoded_size`, replace the file-wide block (the `let lengths: Vec<usize> = ...` statement and the `if lengths.len() >= 2 && ...` block after it) with:

```rust
    // (b) Constraints are counted per layout call, so the two Lengths of a
    // centered box or two one-Length layouts are not an all-Length layout.
    for (start, end) in layout_call_spans(&text) {
        let call = &text[start..end];
        let lengths: Vec<usize> = CONSTRAINT_LENGTH_RE.find_iter(call).map(|m| start + m.start()).collect();
        if lengths.len() >= 2 && !CONSTRAINT_FLEX_RE.is_match(call) {
            let line = text[..lengths[0]].matches('\n').count();
            out.push(with_count(hit("tui-hardcoded-size", file_path, lines, line), "lengthConstraints", lengths.len()));
        }
    }
```

The final `out.sort_by(...)` and `out` are unchanged. `text` is `lines.join("\n")`, which restores the source exactly because `scan_terminal_source` built `lines` with `split('\n')`, so byte offsets map back to line indexes.

- [ ] **Step 4: Run**

```bash
cargo test -p impeccable-core terminal 2>&1 | tail -n 5
cargo test --workspace 2>&1 | grep -E "^test result|FAILED" | sort | uniq -c
```

Expected: all pass, including the unchanged `all_length_layout_is_flagged_once_and_a_flexible_constraint_passes` (line 2, count 3) and `two_length_constraints_on_one_line_still_count_as_two` (line 1, count 2).

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/checks/terminal.rs
git commit -m "$(printf 'Count Constraint::Length per layout call\n\ntui-hardcoded-size walks each Layout::vertical(, Layout::horizontal(,\nLayout::new(, or .constraints( call to its matching close paren and fires\nonce for a call with two or more Length and no flexible constraint,\ninstead of counting Length across the whole file.\n\nAI assistance: drafted with Claude Code.')"
```

---
### Task 8: Fixtures, the moved oracle case, goldens, DELTAS

**Files:**
- Modify: `tests/fixtures/antipatterns/terminal/tui-hardcoded-size/tui-hardcoded-size.rs` (append lines 5 and 6)
- Create: `tests/fixtures/antipatterns/terminal/tui-hardcoded-size/tui-hardcoded-size.measured.rs`
- Create: `tests/fixtures/antipatterns/terminal/tui-hardcoded-size/tui-hardcoded-size.canvas.rs`
- Modify: `tests/fixtures/antipatterns/terminal/tui-emoji-density/tui-emoji-density.rs` (append line 4)
- Modify: `tests/fixtures/antipatterns/terminal/tui-hardcoded-rgb-no-adapt/tui-hardcoded-rgb-no-adapt.tsx` (append lines 4 to 6)
- Modify: `tests/fixtures/antipatterns/terminal/tui-spinner-no-tty-guard/tui-spinner-no-tty-guard.py` (line 1 comment), `tui-spinner-no-tty-guard.tsx` (line 2 comment)
- Modify: `tests/fixtures/antipatterns/terminal/tui-print-in-loop/tui-print-in-loop.py` (line 6 comment)
- Modify: `tests/oracle/cases/detect.mjs` (the `detect-terminal-single-file-advisory` case, line 71)
- Re-record: `tests/oracle/golden/detect-terminal-*.json`
- Modify: `tests/oracle/DELTAS.md` (one section at the end)

**Interfaces:** none (data only).

Every fixture comment below was checked against the signal words in Global Constraints; none contains one.

- [ ] **Step 1: Hardcoded size.** Append to `tui-hardcoded-size.rs` (it has four lines; these become 5 and 6):

```rust
fn header() -> Layout { Layout::vertical([Constraint::Length(1)]) } // pass: one Length in this call
fn sidebar() -> Layout { Layout::horizontal([Constraint::Length(20)]) } // pass: one Length in this call
```

Create `tui-hardcoded-size.measured.rs` (exemption (a); nine lines, ending in a newline):

```rust
use crossterm::terminal;
use ratatui::layout::{Constraint, Layout, Rect};
fn area() -> Rect {
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    Rect::new(0, 0, cols, rows)
}
fn fallback() -> Rect { Rect::new(0, 0, 80, 24) } // pass: this file measures the size, so 80x24 is the fallback
fn fallback_width() -> u16 { let width = 80; width } // pass: the same fallback
fn rigid() -> Layout { Layout::horizontal([Constraint::Length(4), Constraint::Length(8)]) } // flag: the exemption covers literals, not layouts
```

Create `tui-hardcoded-size.canvas.rs` (exemption (c); four lines, ending in a newline):

```rust
use ratatui::layout::Rect;
use ratatui::widgets::canvas::{Canvas, Rectangle};
fn bounds() -> Rect { Rect::new(10, 10, 200, 100) } // pass: a canvas drawing coordinate
fn panel() -> u16 { let width = 80; width } // flag: the canvas exemption covers Rect::new only
```

- [ ] **Step 2: Emoji.** Append to `tui-emoji-density.rs` (becomes line 4):

```rust
const MARKS: [&str; 8] = ["✓", "✗", "★", "☐", "➜", "☆", "✔", "✘"]; // pass: text-presentation marks are not emoji
```

- [ ] **Step 3: chalk.** Append to `tui-hardcoded-rgb-no-adapt.tsx` (becomes lines 4 to 6):

```tsx
import chalk from 'chalk';
export const warn = chalk.bold.hex('#ff00aa')('warn'); // flag: a literal after a chained style
export const themed = (theme) => chalk.hex(theme.primary)('ok'); // pass: a computed argument
```

- [ ] **Step 4: Spinner and print become pass files.** Edit only the trailing comments:

`tui-spinner-no-tty-guard.py` line 1 becomes:

```python
from rich.spinner import Spinner  # pass: Rich stops animating when output is not interactive
```

`tui-spinner-no-tty-guard.tsx` line 2 becomes:

```tsx
import Spinner from 'ink-spinner'; // pass: Ink writes only the last frame when output is not interactive
```

`tui-print-in-loop.py` line 6 becomes:

```python
        print("hello")  # pass: Textual captures print while the app runs
```

- [ ] **Step 5: Move the single-file case.** In `tests/oracle/cases/detect.mjs`, the `detect-terminal-single-file-advisory` object (line 71) becomes:

```js
    { id: 'detect-terminal-single-file-advisory', verb: 'detect', args: ['--no-config', '--json', '--platform', 'terminal', `${TERMINAL}/tui-spinner-no-tty-guard/tui-spinner-no-tty-guard.go`], isolateHome: false },
```

- [ ] **Step 6: See the deltas before recording**

```bash
cargo build --release -p impeccable 2>&1 | tail -n 1
IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/run.mjs detect- 2>&1 | grep -E "^(XX|\?\?) |pass, "
IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/run.mjs hook-terminal 2>&1 | tail -n 1
```

Expected `XX` lines, exactly these six: `detect-terminal-hardcoded-rgb-no-adapt`, `detect-terminal-hardcoded-size`, `detect-terminal-print-in-loop`, `detect-terminal-single-file-advisory`, `detect-terminal-spinner-no-tty-guard`, `detect-terminal-text-all`. Everything else passes, in particular `detect-terminal-emoji-density`, `detect-terminal-gate-web`, `detect-fixture-json-terminal`, `detect-fixture-text-terminal`, the three `detect-dir-*-all-fixtures`, the two `detect-no-advisory-*`, the three `detect-scope-*`, and the three `detect-terminal-product-*`; the `hook-terminal` run ends `0 fail`. `detect-fixture-text-framework-vite` may fail only when a llama-server holds port 8080 (the known baseline); record it and move on. Any other `XX` is a stop: the new `.tsx` lines are web-scannable, so a moved web golden means a web rule reads them. Report it to the controller rather than re-recording it.

- [ ] **Step 7: Record the terminal goldens**

Record by exact id, so no unreviewed golden is rewritten. `record.mjs` filters by prefix; each id below matches only its own case (checked while planning against `allCases()`):

```bash
for id in detect-terminal-hardcoded-rgb-no-adapt detect-terminal-hardcoded-size detect-terminal-print-in-loop \
          detect-terminal-single-file-advisory detect-terminal-spinner-no-tty-guard detect-terminal-text-all \
          detect-terminal-emoji-density; do
  IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/record.mjs --bin "$id"
done
git status --short tests/oracle/golden
```

Expected: exactly the six moved files show as modified; `detect-terminal-emoji-density` is re-recorded (the spec asks for every touched rule) and does not show.

- [ ] **Step 8: Review each moved golden by hand** (`git diff tests/oracle/golden/<file>`):
- `detect-terminal-hardcoded-rgb-no-adapt`: one finding added, `tui-hardcoded-rgb-no-adapt.tsx` line 5, snippet `export const warn = chalk.bold.hex('#ff00aa')('warn'); // flag: a literal after a chained style` (Ink snippets keep the trailing comment, as the existing `.tsx` snippets in this golden do). No finding on line 6.
- `detect-terminal-hardcoded-size`: two findings added, `tui-hardcoded-size.canvas.rs` line 4 and `tui-hardcoded-size.measured.rs` line 9 (with `"lengthConstraints": 2`). No finding on `canvas.rs` line 3 or `measured.rs` lines 4, 7, 8. The `.rs` finding on line 3 still carries `"lengthConstraints": 2` (the old file-wide count would now be 4).
- `detect-terminal-print-in-loop`: the `.py` finding is gone; the `.tsx` finding (line 2) remains, with the new description.
- `detect-terminal-spinner-no-tty-guard`: only the `.go` finding (line 2) remains.
- `detect-terminal-single-file-advisory`: one finding, `tui-spinner-no-tty-guard.go` line 2, snippet `import "github.com/charmbracelet/bubbles/spinner"` (Go snippets come from the comment-blanked text, as in the current `detect-terminal-spinner-no-tty-guard` golden), with the `note` "project signals were not collected; scan the project directory to confirm".
- `detect-terminal-text-all`: the same removals and additions in text form, the new print-in-loop description, and a trailing count that stays `44 advisory notes` (three findings leave, three arrive).
- `detect-terminal-emoji-density` did not move: its `.rs` finding still reports `"emojiCount": 4` although the file now holds eight more dingbats.

- [ ] **Step 9: DELTAS entry.** Append this section to the end of `tests/oracle/DELTAS.md`. It is prose on purpose: `run.mjs` treats any line that starts with ``- `<case-id>` `` as an accepted, never-failing delta, and these goldens are re-recorded, not accepted.

```markdown
## Recorded 2026-09-25: source rules tightening (spec PR 5)

Six `detect-terminal-*` goldens were re-recorded from the binary and reviewed by hand. `tui-print-in-loop` is Ink-only, so its Textual fixture line no longer reports, and its description now reads "`console.log` in an Ink app that sets `patchConsole: false` writes over the rendered frame. Leave `patchConsole` on, or log to a file." `tui-spinner-no-tty-guard` fires only on the Bubbles spinner, so its Rich and Ink fixtures became pass files and `detect-terminal-single-file-advisory` now scans the Go fixture, which still reports with the signals note. `tui-hardcoded-rgb-no-adapt` gained a chained chalk literal (flagged) and a computed `chalk.hex` (passes) in its Ink fixture. `tui-hardcoded-size` gained two pass files, one that measures the terminal and one that uses ratatui's canvas; each also holds one line that still fires, so the exemption is visible in the golden. Its `.rs` fixture gained two one-`Length` layouts that per-call counting leaves alone. `detect-terminal-text-all` carries the same changes and still ends at 44 advisory notes. `detect-terminal-emoji-density` did not move: the eight text-presentation dingbats added to its Rust fixture leave `emojiCount` at 4. No web golden changed; the new `.tsx` lines trip no web rule.
```

- [ ] **Step 10: Replay the whole corpus**

```bash
IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/run.mjs 2>&1 | tail -n 3
```

Expected: `0 fail` and `0 missing goldens`; the accepted-delta count equals Task 1's baseline. The one tolerated failure is `detect-fixture-text-framework-vite` while a llama-server holds port 8080 (the known baseline).

- [ ] **Step 11: Commit**

```bash
git add tests/fixtures/antipatterns/terminal/tui-hardcoded-size/tui-hardcoded-size.rs \
  tests/fixtures/antipatterns/terminal/tui-hardcoded-size/tui-hardcoded-size.measured.rs \
  tests/fixtures/antipatterns/terminal/tui-hardcoded-size/tui-hardcoded-size.canvas.rs \
  tests/fixtures/antipatterns/terminal/tui-emoji-density/tui-emoji-density.rs \
  tests/fixtures/antipatterns/terminal/tui-hardcoded-rgb-no-adapt/tui-hardcoded-rgb-no-adapt.tsx \
  tests/fixtures/antipatterns/terminal/tui-spinner-no-tty-guard/tui-spinner-no-tty-guard.py \
  tests/fixtures/antipatterns/terminal/tui-spinner-no-tty-guard/tui-spinner-no-tty-guard.tsx \
  tests/fixtures/antipatterns/terminal/tui-print-in-loop/tui-print-in-loop.py \
  tests/oracle/cases/detect.mjs tests/oracle/DELTAS.md \
  tests/oracle/golden/detect-terminal-hardcoded-rgb-no-adapt.json \
  tests/oracle/golden/detect-terminal-hardcoded-size.json \
  tests/oracle/golden/detect-terminal-print-in-loop.json \
  tests/oracle/golden/detect-terminal-single-file-advisory.json \
  tests/oracle/golden/detect-terminal-spinner-no-tty-guard.json \
  tests/oracle/golden/detect-terminal-text-all.json
git commit -m "$(printf 'oracle: fixtures and goldens for the PR 5 source rules\n\nRe-recorded from the binary and reviewed by hand: the six\ndetect-terminal goldens the tightened rules move. The single-file\nadvisory case now scans the Go spinner fixture. Two pass files pin the\nmeasuring and canvas exemptions of tui-hardcoded-size.\n\nAI assistance: drafted with Claude Code.')"
```

---
### Task 9: Full gates

**Files:** none changed (fix-forward only if a gate fails, in the task that owns the code).

- [ ] **Step 1: Run the PR 5 delivery gates** (spec section 7)

```bash
cargo test --workspace 2>&1 | grep -E "^test result|FAILED" | sort | uniq -c
T=$HOME/.rustup/toolchains/stable-x86_64-apple-darwin; PATH="$T/bin:$PATH" DYLD_LIBRARY_PATH="$T/lib" cargo xtask bundle --check && echo BUNDLE_FRESH
bun run build 2>&1 | tail -n 3
IMPECCABLE_BIN="$PWD/target/release/impeccable" bun run test 2>&1 | tail -n 15
```

Expected: every cargo `test result` line `ok`; `BUNDLE_FRESH` (Tasks 3 to 7 touch only `terminal.rs`, which the browser bundle does not call, so the assets committed in Task 2 stay fresh; if `--check` fails, run the bundle, confirm only the two assets moved, and commit them with the same trailer rule); `bun run build` passes (no README rule count moves: no rule is added or removed); `bun run test` shows the same results as the Task 1 baseline, with only the known environment failures.

- [ ] **Step 2: A quick real-code smoke over the manual-pass checkouts** (read-only; they live in the scratchpad from PR 2):

```bash
S=/private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/manual-pass
./target/release/impeccable detect --no-config --json --platform terminal "$S/glow" 2>/dev/null | node -e 'const f=JSON.parse(require("fs").readFileSync(0,"utf8"));console.log(f.filter(x=>x.antipattern==="tui-hardcoded-size").map(x=>x.file.split("/").slice(-2).join("/")+":"+x.line).join("\n")||"none")'
./target/release/impeccable detect --no-config --json --platform terminal "$S/ratatui/examples" 2>/dev/null | node -e 'const f=JSON.parse(require("fs").readFileSync(0,"utf8"));console.log(f.filter(x=>x.antipattern==="tui-hardcoded-size").map(x=>x.file.split("/").slice(-3).join("/")+":"+x.line).join("\n")||"none")'
```

Expected, against the PR 2 manual-pass table (`docs/superpowers/research/2026-09-24-terminal-rules-manual-pass.md`, section `tui-hardcoded-size`): glow's `main.go:206` and `ui/rawpaint.go:40` fallbacks are gone (`config_cmd.go:23`, the YAML default inside a raw string, is expected to stay); ratatui's `canvas/src/main.rs:65` and `release-header` `main.rs:85` (`centered`) are gone; the two `Viewport::Fixed` findings stay. Line numbers may have drifted if the checkouts were updated; match by file. Record the before and after counts in the ledger. If a checkout is missing, say so in the ledger instead of skipping silently.

- [ ] **Step 3: No commit** unless a gate forced a fix.

---

### Task 10: Final review, merge to the fork's main, push

**Files:** none new.

- [ ] **Step 0: Bring the branch up to date with main.** If `main` moved since Task 1 (PR 4 or a spec edit), merge it into the branch and resolve conflicts here, never in the merge onto `main`:

```bash
git fetch origin && git merge origin/main
T=$HOME/.rustup/toolchains/stable-x86_64-apple-darwin; PATH="$T/bin:$PATH" DYLD_LIBRARY_PATH="$T/lib" cargo xtask bundle
cargo build --release -p impeccable 2>&1 | tail -n 1
IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/run.mjs 2>&1 | tail -n 3
```

A conflict in `crates/live/assets/antipatterns.json` is resolved by taking either side and re-running `cargo xtask bundle`, which regenerates it from both registry changes. A conflict in `tests/oracle/DELTAS.md` keeps both sections, in date order. A golden conflict is resolved by re-running Task 8 steps 6 to 8 on the merged tree. Commit the merge (with the trailer line) and any regenerated asset.

- [ ] **Step 1: Whole-branch review** per subagent-driven-development (final code reviewer on the most capable model), against spec section 7 "PR 5, source rules" and the section 4 rows tagged "(PR 5)". Findings marked Important or Critical get one fix wave and one scoped re-review.

- [ ] **Step 2: Gates once more on the final head**

```bash
cargo test --workspace 2>&1 | tail -n 3
T=$HOME/.rustup/toolchains/stable-x86_64-apple-darwin; PATH="$T/bin:$PATH" DYLD_LIBRARY_PATH="$T/lib" cargo xtask bundle --check && echo BUNDLE_FRESH
bun run build 2>&1 | tail -n 2
cargo build --release -p impeccable 2>&1 | tail -n 1
IMPECCABLE_BIN="$PWD/target/release/impeccable" bun run test 2>&1 | tail -n 8
git status --short   # only the untracked local files may show; never stage them
git log --format=%B main..HEAD | grep -iE "co-authored|claude-session" && echo "TRAILER FOUND: fix before merging" || echo "trailers clean"
```

- [ ] **Step 3: Merge and push** (the fork owner pre-authorized merging each PR to `main` as it lands; Step 0 already absorbed `main`, so this merge is conflict-free; if `main` moved again, repeat Step 0 first):

```bash
git checkout main && git pull --ff-only origin main
git merge --no-ff terminal-platform-pr5 -m "$(printf 'Merge terminal-platform-pr5: source rules tightening (spec PR 5)\n\nAI assistance: drafted with Claude Code.')"
cargo build --release -p impeccable 2>&1 | tail -n 1
IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/run.mjs detect-terminal 2>&1 | tail -n 1
git push origin main
git log --oneline -3
```

- [ ] **Step 4: Close out.** Ledger completion line; copy the ledger to the session scratchpad; update the memory file `terminal-platform-project.md` (PR 5 merged, merge commit id, the five narrowings, that `ENGINE_VERSION` still needs a bump at release, and that no rule was promoted); write a journal entry if the session held a surprise worth keeping. Never post anything upstream.

---

## Self-review notes

- Spec section 7 PR 5 coverage: `tui-hardcoded-size` (a) Task 6, (b) Task 7, (c) Task 6, canonical case pinned in Task 6's test; `tui-emoji-density` range and `>= 8` Task 4; chalk Task 5; spinner Task 3 with the `.py`/`.tsx` pass files and the moved oracle case in Task 8; print-in-loop Task 2 with the exact description and `cargo xtask bundle`.
- Spec "Tests" paragraph: one outcome-named unit test per narrowing (Tasks 2 to 7); separate pass fixture files for (a) and (c) (Task 8 step 1); the other narrowings add lines to their rule's fixture: two one-`Length` layouts, eight dingbats, a computed `chalk.hex`, a chained literal `chalk.bold.hex` (Task 8 steps 1 to 3); every touched rule's golden and `detect-terminal-text-all` re-recorded and reviewed (Task 8 steps 7 and 8).
- Delivery gates (spec section 7): `cargo test --workspace`, `cargo xtask bundle` and `--check`, re-recorded goldens, `bun run build`, `bun run test` (Tasks 2, 8, 9, 10).
- Type and name consistency: `counts_as_emoji(c: char, next: Option<char>)`, `has_emoji_presentation(c: char)`, `layout_call_spans(text: &str) -> Vec<(usize, usize)>`, `MEASURES_TERMINAL_RE`, `CANVAS_RE`, `SIZE_LITERAL_RE`, `LAYOUT_CALL_RE` are spelled the same in every task that uses them; `text` in `scan_terminal_hardcoded_size` is introduced in Task 6 and reused in Task 7.
- Not in this PR: promotion of any rule; `ENGINE_VERSION`; PRs 4, 6, 7; the Rich spinner forms (`console.status(`, `Live(`) that were never matched and now need not be; `Length(` without the `Constraint::` prefix; the nested `widgets::{canvas::...}` import form.
