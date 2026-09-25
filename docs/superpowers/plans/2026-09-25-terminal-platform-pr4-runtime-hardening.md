# Terminal Platform PR 4: Runtime Engine Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the PR 3 runtime gaps: one overflow verdict per row, two recaptures for the spinner, a 5-second deadline on every tmux call, a window restored after SIGINT or SIGTERM, the user's own `window-size` put back, tmux-matched emoji widths, fewer key-hint false positives, a `paletteRelative` contrast extra, and corrected docs.

**Architecture:** All engine changes stay in `crates/terminal` plus one new function in `crates/common/src/proc.rs` (`clear_interrupt`). `capture.rs` learns three cluster-width rules and `Frame::wide_count`. `rules.rs` gains `Overflow` and `overflow_verdict`, shared by width drift and narrow collapse, plus the pager and first-row key-hint exemptions and `paletteRelative`. `tmux.rs` gets a deadline-polling `Tmux::run`, `Tmux::window_size`, and a `restore` that puts the prior value back. `lib.rs` takes recaptures at 700 ms and 1000 ms and moves the size pass into `size_pass`, which settles in 50 ms steps against an interrupt flag the binary registers with `on_interrupt` only around the size pass. Saved-capture fixtures and oracle goldens pin every observable change; `tests/tmux-engine.test.mjs` proves the prior `window-size` survives and that a real SIGINT restores the window.

**Tech Stack:** Rust workspace (`common`, `terminal`, `foundation`), `unicode-width 0.2`, `libc` (already a unix dependency of `common`), node oracle harness, `cargo xtask bundle`, bun build gates, tmux 3.7c on this machine.

**Spec:** `docs/superpowers/specs/2026-09-24-terminal-platform-design.md`, section 7 "PR 4, runtime engine hardening" (items 1 to 9, its Tests paragraph, and the PR 4 delivery gate), plus the "(PR 4)" corrections in sections 4 and 5. Read section 7 and section 5 before any task; every message, delay, and id below is copied from there.

## Global Constraints

- No manifest version bumps, no `ENGINE_VERSION` bump, no changelog entry.
- Every commit body ends with the line `AI assistance: drafted with Claude Code.` and carries no `Co-Authored-By`, `Claude-Session`, or other agent trailer, even if a harness reminder asks for one (the fork owner's rule).
- Never stage `.serena/`, `.claude/CLAUDE.md`, `.gemini/GEMINI.md`, `.agents/AGENTS.md`, `.superpowers/`, regenerated provider output (`.claude/skills/`, `.cursor/`, `plugin/`, ...), anything under `.impeccable/`, `.env`, or `docs/superpowers/research/2026-09-24-upstream-issue-draft.md`. Stage explicit paths only; never `git add -A` or `git add .`.
- `.env` holds API keys; never print it.
- No em dashes in `skill/**/*.md`, registry descriptions, or docs written here. The one exception is `docs/CLI-CONTRACT.md`'s existing `#### path — title` heading form, which this PR does not add to.
- Every rule stays `severity: Some("advisory")`; no promotion.
- Never run a tmux command without `-L <scratch socket>`; never touch the default socket (the owner's `cockpit` session lives there). Kill scratch servers when done.
- A change to `crates/core`, `crates/foundation` (registry descriptions), or `crates/wasm` owes `cargo xtask bundle` and committing the two refreshed assets (`crates/live/assets/detect-antipatterns-browser.js`, `crates/live/assets/antipatterns.json`). The recipe on this machine (memory `impeccable-wasm-toolchain`): `T=$HOME/.rustup/toolchains/stable-x86_64-apple-darwin; PATH="$T/bin:$PATH" DYLD_LIBRARY_PATH="$T/lib" cargo xtask bundle [--check]`.
- Oracle goldens are recorded only with `IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/record.mjs --bin <case-prefix>` after `cargo build --release -p impeccable`, then reviewed by hand. Never hand-edit a golden. Never touch `tests/oracle/vectors/calls/`.
- Test names state outcomes (`joined_prose_row_under_sixty_columns_reports_nothing`), not function names.
- `crates/terminal` never depends on `crates/context`; `crates/detect` never depends on `crates/terminal`. `crates/terminal` gains one dependency, `impeccable-common` (already a workspace dependency).
- Known environment baseline, recorded and not fixed here: `tests/release.test.mjs` fails on this machine (signed tags forced); a local llama-server on port 8080 breaks oracle case `detect-fixture-text-framework-vite`; bun tests can flake on the 5 s timeout under load (they pass run alone).

## Decisions fixed while planning

1. **`on_interrupt` tolerates a second registration.** `crates/common/src/proc.rs:143` stores the flag pointer in the static `FLAG` (an `AtomicPtr`) and, on unix, reinstalls the same `unix_on_signal` handler for SIGINT and SIGTERM and sets SIGPIPE to `SIG_IGN`; a later call replaces the pointer. On Windows it calls `SetConsoleCtrlHandler(Some(win_ctrl_handler), 1)`, which *adds* a handler each call, so every registration needs a matching removal. Its one caller, `crates/live/src/live_server.rs:52`, runs in the live-server process, never in a `detect` run, and never clears. `clear_interrupt()` therefore: on unix sets SIGINT and SIGTERM to `SIG_DFL`; on Windows calls `SetConsoleCtrlHandler(Some(win_ctrl_handler), 0)` (remove); then nulls `FLAG`. It leaves SIGPIPE ignored: the Rust runtime ignores SIGPIPE before `main`, and nothing in the binary resets it (`grep -rn SIGPIPE crates` finds only `proc.rs`), so "ignored" is the state before `on_interrupt` too.
2. **`clear_interrupt` restores `SIG_DFL`, as the spec says, not the inherited disposition.** A process started with SIGINT ignored (a `cmd &` in a non-interactive shell) becomes interruptible after a size pass. `on_interrupt` already overrides an inherited `SIG_IGN` today, so this is not new, and saving the prior disposition would mean changing `on_interrupt`'s other caller's contract. Cost if wrong: one saved `sighandler_t` per signal later.
3. **Registration happens only around the size pass.** With no `--tmux-sizes`, the engine never resizes, so it reads no `window-size`, registers nothing, and makes exactly PR 3's tmux calls plus one more `capture-pane`. With sizes, the order after the two recaptures is: `show-options` (item 5), `INTERRUPTED.store(false)`, `on_interrupt(&INTERRUPTED)`, the size pass, the restore, then `clear_interrupt()`. Clearing *after* the restore means a Ctrl-C during the restore sets the flag rather than killing the process mid-restore; the restore calls never check the flag.
4. **The flag is injected.** `size_pass`, `capture_sizes`, and `settle` take `&AtomicBool`. Only `detect_pane` passes the process-wide static `INTERRUPTED`; unit tests pass a local flag set from a helper thread and never install a signal handler, because `cargo test` runs tests as threads of one process. The real-signal path is proven by `tests/tmux-engine.test.mjs` (a separate process that receives SIGINT).
5. **An interrupt always wins the message.** A terminal Ctrl-C goes to the whole foreground process group, so an in-flight tmux child can die with an empty stderr and `run` returns `tmux capture-pane: `. When the flag is set after the restore, the error is `interrupted; window restored` or `interrupted; window not restored: <restore error>`, whatever the size pass returned.
6. **tmux children run in their own process group** (`CommandExt::process_group(0)`, unix only, stable since Rust 1.64; this machine has 1.98.1). A terminal Ctrl-C then reaches only the engine, so the in-flight capture and every restore call finish instead of dying. The tmux calls read no terminal (stdin is null, stdout and stderr are pipes), so a background process group costs nothing there. The cost: the children leave the process group that `scripts/run-tests.mjs` kills on its wall-clock cap, so if the engine itself is SIGKILLed mid-call, a hung tmux client is orphaned instead of killed with the group (the 5-second deadline dies with the engine). Accepted because the alternative lets a second Ctrl-C kill the restore's own tmux calls; the owner may veto this in review.
7. **The deadline is a field.** `Tmux { exe, env, timeout }`; `Tmux::locate` sets `TIMEOUT` (5 s). The message comes from `timed_out(verb, timeout)`, so the spec string `tmux <verb>: timed out after 5 seconds` is pinned by a unit test on `timed_out("capture-pane", TIMEOUT)` while the hang tests set `timeout` to 1 s and stay fast. On expiry `run` kills and waits for the child and returns without joining the reader threads (as `git_run` in `crates/context/src/signals.rs:58` does), because a grandchild may still hold a pipe. `Tmux::run` reads stdout and stderr on two threads (`git_run` discards stderr; the engine needs its first line).
8. **Several `--tmux` targets are independent.** An interrupt during target 1's size pass fails target 1 with `interrupted; ...`; `clear_interrupt` then restores the default disposition, and target 2 proceeds. A second Ctrl-C outside a size pass kills the process as before. Stopping the remaining targets would need a change in `crates/detect/src/cli.rs` (`scan_tmux_targets`, line 969), outside this PR's scope.
9. **Restore probe (tmux 3.7c, `-L impeccable-pr4-plan`, killed after).** On a detached 80x24 window with no clients: `show-options -w -v -t app:0.0 window-size` printed nothing (unset); after `set-option -w window-size largest` it printed `largest`; after `resize-window -x 40 -y 24` it printed `manual` at 40x24; after `resize-window -x 80 -y 24` plus `set-option -w -t app:0.0 window-size largest` the window read `80x24` and `#{window-size}` read `largest`, still `80x24` 0.5 s later. The unset path (`-u`) read `80x24` and `latest`. `show-options` on a missing target exits 1 with `no such window: nope:0.0`. So both integration assertions hold at once: the existing test keeps `'80x24 latest'` and a new test sets `largest` first and asserts `'80x24 largest'`.
10. **Capture files need no parser change for two recaptures.** `parse_frames` (`capture.rs:110`) already starts a frame at every `#!capture` line and reads `role=recapture` any number of times, so a PR 3 file with one recapture and a PR 4 file with two both replay. The spinner rule compares frame 0 with each recapture in file order and stops at the first finding; a third recapture is compared too rather than rejected (the spec says "may carry one or two"; refusing three would add an error path nothing needs).
11. **Recapture timing.** Both delays are measured from the end of frame 0's capture with one `Instant`, so the second recapture lands at 1000 ms rather than 700 ms plus a capture plus 300 ms. The live engine pushes the 700 ms frame before the 1000 ms frame.
12. **Wide-glyph count** is `Frame::wide_count(row)`: the number of `wide_tail` cells, which after Task 3's rules is one per cluster the parser measured at two cells (❤️ is one). Item 6 lands in Task 3, before the overflow verdict in Task 4.
13. **An overflowing row gets one verdict and skips every other check.** `overflow_verdict` returns `Fits`, `WidthDrift`, `Collapse`, or `Wrapped`. Width drift's border-column check and collapse's corner and split-word checks run only on `Fits` rows, exactly as PR 3's `continue` after the overflow finding did.
14. **Pager prompt matching trims trailing whitespace** before comparing, because `capture-pane -J` keeps trailing spaces: a bottom row of `:` padded with spaces is still exactly `:`. The first-row exemption reuses `KEY_HINT_RE` unchanged. The unit test uses rows captured from macOS `top` on a scratch server during planning (`Processes: 714 total, 2 running, 712 sleeping, 4442 threads   14:22:30` first, a process row last), which still fire.
15. **`paletteRelative` describes the reported cell.** Findings dedupe on the resolved hex pair, so two cells can share a pair while one uses a named color and the other truecolor; the extra is computed from the style of the first failing cell, the one `line` and `column` point at.
16. **Spec gap: two goldens the Tests paragraph does not list.** `paletteRelative` moves `detect-tmux-capture-low-contrast` and `detect-tmux-capture-light-palette`. Ruling: re-record both in Task 11, confirm the diff is the added key only, and name them in `DELTAS.md`. Every other existing `detect-tmux-*` golden except the spinner must replay unchanged.
17. **Spinner discrimination.** The spec's fixture order (frame 0 `⠋`, recapture at 700 ms `⠙`, recapture at 1000 ms `⠋`) also fires under PR 3's first-recapture-only code, so the golden alone cannot prove "compares with each". A crate test with the differing recapture second (`[frame 0, matching, differing]`) must fire and fails on PR 3's code.
18. **Manual pass record.** The spec's PR 4 gate names no file. The result is appended as a `## PR 4 re-run` section to `docs/superpowers/research/2026-09-24-terminal-runtime-rules-manual-pass.md`, next to PR 3's count of 15. The ratatui clone under `scratchpad/manual-pass/ratatui` has no built examples, so Task 14 rebuilds `demo2` under a ten-minute timebox, as PR 3's Task 11 did.

## File structure

| File | Responsibility |
|---|---|
| `crates/common/src/proc.rs` | `clear_interrupt()` and its test |
| `crates/terminal/Cargo.toml` | `impeccable-common` dependency |
| `crates/terminal/src/capture.rs` | three cluster-width rules in `parse_row`; `Frame::wide_count`; the updated width test |
| `crates/terminal/src/rules.rs` | `Overflow`, `overflow_verdict`, the verdict in `rt_width_drift` and `rt_collapse_narrow`; `PAGER_PROMPT_RE` and the first-row check in `rt_no_key_hints`; `paletteRelative` in `rt_low_contrast`; the spinner loop in `scan_frames` and its snippet |
| `crates/terminal/src/tmux.rs` | `TIMEOUT`, `timed_out`, the deadline in `Tmux::run`, `process_group(0)`, `Tmux::window_size`, `restore(.., window_size)` |
| `crates/terminal/src/lib.rs` | `RECAPTURE_DELAYS_MS`, `INTERRUPT_POLL_MS`, `INTERRUPTED`, `settle`, `size_pass`, the new `detect_pane` flow; fake-tmux tests |
| `crates/foundation/src/registry.rs` | the `tui-rt-spinner-never-rests` description |
| `crates/live/assets/antipatterns.json`, `detect-antipatterns-browser.js` | regenerated by `cargo xtask bundle` |
| `tests/fixtures/terminal-captures/` | `collapse-narrow-prose.txt`, `width-drift-narrow.txt`, `no-key-hints-pager.txt` (new); `spinner-never-rests.txt` (a second recapture) |
| `tests/oracle/golden/` | three new `detect-tmux-capture-*` goldens; spinner, low-contrast, light-palette re-recorded |
| `tests/oracle/DELTAS.md` | one reviewed-delta section |
| `tests/tmux-engine.test.mjs` | the `largest` restore test and the SIGINT test |
| `docs/CLI-CONTRACT.md` | the tmux-scans paragraph and the `crates/terminal` section |
| `docs/superpowers/research/2026-09-24-terminal-runtime-rules-manual-pass.md` | the `## PR 4 re-run` section |

`tests/oracle/cases/detect.mjs` needs no edit: lines 81 to 86 build one `detect-tmux-capture-<name>` case per `.txt` file in `tests/fixtures/terminal-captures/` (`fs.readdirSync(...).filter((f) => f.endsWith('.txt')).sort()`), so each new fixture file is its own case, with args `['--no-config', '--json', '--tmux-capture', '<REPO>/tests/fixtures/terminal-captures/<name>.txt']` and `isolateHome: false`.

---

### Task 1: Branch and baseline

**Files:** none changed.

- [ ] **Step 1: Branch from the fork's main**

```bash
cd /Volumes/Containers/impeccable
git checkout main && git pull --ff-only origin main
git checkout -b terminal-platform-pr4
git log --oneline -1   # a main that contains 09afdf2c (the section 7 spec commit)
```

- [ ] **Step 2: Build and confirm the tooling**

```bash
cargo build --release -p impeccable 2>&1 | tail -n 2
which wasm-pack || cargo install wasm-pack --locked
tmux -V            # 3.2 or newer; this machine has 3.7c
bun --version
```

- [ ] **Step 3: Record the baseline**

```bash
cargo test --workspace 2>&1 | grep -E "^test result|FAILED|panicked" | sort | uniq -c
cargo test -p impeccable-terminal 2>&1 | tail -n 3
IMPECCABLE_BIN="$PWD/target/release/impeccable" bun run test 2>&1 | tail -n 15
IMPECCABLE_BIN="$PWD/target/release/impeccable" node --test tests/tmux-engine.test.mjs 2>&1 | grep -E "^ℹ (pass|fail|skipped)"
```

Known pre-existing failures on this machine, not to be fixed here: `tests/release.test.mjs` (signed tags forced), oracle case `detect-fixture-text-framework-vite` when a llama-server holds port 8080, bun timeouts under load (rerun the file alone), and `dev_url::tests::answers_whatever_scheme_the_page_uses` (flaky). The RTK proxy summarizes `cargo test` output; read the `test result` lines, never grep counts out of a compressed block (Rule 16 marker check).

- [ ] **Step 4: No commit.** Record `BASE=$(git rev-parse HEAD)`, the pass counts, and each known failure in the ledger.

---

### Task 2: `clear_interrupt` in `crates/common`

**Files:**
- Modify: `crates/common/src/proc.rs` (new function after `on_interrupt`, which ends at line 164; one test in `mod tests`)

**Interfaces:**
- Produces: `pub fn impeccable_common::proc::clear_interrupt()`, which Task 10 calls after the restore.
- Consumes: the private `FLAG` static (line 166), `win_ctrl_handler` (line 183), `win::SetConsoleCtrlHandler` (line 281).

- [ ] **Step 1: Write the failing test** at the end of `mod tests` in `proc.rs` (after `detached_child_spawns`, before the module's closing brace):

```rust
    #[cfg(unix)]
    #[test]
    fn a_cleared_interrupt_hands_sigint_and_sigterm_back_to_their_defaults() {
        static CAUGHT: AtomicBool = AtomicBool::new(false);
        on_interrupt(&CAUGHT);
        // raise() delivers to the calling thread before it returns.
        unsafe { libc::raise(libc::SIGTERM) };
        assert!(CAUGHT.load(std::sync::atomic::Ordering::SeqCst), "the registered flag caught SIGTERM");
        clear_interrupt();
        for sig in [libc::SIGINT, libc::SIGTERM] {
            // signal() returns the disposition it replaces.
            let previous = unsafe { libc::signal(sig, libc::SIG_DFL) };
            assert_eq!(previous, libc::SIG_DFL, "signal {sig} is back to its default");
        }
        assert!(FLAG.load(std::sync::atomic::Ordering::SeqCst).is_null(), "no flag stays registered");
    }
```

This is the only test in `impeccable-common` that touches signal dispositions, so running it alongside the others in one process is safe.

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test -p impeccable-common a_cleared_interrupt 2>&1 | tail -n 5
```

Expected: compile error, `cannot find function 'clear_interrupt'`.

- [ ] **Step 3: Implement.** Insert immediately after `on_interrupt`'s closing brace (line 164), before `static FLAG`:

```rust
/// Undo [`on_interrupt`]: SIGINT and SIGTERM go back to their default
/// disposition, so the next one terminates the process again, and the
/// registered flag is forgotten. Windows removes the console control
/// handler `on_interrupt` added. SIGPIPE stays ignored, which is the Rust
/// runtime's own state before `main`.
pub fn clear_interrupt() {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGINT, libc::SIG_DFL);
        libc::signal(libc::SIGTERM, libc::SIG_DFL);
    }
    #[cfg(windows)]
    unsafe {
        win::SetConsoleCtrlHandler(Some(win_ctrl_handler), 0);
    }
    FLAG.store(std::ptr::null_mut(), std::sync::atomic::Ordering::SeqCst);
}
```

Also extend `on_interrupt`'s doc comment's last sentence (line 141 to 142, "Only one flag can be registered per process; later calls replace the earlier one.") by appending: ` [`clear_interrupt`] undoes it.`

- [ ] **Step 4: Run the tests**

```bash
cargo test -p impeccable-common 2>&1 | tail -n 3
cargo build -p impeccable-live 2>&1 | tail -n 1    # the other caller still builds
```

Expected: all pass; the live crate builds unchanged.

- [ ] **Step 5: Commit**

```bash
git add crates/common/src/proc.rs
git commit -m "$(printf 'Add clear_interrupt to hand SIGINT and SIGTERM back to their defaults\n\nAI assistance: drafted with Claude Code.')"
```

---
### Task 3: Emoji widths match tmux (item 6) and `Frame::wide_count`

**Files:**
- Modify: `crates/terminal/src/capture.rs` (`Frame` impl near line 86; `parse_row` lines 181 to 245; test `wide_glyphs_take_two_cells_and_zero_width_marks_take_none` at line 365; new tests)

**Interfaces:**
- Produces: `pub fn Frame::wide_count(&self, row: usize) -> usize`, which Task 4's `overflow_verdict` reads. After this task, ❤️ ✔️ ☀️ ⚠️ 👍🏽 🧑‍💻 each parse to two cells (a glyph cell plus a `wide_tail` cell).
- Consumes: nothing new.

- [ ] **Step 1: Write the failing tests** at the end of `mod tests` in `capture.rs`:

```rust
    #[test]
    fn emoji_sequences_measure_two_cells_like_tmux() {
        // The six sequences probed on tmux 3.7c with #{cursor_x} (spec section 7).
        for s in ["❤\u{FE0F}", "✔\u{FE0F}", "☀\u{FE0F}", "⚠\u{FE0F}", "👍\u{1F3FD}", "🧑\u{200D}💻"] {
            let cells = parse_row(s);
            assert_eq!(cells.len(), 2, "{s:?} measures two cells");
            assert!(cells[0].glyph.is_some() && cells[1].wide_tail, "{s:?} is one glyph and its wide tail");
        }
        assert_eq!(parse_row("❤").len(), 1, "without U+FE0F the heart stays one cell, as tmux draws it");
        assert_eq!(parse_row("日\u{FE0F}").len(), 2, "U+FE0F never widens a glyph that is already two cells");
        assert_eq!(parse_row("a\u{200D}b c").len(), 3, "the character after U+200D takes no cell");
    }

    #[test]
    fn a_row_counts_one_wide_glyph_per_two_cell_cluster() {
        let frame = Frame { rows: vec![parse_row("❤\u{FE0F} 🚀 日 ok")], ..Frame::default() };
        assert_eq!(frame.wide_count(0), 3);
        assert_eq!(frame.wide_count(1), 0, "a missing row holds none");
    }
```

And update the existing test `wide_glyphs_take_two_cells_and_zero_width_marks_take_none`: its input ends in `⚠\u{FE0F}`, which item 6 widens to two cells, so the row is now 10 cells. Replace its two `9`s:

```rust
        let cells = parse_row("a😀b日本⚠\u{FE0F}");
        assert_eq!(cells.len(), 10, "⚠ with U+FE0F is two cells since PR 4");
```

and

```rust
        assert_eq!(frame.row_width(0), 10);
```

(the `frame.text(0) == "a😀b日本⚠"` and `has_wide` assertions stay as they are).

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test -p impeccable-terminal capture:: 2>&1 | tail -n 12
```

Expected: compile error (`no method named 'wide_count'`). After Step 3's first half compiles, the two width tests fail on `❤\u{FE0F}` (1 cell) and `👍\u{1F3FD}` (4 cells).

- [ ] **Step 3: Implement.**

In the `impl Frame` block, after `has_wide` (line 86 to 89), add:

```rust
    /// How many two-cell clusters the row holds: one per wide tail.
    pub fn wide_count(&self, row: usize) -> usize {
        self.rows.get(row).map(|r| r.iter().filter(|c| c.wide_tail).count()).unwrap_or(0)
    }
```

In `parse_row`, add `let mut after_zwj = false;` right after `let mut chars = line.chars().peekable();` (line 183), and replace the block from `if ch == '\r' {` (line 232) to the end of the `match ch.width()` (line 242) with:

```rust
        if ch == '\r' {
            continue;
        }
        // tmux 3.7c's cluster widths (spec section 7, PR 4 item 6): a
        // character joined by U+200D is drawn inside the cluster before it,
        // an emoji modifier recolors the glyph before it, and U+FE0F asks
        // for emoji presentation, which tmux draws two cells wide.
        if after_zwj {
            after_zwj = false;
            continue;
        }
        if ch == '\u{200D}' {
            after_zwj = true;
            continue;
        }
        if ('\u{1F3FB}'..='\u{1F3FF}').contains(&ch) {
            continue;
        }
        if ch == '\u{FE0F}' {
            // Only a one-cell glyph widens: a wide glyph's last cell is its tail.
            if let Some(base) = cells.last().filter(|c| c.glyph.is_some()).map(|c| c.style) {
                cells.push(Cell { glyph: None, wide_tail: true, style: base });
            }
            continue;
        }
        match ch.width().unwrap_or(0) {
            0 => {}
            1 => cells.push(Cell { glyph: Some(ch), wide_tail: false, style: *style }),
            _ => {
                cells.push(Cell { glyph: Some(ch), wide_tail: false, style: *style });
                cells.push(Cell { glyph: None, wide_tail: true, style: *style });
            }
        }
```

Update `parse_row`'s doc comment (lines 176 to 180): after "(zero-width characters take none, two-column glyphs take two)" insert ", with tmux's rules for U+FE0F, emoji modifiers, and U+200D on top of `unicode-width`".

- [ ] **Step 4: Run the tests**

```bash
cargo test -p impeccable-terminal 2>&1 | tail -n 3
```

Expected: all pass, including every `rules::` test (none of their rows carry U+FE0F, a modifier, or U+200D).

- [ ] **Step 5: Replay the existing capture goldens.** None of the eight PR 3 fixtures carries one of the three sequences, so none may move:

```bash
cargo build --release -p impeccable 2>&1 | tail -n 1
IMPECCABLE_ORACLE_PREFIX=detect-tmux IMPECCABLE_BIN="$PWD/target/release/impeccable" node --test tests/oracle.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)"
```

Expected: `pass 1`, `fail 0`. A failure here means a fixture row changed width: stop and report it, do not re-record.

- [ ] **Step 6: Commit**

```bash
git add crates/terminal/src/capture.rs
git commit -m "$(printf 'Match tmux widths for U+FE0F, emoji modifiers, and ZWJ sequences\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 4: One overflow verdict per row (item 1)

**Files:**
- Modify: `crates/terminal/src/rules.rs` (new `Overflow`, `is_box_drawing`, `overflow_verdict` before `rt_width_drift` at line 277; `rt_width_drift`; `rt_collapse_narrow` lines 301 to 338; new tests)

**Interfaces:**
- Consumes: `Frame::wide_count` (Task 3), `Frame::row_width`, `NARROW_COLUMNS`.
- Produces: `pub enum Overflow { Fits, WidthDrift, Collapse, Wrapped }` and `pub fn overflow_verdict(frame: &Frame, r: usize) -> Overflow`, the one classification both rules read.

- [ ] **Step 1: Write the failing tests** at the end of `mod tests` in `rules.rs`:

```rust
    const PROSE: &str = "The quick brown fox jumps over the lazy dog while the index rebuilds in the background.";

    #[test]
    fn joined_prose_row_under_sixty_columns_reports_nothing() {
        let f = frame(40, &[&format!("┌{}┐", "─".repeat(38)), &format!("└{}┘", "─".repeat(38)), PROSE, HINTS]);
        assert_eq!(overflow_verdict(&f, 2), Overflow::Wrapped);
        assert!(rt_collapse_narrow(&f, "x").is_empty(), "a row -J joined is the terminal's wrap");
        assert!(rt_width_drift(&f, "x").is_empty());
    }

    #[test]
    fn emoji_overflow_under_sixty_columns_is_width_drift_only() {
        let top = format!("┌{}┐", "─".repeat(38));
        let rocket = format!("│ 🚀 launch{}│", " ".repeat(29)); // 1 + 1 + 2 + 7 + 29 + 1 = 41 cells
        let bottom = format!("└{}┘", "─".repeat(38));
        let f = frame(40, &[&top, &rocket, &bottom, HINTS]);
        assert_eq!(f.row_width(1), 41);
        assert_eq!(overflow_verdict(&f, 1), Overflow::WidthDrift);
        let out = scan_frames(&[f], Palette::Dark, "x");
        assert_eq!(ids(&out), vec!["tui-rt-width-drift"], "one verdict per row: no collapse beside the drift");
        assert_eq!(out[0].snippet, "row measures 41 cells on a 40-column pane");
    }

    #[test]
    fn heart_with_emoji_presentation_counts_as_one_wide_glyph() {
        let one_over = frame(40, &[&format!("│ ❤\u{FE0F} ok{}│", " ".repeat(33))]); // 1 + 1 + 2 + 3 + 33 + 1 = 41
        assert_eq!((one_over.row_width(0), one_over.wide_count(0)), (41, 1));
        assert_eq!(overflow_verdict(&one_over, 0), Overflow::WidthDrift);
        let two_over = frame(40, &[&format!("│ ❤\u{FE0F} ok{}│", " ".repeat(34))]);
        assert_eq!(overflow_verdict(&two_over, 0), Overflow::Collapse, "42 cells with one wide glyph is more than drift");
    }

    #[test]
    fn boxed_overflow_under_sixty_columns_is_collapse_only() {
        let f = frame(40, &[&format!("┌{}┐", "─".repeat(48)), HINTS]);
        assert_eq!(overflow_verdict(&f, 0), Overflow::Collapse);
        assert_eq!(ids(&rt_collapse_narrow(&f, "x")), vec!["tui-rt-collapse-narrow"]);
        assert!(rt_width_drift(&f, "x").is_empty());
    }

    #[test]
    fn overflow_past_the_wide_glyph_count_at_sixty_columns_or_more_reports_nothing() {
        // A joined row in a wide frame: one emoji plus prose the terminal wrapped.
        let f = frame(80, &[&format!("│ 🚀 {}", "word ".repeat(20)), HINTS]); // 105 cells on 80
        assert_eq!(overflow_verdict(&f, 0), Overflow::Wrapped);
        assert!(rt_width_drift(&f, "x").is_empty());
        assert_eq!(overflow_verdict(&f, 1), Overflow::Fits);
    }
```

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test -p impeccable-terminal rules:: 2>&1 | tail -n 12
```

Expected: compile errors (`cannot find type 'Overflow'`, `cannot find function 'overflow_verdict'`).

- [ ] **Step 3: Implement.** Insert before `pub fn rt_width_drift` (line 277):

```rust
/// What a row wider than its frame is (spec section 7, PR 4 item 1). Each
/// row gets exactly one verdict, shared by width drift and narrow collapse;
/// `line` and `column` index the joined frame `capture-pane -J` returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overflow {
    /// No wider than the frame.
    Fits,
    /// The overflow is no larger than the row's count of wide glyphs: the
    /// app measured some of them as one cell. Any frame width.
    WidthDrift,
    /// Otherwise, in a frame under 60 columns, a row holding a Box Drawing
    /// glyph: the layout itself overflowed.
    Collapse,
    /// Anything else, such as prose the terminal wrapped and `-J` joined.
    Wrapped,
}

fn is_box_drawing(ch: char) -> bool {
    ('\u{2500}'..='\u{257F}').contains(&ch)
}

pub fn overflow_verdict(frame: &Frame, r: usize) -> Overflow {
    let w = frame.row_width(r);
    if frame.width == 0 || w <= frame.width {
        return Overflow::Fits;
    }
    if w - frame.width <= frame.wide_count(r) {
        return Overflow::WidthDrift;
    }
    let boxed = frame.rows[r].iter().any(|c| c.glyph.map(is_box_drawing).unwrap_or(false));
    if frame.width < NARROW_COLUMNS && boxed {
        Overflow::Collapse
    } else {
        Overflow::Wrapped
    }
}
```

Replace the loop body of `rt_width_drift` (lines 281 to 297; the `ends` and `usual_end` lines above it stay) with:

```rust
    for r in 0..frame.rows.len() {
        match overflow_verdict(frame, r) {
            Overflow::WidthDrift => {
                let snippet = format!("row measures {} cells on a {}-column pane", frame.row_width(r), frame.width);
                out.push(rt_finding("tui-rt-width-drift", file, frame, r, frame.width, snippet));
                continue;
            }
            Overflow::Fits => {}
            Overflow::Collapse | Overflow::Wrapped => continue,
        }
        if !frame.has_wide(r) {
            continue;
        }
        if let (Some(end), Some(usual)) = (last_vertical(frame, r), usual_end) {
            if end != usual {
                let snippet = format!("border ends at column {} while other rows end at {}", end + 1, usual + 1);
                out.push(rt_finding("tui-rt-width-drift", file, frame, r, end, snippet));
            }
        }
    }
```

In `rt_collapse_narrow`, replace the overflow block (lines 308 to 313):

```rust
        let w = row.len();
        if frame.width > 0 && w > frame.width {
            let snippet = format!("row overflows the {}-column pane by {} cells", frame.width, w - frame.width);
            out.push(rt_finding("tui-rt-collapse-narrow", file, frame, r, frame.width, snippet));
            continue;
        }
```

with:

```rust
        let w = row.len();
        match overflow_verdict(frame, r) {
            Overflow::Collapse => {
                let snippet = format!("row overflows the {}-column pane by {} cells", frame.width, w - frame.width);
                out.push(rt_finding("tui-rt-collapse-narrow", file, frame, r, frame.width, snippet));
                continue;
            }
            Overflow::Fits => {}
            Overflow::WidthDrift | Overflow::Wrapped => continue,
        }
```

The corner check and the split-word check below it are unchanged.

- [ ] **Step 4: Run the tests**

```bash
cargo test -p impeccable-terminal 2>&1 | tail -n 3
```

Expected: all pass. The PR 3 tests still hold: `emoji_row_wider_than_the_pane_is_width_drift_and_a_shifted_border_too` (13 cells on 12, one rocket: drift), `ascii_rows_never_trip_width_drift` (a 16-cell ASCII row on 12 is `Collapse`, which width drift skips), `narrow_frame_overflow_lost_corner_and_split_word_are_collapse` (a 50-cell border on 40: collapse).

- [ ] **Step 5: Replay the PR 3 capture goldens.** `collapse-narrow.txt`'s overflow row is a box border, and `width-drift.txt` is a 60-column frame whose overflow is one rocket, so neither golden moves:

```bash
cargo build --release -p impeccable 2>&1 | tail -n 1
IMPECCABLE_ORACLE_PREFIX=detect-tmux IMPECCABLE_BIN="$PWD/target/release/impeccable" node --test tests/oracle.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)"
```

Expected: `pass 1`, `fail 0`. A failure means a PR 3 finding moved: stop and report it, do not re-record.

- [ ] **Step 6: Commit**

```bash
git add crates/terminal/src/rules.rs
git commit -m "$(printf 'Give every overflowing row one verdict: width drift, collapse, or nothing\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 5: Key-hint false positives (item 7)

**Files:**
- Modify: `crates/terminal/src/rules.rs` (`PAGER_PROMPT_RE` after `KEY_HINT_RE` at line 38; `is_pager_prompt`; `rt_no_key_hints` lines 150 to 162; one test)

**Interfaces:**
- Consumes: `KEY_HINT_RE`, `MIN_ROWS_FOR_HINTS`, `Frame::text`, `Frame::is_blank`.
- Produces: `rt_no_key_hints` silent on a first row that names a key and on a pager prompt at the bottom.

- [ ] **Step 1: Write the failing test** at the end of `mod tests`:

```rust
    #[test]
    fn a_key_on_the_first_row_or_a_pager_prompt_at_the_bottom_silences_key_hints() {
        let help_on_top = frame(80, &["q quit  / search  ? help", "NAME        SIZE", "a.txt       12K", "b.txt       40K"]);
        assert!(rt_no_key_hints(&help_on_top, "x").is_empty(), "a screen whose first row names the keys");
        for prompt in [":", ":                    ", "(END)", "--More--(42%)", "lines 1-24/200 12%", "README.md lines 25-48"] {
            let pager = frame(80, &["NAME", "     less - opposite of more", "DESCRIPTION", prompt]);
            assert!(rt_no_key_hints(&pager, "x").is_empty(), "pager prompt {prompt:?}");
        }
        // Rows captured from macOS top on a scratch server while planning.
        let top = frame(100, &[
            "Processes: 714 total, 2 running, 712 sleeping, 4442 threads                                14:22:30",
            "Load Avg: 4.94, 5.73, 7.70  CPU usage: 13.12% user, 22.89% sys, 63.98% idle",
            "87568  installd     0.0  00:00.21 2     1    59    1216K 0B    0B    87568 1     sleeping",
        ]);
        assert_eq!(rt_no_key_hints(&top, "x").len(), 1, "top names no key on its first row, so it still fires");
        let colon_inside = frame(80, &["NAME", "     less - opposite of more", "DESCRIPTION", "Status: idle"]);
        assert_eq!(rt_no_key_hints(&colon_inside, "x").len(), 1, "a colon inside text is not a prompt");
    }
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test -p impeccable-terminal a_key_on_the_first_row 2>&1 | tail -n 8
```

Expected: FAIL on `help_on_top` (PR 3 reads only the bottom row).

- [ ] **Step 3: Implement.** After the `KEY_HINT_RE` definition (line 38) add:

```rust
re!(PAGER_PROMPT_RE, r"\(END\)|--More--|lines \d+-\d+");
```

Before `pub fn rt_no_key_hints` add:

```rust
/// A pager's own prompt on the bottom row: exactly `:` once the trailing
/// spaces `capture-pane -J` keeps are trimmed, or a row containing `(END)`,
/// `--More--`, or `lines <n>-<m>`.
fn is_pager_prompt(text: &str) -> bool {
    text.trim_end() == ":" || PAGER_PROMPT_RE.is_match(text)
}
```

In `rt_no_key_hints`, replace the three lines from `let r = *non_blank.last().expect("checked above");` through the `KEY_HINT_RE.is_match(&text)` early return with:

```rust
    // A key named on the first row (a help line, a table header) counts too.
    if KEY_HINT_RE.is_match(&frame.text(non_blank[0])) {
        return Vec::new();
    }
    let r = *non_blank.last().expect("checked above");
    let text = frame.text(r);
    if KEY_HINT_RE.is_match(&text) || is_pager_prompt(&text) {
        return Vec::new();
    }
```

- [ ] **Step 4: Run the tests**

```bash
cargo test -p impeccable-terminal 2>&1 | tail -n 3
```

Expected: all pass, including `bottom_row_without_a_key_is_flagged_and_hints_or_prompts_pass` (its bare frame's first row is a box top, which names no key).

- [ ] **Step 5: Commit**

```bash
git add crates/terminal/src/rules.rs
git commit -m "$(printf 'Silence no-key-hints on a key named up top or a pager prompt below\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 6: `paletteRelative` on low-contrast findings (item 8)

**Files:**
- Modify: `crates/terminal/src/rules.rs` (import `Style`; `palette_relative`; one line in `rt_low_contrast` after the `cellCount` insert at line 120; one test)

**Interfaces:**
- Consumes: `capture::{Color, Style}`.
- Produces: the boolean extra `paletteRelative` on every `tui-rt-low-contrast` finding, inserted after `cellCount` (extras keep insertion order in the JSON output).

- [ ] **Step 1: Write the failing test** at the end of `mod tests`:

```rust
    #[test]
    fn a_ratio_over_theme_colors_is_marked_palette_relative() {
        let f = frame(40, &[
            "\x1b[38;2;120;120;120;48;2;100;100;100mfaint\x1b[0m",
            "\x1b[30mblack on default\x1b[0m",
            "\x1b[38;5;236;48;5;235mdim cube\x1b[0m",
            HINTS,
        ]);
        let out = rt_low_contrast(&f, Palette::Dark, "x");
        assert_eq!(out.len(), 3, "{:?}", out.iter().map(|x| &x.snippet).collect::<Vec<_>>());
        assert_eq!(out[0].extras["paletteRelative"], Value::from(false), "truecolor on truecolor is the same on every theme");
        assert_eq!(out[1].extras["paletteRelative"], Value::from(true), "named black on the default background comes from the theme");
        assert_eq!(out[2].extras["paletteRelative"], Value::from(false), "the 256-color cube and grays are fixed");
        let keys: Vec<&String> = out[0].extras.keys().collect();
        assert_eq!(keys, vec!["column", "frame", "cellCount", "paletteRelative"], "the new extra comes last");
    }
```

`Finding::extras` is a `serde_json::Map<String, Value>` (`crates/foundation/src/findings.rs:29`), and this crate turns on serde_json's `preserve_order`, so the keys come back in insertion order.

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test -p impeccable-terminal a_ratio_over_theme_colors 2>&1 | tail -n 8
```

Expected: FAIL, the `paletteRelative` key is missing (index panic on `extras["paletteRelative"]`).

- [ ] **Step 3: Implement.** Change the import at line 9 to `use crate::capture::{Color, Frame, FrameRole, Style};`. Before `pub fn rt_low_contrast` add:

```rust
/// The terminal's default colors and the 16 named ones come from the user's
/// theme, so a ratio measured over them holds only for the `--palette` the
/// scan assumed (spec section 7, PR 4 item 8).
fn palette_relative(style: &Style) -> bool {
    let themed = |c: Color| matches!(c, Color::Default) || matches!(c, Color::Indexed(n) if n < 16);
    themed(style.fg) || themed(style.bg)
}
```

In `rt_low_contrast`, right after `f.extras.insert("cellCount".into(), Value::from(count));`, add:

```rust
            f.extras.insert("paletteRelative".into(), Value::from(palette_relative(&cell.style)));
```

`cell` is the first failing cell of the pair, the one `line` and `column` point at.

- [ ] **Step 4: Run the tests**

```bash
cargo test -p impeccable-terminal 2>&1 | tail -n 3
```

- [ ] **Step 5: Commit**

```bash
git add crates/terminal/src/rules.rs
git commit -m "$(printf 'Mark low-contrast ratios over theme colors as palette-relative\n\nAI assistance: drafted with Claude Code.')"
```

---
### Task 7: Two recaptures and the spinner rule (item 2)

**Files:**
- Modify: `crates/terminal/src/lib.rs` (`RECAPTURE_DELAY_MS` at line 24 becomes `RECAPTURE_DELAYS_MS`; the recapture in `detect_pane` lines 85 to 87; one test)
- Modify: `crates/terminal/src/rules.rs` (the spinner call in `scan_frames` lines 49 to 51; the snippet at line 169; the existing test `spinner_cycling_across_the_recapture_is_flagged_once`; one new test)
- Modify: `crates/terminal/src/capture.rs` (the `FrameRole::Recapture` doc comment, line 45)
- Modify: `crates/foundation/src/registry.rs` (the `tui-rt-spinner-never-rests` description, line 907)
- Regenerate: `crates/live/assets/antipatterns.json`, `crates/live/assets/detect-antipatterns-browser.js`

**Interfaces:**
- Produces: `pub const RECAPTURE_DELAYS_MS: [u64; 2] = [700, 1000]`; live frames in the order capture, recapture (700 ms), recapture (1000 ms); `scan_frames` comparing frame 0 with each recapture and reporting at most one spinner finding with the snippet `spinner glyph <a> became <b> between captures with no input`.

- [ ] **Step 1: Write the failing tests.** In `rules.rs` `mod tests`, add:

```rust
    #[test]
    fn the_spinner_is_compared_with_each_recapture_and_reported_once() {
        let first = frame(40, &["working ⠋", "│ x │", HINTS]);
        let mut same = frame(40, &["working ⠋", "│ x │", HINTS]);
        same.role = FrameRole::Recapture;
        let mut moved = frame(40, &["working ⠙", "│ x │", HINTS]);
        moved.role = FrameRole::Recapture;
        let spinner = |frames: &[Frame]| {
            scan_frames(frames, Palette::Dark, "x").into_iter().filter(|f| f.antipattern == "tui-rt-spinner-never-rests").count()
        };
        assert_eq!(spinner(&[first.clone(), same.clone(), moved.clone()]), 1, "the differing recapture need not be the first");
        assert_eq!(spinner(&[first.clone(), moved.clone(), moved.clone()]), 1, "two differing recaptures still make one finding");
        assert_eq!(spinner(&[first.clone(), same.clone()]), 0, "a PR 3 file with one matching recapture");
        assert_eq!(spinner(&[first, moved]), 1, "a PR 3 file with one differing recapture still replays");
    }
```

In the existing `spinner_cycling_across_the_recapture_is_flagged_once`, change the snippet assertion to:

```rust
        assert_eq!(out[0].snippet, "spinner glyph ⠋ became ⠙ between captures with no input");
```

In `lib.rs` `mod tests`, add (it reuses the PR 3 helper `fake_tmux_for_restore_scenarios`, which logs each call's first argument):

```rust
    #[cfg(unix)]
    #[test]
    fn a_pane_scan_recaptures_twice_within_one_second() {
        let (dir, fake) = fake_tmux("impeccable-terminal-recaptures", &fake_tmux_for_restore_scenarios(false, false));
        let mut env = HashMap::new();
        env.insert("IMPECCABLE_TMUX".to_string(), fake.to_string_lossy().into_owned());
        let start = std::time::Instant::now();
        TerminalEngine::new(env).detect_pane("smoke:0.0", &ScanOptions::default()).unwrap();
        let elapsed = start.elapsed();
        let log = std::fs::read_to_string(dir.join("calls.log")).unwrap();
        assert_eq!(log.lines().filter(|&c| c == "capture-pane").count(), 3, "frame 0 and two recaptures: {log}");
        assert!(!log.lines().any(|c| c == "resize-window"), "no sizes, no resize");
        assert!(elapsed >= std::time::Duration::from_millis(1000), "the last recapture waits 1000 ms: {elapsed:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }
```

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test -p impeccable-terminal spinner 2>&1 | tail -n 8
cargo test -p impeccable-terminal a_pane_scan_recaptures 2>&1 | tail -n 5
```

Expected: the rules test fails on `[first, same, moved]` (PR 3 reads only the first recapture), the snippet assertion fails, and the lib test counts 2 captures.

- [ ] **Step 3: Implement.**

`lib.rs`: replace

```rust
/// The gap between the first capture and the recapture.
pub const RECAPTURE_DELAY_MS: u64 = 1000;
```

with

```rust
/// When the recaptures are taken, in milliseconds after frame 0 (spec
/// section 7, PR 4 item 2). A 100 ms animation aliases at 1000 ms and not
/// at 700 ms.
pub const RECAPTURE_DELAYS_MS: [u64; 2] = [700, 1000];
```

change `use std::time::Duration;` to `use std::time::{Duration, Instant};`, and in `detect_pane` replace

```rust
        sleep(Duration::from_millis(RECAPTURE_DELAY_MS));
        let second = tmux.capture(target).map_err(EngineError::new)?;
        frames.push(frame_from(&second, &info, &colorterm, FrameRole::Recapture));
```

with

```rust
        // Both delays count from the end of frame 0's capture.
        let taken = Instant::now();
        for delay in RECAPTURE_DELAYS_MS {
            sleep(Duration::from_millis(delay).saturating_sub(taken.elapsed()));
            let again = tmux.capture(target).map_err(EngineError::new)?;
            frames.push(frame_from(&again, &info, &colorterm, FrameRole::Recapture));
        }
```

`rules.rs` `scan_frames`: replace

```rust
    if let Some(second) = frames.iter().find(|f| f.role == FrameRole::Recapture) {
        out.extend(rt_spinner_never_rests(first, second, file));
    }
```

with

```rust
    // Frame 0 against each recapture, in order; one finding at most.
    for recapture in frames.iter().filter(|f| f.role == FrameRole::Recapture) {
        let found = rt_spinner_never_rests(first, recapture, file);
        if !found.is_empty() {
            out.extend(found);
            break;
        }
    }
```

Update `scan_frames`'s doc comment "the spinner against the recapture" to "the spinner against each recapture". In `rt_spinner_never_rests` change the snippet to:

```rust
                let snippet = format!("spinner glyph {a} became {b} between captures with no input");
```

`capture.rs` line 45: the `Recapture` doc becomes `/// The pane's own size again, 700 ms or 1000 ms after the first capture.`

`registry.rs` line 907: the `tui-rt-spinner-never-rests` description becomes exactly:

```rust
        description: "A spinner glyph keeps changing across captures taken within one second with nothing to wait for. Show a spinner only while work is pending and replace it with the result.",
```

- [ ] **Step 4: Run the tests**

```bash
cargo test -p impeccable-terminal 2>&1 | tail -n 3
cargo test -p impeccable-foundation 2>&1 | tail -n 3     # runtime_rows_are_advisory_terminal_rows_at_the_end_of_the_registry checks no em dash
cargo test -p impeccable-core 2>&1 | tail -n 3           # frozen call vectors still replay
grep -rn "RECAPTURE_DELAY_MS\b" crates; echo "expect nothing above"
```

- [ ] **Step 5: Regenerate the two tracked assets**

```bash
T=$HOME/.rustup/toolchains/stable-x86_64-apple-darwin
PATH="$T/bin:$PATH" DYLD_LIBRARY_PATH="$T/lib" cargo xtask bundle 2>&1 | tail -n 2
node -e "const a=JSON.parse(require('fs').readFileSync('crates/live/assets/antipatterns.json','utf8')); console.log(a.length, a.find(x=>x.id==='tui-rt-spinner-never-rests').description)"
git status --short crates/live/assets
```

Expected: `79` and the new description; both asset files modified. The spinner golden moves too; it is re-recorded with the other goldens in Task 11, so `bun run test` stays red on it until then.

- [ ] **Step 6: Commit**

```bash
git add crates/terminal/src/lib.rs crates/terminal/src/rules.rs crates/terminal/src/capture.rs crates/foundation/src/registry.rs crates/live/assets/antipatterns.json crates/live/assets/detect-antipatterns-browser.js
git commit -m "$(printf 'Recapture at 700 ms and 1000 ms so a 100 ms spinner cannot alias\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 8: A 5-second deadline on every tmux call (item 3)

**Files:**
- Modify: `crates/terminal/src/tmux.rs` (imports; `TIMEOUT`, `POLL`, `timed_out`, `read_all`; the `timeout` field on `Tmux` line 63; `Tmux::locate` line 71; `Tmux::run` lines 79 to 97; two tests)

**Interfaces:**
- Produces: `pub const TIMEOUT: Duration` (5 s); `pub fn timed_out(verb: &str, timeout: Duration) -> String`; `pub(crate) timeout: Duration` on `Tmux`, set to `TIMEOUT` by `locate` and lowered only by tests; `Tmux::run` returning `tmux <verb>: timed out after 5 seconds` on expiry. Task 10's timeout-and-restore test sets the field.
- Consumes: the pattern of `git_run` (`crates/context/src/signals.rs:58`), replicated here because `crates/terminal` must not depend on `crates/context`.

- [ ] **Step 1: Write the failing tests** at the end of `mod tests` in `tmux.rs`:

```rust
    #[test]
    fn an_expired_call_names_the_verb_and_the_five_second_deadline() {
        assert_eq!(TIMEOUT, Duration::from_secs(5));
        assert_eq!(timed_out("capture-pane", TIMEOUT), "tmux capture-pane: timed out after 5 seconds");
    }

    #[cfg(unix)]
    #[test]
    fn a_hung_tmux_call_is_killed_at_the_deadline() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("impeccable-tmux-hang-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let fake = dir.join("tmux");
        std::fs::write(&fake, "#!/bin/sh\ncase \"$1\" in\n  -V) echo tmux 3.7c ;;\n  *) exec sleep 30 ;;\nesac\n").unwrap();
        std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut env = HashMap::new();
        env.insert("IMPECCABLE_TMUX".to_string(), fake.to_string_lossy().into_owned());
        env.insert("PATH".to_string(), "/usr/bin:/bin".to_string());
        let mut tmux = Tmux::locate(&env).unwrap();
        assert_eq!(tmux.timeout, TIMEOUT, "locate sets the spec's deadline");
        tmux.timeout = Duration::from_secs(1);
        let start = Instant::now();
        assert_eq!(tmux.run(&["capture-pane", "-p"]).unwrap_err(), timed_out("capture-pane", tmux.timeout));
        assert!(start.elapsed() < Duration::from_secs(3), "killed at the deadline, not after sleep 30: {:?}", start.elapsed());
        let _ = std::fs::remove_dir_all(&dir);
    }
```

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test -p impeccable-terminal tmux:: 2>&1 | tail -n 8
```

Expected: compile errors (`TIMEOUT`, `timed_out`, field `timeout` unknown).

- [ ] **Step 3: Implement.** Replace the imports at lines 5 to 7 with:

```rust
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
```

After `MIN_VERSION` (line 11) add:

```rust
/// Every tmux call's deadline (spec section 7, PR 4 item 3).
pub const TIMEOUT: Duration = Duration::from_secs(5);
/// How often a running tmux call is polled for exit.
const POLL: Duration = Duration::from_millis(5);

/// What an expired call reports: `tmux <verb>: timed out after 5 seconds`.
pub fn timed_out(verb: &str, timeout: Duration) -> String {
    format!("tmux {verb}: timed out after {} seconds", timeout.as_secs())
}

/// Drain a child's pipe on its own thread so a full pipe never stalls it.
fn read_all<R: Read + Send + 'static>(pipe: Option<R>) -> JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut p) = pipe {
            let _ = p.read_to_end(&mut buf);
        }
        buf
    })
}
```

Change the struct to:

```rust
#[derive(Debug)]
pub struct Tmux {
    exe: PathBuf,
    env: HashMap<String, String>,
    /// Each call's deadline: [`TIMEOUT`], lowered only by tests.
    pub(crate) timeout: Duration,
}
```

In `locate`, `let tmux = Tmux { exe, env: env.clone() };` becomes `let tmux = Tmux { exe, env: env.clone(), timeout: TIMEOUT };`.

Replace `run` (doc comment included) with:

```rust
    /// Run tmux with `args` under the deadline: stdout on success,
    /// `tmux <verb>: <first stderr line>` on failure, and
    /// `tmux <verb>: timed out after 5 seconds` once the deadline passes (the
    /// child is killed). On unix the child gets its own process group, so a
    /// Ctrl-C at the terminal reaches the engine and not the call in flight.
    pub fn run(&self, args: &[&str]) -> Result<String, String> {
        let verb = args.first().copied().unwrap_or("");
        let mut cmd = Command::new(&self.exe);
        cmd.args(args)
            .env_clear()
            .envs(&self.env)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        let mut child = cmd.spawn().map_err(|e| format!("could not run {}: {e}", self.exe.display()))?;
        let stdout = read_all(child.stdout.take());
        let stderr = read_all(child.stderr.take());
        let start = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if start.elapsed() >= self.timeout => {
                    let _ = child.kill();
                    let _ = child.wait();
                    // The readers are not joined: a grandchild may still hold a pipe.
                    return Err(timed_out(verb, self.timeout));
                }
                Ok(None) => thread::sleep(POLL),
                Err(e) => return Err(format!("could not run {}: {e}", self.exe.display())),
            }
        };
        let out = stdout.join().unwrap_or_default();
        let err = stderr.join().unwrap_or_default();
        if status.success() {
            return Ok(String::from_utf8_lossy(&out).into_owned());
        }
        let err = String::from_utf8_lossy(&err);
        Err(format!("tmux {verb}: {}", err.lines().next().unwrap_or("").trim()))
    }
```

- [ ] **Step 4: Run the tests**

```bash
cargo test -p impeccable-terminal 2>&1 | tail -n 3
cargo build --release -p impeccable 2>&1 | tail -n 1
IMPECCABLE_BIN="$PWD/target/release/impeccable" node --test tests/tmux-engine.test.mjs 2>&1 | grep -E "^ℹ (pass|fail|skipped)"
```

Expected: all crate tests pass (the PR 3 fake-tmux tests exit quickly, well inside 5 s); the integration test still passes 2 of 2 (the process group changes nothing for a detached server).

- [ ] **Step 5: Commit**

```bash
git add crates/terminal/src/tmux.rs
git commit -m "$(printf 'Give every tmux call a 5-second deadline and its own process group\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 9: Restore the prior `window-size` (item 5)

**Files:**
- Modify: `crates/terminal/src/tmux.rs` (`Tmux::window_size`; `Tmux::restore` lines 136 to 141)
- Modify: `crates/terminal/src/lib.rs` (`detect_pane` reads `window-size` before the size pass; `fake_tmux_for_restore_scenarios` answers `show-options`; new helper `fake_tmux_logging_args`, `window_calls`; two tests)

**Interfaces:**
- Produces: `pub fn Tmux::window_size(&self, target: &str) -> Result<String, String>` (trimmed; empty when the window has no value of its own) and `pub fn Tmux::restore(&self, target: &str, width: usize, height: usize, window_size: &str) -> Result<(), String>`. Test helpers `fake_tmux_logging_args(window_size: &str) -> String` and `window_calls(dir: &Path) -> Vec<String>`, reused by Task 10.
- Consumes: `Tmux::run` (Task 8).

- [ ] **Step 1: Write the failing tests** in `lib.rs` `mod tests`, after `fake_tmux_for_restore_scenarios`:

```rust
    /// A fake tmux that logs every call's full argument list to `calls.log`
    /// and prints `window_size` for `show-options`, as tmux prints a window
    /// option (nothing at all when the window has no value of its own).
    #[cfg(unix)]
    fn fake_tmux_logging_args(window_size: &str) -> String {
        format!(
            r#"#!/bin/sh
DIR="$(dirname "$0")"
echo "$*" >> "$DIR/calls.log"
case "$1" in
  -V) echo "tmux 3.7c" ;;
  list-panes) exit 0 ;;
  display-message) printf '40\t5\t40\t5\t\n' ;;
  capture-pane) echo hi ;;
  show-options) printf '{window_size}' ;;
  resize-window|set-option) exit 0 ;;
  *) exit 1 ;;
esac
"#
        )
    }

    /// The calls that read or change the window, in order.
    #[cfg(unix)]
    fn window_calls(dir: &std::path::Path) -> Vec<String> {
        std::fs::read_to_string(dir.join("calls.log"))
            .unwrap()
            .lines()
            .filter(|l| ["show-options", "resize-window", "set-option"].iter().any(|v| l.starts_with(v)))
            .map(String::from)
            .collect()
    }

    #[cfg(unix)]
    #[test]
    fn restore_puts_back_a_window_size_the_user_had_set() {
        let (dir, fake) = fake_tmux("impeccable-terminal-restore-set", &fake_tmux_logging_args("largest\\n"));
        let mut env = HashMap::new();
        env.insert("IMPECCABLE_TMUX".to_string(), fake.to_string_lossy().into_owned());
        let options = ScanOptions { tmux_sizes: vec![(30, 5)], tmux_settle_ms: Some(0), ..ScanOptions::default() };
        TerminalEngine::new(env).detect_pane("smoke:0.0", &options).unwrap();
        assert_eq!(
            window_calls(&dir),
            vec![
                "show-options -w -v -t smoke:0.0 window-size",
                "resize-window -t smoke:0.0 -x 30 -y 5",
                "resize-window -t smoke:0.0 -x 40 -y 5",
                "set-option -w -t smoke:0.0 window-size largest",
            ]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn restore_unsets_window_size_when_the_window_had_none() {
        let (dir, fake) = fake_tmux("impeccable-terminal-restore-unset", &fake_tmux_logging_args(""));
        let mut env = HashMap::new();
        env.insert("IMPECCABLE_TMUX".to_string(), fake.to_string_lossy().into_owned());
        let options = ScanOptions { tmux_sizes: vec![(30, 5)], tmux_settle_ms: Some(0), ..ScanOptions::default() };
        TerminalEngine::new(env).detect_pane("smoke:0.0", &options).unwrap();
        assert_eq!(
            window_calls(&dir),
            vec![
                "show-options -w -v -t smoke:0.0 window-size",
                "resize-window -t smoke:0.0 -x 30 -y 5",
                "resize-window -t smoke:0.0 -x 40 -y 5",
                "set-option -w -t smoke:0.0 -u window-size",
            ]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
```

(`"largest\\n"` in the Rust source is the four characters `largest\n` in the script, which `printf` turns into a newline, the way tmux ends the value.)

Also make the PR 3 helper answer the new call. In `fake_tmux_for_restore_scenarios`'s `case`, add a line after `list-panes) exit 0 ;;`:

```
  show-options) exit 0 ;;
```

Without it the three PR 3 restore tests would fail on `*) exit 1`.

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test -p impeccable-terminal restore_ 2>&1 | tail -n 10
```

Expected: the two new tests fail (no `show-options` call; the restore always unsets).

- [ ] **Step 3: Implement.** In `tmux.rs`, replace `restore` (doc comment included) with:

```rust
    /// The window's own `window-size` option, trimmed: empty when the window
    /// has none and follows the global value. Read before the first resize,
    /// because `resize-window` replaces any value with `manual`.
    pub fn window_size(&self, target: &str) -> Result<String, String> {
        Ok(self.run(&["show-options", "-w", "-v", "-t", target, "window-size"])?.trim().to_string())
    }

    /// Back to the recorded size, then put back the `window-size` read before
    /// the first resize, or unset the `manual` value `resize-window` left
    /// when there was none, so the window follows its clients again.
    pub fn restore(&self, target: &str, width: usize, height: usize, window_size: &str) -> Result<(), String> {
        self.resize(target, width, height)?;
        if window_size.is_empty() {
            self.run(&["set-option", "-w", "-t", target, "-u", "window-size"]).map(|_| ())
        } else {
            self.run(&["set-option", "-w", "-t", target, "window-size", window_size]).map(|_| ())
        }
    }
```

In `lib.rs` `detect_pane`, inside `if !options.tmux_sizes.is_empty() {`, make the first two lines:

```rust
            let window_size = tmux.window_size(target).map_err(EngineError::new)?;
            let captured = capture_sizes(&tmux, target, options, &colorterm, &mut frames);
            let restored = tmux.restore(target, info.window_width, info.window_height, &window_size);
```

(the `match (captured, restored)` below is unchanged in this task). A `show-options` failure returns before any resize, so the window is untouched.

- [ ] **Step 4: Run the tests**

```bash
cargo test -p impeccable-terminal 2>&1 | tail -n 3
```

Expected: all pass, including `a_failed_capture_and_a_failed_restore_are_both_reported`, `a_failed_restore_is_reported_when_capture_succeeded`, and `a_failed_capture_still_restores_the_window`.

- [ ] **Step 5: Commit**

```bash
git add crates/terminal/src/tmux.rs crates/terminal/src/lib.rs
git commit -m "$(printf 'Restore the window-size the window had before the size pass\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 10: Restore after SIGINT or SIGTERM (item 4)

**Files:**
- Modify: `crates/terminal/Cargo.toml` (one dependency)
- Modify: `crates/terminal/src/lib.rs` (imports; `INTERRUPT_POLL_MS`, `INTERRUPTED`, `settle`, `size_pass`; `capture_sizes` gains the flag; `detect_pane`'s size block; helpers `located`, `pane_40x5`; three tests)

**Interfaces:**
- Consumes: `impeccable_common::proc::{on_interrupt, clear_interrupt}` (Task 2), `Tmux::window_size` and `Tmux::restore` (Task 9), `Tmux.timeout` and `timed_out` (Task 8), `fake_tmux_logging_args`, `window_calls`, `fake_tmux_for_restore_scenarios` (tests).
- Produces: `fn size_pass(tmux, target, options, info, window_size, colorterm, frames, interrupted: &AtomicBool) -> Result<(), String>`, the whole size pass plus its restore; the errors `interrupted; window restored` and `interrupted; window not restored: <message>`.

- [ ] **Step 1: Add the dependency.** In `crates/terminal/Cargo.toml` under `[dependencies]`, after `impeccable-detect = { workspace = true }`:

```toml
impeccable-common = { workspace = true }
```

- [ ] **Step 2: Write the failing tests** in `lib.rs` `mod tests`:

```rust
    /// A fake tmux located the way the engine locates it, for driving
    /// `size_pass` directly with a flag the test controls.
    #[cfg(unix)]
    fn located(dir_name: &str, script: &str) -> (std::path::PathBuf, Tmux) {
        let (dir, fake) = fake_tmux(dir_name, script);
        let mut env = HashMap::new();
        env.insert("IMPECCABLE_TMUX".to_string(), fake.to_string_lossy().into_owned());
        (dir, Tmux::locate(&env).unwrap())
    }

    fn pane_40x5() -> PaneInfo {
        PaneInfo { width: 40, height: 5, window_width: 40, window_height: 5, termfeatures: String::new() }
    }

    #[cfg(unix)]
    #[test]
    fn an_interrupt_mid_settle_stops_the_pass_and_restores_the_window() {
        let (dir, tmux) = located("impeccable-terminal-interrupt", &fake_tmux_logging_args(""));
        let flag = AtomicBool::new(false);
        let options = ScanOptions { tmux_sizes: vec![(30, 5), (20, 5)], tmux_settle_ms: Some(5_000), ..ScanOptions::default() };
        let mut frames = Vec::new();
        let start = Instant::now();
        let result = std::thread::scope(|s| {
            s.spawn(|| {
                sleep(Duration::from_millis(200));
                flag.store(true, Ordering::SeqCst);
            });
            size_pass(&tmux, "smoke:0.0", &options, &pane_40x5(), "", "", &mut frames, &flag)
        });
        assert_eq!(result.unwrap_err(), "interrupted; window restored");
        assert!(start.elapsed() < Duration::from_secs(2), "the 5 s settle stopped within a poll step: {:?}", start.elapsed());
        assert!(frames.is_empty(), "nothing was captured at the interrupted size");
        assert_eq!(
            window_calls(&dir),
            vec![
                "resize-window -t smoke:0.0 -x 30 -y 5",
                "resize-window -t smoke:0.0 -x 40 -y 5",
                "set-option -w -t smoke:0.0 -u window-size",
            ],
            "the second size never ran"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn an_interrupt_with_a_failed_restore_says_the_window_was_not_restored() {
        let (dir, tmux) = located("impeccable-terminal-interrupt-restore-fails", &fake_tmux_for_restore_scenarios(false, true));
        let flag = AtomicBool::new(true); // the signal landed before the first size
        let options = ScanOptions { tmux_sizes: vec![(30, 5)], tmux_settle_ms: Some(0), ..ScanOptions::default() };
        let err = size_pass(&tmux, "smoke:0.0", &options, &pane_40x5(), "", "", &mut Vec::new(), &flag).unwrap_err();
        assert_eq!(err, "interrupted; window not restored: tmux set-option: boom");
        let log = std::fs::read_to_string(dir.join("calls.log")).unwrap();
        assert_eq!(log.lines().filter(|&c| c == "resize-window").count(), 1, "no size was applied; only the restore resized: {log}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn a_capture_that_times_out_still_restores_the_window() {
        let script = r#"#!/bin/sh
DIR="$(dirname "$0")"
echo "$1" >> "$DIR/calls.log"
case "$1" in
  -V) echo "tmux 3.7c" ;;
  list-panes) exit 0 ;;
  display-message) printf '30\t5\t30\t5\t\n' ;;
  capture-pane) exec sleep 30 ;;
  resize-window|set-option) exit 0 ;;
  *) exit 1 ;;
esac
"#;
        let (dir, mut tmux) = located("impeccable-terminal-capture-timeout", script);
        tmux.timeout = Duration::from_secs(1);
        let options = ScanOptions { tmux_sizes: vec![(30, 5)], tmux_settle_ms: Some(0), ..ScanOptions::default() };
        let err = size_pass(&tmux, "smoke:0.0", &options, &pane_40x5(), "", "", &mut Vec::new(), &AtomicBool::new(false)).unwrap_err();
        assert_eq!(err, crate::tmux::timed_out("capture-pane", tmux.timeout));
        let log = std::fs::read_to_string(dir.join("calls.log")).unwrap();
        let calls: Vec<&str> = log.lines().collect();
        assert_eq!(calls[calls.len() - 2..], ["resize-window", "set-option"], "the restore ran after the hung capture: {calls:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }
```

- [ ] **Step 3: Run to verify they fail**

```bash
cargo test -p impeccable-terminal interrupt 2>&1 | tail -n 8
```

Expected: compile error (`cannot find function 'size_pass'`).

- [ ] **Step 4: Implement.** Imports at the top of `lib.rs`:

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};
```

After `DEFAULT_SETTLE_MS` add:

```rust
/// Settle waits sleep in steps this long and check for an interrupt between them.
pub const INTERRUPT_POLL_MS: u64 = 50;

/// Set by SIGINT or SIGTERM while a size pass runs; registered with
/// `impeccable_common::proc::on_interrupt` only around that pass.
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// Wait `total` in 50 ms steps; false as soon as `interrupted` is set.
fn settle(total: Duration, interrupted: &AtomicBool) -> bool {
    let start = Instant::now();
    loop {
        if interrupted.load(Ordering::SeqCst) {
            return false;
        }
        let elapsed = start.elapsed();
        if elapsed >= total {
            return true;
        }
        sleep((total - elapsed).min(Duration::from_millis(INTERRUPT_POLL_MS)));
    }
}
```

Replace `capture_sizes` with:

```rust
/// The `--tmux-sizes` pass: resize, settle, re-read the geometry, capture.
/// Stops at the next check once `interrupted` is set. The caller restores
/// the window whatever this returns.
fn capture_sizes(
    tmux: &Tmux,
    target: &str,
    options: &ScanOptions,
    colorterm: &str,
    frames: &mut Vec<Frame>,
    interrupted: &AtomicBool,
) -> Result<(), String> {
    let settle_for = Duration::from_millis(options.tmux_settle_ms.unwrap_or(DEFAULT_SETTLE_MS));
    for &(w, h) in &options.tmux_sizes {
        if interrupted.load(Ordering::SeqCst) {
            return Err("interrupted".to_string());
        }
        tmux.resize(target, w as usize, h as usize)?;
        if !settle(settle_for, interrupted) {
            return Err("interrupted".to_string());
        }
        let info = tmux.pane_info(target)?;
        let text = tmux.capture(target)?;
        frames.push(frame_from(&text, &info, colorterm, FrameRole::Capture));
    }
    Ok(())
}

/// The size pass and the restore that always follows it. When `interrupted`
/// is set by the end, the interrupt names the failure whatever the pass
/// returned (a terminal Ctrl-C can also fail the tmux call in flight). The
/// restore never checks the flag.
#[allow(clippy::too_many_arguments)]
fn size_pass(
    tmux: &Tmux,
    target: &str,
    options: &ScanOptions,
    info: &PaneInfo,
    window_size: &str,
    colorterm: &str,
    frames: &mut Vec<Frame>,
    interrupted: &AtomicBool,
) -> Result<(), String> {
    let captured = capture_sizes(tmux, target, options, colorterm, frames, interrupted);
    let restored = tmux.restore(target, info.window_width, info.window_height, window_size);
    if interrupted.load(Ordering::SeqCst) {
        return Err(match restored {
            Ok(()) => "interrupted; window restored".to_string(),
            Err(r) => format!("interrupted; window not restored: {r}"),
        });
    }
    match (captured, restored) {
        (Err(c), Err(r)) => Err(format!("{c}; window not restored: {r}")),
        (Ok(()), Err(r)) => Err(format!("window not restored: {r}")),
        (Err(c), Ok(())) => Err(c),
        (Ok(()), Ok(())) => Ok(()),
    }
}
```

In `detect_pane`, replace the whole `if !options.tmux_sizes.is_empty() { ... }` block with:

```rust
        if !options.tmux_sizes.is_empty() {
            let window_size = tmux.window_size(target).map_err(EngineError::new)?;
            // From here to the end of the restore, SIGINT or SIGTERM sets a
            // flag instead of killing the process (spec section 7, item 4).
            INTERRUPTED.store(false, Ordering::SeqCst);
            impeccable_common::proc::on_interrupt(&INTERRUPTED);
            let result = size_pass(&tmux, target, options, &info, &window_size, &colorterm, &mut frames, &INTERRUPTED);
            impeccable_common::proc::clear_interrupt();
            result.map_err(EngineError::new)?;
        }
```

(The `#[allow(clippy::too_many_arguments)]` is there because `size_pass` takes eight arguments; the repo runs no clippy gate, so it only keeps a local `cargo clippy` quiet.)

- [ ] **Step 5: Run the tests**

```bash
cargo test -p impeccable-terminal 2>&1 | tail -n 3
cargo test --workspace 2>&1 | grep -E "^test result|FAILED|panicked" | sort | uniq -c
```

Expected: every terminal test passes (the three PR 3 restore tests now run through `size_pass` and install real handlers around it, which is harmless in a test process that sends no signal); the workspace matches Task 1's baseline plus the new tests.

- [ ] **Step 6: Commit**

```bash
git add crates/terminal/Cargo.toml crates/terminal/src/lib.rs Cargo.lock
git commit -m "$(printf 'Restore the window when SIGINT or SIGTERM lands during a size pass\n\nAI assistance: drafted with Claude Code.')"
```

(`Cargo.lock` gains `impeccable-common` under `impeccable-terminal`; drop it from `git add` if it did not change.)

---
### Task 11: Capture fixtures, goldens, `DELTAS.md`

**Files:**
- Create: `tests/fixtures/terminal-captures/collapse-narrow-prose.txt`, `width-drift-narrow.txt`, `no-key-hints-pager.txt`
- Modify: `tests/fixtures/terminal-captures/spinner-never-rests.txt` (a second recapture)
- Record: `tests/oracle/golden/detect-tmux-capture-{collapse-narrow-prose,width-drift-narrow,no-key-hints-pager}.json` (new); `detect-tmux-capture-spinner-never-rests.json`, `detect-tmux-capture-low-contrast.json`, `detect-tmux-capture-light-palette.json` (re-recorded)
- Modify: `tests/oracle/DELTAS.md` (one section after `## Recorded 2026-09-24: tmux engine (spec PR 3)`)

**Interfaces:**
- Consumes: the release binary with Tasks 3 to 10. `tests/oracle/cases/detect.mjs` lines 81 to 86 turn each new `.txt` into a case automatically (`fs.readdirSync(...).filter((f) => f.endsWith('.txt')).sort()`, then one `{ id: 'detect-tmux-capture-<name>', verb: 'detect', args: ['--no-config', '--json', '--tmux-capture', '<REPO>/tests/fixtures/terminal-captures/<name>.txt'], isolateHome: false }` per file); no case object is added by hand.

- [ ] **Step 1: Write the fixtures.** They are hand-written inputs, so a one-off script creates them and is not kept. From the repo root:

```bash
node - <<'JS'
const fs = require('fs');
const dir = 'tests/fixtures/terminal-captures';
const bar = (w, l, r) => l + '─'.repeat(w - 2) + r;
const HINTS = ' q quit  ? help';
const write = (name, frames) => fs.writeFileSync(`${dir}/${name}.txt`, frames.map(([h, rows]) => [h, ...rows].join('\n') + '\n').join(''));

// 40 columns; the only overflow row is prose the terminal wrapped and -J joined.
write('collapse-narrow-prose', [['#!capture width=40 height=5', [
  bar(40, '┌', '┐'),
  '│ Notes' + ' '.repeat(32) + '│',
  bar(40, '└', '┘'),
  'The quick brown fox jumps over the lazy dog while the index rebuilds in the background.',
  HINTS,
]]]);

// 40 columns; the overflow is one rocket the app padded as one cell (41 cells).
write('width-drift-narrow', [['#!capture width=40 height=4', [
  bar(40, '┌', '┐'),
  '│ 🚀 launch' + ' '.repeat(29) + '│',
  bar(40, '└', '┘'),
  HINTS,
]]]);

// less at its bottom prompt: a bare ':' followed by the spaces -J keeps.
write('no-key-hints-pager', [['#!capture width=80 height=6', [
  'LESS(1)                    General Commands Manual',
  '',
  'NAME',
  '       less - opposite of more',
  '',
  ':' + ' '.repeat(79),
]]]);

// Spinner: frame 0 (⠋), the 700 ms recapture (⠙, differs), the 1000 ms
// recapture (⠋, a 100 ms animation aliased back). Only the new frame's
// neighbor differs, so the finding needs the rule to read more than the
// last recapture.
const spinnerPath = `${dir}/spinner-never-rests.txt`;
const parts = fs.readFileSync(spinnerPath, 'utf8').split(/^(?=#!capture)/m);
if (parts.length !== 2 || !parts[0].startsWith('#!capture width=40 height=5\n')) throw new Error('expected the PR 3 file: one capture, one recapture');
const aliased = parts[0].replace('#!capture width=40 height=5\n', '#!capture width=40 height=5 role=recapture\n');
fs.writeFileSync(spinnerPath, parts[0] + parts[1] + aliased);
console.log(fs.readdirSync(dir).join(' '));
JS
```

- [ ] **Step 2: Check each file with the binary before recording**

```bash
cargo build --release -p impeccable 2>&1 | tail -n 1
B="$PWD/target/release/impeccable"
for f in collapse-narrow-prose width-drift-narrow no-key-hints-pager spinner-never-rests low-contrast; do
  echo "== $f"
  "$B" detect --no-config --json --tmux-capture "tests/fixtures/terminal-captures/$f.txt" | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{for(const f of JSON.parse(s))console.log(f.antipattern,f.line,f.column,f.frame,f.paletteRelative,JSON.stringify(f.snippet))})'
done
grep -c "role=recapture" tests/fixtures/terminal-captures/spinner-never-rests.txt   # 2
```

Expected: `collapse-narrow-prose` nothing; `width-drift-narrow` exactly one line, `tui-rt-width-drift 2 41 40x4 undefined "row measures 41 cells on a 40-column pane"`; `no-key-hints-pager` nothing; `spinner-never-rests` one line at line 2 column 11, frame `40x5`, snippet `"spinner glyph ⠋ became ⠙ between captures with no input"`; `low-contrast` two lines, `faint label` with `paletteRelative` `false` and `black on default` with `true`. A stray finding means a fixture row trips a second rule: fix the fixture, not the rule.

- [ ] **Step 3: Record and review**

```bash
IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/record.mjs --bin detect-tmux 2>&1 | tail -n 3
git status --short tests/oracle/golden
```

Expected `git status`: three new files (`detect-tmux-capture-collapse-narrow-prose.json`, `-width-drift-narrow.json`, `-no-key-hints-pager.json`) and three modified (`-spinner-never-rests.json`, `-low-contrast.json`, `-light-palette.json`). Anything else moved is a regression from Tasks 3 to 10: stop and find it.

Read every new and changed golden with the `Read` tool (not `cat`), and `git diff tests/oracle/golden` for the three modified ones:
- `collapse-narrow-prose` and `no-key-hints-pager`: `stdout` `[]` plus newline, exit 0.
- `width-drift-narrow`: one `tui-rt-width-drift` finding, no `tui-rt-collapse-narrow`, exit 0.
- `spinner-never-rests`: only the `snippet` and `description` lines differ, and the description reads `A spinner glyph keeps changing across captures taken within one second with nothing to wait for. Show a spinner only while work is pending and replace it with the result.`
- `low-contrast` and `light-palette`: the only change is one added `"paletteRelative": <bool>` line after each `cellCount` (dark: `faint label` false, `black on default` true; light: `faint label` false, `clear label` true, because its background is the default).

- [ ] **Step 4: Replay**

```bash
IMPECCABLE_BIN="$PWD/target/release/impeccable" node --test tests/oracle.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)"
```

Expected: pass (or the known port-8080 `detect-fixture-text-framework-vite` failure only, if the llama-server is up; ledger it).

- [ ] **Step 5: `DELTAS.md`.** Append after the PR 3 section:

```markdown
## Recorded 2026-09-25: runtime engine hardening (spec PR 4)

- New: `detect-tmux-capture-collapse-narrow-prose` (a 40-column frame whose only overflow row is joined prose; `[]`), `detect-tmux-capture-width-drift-narrow` (a 40-column frame whose overflow is one emoji; one `tui-rt-width-drift`, no `tui-rt-collapse-narrow`), `detect-tmux-capture-no-key-hints-pager` (less's bare `:` prompt with the trailing spaces `-J` keeps; `[]`). All three come from the fixture loop in `cases/detect.mjs`.
- `detect-tmux-capture-spinner-never-rests`: the fixture gained a second recapture (frame 0 `⠋`, 700 ms `⠙`, 1000 ms `⠋`). The snippet is now `spinner glyph ⠋ became ⠙ between captures with no input` and the registry description says "across captures taken within one second".
- `detect-tmux-capture-low-contrast` and `detect-tmux-capture-light-palette`: each finding gains `paletteRelative` after `cellCount`, true when the foreground or background is the terminal default or one of the 16 named colors. The spec's PR 4 Tests paragraph does not list these two; the change is the new extra only.
- No other golden moved: the width rules, the overflow verdict, and the key-hint exemptions leave the PR 3 fixtures' findings as they were.
```

Quote the exact snippets and booleans from the recorded goldens, not from this plan.

- [ ] **Step 6: Commit**

```bash
git add tests/fixtures/terminal-captures tests/oracle/golden tests/oracle/DELTAS.md
git commit -m "$(printf 'Pin the PR 4 runtime changes with capture fixtures and goldens\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 12: Integration test: the prior `window-size` and a real SIGINT

**Files:**
- Modify: `tests/tmux-engine.test.mjs` (the `node:child_process` import; two `it` blocks between the existing two)

**Interfaces:**
- Consumes: the release binary; the scratch server and `IMPECCABLE_TMUX` wrapper the file's `before()` already sets up (socket `impeccable-test-<pid>`, session `app`, 80x24, running `tests/fixtures/terminal-apps/nested-boxes.sh`).
- Produces: the live proof that a user-set `window-size` survives a scan and that SIGINT mid-settle restores the window and exits 1.

- [ ] **Step 1: Write the tests.** Change the import line to:

```js
import { execFileSync, spawn, spawnSync } from 'node:child_process';
```

After the existing `it('captures the pane, walks the size matrix, and restores the window', ...)` block (which keeps asserting `'80x24 latest'` for a window with no `window-size` of its own), insert:

```js
  it('puts back a window-size the user had set before the scan', () => {
    tmux('set-option', '-w', '-t', 'app:0.0', 'window-size', 'largest');
    try {
      const r = detect('--json', '--tmux', 'app:0.0', '--tmux-sizes', '40x24');
      assert.equal(r.status, 0, r.stderr);
      assert.equal(tmux('display', '-p', '-t', 'app:0.0', '#{window_width}x#{window_height} #{window-size}'), '80x24 largest', 'size and the user value restored');
    } finally {
      tmux('set-option', '-w', '-t', 'app:0.0', '-u', 'window-size');
    }
  });

  it('restores the window and fails loudly when SIGINT lands during the size pass', async () => {
    const child = spawn(BIN, ['detect', '--no-config', '--tmux', 'app:0.0', '--tmux-sizes', '40x24', '--tmux-settle', '5000'], {
      env: engineEnv(BIN, { IMPECCABLE_TMUX: wrapper }),
    });
    let stderr = '';
    child.stderr.setEncoding('utf8').on('data', (d) => { stderr += d; });
    const exited = new Promise((resolve) => child.on('exit', (code, signal) => resolve({ code, signal })));
    // Frame 0, the recaptures (1000 ms), show-options, and the resize take
    // about 1.1 s, so 2.5 s lands inside the 5 s settle. Read the width
    // before signalling, so a failed assertion never leaves the child running.
    await new Promise((resolve) => setTimeout(resolve, 2500));
    const midSettleWidth = tmux('display', '-p', '-t', 'app:0.0', '#{window_width}');
    child.kill('SIGINT');
    const { code, signal } = await exited;
    assert.equal(midSettleWidth, '40', 'the scan was mid-settle at 40 columns when SIGINT arrived');
    assert.equal(signal, null, 'the engine caught SIGINT instead of dying');
    assert.equal(code, 1, stderr);
    assert.match(stderr, /^Error: interrupted; window restored$/m);
    assert.equal(tmux('display', '-p', '-t', 'app:0.0', '#{window_width}x#{window_height} #{window-size}'), '80x24 latest', 'restored after the interrupt');
  });
```

The unknown-pane test stays last. Tests in one `describe` run in order, and the `finally` puts the window back to no `window-size` of its own, so the SIGINT test starts from the state the first test left.

- [ ] **Step 2: Run it**

```bash
cargo build --release -p impeccable 2>&1 | tail -n 1
IMPECCABLE_BIN="$PWD/target/release/impeccable" node --test tests/tmux-engine.test.mjs 2>&1 | grep -E "^ℹ (pass|fail|skipped)|not ok"
tmux -L "impeccable-test-$$" ls 2>&1 | head -n 1   # "no server running": after() killed it
```

Expected: `pass 4`, `fail 0`. If the SIGINT test sees `signal === 'SIGINT'`, the handler was not registered before the resize (check the `detect_pane` order from Task 10). If the width reads `80` at 2.5 s, the resize had not happened yet: raise the wait to 3000 ms and `--tmux-settle` to 6000 together, and note it in the ledger.

- [ ] **Step 3: Commit**

```bash
git add tests/tmux-engine.test.mjs
git commit -m "$(printf 'Prove a user window-size survives a scan and SIGINT restores the window\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 13: `docs/CLI-CONTRACT.md` and the full gates (item 9)

**Files:**
- Modify: `docs/CLI-CONTRACT.md` (the **tmux scans** paragraph, line 116; the `crates/terminal` section, lines 356, 358, 359)

**Interfaces:** none new. Every sentence must match code from Tasks 2 to 12; quote messages and delays from the code.

- [ ] **Step 1: The tmux-scans paragraph.** In line 116, replace the tail

```
`frame` = `WxH`, and for `tui-rt-low-contrast` `cellCount`; every runtime rule is advisory. Palette `dark` (default) or `light`; sizes restore the window (`resize-window` back, then `set-option -w -u window-size`).
```

with

```
`frame` = `WxH`, and for `tui-rt-low-contrast` `cellCount` and `paletteRelative` (true when the foreground or background is the terminal default or one of the 16 named colors, which the user's theme sets, so the ratio holds only for the assumed palette); every runtime rule is advisory. Palette `dark` (default) or `light`. Sizes restore the window: `resize-window` back, then the window's own `window-size` as read before the first resize (`set-option -w -t <target> window-size <value>`, or `-u` when it had none). Every tmux call has a 5-second deadline (`tmux <verb>: timed out after 5 seconds`). SIGINT or SIGTERM during the size pass stops it, restores the window, and fails with `interrupted; window restored` or `interrupted; window not restored: <message>`. `capture-pane -J` keeps trailing spaces and joins rows the terminal wrapped, so `line` and `column` index the joined frame. `tui-rt-truecolor-on-256` needs `client_termfeatures`, which tmux leaves empty unless a client is attached to the scanned session; a saved capture names the features in its header (`termfeatures=`).
```

- [ ] **Step 2: The `crates/terminal` section.**

Replace the whole bullet that begins `- \`TmuxEngine::detect_pane(target, options)\`` (line 356) with:

```
- `TmuxEngine::detect_pane(target, options)`: `Tmux::locate` (`IMPECCABLE_TMUX`, else `PATH`; `tmux -V` must parse to 3.2 or newer) → `list-panes -t <target> -F '#{pane_id}'` (a missing session or window fails here with tmux's own line; tmux 3.7's `display-message -t` does not fail on a bad target) → `display-message -p -t <target> '#{pane_width}\t#{pane_height}\t#{window_width}\t#{window_height}\t#{client_termfeatures}'` → `capture-pane -p -e -J -t <target>` (frame 0, role `capture`) → capture again 700 ms and 1000 ms after frame 0 (role `recapture`) → with `--tmux-sizes`: `show-options -w -v -t <target> window-size` (empty when the window has no value of its own), then SIGINT and SIGTERM set a flag (`impeccable_common::proc::on_interrupt`), then for each `WxH`: `resize-window -t <target> -x W -y H`, sleep `--tmux-settle` (default 300) in 50 ms steps that stop on the flag, `display-message` again, capture → `resize-window` back to the recorded window size, then `set-option -w -t <target> window-size <value>`, or `set-option -w -t <target> -u window-size` when the read was empty (always, even after a failed or timed-out capture or an interrupt) → `impeccable_common::proc::clear_interrupt` → `rules::scan_frames`. A failed restore is reported as `window not restored: <message>`, joined to the capture error with `; ` when both fail; after an interrupt the error is `interrupted; window restored` or `interrupted; window not restored: <message>`. Every tmux call runs in its own process group with a 5-second deadline, after which it is killed and reported as `tmux <verb>: timed out after 5 seconds`.
```

In the frame-policy bullet (line 358), replace `\`tui-rt-spinner-never-rests\` compares frame 0 with the first \`recapture\`` with `` `tui-rt-spinner-never-rests` compares frame 0 with each `recapture` in order and reports once``, and append to the bullet: `` A row wider than its frame gets one verdict: width drift when the overflow is no larger than its count of two-cell glyphs (any frame width), else collapse when the frame is under 60 columns and the row holds a Box Drawing glyph (U+2500 to U+257F), else nothing (a joined prose row is the terminal's wrap); the border-column, corner, and split-word checks read only rows that fit. `tui-rt-no-key-hints` is silent when the first non-blank row names a key or the last is a pager prompt (`:` once trailing spaces are trimmed, or a row containing `(END)`, `--More--`, or `lines <n>-<m>`).``

In the SGR bullet (line 359), replace `widths from \`unicode-width\` (two-cell glyphs own a wide-tail cell, zero-width marks own none)` with `` widths from `unicode-width` (two-cell glyphs own a wide-tail cell, zero-width marks own none) plus tmux 3.7c's three sequence rules: U+FE0F after a one-cell glyph makes it two cells, an emoji modifier (U+1F3FB to U+1F3FF) takes no cell, and a character right after U+200D takes no cell``.

Check: `grep -n "—" docs/CLI-CONTRACT.md | grep -v "^[0-9]*:####"` prints nothing new (no em dash outside the existing heading form).

- [ ] **Step 3: Full gates**

```bash
cargo test --workspace 2>&1 | grep -E "^test result|FAILED|panicked" | sort | uniq -c
T=$HOME/.rustup/toolchains/stable-x86_64-apple-darwin; PATH="$T/bin:$PATH" DYLD_LIBRARY_PATH="$T/lib" cargo xtask bundle --check && echo BUNDLE_FRESH
cargo build --release -p impeccable 2>&1 | tail -n 1
bun run build 2>&1 | tail -n 3
IMPECCABLE_BIN="$PWD/target/release/impeccable" bun run test 2>&1 | tail -n 15
IMPECCABLE_BIN="$PWD/target/release/impeccable" node --test tests/tmux-engine.test.mjs 2>&1 | grep -E "^ℹ (pass|fail|skipped)"
```

Expected: green except the known pre-existing failures from Task 1's baseline; `BUNDLE_FRESH`; the rule count stays 79, so `generateCounts` does not move and no README changes. Compare the pass counts with the baseline plus the new crate tests, the three new oracle cases (counted inside the one oracle test), and the two new integration tests.

- [ ] **Step 4: Commit**

```bash
git add docs/CLI-CONTRACT.md
git commit -m "$(printf 'Document the PR 4 tmux engine contract: deadline, interrupts, restore, widths\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 14: Manual pass over `less` and a 40-column `demo2`

**Files:**
- Modify: `docs/superpowers/research/2026-09-24-terminal-runtime-rules-manual-pass.md` (append `## PR 4 re-run`)
- Possibly modify: `crates/terminal/src/*.rs` (only to fix a bug the pass exposes, with a unit test named for the outcome first and any affected `detect-tmux-*` golden re-recorded)

**Interfaces:** consumes the release binary from Task 13. Produces the spec's PR 4 gate record: the `tui-rt-collapse-narrow` count against PR 3's 15, over the same programs where available.

- [ ] **Step 1: Stage the programs in a scratch server.** Nothing touches the default socket.

```bash
S=/private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/rt-manual-pass-pr4
mkdir -p "$S"; B="$PWD/target/release/impeccable"; SOCK=impeccable-rt-pass-pr4
tmux -L $SOCK kill-server 2>/dev/null
tmux -L $SOCK new-session -d -x 100 -y 30 -s pass -n top 'top'
tmux -L $SOCK new-window -t pass -n vim "vim $PWD/README.md"
tmux -L $SOCK new-window -t pass -n less "less $PWD/CLAUDE.md"
tmux -L $SOCK new-window -t pass -n man 'man tmux'
R=/private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/manual-pass/ratatui
[ -d "$R" ] || git clone --depth 1 https://github.com/ratatui/ratatui.git "$R"
( cd "$R" && timeout 600 cargo build --release --example demo2 2>&1 | tail -n 1 )
[ -x "$R/target/release/examples/demo2" ] && tmux -L $SOCK new-window -t pass -n demo2 "$R/target/release/examples/demo2"
sleep 3
printf '#!/bin/sh\nexec tmux -L %s "$@"\n' "$SOCK" > "$S/tmux-wrap"; chmod +x "$S/tmux-wrap"
tmux -L $SOCK list-windows -t pass -F '#{window_name}' > "$S/windows.txt"; cat "$S/windows.txt"
```

The clone has no built examples today, so `demo2` builds here. If it does not build within ten minutes, record that and run the pass without it; `less` alone then carries the gate, and the record says so.

- [ ] **Step 2: Scan each window at the PR 3 sizes**

```bash
for w in $(cat "$S/windows.txt"); do
  /usr/bin/time -p env IMPECCABLE_TMUX="$S/tmux-wrap" "$B" detect --no-config --json --tmux "pass:$w" --tmux-sizes 80x24,120x40,40x24 > "$S/$w.json" 2> "$S/$w.err"; echo "$w exit=$?"
done
node -e '
const fs=require("fs"); const S=process.argv[1];
for (const f of fs.readdirSync(S).filter(f=>f.endsWith(".json"))) {
  let a=[]; try { a=JSON.parse(fs.readFileSync(`${S}/${f}`,"utf8")); } catch { console.log(f, "unparsable"); continue; }
  const by={}; for (const x of a) by[x.antipattern]=(by[x.antipattern]||0)+1;
  const narrow=a.filter(x=>x.antipattern==="tui-rt-collapse-narrow").length;
  console.log(f, "total", a.length, "collapse-narrow", narrow, JSON.stringify(by));
}' "$S"
tmux -L $SOCK set-option -w -t pass:less window-size largest
IMPECCABLE_TMUX="$S/tmux-wrap" "$B" detect --no-config --json --tmux pass:less --tmux-sizes 40x24 > /dev/null
tmux -L $SOCK display -p -t pass:less '#{window_width}x#{window_height} #{window-size}'   # 100x30 largest
tmux -L $SOCK display -p -t pass:top '#{window_width}x#{window_height} #{window-size}'    # 100x30 latest
```

Read the per-window counts from a small `printf` of the numbers if the output carries a compression marker (Rule 16).

- [ ] **Step 3: Judge the remaining collapse-narrow findings.** For each, print window, frame, line, column, snippet, and look at the joined row (`tmux -L $SOCK capture-pane -p -J -t pass:<w>` after `resize-window -x 40 -y 24`, then restore with the engine or by hand with `-u window-size`). Classify as true positive, false positive, or arguable, one clause each. Also note whether `less`'s bottom `:` prompt still draws a `tui-rt-no-key-hints` finding (it should not) and whether `top` still does (it should).

- [ ] **Step 4: Write the record.** Append to `docs/superpowers/research/2026-09-24-terminal-runtime-rules-manual-pass.md`:

```markdown
## PR 4 re-run

<date, tmux -V, the programs that ran, and demo2's build outcome>

| Window | Total | collapse-narrow (PR 3) | collapse-narrow (PR 4) | Other rules |
|---|---|---|---|---|
| ... one row per window, PR 3 numbers from "Hits per program" above ... |

<the collapse-narrow total against PR 3's 15, each remaining finding classified, the key-hint change on less and top, the largest restore check, scan times>
```

Fill every cell from Step 2 and 3 output; no em dashes. All rules stay `advisory`.

- [ ] **Step 5: Tear down and commit**

```bash
tmux -L $SOCK kill-server
git add docs/superpowers/research/2026-09-24-terminal-runtime-rules-manual-pass.md
git commit -m "$(printf 'Record the PR 4 manual tmux pass over less and demo2\n\nAI assistance: drafted with Claude Code.')"
```

(Add `crates/terminal` and `tests/oracle/golden` paths only if Step 3 led to a fix.)

---

### Task 15: Final review, merge to the fork's main, push

**Files:** none new.

- [ ] **Step 1: Whole-branch review** per subagent-driven-development (final code reviewer on the most capable model), against spec section 7 PR 4 items 1 to 9 and its Tests paragraph. Findings marked Important or Critical get one fix wave and one scoped re-review. Ask the reviewer to probe tmux on a scratch socket rather than trust the fixtures (PR 3's final review found parser bugs the fixtures could not show).

- [ ] **Step 2: Gates once more on the final head**

```bash
cargo test --workspace 2>&1 | grep -E "^test result|FAILED|panicked" | sort | uniq -c
T=$HOME/.rustup/toolchains/stable-x86_64-apple-darwin; PATH="$T/bin:$PATH" DYLD_LIBRARY_PATH="$T/lib" cargo xtask bundle --check && echo BUNDLE_FRESH
cargo build --release -p impeccable 2>&1 | tail -n 1
bun run build 2>&1 | tail -n 2
IMPECCABLE_BIN="$PWD/target/release/impeccable" bun run test 2>&1 | tail -n 8
IMPECCABLE_BIN="$PWD/target/release/impeccable" node --test tests/tmux-engine.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)"
git status --short   # only the untracked local files may show; never stage them
```

- [ ] **Step 3: Merge and push** (the fork owner pre-authorized merging each PR to `main` as it lands):

```bash
git checkout main && git pull --ff-only origin main
git merge --no-ff terminal-platform-pr4 -m "$(printf 'Merge terminal-platform-pr4: runtime engine hardening (spec PR 4)\n\nAI assistance: drafted with Claude Code.')"
git push origin main
git log --oneline -3
```

- [ ] **Step 4: Close out.** Ledger completion line; copy the ledger to the session scratchpad; delete the SDD workspace; update the memory file `terminal-platform-project.md` (PR 4 merged, the merge commit id, the collapse-narrow count against 15, and that `ENGINE_VERSION` still needs a bump at release); write the journal entry. Update the untracked upstream issue draft `docs/superpowers/research/2026-09-24-upstream-issue-draft.md` with one line naming the PR 4 merge commit in its evidence list. Never stage it and never post it.

---

## Self-review notes

- Spec section 7 PR 4 coverage: item 1 (Task 4, `Overflow` and `overflow_verdict`, after item 6 in Task 3); item 2 (Task 7: delays, `scan_frames` loop, snippet, registry description, bundle; Task 11: the spinner fixture); item 3 (Task 8: `TIMEOUT`, `timed_out`, `Tmux::run`; Task 10: the timed-out capture still restores); item 4 (Task 2: `clear_interrupt`; Task 10: order, 50 ms settle, messages; Task 12: real SIGINT); item 5 (Task 9); item 6 (Task 3); item 7 (Task 5); item 8 (Task 6); item 9 (Task 13).
- Tests paragraph: the three new fixtures and goldens and the spinner re-record (Task 11); crate tests for the verdict (Task 4), the three width rules (Task 3), the pager prompts (Task 5), `paletteRelative` (Task 6), the restore sequence for empty and set `window-size` (Task 9); the `largest` integration assertion (Task 12).
- Delivery gate: `cargo test --workspace`, `cargo xtask bundle` and `--check`, goldens reviewed by hand, `bun run build`, `bun run test`, `node --test tests/tmux-engine.test.mjs` (Tasks 13, 15); the manual pass over `less` and a 40-column `demo2` against PR 3's 15 (Task 14).
- Type consistency: `Tmux::restore(&self, &str, usize, usize, &str)` in Tasks 9 and 10; `Tmux::window_size(&self, &str) -> Result<String, String>` in Tasks 9 and 10; `size_pass(.., &AtomicBool) -> Result<(), String>` in Task 10's code and tests; `Frame::wide_count(&self, usize) -> usize` in Tasks 3 and 4; `Overflow::{Fits, WidthDrift, Collapse, Wrapped}` in Task 4; `RECAPTURE_DELAYS_MS: [u64; 2]` in Task 7; `timed_out(&str, Duration) -> String` in Tasks 8 and 10.
- Not in this PR: promotion of any rule, `ENGINE_VERSION`, README counts (still 79), `crates/detect` (so an interrupt fails one `--tmux` target and later targets still run, decision 8), skill text (PR 7).
