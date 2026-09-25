# Terminal Platform PR 3: tmux Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship `impeccable detect --tmux <pane>` and `--tmux-capture <file>`: a new `crates/terminal` engine that captures a running tmux pane (or replays a saved capture), parses it into cell frames, and runs the seven `tui-rt-` runtime rules, all at `advisory`.

**Architecture:** Seven registry rows land at the end of `ANTIPATTERNS`. `crates/detect` grows the engine seam (`TmuxEngine`, `Engines.tmux`, three `ScanOptions` fields, five flags, `scan_tmux_targets`). The new crate `crates/terminal` holds four pure-ish modules: `capture` (SGR parser, frame model, saved-file header), `palette` (SGR colors to sRGB), `rules` (one function per runtime rule over frames), `tmux` (locate, version, geometry, capture, resize, restore). `crates/cli` wires `TerminalEngine::from_process_env()` like the browser engine. The oracle pins the flags and the rules through saved capture fixtures; one integration test drives a scratch tmux server.

**Tech Stack:** Rust workspace (`foundation`, `detect`, `terminal` new, `cli`), `unicode-width 0.2` (the one new dependency), `regex` + `once_cell`, node oracle harness, `cargo xtask bundle`, bun build gates, tmux 3.7 on this machine.

**Spec:** `docs/superpowers/specs/2026-09-24-terminal-platform-design.md`, section 4 (PR 3 table), section 5 (expanded 2026-09-24), section 6 (PR 3). Read sections 4 and 5 before any task; every message, flag, id, and threshold below is copied from there.

## Global Constraints

- Every runtime rule ships at `severity: Some("advisory")`, `platforms: Some(&["terminal"])`, `category` per the section 4 table. No promotion in this PR.
- The seven rows are appended after `tui-print-in-loop`; `ANTIPATTERNS.len()` becomes 79; web output stays byte-identical (the oracle enforces it; any web golden that moves is explained in `tests/oracle/DELTAS.md`).
- Runtime rules live in `crates/terminal/src/rules.rs`, never in `crates/core`; the wasm bundle carries no tmux code. `crates/detect` never depends on `crates/terminal` or `crates/context`; `crates/cli` wires both.
- The engine never launches, types into, or kills a process, and always runs the restore (`resize-window` back, then `set-option -w -u window-size`) after a `--tmux-sizes` pass, even when a capture failed.
- Frame order for a pane: own size, recapture after 1000 ms, then each size after a settle (default 300 ms). Size-invariant rules read frame 0; spinner compares frame 0 with the recapture; geometry rules run on every capture frame and never on the recapture; collapse-narrow only under 60 columns.
- Contrast: `impeccable_core::color::contrast_ratio`, floor 4.5, alphanumeric glyphs only, palettes `dark` (default, white on black) and `light` (black on white), xterm's 16-color table in both.
- Messages and flags are the section 5 tables, byte for byte.
- New oracle cases are recorded with `IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/record.mjs --bin <prefix>` and reviewed by hand. Never hand-edit a golden. Never touch `tests/oracle/vectors/calls/`.
- No manifest version bumps, no `ENGINE_VERSION` bump, no changelog entry.
- Every commit body ends with the line `AI assistance: drafted with Claude Code.` and carries no `Co-Authored-By` or other trailer (the fork owner's rule).
- No em dashes in `skill/**/*.md` (build gate), registry descriptions, or docs written here. The one exception is `docs/CLI-CONTRACT.md`'s existing `#### path — title` heading form, which the new section heading matches.
- Never stage `.serena/`, `.claude/CLAUDE.md`, `.gemini/GEMINI.md`, `.agents/AGENTS.md`, `.superpowers/`, regenerated provider output, `.env`, or `docs/superpowers/research/2026-09-24-upstream-issue-draft.md`.
- Test names state outcomes (spec section 6): `capture_parser_replays_without_tmux`, `missing_tmux_names_the_requirement_and_the_override_wins`, `geometry_rules_skip_the_recapture_and_size_rules_read_the_first_frame`.

## Decisions fixed while planning (now written into spec section 5)

1. **Rules in `crates/terminal`, not `crates/core`.** Nothing but the tmux engine consumes them; keeping them out of core keeps the wasm bundle free of tmux code. Cost if wrong: moving one module later.
2. **`unicode-width` is added** (one crate, no transitive dependencies). A hand-rolled width table would be wrong in exactly the emoji and CJK cases the width-drift rule exists for.
3. **`--tmux-capture <file>` replays a saved capture.** Without it the oracle could not pin the rules and the review flow could not check the captures `terminal.md` already asks agents to save.
4. **Frame roles** (`capture`, `recapture`) and the per-rule frame policy replace the original "two captures one second apart" sentence, so multi-size runs do not duplicate size-invariant findings.
5. **`--tmux` runs the runtime rules on any resolved platform.** The flag is the request; requiring `## Platform: terminal` would only add a failure mode.
6. **The integration test joins the `oracle` suite** as a second command plus one CI step in the oracle job, instead of a new default suite that would need a package script and a CI lane.
7. **The scratch server is reached through an `IMPECCABLE_TMUX` wrapper script** (`exec tmux -L <socket> "$@"`), which also exercises the override path, rather than by parsing `$TMUX`.
8. **Restore is `resize-window` back plus `set-option -w -u window-size`**, because `resize-window` leaves the window in `manual` size mode (verified on tmux 3.7c: `window-size` read `latest` before and `manual` after).

## File structure

| File | Responsibility |
|---|---|
| `crates/foundation/src/registry.rs` | seven `tui-rt-` rows after `tui-print-in-loop`; the two `72` pins become `79` |
| `Cargo.toml` (root) | `impeccable-terminal` workspace dependency |
| `crates/terminal/Cargo.toml` | new crate manifest |
| `crates/terminal/src/lib.rs` | `TerminalEngine`, `frame_from`, `capture_sizes`, the `TmuxEngine` impl |
| `crates/terminal/src/capture.rs` | `Color`, `Style`, `Cell`, `Frame`, `FrameRole`, `HEADER`, `parse_rows`, `parse_frames`, `parse_row` |
| `crates/terminal/src/palette.rs` | `Palette`, `ANSI16`, `indexed`, `hex` |
| `crates/terminal/src/rules.rs` | `scan_frames`, one `rt_<rule>` per rule, `find_boxes` |
| `crates/terminal/src/tmux.rs` | `find_tmux`, `parse_version`, `Tmux`, `PaneInfo`, `NOT_FOUND_MESSAGE` |
| `crates/detect/src/engines.rs` | `TmuxEngine`, `MissingTmuxEngine`, `TMUX_NOT_LINKED`, `Engines.tmux`, `ScanOptions.{tmux_sizes,tmux_settle_ms,palette}` |
| `crates/detect/src/lib.rs` | re-exports |
| `crates/detect/src/cli.rs` | five flags, `scan_tmux_targets`, USAGE, tests with a fake engine |
| `crates/cli/Cargo.toml`, `crates/cli/src/main.rs` | dependency and `tmux: Some(tmux_engine())` |
| `tests/fixtures/terminal-captures/*.txt` | eight saved-capture inputs with headers |
| `tests/fixtures/terminal-apps/nested-boxes.sh` | the deliberately bad TUI the integration test runs |
| `tests/tmux-engine.test.mjs` | scratch-server integration test |
| `scripts/test-suites.mjs`, `.github/workflows/ci.yml` | suite registration, one CI step |
| `tests/oracle/cases/detect.mjs`, `golden/`, `DELTAS.md` | cases, goldens, reviewed deltas |
| `crates/live/assets/antipatterns.json`, `detect-antipatterns-browser.js` | regenerated by `cargo xtask bundle` |
| `README.md`, `README.npm.md`, `docs/CLI-CONTRACT.md`, `docs/ENGINE.md`, root `CLAUDE.md` | counts 72 to 79, flags, crate row, rule-authoring note |
| `skill/reference/terminal.md`, `audit.terminal.md`, `adapt.terminal.md` | the `--tmux` verification bullet and two evidence sentences |
| `docs/superpowers/research/2026-09-24-terminal-runtime-rules-manual-pass.md` | Task 11's record |

---

### Task 1: Branch and baseline

**Files:** none changed.

- [ ] **Step 1: Branch from the fork's main**

```bash
cd /Volumes/Containers/impeccable
git checkout main && git pull --ff-only origin main
git checkout -b terminal-platform-pr3
git log --oneline -1   # a main that contains 86679d9d
```

- [ ] **Step 2: Build and confirm the tooling**

```bash
cargo build --release -p impeccable 2>&1 | tail -n 2
which wasm-pack || cargo install wasm-pack --locked
tmux -V            # 3.2 or newer; this machine has 3.7c
bun --version
```

The wasm bundle recipe on this machine (memory `impeccable-wasm-toolchain`): `T=$HOME/.rustup/toolchains/stable-x86_64-apple-darwin; PATH="$T/bin:$PATH" DYLD_LIBRARY_PATH="$T/lib" cargo xtask bundle [--check]`.

- [ ] **Step 3: Record the baseline**

```bash
cargo test --workspace 2>&1 | grep -E "^test result|FAILED" | sort | uniq -c
IMPECCABLE_BIN="$PWD/target/release/impeccable" bun run test 2>&1 | tail -n 15
node -e "console.log(JSON.parse(require('fs').readFileSync('crates/live/assets/antipatterns.json','utf8')).length)"   # expect 72
```

Known pre-existing failures on this machine, not to be fixed here: `tests/release.test.mjs` (signed tags), the port-8080 oracle flake, bun timeouts in `tests/build.test.js` under load (pass isolated), and `dev_url::tests::answers_whatever_scheme_the_page_uses` (flaky). The RTK proxy summarizes `cargo test` output; read the `test result` line from the summary, never grep counts out of it.

- [ ] **Step 4: No commit.** Record `BASE=$(git rev-parse HEAD)` and the pass counts in the ledger.

---

### Task 2: Registry: the seven runtime rows

**Files:**
- Modify: `crates/foundation/src/registry.rs` (rows after `tui-print-in-loop`; tests)
- Regenerate: `crates/live/assets/antipatterns.json`, `crates/live/assets/detect-antipatterns-browser.js`

**Interfaces:**
- Produces: the ids `tui-rt-low-contrast`, `tui-rt-nested-borders`, `tui-rt-truecolor-on-256`, `tui-rt-width-drift`, `tui-rt-no-key-hints`, `tui-rt-collapse-narrow`, `tui-rt-spinner-never-rests`, which `finding(id, ..)` in Tasks 3 and 5 to 7 requires.

- [ ] **Step 1: Write the failing test** at the end of the existing `mod tests` in `registry.rs` (after the last `#[test]`, before the module's closing brace):

```rust
    const RT_IDS: &[&str] = &[
        "tui-rt-low-contrast",
        "tui-rt-nested-borders",
        "tui-rt-truecolor-on-256",
        "tui-rt-width-drift",
        "tui-rt-no-key-hints",
        "tui-rt-collapse-narrow",
        "tui-rt-spinner-never-rests",
    ];

    #[test]
    fn runtime_rows_are_advisory_terminal_rows_at_the_end_of_the_registry() {
        let tail: Vec<&str> = ANTIPATTERNS.iter().rev().take(RT_IDS.len()).map(|a| a.id).rev().collect();
        assert_eq!(tail, RT_IDS, "runtime rows sit after the source rows so web output never moves");
        for id in RT_IDS {
            let ap = get_antipattern(id).unwrap_or_else(|| panic!("{id} missing"));
            assert_eq!(ap.severity, Some("advisory"), "{id}");
            assert_eq!(ap.platforms, Some(&["terminal"][..]), "{id}");
            assert!(!IMMEDIATE_TIER_RULES.contains(id), "{id} never runs in the hook");
            assert!(!ap.description.contains('\u{2014}'), "{id}: no em dash");
        }
        assert_eq!(get_antipattern("tui-rt-nested-borders").unwrap().category, "slop");
        assert_eq!(get_antipattern("tui-rt-low-contrast").unwrap().category, "quality");
        assert_eq!(ANTIPATTERNS.len(), 79);
    }
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test -p impeccable-foundation runtime_rows 2>&1 | tail -n 5
```

Expected: FAIL (the tail is the eleven `tui-` source rows, length 72).

- [ ] **Step 3: Append the seven rows** immediately after the `tui-print-in-loop` row's closing `},` and before the `];` that closes `ANTIPATTERNS`. Copy exactly:

```rust
    Antipattern {
        id: "tui-rt-low-contrast",
        category: "quality",
        scopes: None,
        severity: Some("advisory"),
        platforms: Some(&["terminal"]),
        name: "Low contrast text in the rendered pane",
        description: "A rendered foreground and background pair measures below 4.5:1 on the reference palette. Pick colors that pass on both a dark and a light terminal theme, or use the terminal's default colors.",
        skill_section: None,
        skill_guideline: None,
    },
    Antipattern {
        id: "tui-rt-nested-borders",
        category: "slop",
        scopes: None,
        severity: Some("advisory"),
        platforms: Some(&["terminal"]),
        name: "Box drawn directly inside another box",
        description: "A bordered region sits inside another bordered region with nothing between them. Borders are the terminal's only container signal; nest at most one level and let padding do the rest.",
        skill_section: None,
        skill_guideline: None,
    },
    Antipattern {
        id: "tui-rt-truecolor-on-256",
        category: "quality",
        scopes: None,
        severity: Some("advisory"),
        platforms: Some(&["terminal"]),
        name: "Truecolor sequences sent to a 256-color client",
        description: "The pane wrote 24-bit color while the client reports 256 colors or fewer, so the colors quantize unpredictably. Detect the color tier and fall back to indexed colors.",
        skill_section: None,
        skill_guideline: None,
    },
    Antipattern {
        id: "tui-rt-width-drift",
        category: "quality",
        scopes: None,
        severity: Some("advisory"),
        platforms: Some(&["terminal"]),
        name: "Row width drifts on emoji or CJK",
        description: "A row containing emoji or CJK measures wider than the pane or breaks the border column. Measure display width with a grapheme-aware library before padding or truncating.",
        skill_section: None,
        skill_guideline: None,
    },
    Antipattern {
        id: "tui-rt-no-key-hints",
        category: "quality",
        scopes: None,
        severity: Some("advisory"),
        platforms: Some(&["terminal"]),
        name: "No key hints on the bottom row",
        description: "The bottom row names no key. Show the two or three keys that matter on every screen; discoverability in a terminal has nowhere else to live.",
        skill_section: None,
        skill_guideline: None,
    },
    Antipattern {
        id: "tui-rt-collapse-narrow",
        category: "quality",
        scopes: None,
        severity: Some("advisory"),
        platforms: Some(&["terminal"]),
        name: "Layout collapses below 60 columns",
        description: "At a narrow width the layout overflows, splits words, or loses the far border. Restructure below 60 columns: hide or stack a region instead of shrinking every pane.",
        skill_section: None,
        skill_guideline: None,
    },
    Antipattern {
        id: "tui-rt-spinner-never-rests",
        category: "quality",
        scopes: None,
        severity: Some("advisory"),
        platforms: Some(&["terminal"]),
        name: "Spinner still cycling with no input",
        description: "A spinner glyph keeps changing across two captures a second apart with nothing to wait for. Show a spinner only while work is pending and replace it with the result.",
        skill_section: None,
        skill_guideline: None,
    },
```

- [ ] **Step 4: Update the two pinned counts.** `grep -n "ANTIPATTERNS.len(), 72" crates/foundation/src/registry.rs` finds two assertions (near lines 1054 and 1210); change both to `79`.

- [ ] **Step 5: Run the tests**

```bash
cargo test -p impeccable-foundation 2>&1 | tail -n 3
cargo test -p impeccable-core 2>&1 | tail -n 3      # frozen call vectors still replay
```

- [ ] **Step 6: Regenerate the two tracked assets**

```bash
T=$HOME/.rustup/toolchains/stable-x86_64-apple-darwin
PATH="$T/bin:$PATH" DYLD_LIBRARY_PATH="$T/lib" cargo xtask bundle 2>&1 | tail -n 2
node -e "const a=JSON.parse(require('fs').readFileSync('crates/live/assets/antipatterns.json','utf8')); console.log(a.length, a.slice(-7).map(x=>x.id).join(','))"   # 79 and the seven ids
git status --short crates/live/assets
```

- [ ] **Step 7: Commit**

```bash
git add crates/foundation/src/registry.rs crates/live/assets/antipatterns.json crates/live/assets/detect-antipatterns-browser.js
git commit -m "$(printf 'Add the seven tui-rt- runtime rule rows to the registry\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 3: The detect seam: `TmuxEngine`, `ScanOptions` fields, five flags, `scan_tmux_targets`

**Files:**
- Modify: `crates/detect/src/engines.rs`, `crates/detect/src/lib.rs`, `crates/detect/src/cli.rs`
- Modify: `crates/cli/src/main.rs:123-127` (`Engines` literal gains `tmux: None` for now; Task 7 wires the real engine)

**Interfaces:**
- Consumes: the seven registry ids from Task 2 (the fake engine in the tests calls `finding("tui-rt-no-key-hints", ..)`).
- Produces: `impeccable_detect::engines::{TmuxEngine, MissingTmuxEngine, TMUX_NOT_LINKED}`; `ScanOptions.tmux_sizes: Vec<(u32, u32)>`, `.tmux_settle_ms: Option<u64>`, `.palette: Option<String>`; `Engines.tmux: Option<&'a dyn TmuxEngine>`. Task 7's `TerminalEngine` implements the trait.

- [ ] **Step 1: Write the failing tests** at the end of `mod tests` in `crates/detect/src/cli.rs`. Also change the existing helper so every `Engines` literal names the new field:

```rust
    fn engines(platform: Option<&'static dyn Fn(&str) -> Option<String>>) -> Engines<'static> {
        static HTML: MissingHtmlEngine = MissingHtmlEngine;
        Engines { html: &HTML, url: None, tmux: None, platform }
    }

    struct FakeTmux;
    impl crate::engines::TmuxEngine for FakeTmux {
        fn detect_pane(&self, target: &str, options: &ScanOptions) -> Result<Vec<Finding>, EngineError> {
            let mut f = impeccable_core::findings::finding(
                "tui-rt-no-key-hints",
                &format!("tmux:{target}"),
                &format!("sizes={:?} palette={:?} settle={:?}", options.tmux_sizes, options.palette, options.tmux_settle_ms),
                1.0,
            );
            impeccable_core::findings::derive_advisory_flag(&mut f);
            Ok(vec![f])
        }
        fn detect_capture(&self, path: &str, _options: &ScanOptions) -> Result<Vec<Finding>, EngineError> {
            Err(EngineError::new(format!("ENOENT: no such file or directory, open '{path}'")))
        }
    }

    fn engines_with_tmux() -> Engines<'static> {
        static HTML: MissingHtmlEngine = MissingHtmlEngine;
        static TMUX: FakeTmux = FakeTmux;
        Engines { html: &HTML, url: None, tmux: Some(&TMUX), platform: None }
    }

    #[test]
    fn tmux_flags_reach_the_engine_and_their_values_never_become_targets() {
        let dir = project("tmux-flags");
        let (code, out, err) = run(
            &dir,
            &["--no-config", "--json", "--tmux", "app:0.0", "--tmux-sizes", "80x24,40x24", "--palette=light", "--tmux-settle", "50"],
            &engines_with_tmux(),
        );
        assert_eq!(code, 0, "advisory findings never fail the run: {err}");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let items = v.as_array().unwrap();
        assert_eq!(items.len(), 1, "no cwd fallback scan happened: {out}");
        assert_eq!(items[0]["file"], "tmux:app:0.0");
        assert_eq!(items[0]["snippet"], "sizes=[(80, 24), (40, 24)] palette=Some(\"light\") settle=Some(50)");
        assert_eq!(items[0]["advisory"], true);
        assert!(!err.contains("cannot access"), "a flag value was taken as a target: {err}");
    }

    #[test]
    fn tmux_pane_and_file_targets_scan_together() {
        let dir = project("tmux-mixed");
        let (_, out, _) = run(&dir, &["--no-config", "--json", "--platform", "terminal", "--tmux", "app:0.0", "src"], &engines_with_tmux());
        assert!(out.contains("\"tui-double-border\"") && out.contains("tmux:app:0.0"), "{out}");
    }

    #[test]
    fn unreadable_capture_file_is_an_operational_failure() {
        let dir = project("tmux-capture");
        let (code, _, err) = run(&dir, &["--no-config", "--json", "--tmux-capture", "missing.txt"], &engines_with_tmux());
        assert_eq!(code, 1);
        let resolved = jsp::resolve(&dir.to_string_lossy(), &["missing.txt"]);
        assert_eq!(err, format!("Error: cannot scan missing.txt: ENOENT: no such file or directory, open '{resolved}'\n"));
    }

    #[test]
    fn tmux_flag_values_are_validated_with_the_spec_messages() {
        let dir = project("tmux-errors");
        let cases: &[(&[&str], &str)] = &[
            (&["--tmux"], "Error: --tmux requires a tmux target, e.g. --tmux app:0.0\n"),
            (&["--tmux-capture"], "Error: --tmux-capture requires a path to a saved capture\n"),
            (&["--tmux", "a:0", "--tmux-sizes", "wide"], "Error: --tmux-sizes requires comma-separated WxH values, e.g. --tmux-sizes 80x24,120x40,40x24\n"),
            (&["--tmux", "a:0", "--tmux-sizes=80x24,"], "Error: --tmux-sizes requires comma-separated WxH values, e.g. --tmux-sizes 80x24,120x40,40x24\n"),
            (&["--tmux", "a:0", "--tmux-settle", "soon"], "Error: --tmux-settle requires a whole number of milliseconds\n"),
            (&["--tmux", "a:0", "--palette", "sepia"], "Error: --palette requires dark or light\n"),
        ];
        for (args, expected) in cases {
            let mut full = vec!["--no-config"];
            full.extend_from_slice(args);
            let (code, _, err) = run(&dir, &full, &engines_with_tmux());
            assert_eq!((code, err.as_str()), (1, *expected), "{args:?}");
        }
    }

    #[test]
    fn build_without_the_tmux_engine_says_so() {
        let dir = project("tmux-missing-engine");
        let (code, _, err) = run(&dir, &["--no-config", "--tmux", "a:0"], &engines(None));
        assert_eq!(code, 1);
        assert_eq!(err, "Error: impeccable detect: tmux engine is not linked into this build\n");
    }
```

The test module needs `use crate::engines::{EngineError, ScanOptions};` next to its existing imports (`Finding` is already imported at the top of the file).

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test -p impeccable-detect tmux 2>&1 | tail -n 5
```

Expected: compile errors (`tmux` field, `TmuxEngine` missing).

- [ ] **Step 3: The seam** in `engines.rs`. Extend `ScanOptions` after `signals`:

```rust
    /// `--tmux-sizes` (tmux scans only): extra WxH captures after the pane's
    /// own size, restored afterwards.
    pub tmux_sizes: Vec<(u32, u32)>,
    /// `--tmux-settle` (tmux scans only); `None` is the engine default (300 ms).
    pub tmux_settle_ms: Option<u64>,
    /// `--palette` (tmux scans only): `dark` (default) or `light`.
    pub palette: Option<String>,
```

After the `SharedBrowser` trait add:

```rust
/// The tmux engine (spec section 5). Implemented by crates/terminal; `detect`
/// never depends on that crate.
pub trait TmuxEngine {
    /// Capture the running pane `target` (`session:window.pane`) at its own
    /// size, again one second later, and at each `options.tmux_sizes`, then
    /// run the runtime rules. Findings name `tmux:<target>`.
    fn detect_pane(&self, target: &str, options: &ScanOptions) -> Result<Vec<Finding>, EngineError>;
    /// Replay a saved `tmux capture-pane -p -e -J` file (one or more frames).
    fn detect_capture(&self, path: &str, options: &ScanOptions) -> Result<Vec<Finding>, EngineError>;
}
```

Extend `Engines` between `url` and `platform`:

```rust
    /// The tmux engine behind `--tmux` / `--tmux-capture`; `None` in a build
    /// that does not link crates/terminal.
    pub tmux: Option<&'a dyn TmuxEngine>,
```

After `MissingUrlEngine` add:

```rust
/// A build without crates/terminal (the `cli` binary always links it), so a
/// missing engine is an internal error reported like `MissingHtmlEngine`'s.
pub const TMUX_NOT_LINKED: &str = "impeccable detect: tmux engine is not linked into this build";

pub struct MissingTmuxEngine;

impl TmuxEngine for MissingTmuxEngine {
    fn detect_pane(&self, _target: &str, _options: &ScanOptions) -> Result<Vec<Finding>, EngineError> {
        Err(EngineError::new(TMUX_NOT_LINKED))
    }
    fn detect_capture(&self, _path: &str, _options: &ScanOptions) -> Result<Vec<Finding>, EngineError> {
        Err(EngineError::new(TMUX_NOT_LINKED))
    }
}
```

In `lib.rs` change the re-export to:

```rust
pub use engines::{
    Engines, HtmlEngine, MissingHtmlEngine, MissingTmuxEngine, MissingUrlEngine, ScanOptions,
    TmuxEngine, UrlEngine,
};
```

In `crates/cli/src/main.rs` add `tmux: None,` between `url: Some(url_engine()),` and `platform: Some(&RESOLVE),` so the workspace builds (Task 7 replaces it).

- [ ] **Step 4: The flags** in `cli.rs`. Add after the `FILE_URL_RE` regex:

```rust
re!(TMUX_SIZE_RE, format!("^({D}{{1,4}})[xX]({D}{{1,4}})$"));

/// A flag whose value is missing or malformed: the message, exit 1.
fn usage_error(io: &mut Io, message: &str) -> Result<i32, Exit> {
    io.err(message);
    Err(Exit(1))
}
```

In `detect_cli`, directly after the `--platform` loop (before the `// The flag wins; ...` comment) add:

```rust
    // --tmux, --tmux-capture, --tmux-sizes, --tmux-settle, --palette (spec
    // section 5). Each value is spliced out like --viewport's so it never
    // becomes a target; the --flag=value form works for all five.
    const TMUX_SIZES_ERROR: &str = "Error: --tmux-sizes requires comma-separated WxH values, e.g. --tmux-sizes 80x24,120x40,40x24\n";
    const TMUX_SETTLE_ERROR: &str = "Error: --tmux-settle requires a whole number of milliseconds\n";
    let mut tmux_panes: Vec<String> = Vec::new();
    let mut tmux_captures: Vec<String> = Vec::new();
    let mut tmux_sizes: Vec<(u32, u32)> = Vec::new();
    let mut tmux_settle_ms: Option<u64> = None;
    let mut palette: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        let (flag, inline_value) = match args[i].split_once('=') {
            Some((f, v)) if f.starts_with("--") => (f.to_string(), Some(v.to_string())),
            _ => (args[i].clone(), None),
        };
        if !matches!(flag.as_str(), "--tmux" | "--tmux-capture" | "--tmux-sizes" | "--tmux-settle" | "--palette") {
            i += 1;
            continue;
        }
        let value = inline_value
            .clone()
            .or_else(|| args.get(i + 1).cloned())
            .filter(|v| !v.starts_with("--"));
        match (flag.as_str(), value) {
            ("--tmux", Some(v)) => tmux_panes.push(v),
            ("--tmux", None) => return usage_error(io, "Error: --tmux requires a tmux target, e.g. --tmux app:0.0\n"),
            ("--tmux-capture", Some(v)) => tmux_captures.push(v),
            ("--tmux-capture", None) => return usage_error(io, "Error: --tmux-capture requires a path to a saved capture\n"),
            ("--tmux-sizes", Some(v)) => {
                for part in v.split(',') {
                    let Some(m) = TMUX_SIZE_RE.captures(impeccable_core::js::trim(part)) else {
                        return usage_error(io, TMUX_SIZES_ERROR);
                    };
                    tmux_sizes.push((m[1].parse().unwrap_or(0), m[2].parse().unwrap_or(0)));
                }
            }
            ("--tmux-sizes", None) => return usage_error(io, TMUX_SIZES_ERROR),
            ("--tmux-settle", Some(v)) => match v.parse::<u64>() {
                Ok(ms) => tmux_settle_ms = Some(ms),
                Err(_) => return usage_error(io, TMUX_SETTLE_ERROR),
            },
            ("--tmux-settle", None) => return usage_error(io, TMUX_SETTLE_ERROR),
            ("--palette", Some(v)) if v == "dark" || v == "light" => palette = Some(v),
            ("--palette", _) => return usage_error(io, "Error: --palette requires dark or light\n"),
            _ => unreachable!("every tmux flag is matched above"),
        }
        let n = if inline_value.is_some() { 1 } else { 2 };
        for _ in 0..n {
            if i < args.len() {
                args.remove(i);
            }
        }
    }
    let has_tmux_targets = !tmux_panes.is_empty() || !tmux_captures.is_empty();
```

Extend the `base` literal with `tmux_sizes, tmux_settle_ms, palette: palette.clone(),` after `signals: None,`.

Change the stdin condition and the cwd default:

```rust
    if !stdin_tty && targets.is_empty() && !has_tmux_targets {
        all = ctx.handle_stdin().map_err(|e| fatal(ctx.io, e))?;
    } else {
        let paths: Vec<String> = if targets.is_empty() && !has_tmux_targets {
            vec![cwd.clone()]
        } else {
            targets.clone()
        };
```

and after `result?;` in that branch add `scan_tmux_targets(&mut ctx, &tmux_panes, &tmux_captures, &mut all);`.

Add the function after `scan_targets`:

```rust
/// `--tmux` panes and `--tmux-capture` files (spec section 5). A pane failure
/// is reported like a URL scan's (`Error: <message>`); a capture file like a
/// local file's. Both set the operational failure and the run continues.
fn scan_tmux_targets(ctx: &mut Ctx, panes: &[String], captures: &[String], all: &mut Vec<Finding>) {
    use crate::engines::{MissingTmuxEngine, TmuxEngine};
    let options = ctx.base.clone();
    for target in panes {
        let result = match ctx.engines.tmux {
            Some(engine) => engine.detect_pane(target, &options),
            None => MissingTmuxEngine.detect_pane(target, &options),
        };
        match result {
            Ok(f) => all.extend(f),
            Err(e) => {
                ctx.had_operational_failure = true;
                ctx.io.err(&format!("Error: {}\n", e.message));
            }
        }
    }
    for path in captures {
        let resolved = jsp::resolve(&ctx.cwd, &[path]);
        let result = match ctx.engines.tmux {
            Some(engine) => engine.detect_capture(&resolved, &options),
            None => MissingTmuxEngine.detect_capture(&resolved, &options),
        };
        match result {
            Ok(f) => all.extend(f),
            Err(e) => {
                let message = e.message.clone();
                ctx.report_local_scan_failure(path, &message);
            }
        }
    }
}
```

- [ ] **Step 5: USAGE.** In the `USAGE` constant insert after the `--platform` line:

```
  --tmux <target>     Capture a running tmux pane (session:window.pane) and run
                      the terminal runtime rules over it (tmux 3.2 or newer)
  --tmux-capture <f>  Run the runtime rules over a saved capture-pane -p -e -J file
  --tmux-sizes <list> Also capture the pane at each WxH and restore its size after,
                      e.g. --tmux-sizes 80x24,120x40,40x24
  --tmux-settle <ms>  Wait after each resize before capturing (default 300)
  --palette <name>    Contrast palette for tmux scans: dark (default) or light
```

Under `Detection modes:` add after the two `URLs` lines:

```
  tmux panes     --tmux captures the rendered grid with colors, never launching
                 or typing; --tmux-capture replays a saved capture without tmux
```

Under `Examples:` add as the last line: `  impeccable detect --tmux app:0.0 --tmux-sizes 80x24,120x40,40x24`.

- [ ] **Step 6: Run the tests**

```bash
cargo test -p impeccable-detect 2>&1 | tail -n 3
cargo build --workspace --all-targets 2>&1 | tail -n 2
```

- [ ] **Step 7: Prove web output is untouched** before anything else changes:

```bash
cargo build --release -p impeccable 2>&1 | tail -n 1
IMPECCABLE_BIN="$PWD/target/release/impeccable" node --test tests/oracle.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)"
```

Expected: exactly one failure, `detect-help` (the USAGE text grew); Task 8 re-records it.

- [ ] **Step 8: Commit**

```bash
git add crates/detect/src/engines.rs crates/detect/src/lib.rs crates/detect/src/cli.rs crates/cli/src/main.rs
git commit -m "$(printf 'Add the tmux engine seam and the --tmux flags to impeccable detect\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 4: `crates/terminal`: the crate, the capture parser, the palette

**Files:**
- Create: `crates/terminal/Cargo.toml`, `crates/terminal/src/lib.rs` (module list only for now), `crates/terminal/src/capture.rs`, `crates/terminal/src/palette.rs`
- Modify: `Cargo.toml` (root, `[workspace.dependencies]`)

**Interfaces:**
- Produces: `capture::{Color, Style, Cell, Frame, FrameRole, HEADER, parse_rows, parse_frames, parse_row}` and `palette::{Palette, ANSI16, indexed, hex}` for Tasks 5 to 7. `Frame` methods: `glyph(row, col) -> Option<char>`, `row_width(row)`, `text(row) -> String`, `is_blank(row)`, `has_wide(row)`. `Palette::parse(&str) -> Option<Palette>`, `Palette::resolve(&Style) -> (Rgba, Rgba)`.

- [ ] **Step 1: The manifest and the workspace entry.** Create `crates/terminal/Cargo.toml`:

```toml
[package]
name = "impeccable-terminal"
edition.workspace = true
version.workspace = true
license.workspace = true
publish.workspace = true

[dependencies]
impeccable-core = { workspace = true }
impeccable-detect = { workspace = true }
serde_json = { workspace = true, features = ["preserve_order", "float_roundtrip"] }
regex = { workspace = true }
once_cell = { workspace = true }
unicode-width = "0.2"
```

In the root `Cargo.toml`, after `impeccable-bundle = { path = "crates/bundle" }` add `impeccable-terminal = { path = "crates/terminal" }`. (`members = ["crates/*"]` already picks the crate up.)

Create `crates/terminal/src/lib.rs` with just:

```rust
//! impeccable-terminal: the tmux engine of `impeccable detect` (spec section
//! 5). Captures a running tmux pane, or replays a saved capture, parses it
//! into frames, and runs the `tui-rt-` runtime rules. Wired into the binary
//! through [`impeccable_detect::engines::TmuxEngine`]; the engine never
//! launches or kills a process and always restores a window size it changed.

pub mod capture;
pub mod palette;
```

- [ ] **Step 2: Write `capture.rs` with its tests** (the tests are the failing half; write the whole file, run, and fix until green):

```rust
//! Capture parsing: `tmux capture-pane -p -e -J` text into a cell grid
//! (spec section 5, "Capture and the frame model"). Pure functions over
//! text, so goldens replay from saved files without tmux.

use unicode_width::UnicodeWidthChar;

/// A color as the SGR stream named it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Color {
    #[default]
    Default,
    /// `30..37` and `40..47` as 0 to 7, `90..97` and `100..107` as 8 to 15,
    /// `38;5;n` / `48;5;n` as 0 to 255.
    Indexed(u8),
    /// `38;2;r;g;b` / `48;2;r;g;b`.
    Rgb(u8, u8, u8),
}

/// The SGR state a cell was written with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub dim: bool,
    pub blink: bool,
    pub reverse: bool,
}

/// One column of one row. A two-column glyph owns its cell and the next,
/// which carries `glyph: None` and `wide_tail: true`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub glyph: Option<char>,
    pub wide_tail: bool,
    pub style: Style,
}

/// Which capture of the run a frame is (spec section 5 vocabulary).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FrameRole {
    /// A capture at a size the run asked for.
    #[default]
    Capture,
    /// The pane's own size again, one second after the first capture.
    Recapture,
}

/// One parsed capture.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Frame {
    pub width: usize,
    pub height: usize,
    /// `#{client_termfeatures}` (for example `256,RGB`); empty when unknown.
    pub termfeatures: String,
    /// `COLORTERM` in the engine's environment; empty when unset.
    pub colorterm: String,
    pub role: FrameRole,
    pub rows: Vec<Vec<Cell>>,
}

impl Frame {
    pub fn glyph(&self, row: usize, col: usize) -> Option<char> {
        self.rows.get(row)?.get(col)?.glyph
    }

    pub fn row_width(&self, row: usize) -> usize {
        self.rows.get(row).map(Vec::len).unwrap_or(0)
    }

    /// The row as text, one character per glyph (wide tails skipped).
    pub fn text(&self, row: usize) -> String {
        self.rows
            .get(row)
            .map(|r| r.iter().filter(|c| !c.wide_tail).map(|c| c.glyph.unwrap_or(' ')).collect())
            .unwrap_or_default()
    }

    pub fn is_blank(&self, row: usize) -> bool {
        self.rows
            .get(row)
            .map(|r| r.iter().all(|c| c.glyph.map(char::is_whitespace).unwrap_or(true)))
            .unwrap_or(true)
    }

    /// Does the row hold a two-column glyph (emoji, CJK)?
    pub fn has_wide(&self, row: usize) -> bool {
        self.rows.get(row).map(|r| r.iter().any(|c| c.wide_tail)).unwrap_or(false)
    }
}

/// The line that introduces a frame in a saved capture file.
pub const HEADER: &str = "#!capture";

/// Rows of captured text into cells. A trailing newline does not add a row.
pub fn parse_rows(text: &str) -> Vec<Vec<Cell>> {
    let mut lines: Vec<&str> = text.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    lines.into_iter().map(parse_row).collect()
}

/// A saved capture file into frames: every `#!capture` line starts a frame;
/// a file without one is a single frame measured from its rows.
pub fn parse_frames(text: &str) -> Result<Vec<Frame>, String> {
    let mut groups: Vec<(Option<&str>, Vec<&str>)> = Vec::new();
    for line in text.split('\n') {
        if let Some(rest) = line.strip_prefix(HEADER) {
            groups.push((Some(rest), Vec::new()));
        } else if let Some(last) = groups.last_mut() {
            last.1.push(line);
        } else {
            groups.push((None, vec![line]));
        }
    }
    let mut frames = Vec::new();
    for (header, mut lines) in groups {
        if lines.last() == Some(&"") {
            lines.pop();
        }
        let mut frame = Frame::default();
        if let Some(h) = header {
            apply_header(&mut frame, h)?;
        }
        frame.rows = lines.into_iter().map(parse_row).collect();
        if frame.width == 0 {
            frame.width = frame.rows.iter().map(Vec::len).max().unwrap_or(0);
        }
        if frame.height == 0 {
            frame.height = frame.rows.len();
        }
        frames.push(frame);
    }
    Ok(frames)
}

fn apply_header(frame: &mut Frame, header: &str) -> Result<(), String> {
    for pair in header.split_whitespace() {
        let Some((key, value)) = pair.split_once('=') else {
            return Err(format!("capture header: expected key=value, got {pair:?}"));
        };
        match key {
            "width" => {
                frame.width = value
                    .parse()
                    .map_err(|_| format!("capture header: width {value:?} is not a number"))?
            }
            "height" => {
                frame.height = value
                    .parse()
                    .map_err(|_| format!("capture header: height {value:?} is not a number"))?
            }
            "termfeatures" => frame.termfeatures = value.to_string(),
            "colorterm" => frame.colorterm = value.to_string(),
            "role" => {
                frame.role = match value {
                    "capture" => FrameRole::Capture,
                    "recapture" => FrameRole::Recapture,
                    _ => return Err(format!("capture header: unknown role {value:?}")),
                }
            }
            _ => return Err(format!("capture header: unknown key {key:?}")),
        }
    }
    Ok(())
}

/// One captured row. SGR sequences update the style; every other character
/// is a cell (zero-width characters take none, two-column glyphs take two).
pub fn parse_row(line: &str) -> Vec<Cell> {
    let mut cells = Vec::new();
    let mut style = Style::default();
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                let mut params = String::new();
                let mut terminator = None;
                for c in chars.by_ref() {
                    if c.is_ascii_digit() || c == ';' || c == ':' {
                        params.push(c);
                    } else {
                        terminator = Some(c);
                        break;
                    }
                }
                if terminator == Some('m') {
                    apply_sgr(&mut style, &params);
                }
            }
            continue;
        }
        if ch == '\r' {
            continue;
        }
        match ch.width().unwrap_or(0) {
            0 => {}
            1 => cells.push(Cell { glyph: Some(ch), wide_tail: false, style }),
            _ => {
                cells.push(Cell { glyph: Some(ch), wide_tail: false, style });
                cells.push(Cell { glyph: None, wide_tail: true, style });
            }
        }
    }
    cells
}

/// The SGR parameters tmux emits: reset, bold, dim, blink, reverse and their
/// offs, the 16 named colors, `38;5;n` / `48;5;n`, `38;2;r;g;b` / `48;2;r;g;b`.
/// Anything else is ignored.
fn apply_sgr(style: &mut Style, params: &str) {
    let parts: Vec<&str> = if params.is_empty() { vec!["0"] } else { params.split([';', ':']).collect() };
    let num = |i: usize| parts.get(i).and_then(|p| p.parse::<u16>().ok());
    let mut i = 0;
    while i < parts.len() {
        let Some(p) = num(i) else {
            i += 1;
            continue;
        };
        match p {
            0 => *style = Style::default(),
            1 => style.bold = true,
            2 => style.dim = true,
            5 | 6 => style.blink = true,
            7 => style.reverse = true,
            22 => {
                style.bold = false;
                style.dim = false;
            }
            25 => style.blink = false,
            27 => style.reverse = false,
            30..=37 => style.fg = Color::Indexed((p - 30) as u8),
            90..=97 => style.fg = Color::Indexed((p - 90 + 8) as u8),
            39 => style.fg = Color::Default,
            40..=47 => style.bg = Color::Indexed((p - 40) as u8),
            100..=107 => style.bg = Color::Indexed((p - 100 + 8) as u8),
            49 => style.bg = Color::Default,
            38 | 48 => {
                let color = match num(i + 1) {
                    Some(5) => {
                        let index = num(i + 2).unwrap_or(0).min(255) as u8;
                        i += 2;
                        Some(Color::Indexed(index))
                    }
                    Some(2) => {
                        // `38;2;r;g;b`; the colon form may carry an empty
                        // color-space id (`38:2::r:g:b`).
                        let skip = usize::from(parts.get(i + 2) == Some(&""));
                        let channel = |k: usize| num(i + 2 + skip + k).unwrap_or(0).min(255) as u8;
                        let rgb = Color::Rgb(channel(0), channel(1), channel(2));
                        i += 4 + skip;
                        Some(rgb)
                    }
                    _ => None,
                };
                if let Some(c) = color {
                    if p == 38 {
                        style.fg = c
                    } else {
                        style.bg = c
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sgr_colors_and_attributes_land_on_the_cells_they_precede() {
        let cells = parse_row("\x1b[1m\x1b[38;2;120;120;120m\x1b[48;2;100;100;100mhi\x1b[0m \x1b[31mred\x1b[39m!");
        assert_eq!(cells.len(), 7);
        assert_eq!(
            cells[0].style,
            Style { fg: Color::Rgb(120, 120, 120), bg: Color::Rgb(100, 100, 100), bold: true, ..Style::default() }
        );
        assert_eq!(cells[2].style, Style::default(), "0 resets everything");
        assert_eq!(cells[3].style.fg, Color::Indexed(1));
        assert_eq!(cells[6].style.fg, Color::Default, "39 resets the foreground only");
        assert_eq!(cells[6].glyph, Some('!'));
    }

    #[test]
    fn bright_indexed_and_combined_parameters_map_into_the_256_index_space() {
        let cells = parse_row("\x1b[97;104ma\x1b[38;5;208mb\x1b[7mc\x1b[38;2;1;2;3;48;2;4;5;6md");
        assert_eq!((cells[0].style.fg, cells[0].style.bg), (Color::Indexed(15), Color::Indexed(12)));
        assert_eq!(cells[1].style.fg, Color::Indexed(208));
        assert!(cells[2].style.reverse);
        assert_eq!((cells[3].style.fg, cells[3].style.bg), (Color::Rgb(1, 2, 3), Color::Rgb(4, 5, 6)));
    }

    #[test]
    fn wide_glyphs_take_two_cells_and_zero_width_marks_take_none() {
        let cells = parse_row("a😀b日本⚠\u{FE0F}");
        assert_eq!(cells.len(), 9);
        assert!(cells[2].wide_tail && cells[2].glyph.is_none());
        let frame = Frame { rows: vec![cells], ..Frame::default() };
        assert_eq!(frame.text(0), "a😀b日本⚠");
        assert!(frame.has_wide(0));
        assert_eq!(frame.row_width(0), 9);
    }

    #[test]
    fn capture_parser_replays_without_tmux() {
        let text = "#!capture width=40 height=3 termfeatures=256 colorterm=truecolor\nrow one\n\n\n#!capture width=40 height=3 role=recapture\nrow two\n";
        let frames = parse_frames(text).unwrap();
        assert_eq!(frames.len(), 2);
        assert_eq!((frames[0].width, frames[0].height, frames[0].rows.len()), (40, 3, 2));
        assert!(frames[0].is_blank(1), "an empty row inside a frame is kept");
        assert_eq!(frames[0].termfeatures, "256");
        assert_eq!(frames[0].colorterm, "truecolor");
        assert_eq!(frames[0].role, FrameRole::Capture);
        assert_eq!(frames[1].role, FrameRole::Recapture);
        assert_eq!(frames[1].text(0), "row two");
    }

    #[test]
    fn headerless_capture_measures_itself_and_knows_no_client() {
        let frames = parse_frames("ab\nabcd\n").unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!((frames[0].width, frames[0].height), (4, 2));
        assert_eq!(frames[0].termfeatures, "");
        assert_eq!(frames[0].role, FrameRole::Capture);
    }

    #[test]
    fn bad_headers_are_errors_not_guesses() {
        assert_eq!(parse_frames("#!capture width=wide\n").unwrap_err(), "capture header: width \"wide\" is not a number");
        assert_eq!(parse_frames("#!capture role=third\n").unwrap_err(), "capture header: unknown role \"third\"");
        assert_eq!(parse_frames("#!capture depth=8\n").unwrap_err(), "capture header: unknown key \"depth\"");
        assert_eq!(parse_frames("#!capture width\n").unwrap_err(), "capture header: expected key=value, got \"width\"");
    }
}
```

- [ ] **Step 3: Write `palette.rs` with its tests**:

```rust
//! SGR colors to sRGB for the contrast rule (spec section 5, "Palette").

use impeccable_core::color::Rgba;

use crate::capture::{Color, Style};

/// The reference palette contrast is measured on. The two differ only in
/// the default foreground and background.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Palette {
    #[default]
    Dark,
    Light,
}

impl Palette {
    pub fn parse(name: &str) -> Option<Palette> {
        match name {
            "dark" => Some(Palette::Dark),
            "light" => Some(Palette::Light),
            _ => None,
        }
    }

    fn default_fg(self) -> (u8, u8, u8) {
        match self {
            Palette::Dark => (0xff, 0xff, 0xff),
            Palette::Light => (0x00, 0x00, 0x00),
        }
    }

    fn default_bg(self) -> (u8, u8, u8) {
        match self {
            Palette::Dark => (0x00, 0x00, 0x00),
            Palette::Light => (0xff, 0xff, 0xff),
        }
    }

    /// The (foreground, background) a cell renders with, after `reverse`.
    pub fn resolve(self, style: &Style) -> (Rgba, Rgba) {
        let fg = self.color(style.fg, true);
        let bg = self.color(style.bg, false);
        if style.reverse {
            (bg, fg)
        } else {
            (fg, bg)
        }
    }

    fn color(self, color: Color, foreground: bool) -> Rgba {
        let (r, g, b) = match color {
            Color::Default => {
                if foreground {
                    self.default_fg()
                } else {
                    self.default_bg()
                }
            }
            Color::Indexed(i) => indexed(i),
            Color::Rgb(r, g, b) => (r, g, b),
        };
        Rgba { r: r as f64, g: g as f64, b: b as f64, a: None }
    }
}

/// xterm's sixteen named colors, the table both palettes share: the rule is
/// about the pairs the app chose, not about a theme.
pub const ANSI16: [(u8, u8, u8); 16] = [
    (0x00, 0x00, 0x00), (0xcd, 0x00, 0x00), (0x00, 0xcd, 0x00), (0xcd, 0xcd, 0x00),
    (0x00, 0x00, 0xee), (0xcd, 0x00, 0xcd), (0x00, 0xcd, 0xcd), (0xe5, 0xe5, 0xe5),
    (0x7f, 0x7f, 0x7f), (0xff, 0x00, 0x00), (0x00, 0xff, 0x00), (0xff, 0xff, 0x00),
    (0x5c, 0x5c, 0xff), (0xff, 0x00, 0xff), (0x00, 0xff, 0xff), (0xff, 0xff, 0xff),
];

/// The 256-color space: the named sixteen, the 6x6x6 cube, the 24 grays.
pub fn indexed(index: u8) -> (u8, u8, u8) {
    match index {
        0..=15 => ANSI16[index as usize],
        16..=231 => {
            let n = index - 16;
            let level = |v: u8| if v == 0 { 0 } else { 55 + 40 * v };
            (level(n / 36), level((n / 6) % 6), level(n % 6))
        }
        _ => {
            let v = 8 + 10 * (index - 232);
            (v, v, v)
        }
    }
}

/// `#rrggbb`, lowercase, for snippets.
pub fn hex(color: &Rgba) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r as u8, color.g as u8, color.b as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexed_cube_and_grays_match_xterm() {
        assert_eq!(indexed(16), (0, 0, 0));
        assert_eq!(indexed(196), (255, 0, 0));
        assert_eq!(indexed(208), (255, 135, 0));
        assert_eq!(indexed(232), (8, 8, 8));
        assert_eq!(indexed(255), (238, 238, 238));
    }

    #[test]
    fn reverse_swaps_foreground_and_background() {
        let style = Style { fg: Color::Indexed(1), bg: Color::Default, reverse: true, ..Style::default() };
        let (fg, bg) = Palette::Dark.resolve(&style);
        assert_eq!(hex(&fg), "#000000");
        assert_eq!(hex(&bg), "#cd0000");
    }

    #[test]
    fn light_palette_inverts_only_the_defaults() {
        let plain = Style::default();
        let (dfg, dbg) = Palette::Dark.resolve(&plain);
        let (lfg, lbg) = Palette::Light.resolve(&plain);
        assert_eq!((hex(&dfg), hex(&dbg)), ("#ffffff".to_string(), "#000000".to_string()));
        assert_eq!((hex(&lfg), hex(&lbg)), ("#000000".to_string(), "#ffffff".to_string()));
        let red = Style { fg: Color::Indexed(9), ..Style::default() };
        assert_eq!(hex(&Palette::Light.resolve(&red).0), hex(&Palette::Dark.resolve(&red).0));
        assert_eq!(Palette::parse("sepia"), None);
        assert_eq!(Palette::parse("light"), Some(Palette::Light));
    }
}
```

- [ ] **Step 4: Run the tests** (the first build fetches `unicode-width`; a fetch failure is a blocker to report, not to work around)

```bash
cargo test -p impeccable-terminal 2>&1 | tail -n 5
git status --short Cargo.lock    # modified: the new crate and unicode-width
```

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock crates/terminal
git commit -m "$(printf 'Add crates/terminal with the capture parser and the contrast palette\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 5: Runtime rules, part 1: `rules.rs` scaffold, contrast, truecolor, key hints, spinner

**Files:**
- Create: `crates/terminal/src/rules.rs`
- Modify: `crates/terminal/src/lib.rs` (`pub mod rules;`)

**Interfaces:**
- Consumes: Task 4's `capture` and `palette` modules; Task 2's registry ids.
- Produces: `rules::scan_frames(frames: &[Frame], palette: Palette, file: &str) -> Vec<Finding>` (Task 7 calls it), `rt_low_contrast`, `rt_truecolor_on_256`, `rt_no_key_hints`, `rt_spinner_never_rests`, the shared `rt_finding` helper, and the glyph-set constants Task 6 extends. Task 6 fills in the three geometry rules that this task stubs as returning `Vec::new()`.

- [ ] **Step 1: Write the failing tests.** Create `rules.rs` with the module skeleton and the tests first:

```rust
//! The runtime rules (spec section 4, PR 3 table): pure functions over the
//! frames of one target. Findings name the registry rows in
//! `crates/foundation` (`tui-rt-*`), all advisory.

use impeccable_core::color::contrast_ratio;
use impeccable_core::findings::{derive_advisory_flag, finding, Finding};
use serde_json::Value;

use crate::capture::{Color, Frame, FrameRole};
use crate::palette::{hex, Palette};

macro_rules! re {
    ($name:ident, $pat:expr) => {
        static $name: once_cell::sync::Lazy<regex::Regex> =
            once_cell::sync::Lazy::new(|| regex::Regex::new($pat).expect(stringify!($name)));
    };
}

/// WCAG AA for body text; the terminal has no large-text tier.
pub const CONTRAST_FLOOR: f64 = 4.5;
/// `adapt.terminal.md`'s Narrow class starts under 60 columns.
pub const NARROW_COLUMNS: usize = 60;
/// Fewer non-blank rows than this is a prompt, not a screen.
pub const MIN_ROWS_FOR_HINTS: usize = 3;

pub(crate) const TOP_LEFT: &str = "┌╭╔┏";
pub(crate) const TOP_RIGHT: &str = "┐╮╗┓";
pub(crate) const BOTTOM_LEFT: &str = "└╰╚┗";
pub(crate) const BOTTOM_RIGHT: &str = "┘╯╝┛";
pub(crate) const VERTICAL: &str = "│║┃┆┇┊┋╎╏";
pub(crate) const TEE: &str = "├┤┬┴┼╠╣╦╩╬┣┫┳┻╋╟╢╤╧╪";
/// Glyphs a spinner cycles through, beyond the braille block U+2800 to U+28FF.
const SPINNER_GLYPHS: &str = "|/-\\◐◓◑◒◴◵◶◷◜◝◞◟▁▂▃▄▅▆▇█▏▎▍▌▋▊▉";

re!(
    KEY_HINT_RE,
    r"(?i)(?:^|[\s:|/,(\[<])(?:q|esc|enter|tab|space|ctrl|alt|shift|f\d{1,2}|[hjkl])(?:$|[\s:|/,)\]>])|[?↑↓←→⏎⌃⌘]|<[^>\s]{1,12}>|\[[^\]\s]{1,12}\]|\^[A-Z]"
);

/// Every runtime rule over the frames of one target, in the frame policy
/// spec section 5 fixes: size-invariant rules on the first frame, the spinner
/// against the recapture, geometry on every capture frame.
pub fn scan_frames(frames: &[Frame], palette: Palette, file: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    let Some(first) = frames.first() else { return out };
    out.extend(rt_low_contrast(first, palette, file));
    out.extend(rt_truecolor_on_256(first, file));
    out.extend(rt_no_key_hints(first, file));
    if let Some(second) = frames.iter().find(|f| f.role == FrameRole::Recapture) {
        out.extend(rt_spinner_never_rests(first, second, file));
    }
    for frame in frames.iter().filter(|f| f.role == FrameRole::Capture) {
        out.extend(rt_nested_borders(frame, file));
        out.extend(rt_width_drift(frame, file));
        if frame.width < NARROW_COLUMNS {
            out.extend(rt_collapse_narrow(frame, file));
        }
    }
    for f in &mut out {
        derive_advisory_flag(f);
    }
    out
}

/// A finding at a cell: `line` is the 1-based row, `column` the 1-based
/// cell, `frame` the size it was seen at.
pub(crate) fn rt_finding(id: &str, file: &str, frame: &Frame, row: usize, col: usize, snippet: String) -> Finding {
    let mut f = finding(id, file, &snippet, (row + 1) as f64);
    f.extras.insert("column".into(), Value::from(col + 1));
    f.extras.insert("frame".into(), Value::String(format!("{}x{}", frame.width, frame.height)));
    f
}

fn is_text(ch: char) -> bool {
    ch.is_alphanumeric()
}

fn is_spinner(ch: char) -> bool {
    SPINNER_GLYPHS.contains(ch) || ('\u{2800}'..='\u{28FF}').contains(&ch)
}

pub fn rt_low_contrast(frame: &Frame, palette: Palette, file: &str) -> Vec<Finding> {
    todo!()
}

pub fn rt_truecolor_on_256(frame: &Frame, file: &str) -> Vec<Finding> {
    todo!()
}

pub fn rt_no_key_hints(frame: &Frame, file: &str) -> Vec<Finding> {
    todo!()
}

pub fn rt_spinner_never_rests(first: &Frame, second: &Frame, file: &str) -> Vec<Finding> {
    todo!()
}

// Task 6 replaces these three bodies.
pub fn rt_nested_borders(_frame: &Frame, _file: &str) -> Vec<Finding> {
    Vec::new()
}

pub fn rt_width_drift(_frame: &Frame, _file: &str) -> Vec<Finding> {
    Vec::new()
}

pub fn rt_collapse_narrow(_frame: &Frame, _file: &str) -> Vec<Finding> {
    Vec::new()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::capture::parse_rows;

    pub(crate) fn frame(width: usize, rows: &[&str]) -> Frame {
        Frame { width, height: rows.len(), rows: parse_rows(&rows.join("\n")), ..Frame::default() }
    }

    pub(crate) fn ids(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|f| f.antipattern.as_str()).collect()
    }

    pub(crate) const HINTS: &str = " q quit  ? help";

    #[test]
    fn gray_on_gray_text_is_low_contrast_and_borders_never_count() {
        let gray = "\x1b[38;2;120;120;120;48;2;100;100;100m";
        let f = frame(40, &[
            "┌──────┐",
            &format!("│ {gray}faint\x1b[0m │"),
            &format!("{gray}└──────┘\x1b[0m"),
            HINTS,
        ]);
        let out = rt_low_contrast(&f, Palette::Dark, "x");
        assert_eq!(out.len(), 1, "{:?}", ids(&out));
        assert_eq!(out[0].line, 2.0);
        assert_eq!(out[0].extras["column"], Value::from(3));
        assert_eq!(out[0].extras["cellCount"], Value::from(5));
        assert_eq!(out[0].extras["frame"], Value::from("40x4"));
        assert_eq!(out[0].snippet, "\"faint\" #787878 on #646464: 1.3:1 (need 4.5:1)");
    }

    #[test]
    fn black_text_fails_on_the_dark_default_and_passes_on_light() {
        let f = frame(40, &["title", "\x1b[30mblack on default\x1b[0m", HINTS]);
        let dark = rt_low_contrast(&f, Palette::Dark, "x");
        assert_eq!(dark.len(), 1);
        assert_eq!(dark[0].snippet, "\"black on default\" #000000 on #000000: 1.0:1 (need 4.5:1)");
        assert!(rt_low_contrast(&f, Palette::Light, "x").is_empty());
    }

    #[test]
    fn each_failing_pair_reports_once() {
        let f = frame(40, &["\x1b[30ma\x1b[0m \x1b[30mb\x1b[0m", "\x1b[30mc\x1b[0m", HINTS]);
        let out = rt_low_contrast(&f, Palette::Dark, "x");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].extras["cellCount"], Value::from(3));
    }

    #[test]
    fn truecolor_fires_only_when_the_client_is_known_to_lack_rgb() {
        let mut f = frame(40, &["\x1b[38;2;200;100;50mwarm\x1b[0m", "body", HINTS]);
        assert!(rt_truecolor_on_256(&f, "x").is_empty(), "unknown client stays silent");
        f.termfeatures = "256".into();
        let out = rt_truecolor_on_256(&f, "x");
        assert_eq!(out.len(), 1);
        assert_eq!((out[0].line, out[0].extras["column"].clone()), (1.0, Value::from(1)));
        assert_eq!(out[0].snippet, "truecolor SGR 38;2;200;100;50 while the client reports 256");
        f.termfeatures = "256,RGB".into();
        assert!(rt_truecolor_on_256(&f, "x").is_empty());
        f.termfeatures = "256".into();
        f.colorterm = "truecolor".into();
        assert!(rt_truecolor_on_256(&f, "x").is_empty(), "COLORTERM vouches for the outer terminal");
    }

    #[test]
    fn bottom_row_without_a_key_is_flagged_and_hints_or_prompts_pass() {
        let bare = frame(40, &["┌──┐", "│ab│", "└──┘", "Loading complete."]);
        let out = rt_no_key_hints(&bare, "x");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].line, 4.0);
        assert_eq!(out[0].snippet, "bottom row \"Loading complete.\" names no key");
        for hints in [" q quit  ? help", "[Enter] open  <Esc> back", "^C exit", "j/k move", "Press F1 for help"] {
            assert!(rt_no_key_hints(&frame(40, &["┌──┐", "│ab│", hints]), "x").is_empty(), "{hints}");
        }
        assert!(rt_no_key_hints(&frame(40, &["$ ", "Loading complete."]), "x").is_empty(), "a prompt is not a screen");
        assert!(rt_no_key_hints(&frame(40, &["┌──┐", "│ab│", "└──┘", HINTS, "", ""]), "x").is_empty(), "trailing blank rows are skipped");
    }

    #[test]
    fn spinner_cycling_across_the_recapture_is_flagged_once() {
        let a = frame(40, &["working ⠋  ⠙", "│ x │", HINTS]);
        let mut b = frame(40, &["working ⠙  ⠹", "│ x │", HINTS]);
        b.role = FrameRole::Recapture;
        let out = rt_spinner_never_rests(&a, &b, "x");
        assert_eq!(out.len(), 1, "one finding per pair, at the first cycling cell");
        assert_eq!((out[0].line, out[0].extras["column"].clone()), (1.0, Value::from(9)));
        assert_eq!(out[0].snippet, "spinner glyph ⠋ became ⠙ after one second with no input");
        assert!(rt_spinner_never_rests(&a, &a, "x").is_empty(), "a resting glyph is fine");
        let c = frame(40, &["working ✓  ✓", "│ x │", HINTS]);
        assert!(rt_spinner_never_rests(&a, &c, "x").is_empty(), "a spinner that resolved is fine");
    }
}
```

Add `pub mod rules;` to `lib.rs`.

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test -p impeccable-terminal rules 2>&1 | tail -n 5
```

Expected: the four tests panic on `todo!()`.

- [ ] **Step 3: Implement the four rules** by replacing the four `todo!()` bodies:

```rust
pub fn rt_low_contrast(frame: &Frame, palette: Palette, file: &str) -> Vec<Finding> {
    // One finding per distinct (fg, bg) pair, at the first text cell that fails.
    let mut seen: Vec<(String, String)> = Vec::new();
    let mut out = Vec::new();
    for (r, row) in frame.rows.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            let Some(ch) = cell.glyph else { continue };
            if !is_text(ch) {
                continue;
            }
            let (fg, bg) = palette.resolve(&cell.style);
            let ratio = contrast_ratio(&fg, &bg);
            if ratio >= CONTRAST_FLOOR {
                continue;
            }
            let key = (hex(&fg), hex(&bg));
            if seen.contains(&key) {
                continue;
            }
            seen.push(key.clone());
            let run: String = row[c..]
                .iter()
                .take_while(|x| x.style == cell.style)
                .filter_map(|x| x.glyph)
                .take(24)
                .collect();
            let count = frame
                .rows
                .iter()
                .flatten()
                .filter(|x| x.glyph.map(is_text).unwrap_or(false) && palette.resolve(&x.style) == (fg, bg))
                .count();
            let snippet = format!("\"{run}\" {} on {}: {ratio:.1}:1 (need 4.5:1)", key.0, key.1);
            let mut f = rt_finding("tui-rt-low-contrast", file, frame, r, c, snippet);
            f.extras.insert("cellCount".into(), Value::from(count));
            out.push(f);
        }
    }
    out
}

pub fn rt_truecolor_on_256(frame: &Frame, file: &str) -> Vec<Finding> {
    if frame.termfeatures.is_empty() {
        return Vec::new(); // no client reported its features
    }
    let client_has_rgb = frame.termfeatures.split(',').any(|f| f.trim().eq_ignore_ascii_case("RGB"))
        || matches!(frame.colorterm.as_str(), "truecolor" | "24bit");
    if client_has_rgb {
        return Vec::new();
    }
    for (r, row) in frame.rows.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            let sgr = match (cell.style.fg, cell.style.bg) {
                (Color::Rgb(red, green, blue), _) => format!("38;2;{red};{green};{blue}"),
                (_, Color::Rgb(red, green, blue)) => format!("48;2;{red};{green};{blue}"),
                _ => continue,
            };
            let snippet = format!("truecolor SGR {sgr} while the client reports {}", frame.termfeatures);
            return vec![rt_finding("tui-rt-truecolor-on-256", file, frame, r, c, snippet)];
        }
    }
    Vec::new()
}

pub fn rt_no_key_hints(frame: &Frame, file: &str) -> Vec<Finding> {
    let non_blank: Vec<usize> = (0..frame.rows.len()).filter(|&r| !frame.is_blank(r)).collect();
    if non_blank.len() < MIN_ROWS_FOR_HINTS {
        return Vec::new();
    }
    let r = *non_blank.last().expect("checked above");
    let text = frame.text(r);
    if KEY_HINT_RE.is_match(&text) {
        return Vec::new();
    }
    let shown: String = text.trim().chars().take(60).collect();
    vec![rt_finding("tui-rt-no-key-hints", file, frame, r, 0, format!("bottom row \"{shown}\" names no key"))]
}

pub fn rt_spinner_never_rests(first: &Frame, second: &Frame, file: &str) -> Vec<Finding> {
    for (r, row) in first.rows.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            let (Some(a), Some(b)) = (cell.glyph, second.glyph(r, c)) else { continue };
            if a != b && is_spinner(a) && is_spinner(b) {
                let snippet = format!("spinner glyph {a} became {b} after one second with no input");
                return vec![rt_finding("tui-rt-spinner-never-rests", file, first, r, c, snippet)];
            }
        }
    }
    Vec::new()
}
```

- [ ] **Step 4: Run the tests**

```bash
cargo test -p impeccable-terminal 2>&1 | tail -n 3
```

If the contrast snippet's `1.3:1` differs in the last digit, the palette or the luminance path is wrong, not the test: (120,120,120) on (100,100,100) is 1.34:1 under WCAG.

- [ ] **Step 5: Commit**

```bash
git add crates/terminal/src/lib.rs crates/terminal/src/rules.rs
git commit -m "$(printf 'Add the contrast, truecolor, key-hint, and spinner runtime rules\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 6: Runtime rules, part 2: nested borders, width drift, narrow collapse

**Files:**
- Modify: `crates/terminal/src/rules.rs` (replace the three stub bodies; add `Rect`, `find_boxes`, `last_vertical`, `mode_of`; add tests)

**Interfaces:**
- Consumes: Task 5's scaffold, constants, `rt_finding`, and test helpers.
- Produces: `rt_nested_borders`, `rt_width_drift`, `rt_collapse_narrow` with the section 4 signals; `find_boxes(&Frame) -> Vec<Rect>` (public, used by tests).

- [ ] **Step 1: Write the failing tests** inside `mod tests`:

```rust
    #[test]
    fn box_directly_inside_a_box_is_flagged_and_content_between_is_not() {
        let nested = frame(40, &[
            "┌──────────┐",
            "│┌────────┐│",
            "││ inner  ││",
            "│└────────┘│",
            "└──────────┘",
            HINTS,
        ]);
        let out = rt_nested_borders(&nested, "x");
        assert_eq!(out.len(), 1, "{:?}", ids(&out));
        assert_eq!((out[0].line, out[0].extras["column"].clone()), (2.0, Value::from(2)));
        assert_eq!(out[0].snippet, "10x3 box drawn directly inside a 12x5 box");
        let spaced = frame(40, &[
            "┌ Title ────┐",
            "│ label     │",
            "│ ┌───────┐ │",
            "│ │ inner │ │",
            "│ └───────┘ │",
            "└───────────┘",
            HINTS,
        ]);
        assert!(rt_nested_borders(&spaced, "x").is_empty(), "text between the borders is content");
        let siblings = frame(40, &["┌──┐ ┌──┐", "│a │ │b │", "└──┘ └──┘", HINTS]);
        assert!(rt_nested_borders(&siblings, "x").is_empty());
        assert_eq!(find_boxes(&siblings).len(), 2);
        let split = frame(40, &["┌────┬────┐", "│ a  │ b  │", "└────┴────┘", HINTS]);
        assert_eq!(find_boxes(&split).len(), 1, "a divider with tees is one box");
    }

    #[test]
    fn emoji_row_wider_than_the_pane_is_width_drift_and_a_shifted_border_too() {
        let f = frame(12, &[
            "┌──────────┐",
            "│ plain    │",
            "│ 🚀 launch │",   // padded as if the rocket were one cell: 13 cells
            "│ 日本 tok│",      // padded as if 日本 were six cells: 11 cells
            "└──────────┘",
            HINTS,
        ]);
        let out = rt_width_drift(&f, "x");
        assert_eq!(ids(&out), vec!["tui-rt-width-drift", "tui-rt-width-drift"]);
        assert_eq!((out[0].line, out[0].extras["column"].clone()), (3.0, Value::from(13)));
        assert_eq!(out[0].snippet, "row measures 13 cells on a 12-column pane");
        assert_eq!((out[1].line, out[1].extras["column"].clone()), (4.0, Value::from(11)));
        assert_eq!(out[1].snippet, "border ends at column 11 while other rows end at 12");
    }

    #[test]
    fn ascii_rows_never_trip_width_drift() {
        let f = frame(12, &["┌──────────┐", "│ plain    │", "│ overflow row │", "└──────────┘", HINTS]);
        assert!(rt_width_drift(&f, "x").is_empty());
    }

    #[test]
    fn narrow_frame_overflow_lost_corner_and_split_word_are_collapse() {
        let f = frame(40, &[
            &format!("┌{}┐", "─".repeat(48)),
            "┌────────",
            "Processing the selected directory record",
            "s to the index",
            "q quit",
        ]);
        let out = rt_collapse_narrow(&f, "x");
        assert_eq!(ids(&out), vec!["tui-rt-collapse-narrow"; 3]);
        assert_eq!(out[0].snippet, "row overflows the 40-column pane by 10 cells");
        assert_eq!(out[1].snippet, "border loses its right corner");
        assert_eq!(out[2].snippet, "word split at the pane edge: \"tory record\"");
        assert_eq!(out[2].line, 3.0);
    }

    #[test]
    fn geometry_rules_skip_the_recapture_and_size_rules_read_the_first_frame() {
        let first = frame(40, &["┌──┐", "│ab│", "└──┘", HINTS]);
        let mut recap = frame(40, &["┌────┐", "│┌──┐│", "││ab││", "│└──┘│", "└────┘", HINTS]);
        recap.role = FrameRole::Recapture;
        let narrow = frame(20, &["┌──────────────────────┐", "text", HINTS]);
        let out = scan_frames(&[first, recap, narrow], Palette::Dark, "tmux:app:0.0");
        let found = ids(&out);
        assert!(!found.contains(&"tui-rt-nested-borders"), "the recapture is never scanned for geometry: {found:?}");
        assert!(found.contains(&"tui-rt-collapse-narrow"), "{found:?}");
        assert!(
            out.iter().filter(|f| f.antipattern == "tui-rt-collapse-narrow").all(|f| f.extras["frame"] == Value::from("20x3")),
            "collapse only on the narrow frame"
        );
        assert!(out.iter().all(|f| f.advisory == Some(true) && f.file == "tmux:app:0.0"));
        let wide = frame(80, &["┌──────────────────────────────────────────────────────────────────────────────────────┐", "text", HINTS]);
        assert!(!ids(&scan_frames(&[wide], Palette::Dark, "x")).contains(&"tui-rt-collapse-narrow"), "80 columns is not narrow");
    }
```

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test -p impeccable-terminal rules 2>&1 | tail -n 8
```

- [ ] **Step 3: Implement.** Replace the three stubs and add the helpers above `#[cfg(test)]`:

```rust
/// A closed box-drawing rectangle, inclusive cell coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub top: usize,
    pub left: usize,
    pub bottom: usize,
    pub right: usize,
}

fn glyph_in(frame: &Frame, r: usize, c: usize, set: &str) -> bool {
    frame.glyph(r, c).map(|g| set.contains(g)).unwrap_or(false)
}

fn is_edge_vertical(frame: &Frame, r: usize, c: usize) -> bool {
    glyph_in(frame, r, c, VERTICAL) || glyph_in(frame, r, c, TEE)
}

/// Every closed rectangle: a top-left corner, horizontals or a title to the
/// first top-right corner, verticals (or tees) down both sides to matching
/// bottom corners.
pub fn find_boxes(frame: &Frame) -> Vec<Rect> {
    let mut out = Vec::new();
    for (top, row) in frame.rows.iter().enumerate() {
        for (left, cell) in row.iter().enumerate() {
            if !cell.glyph.map(|g| TOP_LEFT.contains(g)).unwrap_or(false) {
                continue;
            }
            let Some(right) = (left + 1..row.len()).find(|&c| glyph_in(frame, top, c, TOP_RIGHT)) else { continue };
            let top_edge_broken = (left + 1..right).any(|c| {
                glyph_in(frame, top, c, VERTICAL)
                    || glyph_in(frame, top, c, TOP_LEFT)
                    || glyph_in(frame, top, c, BOTTOM_LEFT)
                    || glyph_in(frame, top, c, BOTTOM_RIGHT)
            });
            if top_edge_broken {
                continue;
            }
            let Some(bottom) = (top + 1..frame.rows.len()).find(|&r| glyph_in(frame, r, left, BOTTOM_LEFT)) else { continue };
            if !glyph_in(frame, bottom, right, BOTTOM_RIGHT) {
                continue;
            }
            let sides_closed = (top + 1..bottom).all(|r| is_edge_vertical(frame, r, left) && is_edge_vertical(frame, r, right));
            if sides_closed {
                out.push(Rect { top, left, bottom, right });
            }
        }
    }
    out
}

pub fn rt_nested_borders(frame: &Frame, file: &str) -> Vec<Finding> {
    let boxes = find_boxes(frame);
    let mut out = Vec::new();
    for inner in &boxes {
        let enclosing = boxes
            .iter()
            .filter(|o| o.top < inner.top && o.left < inner.left && o.right > inner.right && o.bottom > inner.bottom)
            .max_by_key(|o| o.top);
        let Some(outer) = enclosing else { continue };
        // Immediate nesting: the ring between the two borders holds nothing.
        let ring_blank = (outer.top + 1..outer.bottom).all(|r| {
            (outer.left + 1..outer.right).all(|c| {
                let inside_inner = r >= inner.top && r <= inner.bottom && c >= inner.left && c <= inner.right;
                inside_inner || frame.glyph(r, c).map(char::is_whitespace).unwrap_or(true)
            })
        });
        if !ring_blank {
            continue;
        }
        let snippet = format!(
            "{}x{} box drawn directly inside a {}x{} box",
            inner.right - inner.left + 1,
            inner.bottom - inner.top + 1,
            outer.right - outer.left + 1,
            outer.bottom - outer.top + 1
        );
        out.push(rt_finding("tui-rt-nested-borders", file, frame, inner.top, inner.left, snippet));
    }
    out
}

/// The cell index of the row's last vertical border or right corner.
fn last_vertical(frame: &Frame, r: usize) -> Option<usize> {
    frame.rows.get(r)?.iter().rposition(|c| {
        c.glyph.map(|g| VERTICAL.contains(g) || TOP_RIGHT.contains(g) || BOTTOM_RIGHT.contains(g)).unwrap_or(false)
    })
}

/// The most common value, when at least two rows agree; ties go to the larger.
fn mode_of(values: &[usize]) -> Option<usize> {
    let mut best: Option<(usize, usize)> = None;
    for &v in values {
        let count = values.iter().filter(|&&x| x == v).count();
        if best.map(|(bv, bc)| count > bc || (count == bc && v > bv)).unwrap_or(true) {
            best = Some((v, count));
        }
    }
    best.filter(|&(_, count)| count >= 2).map(|(v, _)| v)
}

pub fn rt_width_drift(frame: &Frame, file: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    let ends: Vec<usize> = (0..frame.rows.len()).filter_map(|r| last_vertical(frame, r)).collect();
    let usual_end = mode_of(&ends);
    for r in 0..frame.rows.len() {
        if !frame.has_wide(r) {
            continue;
        }
        let w = frame.row_width(r);
        if frame.width > 0 && w > frame.width {
            let snippet = format!("row measures {w} cells on a {}-column pane", frame.width);
            out.push(rt_finding("tui-rt-width-drift", file, frame, r, frame.width, snippet));
            continue;
        }
        if let (Some(end), Some(usual)) = (last_vertical(frame, r), usual_end) {
            if end != usual {
                let snippet = format!("border ends at column {} while other rows end at {}", end + 1, usual + 1);
                out.push(rt_finding("tui-rt-width-drift", file, frame, r, end, snippet));
            }
        }
    }
    out
}

pub fn rt_collapse_narrow(frame: &Frame, file: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    for r in 0..frame.rows.len() {
        let row = &frame.rows[r];
        if row.is_empty() {
            continue;
        }
        let w = row.len();
        if frame.width > 0 && w > frame.width {
            let snippet = format!("row overflows the {}-column pane by {} cells", frame.width, w - frame.width);
            out.push(rt_finding("tui-rt-collapse-narrow", file, frame, r, frame.width, snippet));
            continue;
        }
        let text = frame.text(r);
        let has_left = text.chars().any(|g| TOP_LEFT.contains(g) || BOTTOM_LEFT.contains(g));
        let has_right = text.chars().any(|g| TOP_RIGHT.contains(g) || BOTTOM_RIGHT.contains(g));
        if has_left && !has_right {
            out.push(rt_finding("tui-rt-collapse-narrow", file, frame, r, w - 1, "border loses its right corner".to_string()));
            continue;
        }
        // A word split at the right edge: the row fills the pane, ends in a
        // letter, and the next row starts with one.
        let ends_in_letter = row.last().and_then(|c| c.glyph).map(char::is_alphabetic).unwrap_or(false);
        let next_starts_with_letter = frame
            .rows
            .get(r + 1)
            .and_then(|n| n.first())
            .and_then(|c| c.glyph)
            .map(char::is_alphabetic)
            .unwrap_or(false);
        if w == frame.width && ends_in_letter && next_starts_with_letter {
            let tail: String = text.chars().rev().take(11).collect::<Vec<_>>().into_iter().rev().collect();
            let snippet = format!("word split at the pane edge: \"{}\"", tail.trim());
            out.push(rt_finding("tui-rt-collapse-narrow", file, frame, r, w - 1, snippet));
        }
    }
    out
}
```

- [ ] **Step 4: Run the tests**

```bash
cargo test -p impeccable-terminal 2>&1 | tail -n 3
```

If `emoji_row_wider_than_the_pane...` fails on the counts, check the fixture rows cell by cell with `parse_row` before touching the rule: the rocket row must be 13 cells and the CJK row 11.

- [ ] **Step 5: Commit**

```bash
git add crates/terminal/src/rules.rs
git commit -m "$(printf 'Add the nested-border, width-drift, and narrow-collapse runtime rules\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 7: The tmux driver, the `TmuxEngine` impl, and the binary wiring

**Files:**
- Create: `crates/terminal/src/tmux.rs`
- Modify: `crates/terminal/src/lib.rs` (`TerminalEngine`, `frame_from`, `capture_sizes`, the impl)
- Modify: `crates/cli/Cargo.toml`, `crates/cli/src/main.rs`

**Interfaces:**
- Consumes: Task 3's `TmuxEngine` trait and `ScanOptions` fields; Tasks 4 to 6.
- Produces: `impeccable_terminal::TerminalEngine::{new, from_process_env}`; `tmux::{find_tmux, parse_version, Tmux, PaneInfo, NOT_FOUND_MESSAGE, MIN_VERSION}`; the binary's `--tmux` works end to end (Task 9 tests it against a scratch server).

- [ ] **Step 1: Write `tmux.rs` with its tests**:

```rust
//! The tmux side of the engine: find the executable, check its version, read
//! pane geometry, capture, resize, restore (spec section 5, "Requirements and
//! degradation" and "Multi-size pass").

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const NOT_FOUND_MESSAGE: &str = "tmux 3.2 or newer is required for --tmux and was not found on PATH. Install tmux, or point IMPECCABLE_TMUX at the executable.";
/// `resize-window -x -y` and `client_termfeatures` both arrived in 3.2.
pub const MIN_VERSION: (u32, u32) = (3, 2);

/// `IMPECCABLE_TMUX` (a path, reported when it does not exist), else the first
/// `tmux` on `PATH`.
pub fn find_tmux(env: &HashMap<String, String>) -> Result<PathBuf, String> {
    if let Some(raw) = env.get("IMPECCABLE_TMUX").map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let path = PathBuf::from(raw);
        return if path.is_file() {
            Ok(path)
        } else {
            Err(format!("tmux was not found at the configured path ({raw}) from IMPECCABLE_TMUX"))
        };
    }
    let path_var = env.get("PATH").cloned().unwrap_or_default();
    let separator = if cfg!(windows) { ';' } else { ':' };
    let name = if cfg!(windows) { "tmux.exe" } else { "tmux" };
    for dir in path_var.split(separator).filter(|d| !d.is_empty()) {
        let candidate = Path::new(dir).join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(NOT_FOUND_MESSAGE.to_string())
}

/// `tmux -V` into `(major, minor)`: "tmux 3.7c" is (3, 7), "tmux next-3.8" is (3, 8).
pub fn parse_version(output: &str) -> Option<(u32, u32)> {
    let digits: String = output
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let mut parts = digits.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().ok()?;
    Some((major, minor))
}

/// What `display-message` reports about the target before the first capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneInfo {
    pub width: usize,
    pub height: usize,
    pub window_width: usize,
    pub window_height: usize,
    /// `#{client_termfeatures}`; empty when no client is attached.
    pub termfeatures: String,
}

/// A located, version-checked tmux executable and the environment it runs
/// with (so `$TMUX` reaches it and a `PATH` override is honored).
pub struct Tmux {
    exe: PathBuf,
    env: HashMap<String, String>,
}

impl Tmux {
    pub fn locate(env: &HashMap<String, String>) -> Result<Tmux, String> {
        let exe = find_tmux(env)?;
        let tmux = Tmux { exe, env: env.clone() };
        let version = tmux.run(&["-V"])?;
        match parse_version(&version) {
            Some(v) if v >= MIN_VERSION => Ok(tmux),
            _ => Err(format!("tmux 3.2 or newer is required for --tmux; found {}", version.trim())),
        }
    }

    /// Run tmux with `args`: stdout on success, `tmux <verb>: <first stderr line>` otherwise.
    pub fn run(&self, args: &[&str]) -> Result<String, String> {
        let output = Command::new(&self.exe)
            .args(args)
            .env_clear()
            .envs(&self.env)
            .stdin(Stdio::null())
            .output()
            .map_err(|e| format!("could not run {}: {e}", self.exe.display()))?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!(
            "tmux {}: {}",
            args.first().copied().unwrap_or(""),
            stderr.lines().next().unwrap_or("").trim()
        ))
    }

    pub fn pane_info(&self, target: &str) -> Result<PaneInfo, String> {
        let out = self.run(&[
            "display-message",
            "-p",
            "-t",
            target,
            "#{pane_width}\t#{pane_height}\t#{window_width}\t#{window_height}\t#{client_termfeatures}",
        ])?;
        let line = out.trim_end_matches('\n');
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 5 {
            return Err(format!("tmux display-message: unexpected output {line:?}"));
        }
        let number = |s: &str| s.trim().parse::<usize>().map_err(|_| format!("tmux display-message: {s:?} is not a number"));
        Ok(PaneInfo {
            width: number(parts[0])?,
            height: number(parts[1])?,
            window_width: number(parts[2])?,
            window_height: number(parts[3])?,
            termfeatures: parts[4].trim().to_string(),
        })
    }

    /// The visible grid with SGR sequences kept and wrapped lines joined.
    pub fn capture(&self, target: &str) -> Result<String, String> {
        self.run(&["capture-pane", "-p", "-e", "-J", "-t", target])
    }

    pub fn resize(&self, target: &str, width: usize, height: usize) -> Result<(), String> {
        self.run(&["resize-window", "-t", target, "-x", &width.to_string(), "-y", &height.to_string()]).map(|_| ())
    }

    /// Back to the recorded size, then drop the `manual` window-size that
    /// `resize-window` set so the window follows its clients again.
    pub fn restore(&self, target: &str, width: usize, height: usize) -> Result<(), String> {
        self.resize(target, width, height)?;
        self.run(&["set-option", "-w", "-t", target, "-u", "window-size"]).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version_reads_release_and_next_builds() {
        assert_eq!(parse_version("tmux 3.7c\n"), Some((3, 7)));
        assert_eq!(parse_version("tmux 3.2a"), Some((3, 2)));
        assert_eq!(parse_version("tmux next-3.8"), Some((3, 8)));
        assert_eq!(parse_version("tmux master"), None);
        assert!((3, 1) < MIN_VERSION && (3, 2) >= MIN_VERSION);
    }

    #[cfg(unix)]
    #[test]
    fn missing_tmux_names_the_requirement_and_the_override_wins() {
        let mut env = HashMap::new();
        env.insert("PATH".to_string(), "/nonexistent".to_string());
        assert_eq!(find_tmux(&env).unwrap_err(), NOT_FOUND_MESSAGE);
        env.insert("IMPECCABLE_TMUX".to_string(), "/nonexistent/tmux".to_string());
        assert_eq!(
            find_tmux(&env).unwrap_err(),
            "tmux was not found at the configured path (/nonexistent/tmux) from IMPECCABLE_TMUX"
        );
        let dir = std::env::temp_dir().join(format!("impeccable-tmux-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let fake = dir.join("tmux");
        std::fs::write(&fake, "#!/bin/sh\necho tmux 3.7c\n").unwrap();
        env.remove("IMPECCABLE_TMUX");
        env.insert("PATH".to_string(), dir.to_string_lossy().into_owned());
        assert_eq!(find_tmux(&env).unwrap(), fake);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn an_old_tmux_is_refused_with_its_version_in_the_message() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("impeccable-tmux-old-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let fake = dir.join("tmux");
        std::fs::write(&fake, "#!/bin/sh\necho tmux 3.1b\n").unwrap();
        std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut env = HashMap::new();
        env.insert("IMPECCABLE_TMUX".to_string(), fake.to_string_lossy().into_owned());
        env.insert("PATH".to_string(), "/usr/bin:/bin".to_string());
        assert_eq!(Tmux::locate(&env).unwrap_err(), "tmux 3.2 or newer is required for --tmux; found tmux 3.1b");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
```

- [ ] **Step 2: Finish `lib.rs`.** Replace its contents with:

```rust
//! impeccable-terminal: the tmux engine of `impeccable detect` (spec section
//! 5). Captures a running tmux pane, or replays a saved capture, parses it
//! into frames, and runs the `tui-rt-` runtime rules. Wired into the binary
//! through [`impeccable_detect::engines::TmuxEngine`]; the engine never
//! launches or kills a process and always restores a window size it changed.

pub mod capture;
pub mod palette;
pub mod rules;
pub mod tmux;

use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

use impeccable_core::findings::Finding;
use impeccable_detect::engines::{EngineError, ScanOptions, TmuxEngine};

use capture::{parse_frames, parse_rows, Frame, FrameRole};
use palette::Palette;
use tmux::{PaneInfo, Tmux};

/// The gap between the first capture and the recapture.
pub const RECAPTURE_DELAY_MS: u64 = 1000;
/// `--tmux-settle` when the flag is absent.
pub const DEFAULT_SETTLE_MS: u64 = 300;

/// The tmux engine, holding the process environment it reads
/// (`IMPECCABLE_TMUX`, `PATH`, `TMUX`, `COLORTERM`).
pub struct TerminalEngine {
    env: HashMap<String, String>,
}

impl TerminalEngine {
    pub fn new(env: HashMap<String, String>) -> Self {
        TerminalEngine { env }
    }

    pub fn from_process_env() -> Self {
        TerminalEngine::new(std::env::vars().collect())
    }

    fn colorterm(&self) -> String {
        self.env.get("COLORTERM").cloned().unwrap_or_default()
    }
}

fn palette_for(options: &ScanOptions) -> Palette {
    options.palette.as_deref().and_then(Palette::parse).unwrap_or_default()
}

/// A frame from a live capture, sized by the pane rather than by a header.
fn frame_from(text: &str, info: &PaneInfo, colorterm: &str, role: FrameRole) -> Frame {
    Frame {
        width: info.width,
        height: info.height,
        termfeatures: info.termfeatures.clone(),
        colorterm: colorterm.to_string(),
        role,
        rows: parse_rows(text),
    }
}

/// The `--tmux-sizes` pass: resize, settle, re-read the geometry, capture.
/// The caller restores the window whatever this returns.
fn capture_sizes(tmux: &Tmux, target: &str, options: &ScanOptions, colorterm: &str, frames: &mut Vec<Frame>) -> Result<(), String> {
    let settle = Duration::from_millis(options.tmux_settle_ms.unwrap_or(DEFAULT_SETTLE_MS));
    for &(w, h) in &options.tmux_sizes {
        tmux.resize(target, w as usize, h as usize)?;
        sleep(settle);
        let info = tmux.pane_info(target)?;
        let text = tmux.capture(target)?;
        frames.push(frame_from(&text, &info, colorterm, FrameRole::Capture));
    }
    Ok(())
}

impl TmuxEngine for TerminalEngine {
    fn detect_pane(&self, target: &str, options: &ScanOptions) -> Result<Vec<Finding>, EngineError> {
        let tmux = Tmux::locate(&self.env).map_err(EngineError::new)?;
        let info = tmux.pane_info(target).map_err(EngineError::new)?;
        let colorterm = self.colorterm();
        let first = tmux.capture(target).map_err(EngineError::new)?;
        let mut frames = vec![frame_from(&first, &info, &colorterm, FrameRole::Capture)];
        sleep(Duration::from_millis(RECAPTURE_DELAY_MS));
        let second = tmux.capture(target).map_err(EngineError::new)?;
        frames.push(frame_from(&second, &info, &colorterm, FrameRole::Recapture));
        if !options.tmux_sizes.is_empty() {
            let captured = capture_sizes(&tmux, target, options, &colorterm, &mut frames);
            let restored = tmux.restore(target, info.window_width, info.window_height);
            captured.map_err(EngineError::new)?;
            restored.map_err(EngineError::new)?;
        }
        Ok(rules::scan_frames(&frames, palette_for(options), &format!("tmux:{target}")))
    }

    fn detect_capture(&self, path: &str, options: &ScanOptions) -> Result<Vec<Finding>, EngineError> {
        let text = std::fs::read_to_string(path).map_err(|e| {
            EngineError::new(match e.kind() {
                std::io::ErrorKind::NotFound => format!("ENOENT: no such file or directory, open '{path}'"),
                std::io::ErrorKind::PermissionDenied => format!("EACCES: permission denied, open '{path}'"),
                _ => format!("{e}"),
            })
        })?;
        let frames = parse_frames(&text).map_err(EngineError::new)?;
        Ok(rules::scan_frames(&frames, palette_for(options), path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_frames_carry_the_pane_geometry_and_client_features() {
        let info = PaneInfo { width: 60, height: 8, window_width: 60, window_height: 8, termfeatures: "256,RGB".into() };
        let f = frame_from("\x1b[31mhi\x1b[0m\n\n", &info, "truecolor", FrameRole::Recapture);
        assert_eq!((f.width, f.height, f.rows.len()), (60, 8, 2));
        assert_eq!(f.termfeatures, "256,RGB");
        assert_eq!(f.colorterm, "truecolor");
        assert_eq!(f.role, FrameRole::Recapture);
    }

    #[test]
    fn palette_flag_falls_back_to_dark() {
        assert_eq!(palette_for(&ScanOptions::default()), Palette::Dark);
        assert_eq!(palette_for(&ScanOptions { palette: Some("light".into()), ..ScanOptions::default() }), Palette::Light);
    }

    #[test]
    fn saved_capture_replays_through_the_engine() {
        let dir = std::env::temp_dir().join(format!("impeccable-terminal-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("bare.txt");
        std::fs::write(&file, "#!capture width=40 height=4\n┌──┐\n│ab│\n└──┘\nLoading complete.\n").unwrap();
        let engine = TerminalEngine::new(HashMap::new());
        let out = engine.detect_capture(&file.to_string_lossy(), &ScanOptions::default()).unwrap();
        assert_eq!(out.iter().map(|f| f.antipattern.as_str()).collect::<Vec<_>>(), vec!["tui-rt-no-key-hints"]);
        assert_eq!(out[0].file, file.to_string_lossy());
        let missing = engine.detect_capture("/nonexistent/x.txt", &ScanOptions::default()).unwrap_err();
        assert_eq!(missing.message, "ENOENT: no such file or directory, open '/nonexistent/x.txt'");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
```

- [ ] **Step 3: Wire the binary.** In `crates/cli/Cargo.toml` add `impeccable-terminal = { path = "../terminal" }` after the `impeccable-browser` line. In `crates/cli/src/main.rs` replace `tmux: None,` with `tmux: Some(tmux_engine()),`, extend the `engines()` doc comment to name the tmux engine (crates/terminal), and add after the browser-engine block:

```rust
// --- tmux engine (crates/terminal) ----------------------------------------
/// The tmux engine behind `detect --tmux`, built once from the process
/// environment (`IMPECCABLE_TMUX`, `PATH`, `TMUX`, `COLORTERM`).
fn tmux_engine() -> &'static impeccable_terminal::TerminalEngine {
    static ENGINE: std::sync::OnceLock<impeccable_terminal::TerminalEngine> =
        std::sync::OnceLock::new();
    ENGINE.get_or_init(impeccable_terminal::TerminalEngine::from_process_env)
}
// -------------------------------------------------------------------------
```

- [ ] **Step 4: Tests and a live smoke** against a scratch server (this is the first end-to-end run; it is not the integration test, which Task 9 writes):

```bash
cargo test -p impeccable-terminal 2>&1 | tail -n 3
cargo build --release -p impeccable 2>&1 | tail -n 1
S=impeccable-smoke; tmux -L $S kill-server 2>/dev/null
tmux -L $S new-session -d -x 60 -y 8 -s smoke 'printf "┌────┐\n│┌──┐│\n││ab││\n│└──┘│\n└────┘\nLoading complete."; sleep 60'
sleep 0.5
W=$(mktemp -d)/tmux-wrap; printf '#!/bin/sh\nexec tmux -L %s "$@"\n' "$S" > "$W"; chmod +x "$W"
IMPECCABLE_TMUX="$W" ./target/release/impeccable detect --no-config --tmux smoke:0.0 --tmux-sizes 40x6; echo "exit=$?"
tmux -L $S display -p -t smoke:0.0 '#{window_width}x#{window_height} #{window-size}'   # 60x8 latest
IMPECCABLE_TMUX="$W" ./target/release/impeccable detect --no-config --tmux nope:0.0; echo "exit=$?"
PATH=/nonexistent ./target/release/impeccable detect --no-config --tmux smoke:0.0; echo "exit=$?"
tmux -L $S kill-server
```

Expected: the first run lists `tui-rt-nested-borders` and `tui-rt-no-key-hints` under the Advisory section and exits 0, taking a little over a second (the recapture); the display line reads `60x8 latest`; the second run prints `Error: tmux list-panes: can't find session: nope` and exits 1; the third prints the `NOT_FOUND_MESSAGE` line with `Error: ` in front and exits 1.

- [ ] **Step 5: Commit**

```bash
git add crates/terminal crates/cli/Cargo.toml crates/cli/src/main.rs Cargo.lock
git commit -m "$(printf 'Drive tmux from the terminal engine and wire it into the binary\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 8: Capture fixtures, oracle cases, goldens

**Files:**
- Create: `tests/fixtures/terminal-captures/{low-contrast,nested-borders,truecolor-on-256,width-drift,no-key-hints,collapse-narrow,spinner-never-rests,sizes-pass}.txt`
- Modify: `tests/oracle/cases/detect.mjs` (new cases after the terminal cases, before `// Flag surface and errors`)
- Modify: `tests/oracle/DELTAS.md` (new section)
- Record: `tests/oracle/golden/detect-tmux-*.json`, `detect-help.json`

**Interfaces:**
- Consumes: the binary from Task 7 with every flag.
- Produces: the fixture files Task 10 documents and Task 11 reuses as the `--tmux-capture` example.

- [ ] **Step 1: Create the fixtures.** They are hand-written inputs, not goldens, so a script creates them once and is not kept. Run from the repo root:

```bash
node - <<'JS'
const fs = require('fs');
const dir = 'tests/fixtures/terminal-captures';
fs.mkdirSync(dir, { recursive: true });
const E = '\x1b[';
const hdr = (w, h, extra = '') => `#!capture width=${w} height=${h}${extra ? ' ' + extra : ''}`;
const bar = (w, l, r) => l + '─'.repeat(w - 2) + r;
const seg = (text, sgr) => ({ text, sgr, cells: [...text].length });
// A bordered row: visible width is counted in code points (the fixtures below
// only put ASCII inside `line`; the wide-glyph rows are written by hand).
const line = (w, segs) => {
  const visible = segs.reduce((n, s) => n + s.cells, 0);
  return '│' + segs.map((s) => (s.sgr ? E + s.sgr + 'm' + s.text + E + '0m' : s.text)).join('') + ' '.repeat(Math.max(0, w - 2 - visible)) + '│';
};
const HINTS = ' q quit  ? help';
const write = (name, frames) => fs.writeFileSync(`${dir}/${name}.txt`, frames.map(([h, rows]) => [h, ...rows].join('\n') + '\n').join(''));

write('low-contrast', [[hdr(40, 6, 'termfeatures=256,RGB colorterm=truecolor'), [
  bar(40, '┌', '┐'),
  line(40, [seg(' '), seg('faint label', '38;2;120;120;120;48;2;100;100;100'), seg('  '), seg('clear label', '38;2;255;255;255')]),
  line(40, [seg(' '), seg('black on default', '30')]),
  line(40, [seg(' status: ok')]),
  bar(40, '└', '┘'),
  HINTS,
]]]);

write('nested-borders', [[hdr(40, 7), [
  bar(40, '┌', '┐'),
  '│' + bar(38, '┌', '┐') + '│',
  '││' + ' Panel'.padEnd(36) + '││',
  '││' + ' content'.padEnd(36) + '││',
  '│' + bar(38, '└', '┘') + '│',
  bar(40, '└', '┘'),
  HINTS,
]]]);

write('truecolor-on-256', [[hdr(40, 5, 'termfeatures=256'), [
  bar(40, '┌', '┐'),
  line(40, [seg(' '), seg('warm title', '38;2;200;100;50')]),
  line(40, [seg(' body text')]),
  bar(40, '└', '┘'),
  HINTS,
]]]);

write('width-drift', [[hdr(30, 6), [
  bar(30, '┌', '┐'),
  '│ plain row' + ' '.repeat(18) + '│',
  '│ 🚀 launch' + ' '.repeat(19) + '│',   // the app counted the rocket as one cell: 31 cells on a 30-column pane
  '│ 日本 tokyo' + ' '.repeat(15) + '│',  // the app counted 日本 as six cells: the border lands early
  bar(30, '└', '┘'),
  HINTS,
]]]);

write('no-key-hints', [[hdr(40, 5), [
  bar(40, '┌', '┐'),
  line(40, [seg(' Import finished')]),
  line(40, [seg(' 42 files written')]),
  bar(40, '└', '┘'),
  'Loading complete.',
]]]);

const sentence = 'Processing the selected directory records to the index';
write('collapse-narrow', [[hdr(40, 5), [
  bar(50, '┌', '┐'),
  '┌' + '─'.repeat(30),
  sentence.slice(0, 40),
  sentence.slice(40),
  'q quit',
]]]);

const spin = (g) => [bar(40, '┌', '┐'), line(40, [seg(' Syncing ' + g)]), line(40, [seg(' 3 of 10 files')]), bar(40, '└', '┘'), HINTS];
write('spinner-never-rests', [[hdr(40, 5), spin('⠋')], [hdr(40, 5, 'role=recapture'), spin('⠙')]]);

const good = (w) => [bar(w, '┌', '┐'), line(w, [seg(' Files')]), line(w, [seg(' 3 selected')]), bar(w, '└', '┘'), HINTS];
write('sizes-pass', [[hdr(80, 6, 'termfeatures=256,RGB'), good(80)], [hdr(80, 6, 'termfeatures=256,RGB role=recapture'), good(80)], [hdr(40, 6, 'termfeatures=256,RGB'), good(40)]]);
console.log(fs.readdirSync(dir).join(' '));
JS
```

Then check each file with the binary before recording anything:

```bash
B="$PWD/target/release/impeccable"
for f in tests/fixtures/terminal-captures/*.txt; do echo "== $f"; "$B" detect --no-config --json --tmux-capture "$f" | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{for(const f of JSON.parse(s))console.log(f.antipattern,f.line,f.column,f.frame,JSON.stringify(f.snippet))})'; done
"$B" detect --no-config --json --palette light --tmux-capture tests/fixtures/terminal-captures/low-contrast.txt | grep -c '"tui-rt-low-contrast"'
```

Expected, one rule per file and nothing else: `low-contrast` two findings on dark (`faint label` 1.3:1, `black on default` 1.0:1) and two on light (`faint label`, `clear label` 1.0:1); `nested-borders` one at line 2 column 2 (`38x4 box drawn directly inside a 40x6 box`); `truecolor-on-256` one at line 2 column 3; `width-drift` two (line 3 `row measures 31 cells on a 30-column pane`, line 4 `border ends at column 28 while other rows end at 30`); `no-key-hints` one at line 5; `collapse-narrow` three (lines 1, 2, 3); `spinner-never-rests` one at line 2 column 11; `sizes-pass` `[]`. A stray finding means a fixture row trips a second rule: fix the fixture, not the rule, unless the rule is wrong by the spec's definition (then fix the rule with a unit test first).

- [ ] **Step 2: Add the cases** to `tests/oracle/cases/detect.mjs` after the `detect-terminal-product-flag-web` line and before `// Flag surface and errors`:

```js
    // The tmux engine (spec section 5) through saved captures, so the oracle
    // needs no tmux. One golden per fixture, plus the text renderer, the light
    // palette, the failure paths, and every flag's validation message.
  );
  const CAPTURES = `<REPO>/tests/fixtures/terminal-captures`;
  const CAPTURE_FILES = fs.readdirSync(path.join(REPO_ROOT, 'tests', 'fixtures', 'terminal-captures'))
    .filter((f) => f.endsWith('.txt')).sort();
  for (const file of CAPTURE_FILES) {
    out.push({ id: `detect-tmux-capture-${file.replace(/\.txt$/, '')}`, verb: 'detect', args: ['--no-config', '--json', '--tmux-capture', `${CAPTURES}/${file}`], isolateHome: false });
  }
  out.push(
    { id: 'detect-tmux-capture-text', verb: 'detect', args: ['--no-config', '--tmux-capture', `${CAPTURES}/nested-borders.txt`], isolateHome: false },
    { id: 'detect-tmux-capture-light-palette', verb: 'detect', args: ['--no-config', '--json', '--palette', 'light', '--tmux-capture', `${CAPTURES}/low-contrast.txt`], isolateHome: false },
    { id: 'detect-tmux-capture-with-files', verb: 'detect', args: ['--no-config', '--json', '--tmux-capture', `${CAPTURES}/no-key-hints.txt`, `<REPO>/tests/fixtures/antipatterns/blinking-cursor.html`], isolateHome: false },
    { id: 'detect-tmux-capture-missing', verb: 'detect', args: ['--no-config', '--json', '--tmux-capture', `${CAPTURES}/does-not-exist.txt`], isolateHome: false },
    // No tmux anywhere: PATH names a directory that does not exist and the
    // override is unset, so the requirement line is the whole output.
    { id: 'detect-tmux-missing-binary', verb: 'detect', args: ['--no-config', '--tmux', 'nope:0.0'], env: { PATH: '/nonexistent', IMPECCABLE_TMUX: null }, isolateHome: false },
    { id: 'detect-tmux-flag-no-target', verb: 'detect', args: ['--no-config', '--tmux'], isolateHome: false },
    { id: 'detect-tmux-capture-no-path', verb: 'detect', args: ['--no-config', '--tmux-capture'], isolateHome: false },
    { id: 'detect-tmux-sizes-invalid', verb: 'detect', args: ['--no-config', '--tmux', 'nope:0.0', '--tmux-sizes', 'wide'], isolateHome: false },
    { id: 'detect-tmux-settle-invalid', verb: 'detect', args: ['--no-config', '--tmux', 'nope:0.0', '--tmux-settle', 'soon'], isolateHome: false },
    { id: 'detect-palette-invalid', verb: 'detect', args: ['--no-config', '--tmux-capture', `${CAPTURES}/low-contrast.txt`, '--palette', 'sepia'], isolateHome: false },
```

(The block keeps the file's existing `out.push(` open call: the first line above closes the previous `out.push(...)` with `);` and the last case list is closed by the `// Flag surface and errors` cases that follow, which already sit inside an `out.push(` call. Read the surrounding lines and keep the parentheses balanced; `node -e "import('./tests/oracle/cases/detect.mjs').then(m=>console.log(m.default().length))"` must print a count.)

- [ ] **Step 3: Record and review**

```bash
IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/record.mjs --bin detect-tmux 2>&1 | tail -n 3
IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/record.mjs --bin detect-palette 2>&1 | tail -n 3
IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/record.mjs --bin detect-help 2>&1 | tail -n 3
git status --short tests/oracle/golden | grep -v "detect-tmux\|detect-palette\|detect-help"; echo "expect nothing above: no other golden moved"
```

Read every new golden by hand (`Read` tool, not `cat`): exit codes (0 for every capture case, 1 for the four failure and five validation cases), the `Error:` lines byte for byte against spec section 5's tables, and `detect-tmux-missing-binary` carrying exactly `Error: tmux 3.2 or newer is required for --tmux and was not found on PATH. Install tmux, or point IMPECCABLE_TMUX at the executable.\n`. `detect-tmux-capture-with-files` shows the HTML finding first and the tmux finding after it.

- [ ] **Step 4: Replay**

```bash
IMPECCABLE_BIN="$PWD/target/release/impeccable" node --test tests/oracle.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)"
```

- [ ] **Step 5: `DELTAS.md`.** Append a section `## Recorded 2026-09-24: tmux engine (spec PR 3)` that says: the new `detect-tmux-*` and `detect-palette-invalid` cases pin the five flags and the seven runtime rules through saved captures; `detect-help` moved for the new usage lines; no other golden changed because the runtime rules run only behind `--tmux` / `--tmux-capture`; the missing-binary case sets `PATH` to `/nonexistent` and unsets `IMPECCABLE_TMUX`. Quote the exact stderr lines from the goldens, not from memory.

- [ ] **Step 6: Commit**

```bash
git add tests/fixtures/terminal-captures tests/oracle/cases/detect.mjs tests/oracle/golden tests/oracle/DELTAS.md
git commit -m "$(printf 'Pin the tmux engine with saved-capture fixtures and oracle goldens\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 9: The scratch-server integration test

**Files:**
- Create: `tests/fixtures/terminal-apps/nested-boxes.sh`, `tests/tmux-engine.test.mjs`
- Modify: `scripts/test-suites.mjs` (oracle suite), `.github/workflows/ci.yml` (oracle job)

**Interfaces:**
- Consumes: the binary through `tests/lib/engine-bin.mjs` (`findEngineBinary`, `engineEnv`, `ENGINE_MISSING_MESSAGE`); the `IMPECCABLE_TMUX` override from Task 7.
- Produces: the one test that proves a live pane round-trips, including the restore.

- [ ] **Step 1: The fixture app.** Create `tests/fixtures/terminal-apps/nested-boxes.sh` and `chmod +x` it:

```sh
#!/bin/sh
# A deliberately bad TUI for tests/tmux-engine.test.mjs: a box drawn directly
# inside a box, gray-on-gray truecolor text, no key hints, and a spinner that
# never stops. Draws once at 60 columns, then animates one cell forever.
printf '\033[2J\033[H'
inner() { printf '││%-56s││\n' "$1"; }
printf '┌──────────────────────────────────────────────────────────┐\n'
printf '│┌────────────────────────────────────────────────────────┐│\n'
inner ' Impeccable fixture'
inner ''
printf '││ \033[38;2;120;120;120m\033[48;2;100;100;100m%-54s\033[0m ││\n' 'faint status text nobody can read'
inner ''
inner ' working  '
printf '│└────────────────────────────────────────────────────────┘│\n'
printf '└──────────────────────────────────────────────────────────┘\n'
printf 'Loading complete.'
while :; do
  for g in '⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏'; do
    # Save the cursor, write the glyph at row 7 column 13, restore.
    printf '\0337\033[7;13H%s\0338' "$g"
    sleep 0.1
  done
done
```

Check it draws: `sh tests/fixtures/terminal-apps/nested-boxes.sh` in a spare terminal for two seconds, then Ctrl-C.

- [ ] **Step 2: The test.** Create `tests/tmux-engine.test.mjs`:

```js
/**
 * The tmux engine against a scratch tmux server (spec section 5, "Oracle and
 * tests"). Starts `tests/fixtures/terminal-apps/nested-boxes.sh` in a detached
 * server on its own socket, points the engine at that server through an
 * IMPECCABLE_TMUX wrapper that adds `-L`, and checks the findings and the
 * restore. Skips without a binary or without tmux. The server is killed in
 * after(); the runner's process-group guard covers a SIGKILLed test process.
 */
import { after, before, describe, it } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { ENGINE_MISSING_MESSAGE, engineEnv, findEngineBinary } from './lib/engine-bin.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const BIN = findEngineBinary();
const TMUX_OK = spawnSync('tmux', ['-V'], { encoding: 'utf8' }).status === 0;
const SOCKET = `impeccable-test-${process.pid}`;
const APP = path.join(ROOT, 'tests', 'fixtures', 'terminal-apps', 'nested-boxes.sh');

const tmux = (...args) => execFileSync('tmux', ['-L', SOCKET, ...args], { encoding: 'utf8' }).trim();
let wrapper;

function detect(...args) {
  return spawnSync(BIN, ['detect', '--no-config', ...args], {
    encoding: 'utf8',
    env: engineEnv(BIN, { IMPECCABLE_TMUX: wrapper }),
    timeout: 30_000,
  });
}

describe('tmux engine against a scratch server', { skip: !BIN ? ENGINE_MISSING_MESSAGE : !TMUX_OK ? 'tmux is not installed' : false }, () => {
  before(() => {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'impeccable-tmux-'));
    wrapper = path.join(dir, 'tmux');
    fs.writeFileSync(wrapper, `#!/bin/sh\nexec tmux -L ${SOCKET} "$@"\n`, { mode: 0o755 });
    tmux('new-session', '-d', '-x', '80', '-y', '24', '-s', 'app', `sh ${APP}`);
    spawnSync('sleep', ['0.5']);
  });

  after(() => {
    try { tmux('kill-server'); } catch { /* already gone */ }
    if (wrapper) fs.rmSync(path.dirname(wrapper), { recursive: true, force: true });
  });

  it('captures the pane, walks the size matrix, and restores the window', () => {
    const r = detect('--json', '--tmux', 'app:0.0', '--tmux-sizes', '40x24');
    assert.equal(r.status, 0, r.stderr);
    const findings = JSON.parse(r.stdout);
    const ids = new Set(findings.map((f) => f.antipattern));
    for (const id of ['tui-rt-nested-borders', 'tui-rt-low-contrast', 'tui-rt-no-key-hints', 'tui-rt-spinner-never-rests']) {
      assert.ok(ids.has(id), `${id} missing from ${[...ids].join(', ')}`);
    }
    assert.ok(findings.some((f) => f.antipattern === 'tui-rt-collapse-narrow' && f.frame === '40x24'), 'the 40-column frame collapses');
    assert.ok(findings.every((f) => f.file === 'tmux:app:0.0' && f.advisory === true));
    assert.equal(tmux('display', '-p', '-t', 'app:0.0', '#{window_width}x#{window_height} #{window-size}'), '80x24 latest', 'window size and mode restored');
  });

  it('reports an unknown pane as an operational failure and touches nothing', () => {
    const r = detect('--tmux', 'nope:0.0');
    assert.equal(r.status, 1);
    assert.match(r.stderr, /^Error: tmux list-panes: can.t find session: nope/);
    assert.equal(r.stdout, '');
  });
});
```

- [ ] **Step 3: Run it**

```bash
IMPECCABLE_BIN="$PWD/target/release/impeccable" node --test tests/tmux-engine.test.mjs 2>&1 | grep -E "^ℹ (pass|fail|skipped)|not ok" 
tmux -L "impeccable-test-$$" ls 2>&1 | head -n 1   # any leftover server from a crashed run shows here
```

If `#{window-size}` reads `manual` after the run, the restore's `set-option -u` did not reach the window: check that `restore()` passes `-w -t <target>`. If `client_termfeatures` questions come up, they do not affect this test (no truecolor assertion, by design: the detached server has no client).

- [ ] **Step 4: Register the file.** In `scripts/test-suites.mjs`, oracle suite: add `/^crates\/terminal\//,` to `triggers` (after the `skill` pattern) and a second command:

```js
    commands: [
      {
        runner: 'node',
        timeoutMs: 900000,
        files: ['tests/oracle.test.mjs'],
      },
      // The tmux engine against a scratch server; skips without tmux or a binary.
      {
        runner: 'node',
        timeoutMs: 120000,
        files: ['tests/tmux-engine.test.mjs'],
      },
    ],
```

In `.github/workflows/ci.yml`, oracle job, add after the `Replay oracle goldens` step:

```yaml
      - name: Run the tmux engine against a scratch server
        env:
          IMPECCABLE_BIN: ${{ github.workspace }}/target/release/impeccable
        run: node --test tests/tmux-engine.test.mjs
```

Then:

```bash
node --test tests/test-suites.test.mjs tests/ci-test-plan.test.mjs tests/workflow-security.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)"
node scripts/run-tests.mjs --list | grep -n tmux
```

- [ ] **Step 5: Commit**

```bash
git add tests/fixtures/terminal-apps/nested-boxes.sh tests/tmux-engine.test.mjs scripts/test-suites.mjs .github/workflows/ci.yml
git commit -m "$(printf 'Add the scratch-server integration test for the tmux engine\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 10: Documentation, skill text, counts, and the full gates

**Files:**
- Modify: `README.md`, `README.npm.md` (every `72 deterministic` phrase becomes `79`)
- Modify: `docs/CLI-CONTRACT.md` (flag table near line 99; USAGE block near line 122; a **tmux scans** paragraph after the **Platform** paragraph near line 109; registry list at line 253; a new `#### crates/terminal — tmux pane scans` section after the URL-scans section's `Tests:` line near line 336)
- Modify: `docs/ENGINE.md` (Layout block: a `terminal` row after `browser`; one sentence in "Rule packs")
- Modify: `CLAUDE.md` (root: the `terminal` bullet at line 55, the web-only paragraph at line 63, the rule-authoring paragraph at line 363)
- Modify: `skill/reference/terminal.md` (one bullet in "Verifying the build"), `skill/reference/audit.terminal.md` (one sentence in the intro paragraph), `skill/reference/adapt.terminal.md` (one sentence after the width-class table)
- Record: `tests/oracle/golden/context-terminal.json` (moves because `terminal.md` changed)

**Interfaces:** none new. Every statement must match code that landed in Tasks 2 to 9; quote ids, flags, and messages from the code.

- [ ] **Step 1: Counts**

```bash
sed -i '' 's/72 deterministic/79 deterministic/g' README.md README.npm.md
grep -n "deterministic" README.md README.npm.md | grep -v 79; echo "expect nothing above"
bun run build 2>&1 | tail -n 5     # generateCounts must pass
```

- [ ] **Step 2: `docs/CLI-CONTRACT.md`.**
- Flag table, after the `--platform` row, five rows (the value column and the error column copied from spec section 5's "Flags and messages" table; note that each is spliced out like `--viewport` and accepts `--flag=value`).
- USAGE block: paste the seven `--tmux*` / `--palette` lines, the `tmux panes` detection-mode lines, and the new example exactly as `crates/detect/src/cli.rs` has them (diff the two blocks: `sed -n '/^pub const USAGE/,/^";/p' crates/detect/src/cli.rs`).
- After the **Platform** paragraph add: ``**tmux scans**: `--tmux <target>` (repeatable) and `--tmux-capture <file>` (repeatable) are scanned after the file and URL targets, on any resolved platform; when either is present and no file target is given there is no cwd fallback and stdin is not read. A pane failure prints `Error: <message>` (tmux missing: `tmux 3.2 or newer is required for --tmux and was not found on PATH. Install tmux, or point IMPECCABLE_TMUX at the executable.`; too old: `tmux 3.2 or newer is required for --tmux; found <tmux -V>`; bad target: `tmux list-panes: <tmux's first stderr line>`, from the `list-panes -t <target>` check that precedes `display-message`) and sets the operational failure; a capture file failure prints `Error: cannot scan <file>: <ENOENT/EACCES message>`. Findings carry `file` `tmux:<target>` (pane) or the resolved path (capture), `line` = 1-based row, `column` = 1-based cell, `frame` = `WxH`, and for `tui-rt-low-contrast` `cellCount`; every runtime rule is advisory. Palette `dark` (default) or `light`; sizes restore the window (`resize-window` back, then `set-option -w -u window-size`).``
- Line 253: `(72 ids, in order)` becomes `(79 ids, in order)`; append the seven `tui-rt-` ids in registry order after `tui-print-in-loop`; after the `platforms` sentence add: ``The seven `tui-rt-` rows run only in the tmux engine (crates/terminal), never in the text engine, the hook, or the browser adapters.``
- New section after the URL-scans `Tests:` line:

```
#### `crates/terminal` — tmux pane scans

- `TmuxEngine::detect_pane(target, options)`: `Tmux::locate` (`IMPECCABLE_TMUX`, else `PATH`; `tmux -V` must parse to 3.2 or newer) → `display-message -p -t <target> '#{pane_width}\t#{pane_height}\t#{window_width}\t#{window_height}\t#{client_termfeatures}'` → `capture-pane -p -e -J -t <target>` (frame 0, role `capture`) → sleep 1000 ms → capture again (role `recapture`) → for each `--tmux-sizes WxH`: `resize-window -t <target> -x W -y H`, sleep `--tmux-settle` (default 300), `display-message` again, capture → `resize-window` back to the recorded window size and `set-option -w -t <target> -u window-size` (always, even after a failed capture) → `rules::scan_frames`.
- `TmuxEngine::detect_capture(path, options)`: read the file (ENOENT/EACCES messages as for local files) → `capture::parse_frames` (frames introduced by `#!capture width= height= termfeatures= colorterm= role=` lines; a headerless file is one frame measured from its rows) → `rules::scan_frames`.
- Frame policy: `tui-rt-low-contrast`, `tui-rt-truecolor-on-256`, `tui-rt-no-key-hints` read frame 0; `tui-rt-spinner-never-rests` compares frame 0 with the first `recapture`; `tui-rt-nested-borders` and `tui-rt-width-drift` run on every `capture` frame; `tui-rt-collapse-narrow` on `capture` frames under 60 columns.
- SGR parsed: 0, 1, 2, 5/6, 7, 22, 25, 27, 30-37, 39, 40-47, 49, 90-97, 100-107, `38;5;n`, `48;5;n`, `38;2;r;g;b`, `48;2;r;g;b`; widths from `unicode-width` (two-cell glyphs own a wide-tail cell, zero-width marks own none).
- Palette: default fg/bg white-on-black (`dark`) or black-on-white (`light`); ANSI 0-15 from xterm's table; 256-color cube and grays by formula; truecolor as written; `reverse` swaps. Contrast is `contrast_ratio` with floor 4.5 over alphanumeric glyphs; one finding per distinct pair.
- Tests: `cargo test -p impeccable-terminal`, `tests/tmux-engine.test.mjs` (scratch server), oracle cases `detect-tmux-*`.
```

- [ ] **Step 3: `docs/ENGINE.md`.** In the Layout block add after the `browser` line: `  terminal     the tmux engine: capture parser, contrast palette, runtime rules, tmux driver`. In "Rule packs" add one sentence at the end of the paragraph that lists the hooks: "Rule packs do not extend the tmux engine; its rules are the seven `tui-rt-` rows in the registry."

- [ ] **Step 4: Root `CLAUDE.md`.**
- Line 55 bullet: after `run over its source,` insert `` `impeccable detect --tmux <pane>` captures a running pane and runs the `tui-rt-` runtime rules (`crates/terminal`), ``.
- Line 63 paragraph, append: "`impeccable detect --tmux <session:window.pane>` (tmux 3.2 or newer) captures an already running pane, optionally at each `--tmux-sizes WxH`, and runs the runtime rules; `--tmux-capture <file>` replays a saved capture. The engine never launches a process and restores the window size it changed."
- Line 363 paragraph, append: "A runtime rule (`tui-rt-`) lives in `crates/terminal/src/rules.rs`, its fixture is a saved capture under `tests/fixtures/terminal-captures/<rule>.txt` with a `#!capture` header, and its oracle case is `detect-tmux-capture-<rule>`."

- [ ] **Step 5: Skill text.**
- `skill/reference/terminal.md`, after the "Run the matrix." bullet, add one bullet (it must start with `- **` and end with a rule marker; `tests/skill-reference.test.mjs` pins both):

```
- **Scan the pane.** With the app running in tmux, run `{{scripts_path}}/impeccable detect --json --tmux <target> --tmux-sizes 80x24,120x40,40x24` (add `--palette light` for the light-theme pass). It captures the pane at each size, restores it, and reports the `tui-rt-` runtime rules: contrast, nested borders, truecolor on a 256-color client, width drift on emoji or CJK, missing key hints, narrow collapse, a spinner that never rests. A saved `tmux capture-pane -p -e -J` file replays with `--tmux-capture <file>`. Every `tui-rt-` finding is advisory. <!-- rule:terminal-verify-detect -->
```

- `skill/reference/audit.terminal.md`, line 3, after `so the project signals (TTY guards, adaptive color helpers, icon fallbacks) are collected.` insert: `With the app running in a tmux pane, add `--tmux <target> --tmux-sizes 80x24,120x40,40x24` to the same command for the `tui-rt-` runtime rules over the rendered grid; those are advisory too, and a saved capture replays with `--tmux-capture <file>`.`
- `skill/reference/adapt.terminal.md`, after the width-class table (before the "Restructure, do not squeeze." bullet), add a paragraph: `` `{{scripts_path}}/impeccable detect --json --tmux <target> --tmux-sizes 80x24,120x40,40x24` captures all three classes in one run and reports what collapses below 60 columns (`tui-rt-collapse-narrow`) and what drifts on emoji or CJK (`tui-rt-width-drift`). ``

Then:

```bash
bun run build 2>&1 | tail -n 3
node --test tests/skill-reference.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)"
IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/record.mjs --bin context-terminal 2>&1 | tail -n 2
git status --short tests/oracle/golden    # only context-terminal.json
```

Read the re-recorded `context-terminal.json` diff: the new bullet is the only change. This is not Setup text; `bun run test:skill-behavior` is not owed (ledger it).

- [ ] **Step 6: Full gates**

```bash
cargo test --workspace 2>&1 | tail -n 3
T=$HOME/.rustup/toolchains/stable-x86_64-apple-darwin; PATH="$T/bin:$PATH" DYLD_LIBRARY_PATH="$T/lib" cargo xtask bundle --check && echo BUNDLE_FRESH
cargo build --release -p impeccable 2>&1 | tail -n 1
bun run build 2>&1 | tail -n 3
IMPECCABLE_BIN="$PWD/target/release/impeccable" bun run test 2>&1 | tail -n 15
```

Expected: green except the known pre-existing failures from Task 1's baseline. Compare pass counts with the baseline plus the new oracle cases and the two integration tests.

- [ ] **Step 7: Commit**

```bash
git add README.md README.npm.md docs/CLI-CONTRACT.md docs/ENGINE.md CLAUDE.md skill/reference/terminal.md skill/reference/audit.terminal.md skill/reference/adapt.terminal.md tests/oracle/golden/context-terminal.json
git commit -m "$(printf 'Document the tmux engine, the --tmux flags, and the 79-rule count\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 11: Manual pass over real terminal programs

**Files:**
- Create: `docs/superpowers/research/2026-09-24-terminal-runtime-rules-manual-pass.md`
- Possibly modify: `crates/terminal/src/rules.rs` or `capture.rs` (only to fix a bug the pass exposes; each fix adds a unit test first and re-records any affected `detect-tmux-*` golden)

**Interfaces:** consumes the release binary from Task 10. Produces the calibration record spec section 6 requires before any runtime rule leaves `advisory`, plus answers to the two PR 3 open questions (the one-second spinner window; whether `client_termfeatures` is populated from outside the session).

- [ ] **Step 1: Stage programs in a scratch server.** Everything runs in `tmux -L impeccable-rt-pass`; nothing touches the user's own tmux server.

```bash
S=/private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/rt-manual-pass
mkdir -p "$S"; B="$PWD/target/release/impeccable"; SOCK=impeccable-rt-pass
tmux -L $SOCK kill-server 2>/dev/null
tmux -L $SOCK new-session -d -x 100 -y 30 -s pass -n top 'top'
tmux -L $SOCK new-window -t pass -n vim "vim $PWD/README.md"
tmux -L $SOCK new-window -t pass -n less "less $PWD/CLAUDE.md"
tmux -L $SOCK new-window -t pass -n man 'man tmux'
# A Ratatui app: the PR 2 clone if it is still in the scratchpad, else a shallow clone.
R=/private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/manual-pass/ratatui
[ -d "$R" ] || git clone --depth 1 https://github.com/ratatui/ratatui.git "$R"
( cd "$R" && timeout 600 cargo build --release --example demo2 2>&1 | tail -n 1 ) && tmux -L $SOCK new-window -t pass -n demo2 "$(ls "$R"/target/release/examples/demo2 2>/dev/null | head -n 1)"
for extra in htop btop lazygit gitui nvim; do command -v $extra >/dev/null && tmux -L $SOCK new-window -t pass -n $extra "$extra"; done
sleep 3
printf '#!/bin/sh\nexec tmux -L %s "$@"\n' "$SOCK" > "$S/tmux-wrap"; chmod +x "$S/tmux-wrap"
tmux -L $SOCK list-windows -t pass -F '#{window_name}' > "$S/windows.txt"; cat "$S/windows.txt"
```

If the Ratatui example does not build within ten minutes, skip it and record the substitution; the pass must still cover at least four programs.

- [ ] **Step 2: Scan each window at three sizes and record timings**

```bash
for w in $(cat "$S/windows.txt"); do
  /usr/bin/time -p env IMPECCABLE_TMUX="$S/tmux-wrap" "$B" detect --no-config --json --tmux "pass:$w" --tmux-sizes 80x24,120x40,40x24 > "$S/$w.json" 2> "$S/$w.err"; echo "$w exit=$?"
done
node -e '
const fs=require("fs"); const S=process.argv[1];
for (const f of fs.readdirSync(S).filter(f=>f.endsWith(".json"))) {
  let a=[]; try { a=JSON.parse(fs.readFileSync(`${S}/${f}`,"utf8")); } catch { console.log(f, "unparsable"); continue; }
  const by={}; for (const x of a) by[x.antipattern]=(by[x.antipattern]||0)+1;
  console.log(f, a.length, JSON.stringify(by));
}' "$S"
tmux -L $SOCK display -p -t pass:top '#{window_width}x#{window_height} #{window-size}'   # 100x30 latest: the restore held
IMPECCABLE_TMUX="$S/tmux-wrap" "$B" detect --no-config --json --tmux pass:top --palette light > "$S/top-light.json"
```

Also save one capture and replay it, the review-flow path `terminal.md` now describes:

```bash
tmux -L $SOCK capture-pane -p -e -J -t pass:less > "$S/less-capture.txt"
"$B" detect --no-config --json --tmux-capture "$S/less-capture.txt" | head -c 600
```

- [ ] **Step 3: Judge samples.** For every rule with hits, print up to ten findings (window, frame, line, column, snippet) and look at the corresponding capture (`tmux -L $SOCK capture-pane -p -t pass:<w> | sed -n '<line>p'`). Classify each as true positive, false positive, or arguable, with the reason in one clause. A false-positive rate above roughly a third marks a rule "tighten before promotion"; a rule with no hits anywhere is "unmeasured". Where a false positive is a plain bug (an SGR parameter misread, a box-drawing glyph missing from a set, a width miscounted), fix it with a unit test named for the outcome, rebuild, re-record only the affected `detect-tmux-*` golden, and note the fix.

- [ ] **Step 4: Answer the two open questions** from the evidence: how often `tui-rt-spinner-never-rests` fired on programs that were idle versus programs doing work (top redraws every second; vim and less are idle), and what `client_termfeatures` reported (`tmux -L $SOCK display -p -t pass:top '#{client_termfeatures}'` in the detached server, and once more after attaching a client in a second terminal if one is available; if none is, say so).

- [ ] **Step 5: Write the record** at `docs/superpowers/research/2026-09-24-terminal-runtime-rules-manual-pass.md`: the programs and versions, scan times, a table `rule | program | hits | sampled | false positives | verdict`, the restore check, the two answers, the fixes made (if any), and the promotion recommendation per rule (all stay `advisory` in this PR). No em dashes.

- [ ] **Step 6: Tear down and commit**

```bash
tmux -L $SOCK kill-server
git add docs/superpowers/research/2026-09-24-terminal-runtime-rules-manual-pass.md crates/terminal tests/oracle/golden
git commit -m "$(printf 'Record the terminal runtime-rule manual pass\n\nAI assistance: drafted with Claude Code.')"
```

(Drop paths from `git add` that did not change.)

---

### Task 12: Final review, merge to the fork's main, push

**Files:** none new.

- [ ] **Step 1: Whole-branch review** per subagent-driven-development (final code reviewer on the most capable model). Findings marked Important or Critical get one fix wave and one scoped re-review.

- [ ] **Step 2: Gates once more on the final head**

```bash
cargo test --workspace 2>&1 | tail -n 3
T=$HOME/.rustup/toolchains/stable-x86_64-apple-darwin; PATH="$T/bin:$PATH" DYLD_LIBRARY_PATH="$T/lib" cargo xtask bundle --check && echo BUNDLE_FRESH
bun run build 2>&1 | tail -n 2
IMPECCABLE_BIN="$PWD/target/release/impeccable" bun run test 2>&1 | tail -n 8
IMPECCABLE_BIN="$PWD/target/release/impeccable" node --test tests/tmux-engine.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)"
git status --short   # only the untracked local files may show; never stage them
```

- [ ] **Step 3: Merge and push** (the fork owner pre-authorized merging each PR to `main` as it lands):

```bash
git checkout main && git pull --ff-only origin main
git merge --no-ff terminal-platform-pr3 -m "$(printf 'Merge terminal-platform-pr3: tmux runtime engine (spec PR 3)\n\nAI assistance: drafted with Claude Code.')"
git push origin main
git log --oneline -3
```

- [ ] **Step 4: Close out.** Ledger completion line; copy the ledger to the session scratchpad; delete the SDD workspace; update the memory file `terminal-platform-project.md` (PR 3 merged, commit id, the manual-pass verdicts, the two open-question answers, and that `ENGINE_VERSION` still needs a bump at release); write the journal entry. Update the untracked upstream issue draft `docs/superpowers/research/2026-09-24-upstream-issue-draft.md`: its question 4 becomes a "Part 3" paragraph naming the engine, the flags, and the merge commit, and the evidence list gains that commit. Never post it.

---

## Self-review notes

- Spec section 5 coverage: crate and modules (Tasks 4 to 7); seam, `Engines.tmux`, `ScanOptions` fields, flags and messages (Task 3); pane or saved capture, no cwd fallback, `IMPECCABLE_TMUX` (Tasks 3, 7); capture format, header keys, headerless fallback (Task 4); frame order, settle, restore incl. `set-option -u window-size` (Task 7); rules-over-frames policy (Task 5 `scan_frames`, Task 6 test); palette (Task 4); output fields (`file`, `line`, `column`, `frame`, `cellCount`; Tasks 5, 7); requirements and degradation messages (Tasks 3, 7, 8); oracle fixtures and cases (Task 8); integration test, suite, CI step (Task 9).
- Spec section 4 PR 3 table: seven rows (Task 2), seven rules with the stated signals (Tasks 5, 6), categories checked by the registry test.
- Spec section 6 PR 3 files: every listed file appears in a task; gates in Tasks 10 and 12; the manual pass in Task 11.
- Type consistency: `TmuxEngine::{detect_pane, detect_capture}(&self, &str, &ScanOptions) -> Result<Vec<Finding>, EngineError>` is spelled the same in Tasks 3, 7; `ScanOptions.tmux_sizes: Vec<(u32, u32)>`, `.tmux_settle_ms: Option<u64>`, `.palette: Option<String>` in Tasks 3, 7; `Frame`, `FrameRole`, `parse_rows`, `parse_frames` in Tasks 4 to 7; `PaneInfo` fields in Task 7 and its `lib.rs` test; `rt_finding(id, file, frame, row, col, snippet)` in Tasks 5, 6.
- Not in this PR: promotion of any rule, `ENGINE_VERSION`, the known gaps listed in the spec, `.claude/CLAUDE.md` (untracked on this machine).
