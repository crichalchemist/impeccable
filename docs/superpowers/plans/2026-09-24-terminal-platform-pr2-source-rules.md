# Terminal Platform PR 2: Source Rules Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the eleven `tui-` source rules so `impeccable detect` and the design hook scan Ratatui, Bubble Tea, Textual, and Ink source on a `terminal` project, with every rule at `advisory`.

**Architecture:** Registry rows gain a `platforms` field; the eleven new rows sit at the end of `ANTIPATTERNS`. A new pure module `crates/core/src/checks/terminal.rs` holds framework detection, project signals, comment blanking, and one scan function per rule. `crates/detect` threads a resolved platform and the collected signals into `detect_text`, walks the terminal extensions only on a terminal project, and takes a `--platform` flag; `crates/cli` supplies the PRODUCT.md resolver through `Engines` so `crates/detect` never depends on `crates/context`. The hook widens its extension gate on terminal projects and passes the same platform into the text engine.

**Tech Stack:** Rust (workspace crates `foundation`, `core`, `detect`, `hook`, `context`, `cli`), `regex` + `once_cell` (already dependencies of `core` and `detect`), node oracle harness (`tests/oracle/`), `cargo xtask bundle` (needs `wasm-pack`), bun build gates.

**Spec:** `docs/superpowers/specs/2026-09-24-terminal-platform-design.md`, sections 3, 4 (PR 2 source rules), and 6 (PR 2). Read section 3 and the PR 2 table in section 4 before any task.

## Global Constraints

- Every new rule ships at `severity: Some("advisory")`. Promotion to `warning` happens only after the manual pass in Task 10, and not in this PR.
- Rules keep `category: "slop"` or `"quality"`. No new category. The `platforms` field alone marks terminal rules.
- `platforms: Option<&'static [&'static str]>`; `None` means web (every existing row). Terminal rules declare `Some(&["terminal"])`.
- `crates/detect` does not depend on `crates/context` and must not start to. `crates/cli` builds the resolver and hands it to `detect` through `Engines`.
- `SCANNABLE_EXTENSIONS` is unchanged for web. `TERMINAL_EXTENSIONS` is `.rs`, `.go`, `.py`, `.tcss` and is consulted only on a terminal project.
- Built-in web output stays byte-identical: new rows go at the end of `ANTIPATTERNS`, and no web code path changes its output when the platform is not `terminal`. The oracle enforces this; any web golden that changes must be explained in `tests/oracle/DELTAS.md`.
- Framework detection is per file (spec section 3 table). A file that fails it gets no terminal rules.
- Absence rules consult `ProjectSignals`. In a single-file scan the set is absent and those rules report with the note `project signals were not collected; scan the project directory to confirm`.
- `hook_lib.rs`'s native skip does not gain `terminal`.
- New oracle cases are recorded with `IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/record.mjs --bin <prefix>` and reviewed by hand. Never hand-edit a golden. Never touch `tests/oracle/vectors/calls/`.
- No manifest version bumps, no `ENGINE_VERSION` bump, no changelog entry.
- Every commit body ends with the line `AI assistance: drafted with Claude Code.` and carries no `Co-Authored-By` or other trailer (the fork owner's rule).
- No em dashes anywhere in `skill/**/*.md` (build gate). Registry descriptions and docs written in this PR also avoid them.
- Test names state outcomes (spec section 6): `web_project_with_rust_backend_gets_no_terminal_rules`, `terminal_platform_keeps_hook_active`, `hardcoded_truecolor_flagged_only_in_tui_files`, `single_file_scan_downgrades_absence_rules`.

## Rulings made while planning (deviations from the spec text, with reasons)

1. **Fixture layout.** Spec section 3 names `tests/fixtures/antipatterns/terminal/<rule>.<ext>`. This plan uses one subdirectory per rule, `tests/fixtures/antipatterns/terminal/<rule>/<rule>.<ext>`, so the per-rule oracle case is a directory scan and collects project signals from that rule's own files. A flat layout would make every per-rule golden a single-file scan and drown the absence rules in the downgrade note.
2. **`SKIP_DIRS`.** Spec says `SKIP_DIRS` gains `target`, `.venv`, `vendor`. The oracle workspace `detect-config` carries `src/vendor/ignored.html` and the case `detect-config-vendor-ignored` proves `ignoreFiles` on it; a global skip would make that case vacuous and change web output. The three names live in `TERMINAL_SKIP_DIRS` and are skipped only on a terminal project.
3. **Hook surfacing.** Spec section 3 says the hook's immediate tier runs the cheap rules. The hook drops advisory findings unless `.impeccable/config.json` sets `hook.advisoryRules` to `include`, and every PR 2 rule is advisory, so by default the hook runs on a terminal project and surfaces nothing. This PR proves the hook runs (extension gate, scan, audit) and that findings flow when advisory is included; default edit-time surfacing arrives with promotion. The five cheap rules still join `IMMEDIATE_TIER_RULES` now so promotion is a severity change only.
4. **Python docstring blanking.** Spec says comment stripping removes `#` comments and docstrings. Textual apps hold their stylesheet in `CSS = """..."""`, which carries `border: double` and `text-style: blink`. Only a triple-quoted literal that is the first token on its line (a bare docstring statement) is blanked.
5. **Density rules report once per file.** `tui-emoji-density` and `tui-nerd-glyph-no-fallback` are file-level thresholds, so each reports one finding at the first hit with a count in `extras`, instead of one finding per line.
6. **Platform resolution root.** `detect` resolves the platform once per run from the process cwd (the resolver is `resolve_project_platform(cwd, env)`), not per target. Running `impeccable detect /other/project` from an unrelated directory resolves web; `--platform terminal` is the override for that case, CI, and the oracle.
7. **`tui-hardcoded-size` test-path exemption.** Spec says "outside paths containing `test`". Only the file name and its parent directory are checked, because every fixture in this repo lives under `tests/fixtures/` and an absolute-path check would make the rule's own fixture scan to nothing.

## File structure

| File | Responsibility |
|---|---|
| `crates/foundation/src/registry.rs` | `platforms` field, `rule_runs_on_platform`, eleven rows at the end, five ids added to `IMMEDIATE_TIER_RULES` |
| `crates/core/src/checks/terminal.rs` (new) | `Stack`, `TERMINAL_EXTENSIONS`, `classify_terminal_source`, `ProjectSignals`, `strip_c_comments`, `strip_python_comments`, `scan_terminal_source`, one `scan_terminal_<rule>` per rule |
| `crates/core/src/checks/mod.rs` | `pub mod terminal;` |
| `crates/context/src/context.rs` | `resolve_project_platform(cwd, env)` (the one PRODUCT.md platform lookup) |
| `crates/hook/src/hook_lib.rs` | calls the context resolver; `allowed_exts(platform)`; `HookScanOptions` gains `platform` and `signals` |
| `crates/hook/src/hook.rs`, `before_edit.rs` | platform-aware extension gate; Stop pass collects signals |
| `crates/detect/src/detect_text.rs` | `TextOptions.platform`, `TextOptions.signals`, the terminal branch |
| `crates/detect/src/file_system.rs` | `TERMINAL_SKIP_DIRS`, `has_scannable_extension_for`, `walk_dir_reporting_for` |
| `crates/detect/src/engines.rs` | `ScanOptions.platform`, `ScanOptions.signals`, `Engines.platform` |
| `crates/detect/src/cli.rs` | `--platform` flag, resolver call, signals pre-pass, USAGE line |
| `crates/detect/tests/terminal.rs` (new) | integration tests over `detect_text` and the walker |
| `crates/cli/src/main.rs` | `Engines.platform: Some(&resolve_platform)` |
| `tests/fixtures/antipatterns/terminal/<rule>/` | one fixture per stack per rule |
| `tests/oracle/workspaces/detect-terminal-project/` | PRODUCT.md `terminal` + Ratatui source, used by detect and hook cases |
| `tests/oracle/cases/detect.mjs`, `hooks.mjs`, `golden/`, `DELTAS.md` | new cases, recorded goldens, reviewed deltas |
| `crates/live/assets/antipatterns.json`, `crates/live/assets/detect-antipatterns-browser.js` | regenerated by `cargo xtask bundle` |
| `README.md`, `README.npm.md`, `docs/CLI-CONTRACT.md`, `docs/ENGINE.md`, root `CLAUDE.md` | counts (61 to 72), flag, walker, registry list, module map, Platform paragraph |
| `docs/superpowers/research/2026-09-24-terminal-rules-manual-pass.md` (new) | Task 10's false-positive record and calibration decisions |

---

### Task 1: Branch and baseline

**Files:** none changed.

- [ ] **Step 1: Branch from the fork's main**

```bash
cd /Volumes/Containers/impeccable
git checkout main && git pull --ff-only origin main
git checkout -b terminal-platform-pr2
git log --oneline -1   # expect ed600fd0 or a later main that contains it
```

- [ ] **Step 2: Build the engine and confirm the tooling**

```bash
cargo build --release -p impeccable 2>&1 | tail -n 2
which wasm-pack || cargo install wasm-pack --locked
bun --version    # Homebrew bun 1.4.x
```

- [ ] **Step 3: Record the baseline**

```bash
cargo test --workspace 2>&1 | grep -E "^test result|FAILED" | sort | uniq -c
IMPECCABLE_BIN="$PWD/target/release/impeccable" bun run test 2>&1 | tail -n 15
node -e "console.log(JSON.parse(require('fs').readFileSync('crates/live/assets/antipatterns.json','utf8')).length)"   # expect 61
```

Known pre-existing failures on this machine, not to be fixed here: `tests/release.test.mjs` (3 tests, git config forces signed tags) and the oracle case `detect-fixture-text-framework-vite` if anything holds port 8080. Write the pass counts into the SDD ledger.

- [ ] **Step 4: No commit** (nothing changed). Record `BASE=$(git rev-parse HEAD)` in the ledger.

---

### Task 2: Registry: the `platforms` field and the eleven rows

**Files:**
- Modify: `crates/foundation/src/registry.rs` (struct at lines 14-29, rows end at line 643, `IMMEDIATE_TIER_RULES` at 655-673)
- Modify: `crates/detect/tests/rule_pack.rs:13-22` (the test pack's row literal)
- Modify: any other `Antipattern {` literal found by the grep in Step 3
- Regenerate: `crates/live/assets/antipatterns.json`, `crates/live/assets/detect-antipatterns-browser.js`

**Interfaces:**
- Produces: `Antipattern.platforms: Option<&'static [&'static str]>`; `pub fn rule_runs_on_platform(ap: &Antipattern, platform: Option<&str>) -> bool`; the eleven ids below, all resolvable through `get_antipattern`.

- [ ] **Step 1: Write the failing tests** at the end of the existing `mod tests` in `registry.rs` (after the last `#[test]`, before the module's closing brace):

```rust
    const TUI_IDS: &[&str] = &[
        "tui-figlet-banner",
        "tui-gradient-title",
        "tui-blink-attribute",
        "tui-emoji-density",
        "tui-double-border",
        "tui-hardcoded-rgb-no-adapt",
        "tui-spinner-no-tty-guard",
        "tui-hardcoded-size",
        "tui-grapheme-unsafe-truncate",
        "tui-nerd-glyph-no-fallback",
        "tui-print-in-loop",
    ];

    #[test]
    fn terminal_rows_are_advisory_and_declare_the_terminal_platform() {
        for id in TUI_IDS {
            let ap = get_antipattern(id).unwrap_or_else(|| panic!("{id} missing"));
            assert_eq!(ap.severity, Some("advisory"), "{id}");
            assert_eq!(ap.platforms, Some(&["terminal"][..]), "{id}");
            assert!(ap.category == "slop" || ap.category == "quality", "{id}");
        }
    }

    #[test]
    fn web_rows_declare_no_platform_and_come_first() {
        let first_tui = ANTIPATTERNS.iter().position(|ap| ap.id.starts_with("tui-")).unwrap();
        assert_eq!(first_tui, 61, "web rows stay byte-identical: tui rows start after them");
        for ap in &ANTIPATTERNS[..first_tui] {
            assert_eq!(ap.platforms, None, "{}", ap.id);
        }
        assert_eq!(ANTIPATTERNS.len(), 72);
    }

    #[test]
    fn terminal_row_never_runs_on_a_web_project() {
        let tui = get_antipattern("tui-double-border").unwrap();
        let web = get_antipattern("side-tab").unwrap();
        assert!(!rule_runs_on_platform(tui, None));
        assert!(!rule_runs_on_platform(tui, Some("web")));
        assert!(!rule_runs_on_platform(tui, Some("ios")));
        assert!(rule_runs_on_platform(tui, Some("terminal")));
        assert!(rule_runs_on_platform(web, None));
        assert!(rule_runs_on_platform(web, Some("terminal")));
    }

    #[test]
    fn cheap_terminal_rules_join_the_immediate_tier() {
        for id in ["tui-figlet-banner", "tui-gradient-title", "tui-blink-attribute", "tui-double-border", "tui-hardcoded-rgb-no-adapt"] {
            assert!(IMMEDIATE_TIER_RULES.contains(&id), "{id}");
        }
        assert!(!IMMEDIATE_TIER_RULES.contains(&"tui-print-in-loop"));
    }
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p impeccable-foundation registry 2>&1 | tail -n 20`
Expected: compile error (`no field platforms`, `rule_runs_on_platform` not found).

- [ ] **Step 3: Add the field to the struct and to every existing literal**

Insert after the `severity` field (line 22) in `Antipattern`:

```rust
    /// Platforms the rule runs on. `None` is the web rulebook (every row the
    /// JS shipped). A terminal rule declares `Some(&["terminal"])`, and the
    /// text engine never runs it outside that platform
    /// ([`rule_runs_on_platform`]).
    pub platforms: Option<&'static [&'static str]>,
```

Then add `platforms: None,` after every `severity:` line in the 61 existing rows, and in every other `Antipattern { ... }` literal in the workspace:

```bash
perl -0pi -e 's/^(\s+)severity: ([^\n]*),\n/$1severity: $2,\n$1platforms: None,\n/mg' crates/foundation/src/registry.rs
grep -c "platforms: None" crates/foundation/src/registry.rs        # expect 61
grep -rln "Antipattern {" crates --include='*.rs' | grep -v registry.rs   # add `platforms: None,` after `severity` in each hit (crates/detect/tests/rule_pack.rs at least)
cargo build --workspace --all-targets 2>&1 | grep -E "^error" | head   # expect nothing
```

If the perl count is not 61, undo with `git checkout crates/foundation/src/registry.rs` and insert by hand; do not leave a row without the field.

- [ ] **Step 4: Add `rule_runs_on_platform`** directly after `pub fn get_antipattern` (find it with `grep -n "pub fn get_antipattern" crates/foundation/src/registry.rs`):

```rust
/// Whether `ap` runs when the resolved platform is `platform` (`None` means
/// web, the default when PRODUCT.md has no `## Platform`). Rows without
/// `platforms` are the web rulebook and run wherever a web file is scanned,
/// including a terminal project's `.html`; a row that names platforms runs
/// only on one of them.
pub fn rule_runs_on_platform(ap: &Antipattern, platform: Option<&str>) -> bool {
    match ap.platforms {
        None => true,
        Some(list) => platform.map(|p| list.contains(&p)).unwrap_or(false),
    }
}
```

- [ ] **Step 5: Append the eleven rows** immediately before the `];` that closes `ANTIPATTERNS` (line 643 before this task). Copy exactly; descriptions are user-facing and contain no em dashes:

```rust
    // Terminal source rules (spec docs/superpowers/specs/2026-09-24-terminal-platform-design.md,
    // section 4). All advisory until the manual pass promotes them; `platforms`
    // keeps them off web projects. New rows stay after this comment so the
    // web rulebook above remains byte-identical.
    Antipattern {
        id: "tui-figlet-banner",
        category: "slop",
        scopes: None,
        severity: Some("advisory"),
        platforms: Some(&["terminal"]),
        name: "FIGlet banner",
        description: "A block-letter banner (figlet, pyfiglet, ink-big-text, cfonts) or hand-drawn block glyphs at the top of a terminal UI is the most recognizable generated-CLI tell. Print the name once in plain text and spend the rows on the task.",
        skill_section: None,
        skill_guideline: None,
    },
    Antipattern {
        id: "tui-gradient-title",
        category: "slop",
        scopes: None,
        severity: Some("advisory"),
        platforms: Some(&["terminal"]),
        name: "Gradient title",
        description: "A gradient across a title (gradient-string, ink-gradient, rich_gradient, lipgloss.Blend) is decoration the terminal cannot render consistently. Use one accent color.",
        skill_section: None,
        skill_guideline: None,
    },
    Antipattern {
        id: "tui-blink-attribute",
        category: "slop",
        scopes: None,
        severity: Some("advisory"),
        platforms: Some(&["terminal"]),
        name: "Blinking text",
        description: "The blink attribute (SLOW_BLINK, [blink], text-style: blink, ESC[5m) is unreadable, ignored by many emulators, and hostile to attention. Show state with color or a static marker.",
        skill_section: None,
        skill_guideline: None,
    },
    Antipattern {
        id: "tui-emoji-density",
        category: "slop",
        scopes: None,
        severity: Some("advisory"),
        platforms: Some(&["terminal"]),
        name: "Emoji as UI vocabulary",
        description: "Emoji used as status glyphs and bullets render at unpredictable widths and break column alignment. Use ASCII or a single-width symbol set with a plain fallback.",
        skill_section: None,
        skill_guideline: None,
    },
    Antipattern {
        id: "tui-double-border",
        category: "slop",
        scopes: None,
        severity: Some("advisory"),
        platforms: Some(&["terminal"]),
        name: "Double-line border",
        description: "Double-line box borders draw the eye to the frame instead of the content and stack badly when panels nest. Use a plain or rounded single border, and drop borders on inner panels.",
        skill_section: None,
        skill_guideline: None,
    },
    Antipattern {
        id: "tui-hardcoded-rgb-no-adapt",
        category: "slop",
        scopes: None,
        severity: Some("advisory"),
        platforms: Some(&["terminal"]),
        name: "Hardcoded truecolor without adaptation",
        description: "A literal RGB or hex color with no light/dark or NO_COLOR adaptation assumes one terminal theme. Use the 16 ANSI colors or an adaptive color helper.",
        skill_section: None,
        skill_guideline: None,
    },
    Antipattern {
        id: "tui-spinner-no-tty-guard",
        category: "quality",
        scopes: None,
        severity: Some("advisory"),
        platforms: Some(&["terminal"]),
        name: "Spinner without a TTY guard",
        description: "A spinner that runs when stdout is not a TTY floods logs and CI output with frames. Check isatty (or a CI / --no-progress flag) and print a static line instead.",
        skill_section: None,
        skill_guideline: None,
    },
    Antipattern {
        id: "tui-hardcoded-size",
        category: "quality",
        scopes: None,
        severity: Some("advisory"),
        platforms: Some(&["terminal"]),
        name: "Hardcoded terminal size",
        description: "Fixed 80x24 assumptions and all-Length layouts do not resize with the terminal. Read the size at render time and use flexible constraints.",
        skill_section: None,
        skill_guideline: None,
    },
    Antipattern {
        id: "tui-grapheme-unsafe-truncate",
        category: "quality",
        scopes: None,
        severity: Some("advisory"),
        platforms: Some(&["terminal"]),
        name: "Byte or char truncation before an ellipsis",
        description: "Slicing a string by bytes or chars before an ellipsis splits wide characters and grapheme clusters. Truncate by display width (unicode-width, runewidth, string-width, wcwidth).",
        skill_section: None,
        skill_guideline: None,
    },
    Antipattern {
        id: "tui-nerd-glyph-no-fallback",
        category: "quality",
        scopes: None,
        severity: Some("advisory"),
        platforms: Some(&["terminal"]),
        name: "Nerd Font glyph without a fallback",
        description: "Private-use glyphs render as boxes without a patched font. Offer an ASCII or plain icon set behind an option.",
        skill_section: None,
        skill_guideline: None,
    },
    Antipattern {
        id: "tui-print-in-loop",
        category: "quality",
        scopes: None,
        severity: Some("advisory"),
        platforms: Some(&["terminal"]),
        name: "Console output inside a TUI render loop",
        description: "print or console.log inside a running full-screen app corrupts the frame. Log to a file or use the framework's log or patchConsole facility.",
        skill_section: None,
        skill_guideline: None,
    },
```

- [ ] **Step 6: Extend `IMMEDIATE_TIER_RULES`** by appending before its closing `];`:

```rust
    // Terminal single-property slop, cheap to fix at the edit site. Advisory
    // today, so the hook surfaces them only with `hook.advisoryRules: include`;
    // listed now so promotion is a severity change only.
    "tui-figlet-banner",
    "tui-gradient-title",
    "tui-blink-attribute",
    "tui-double-border",
    "tui-hardcoded-rgb-no-adapt",
```

- [ ] **Step 7: Run the tests**

Run: `cargo test -p impeccable-foundation 2>&1 | grep -E "^test result|FAILED|panicked"`
Expected: all pass. Then `cargo test --workspace 2>&1 | grep -E "^test result" | grep -v " 0 failed" ; echo "non-green lines above, expect none"`. The `crates/core` vector replay (`vectors.rs`) must still pass: the frozen vectors do not carry `platforms`, so a failure there means the field changed serialization; it must not (the field is not serialized anywhere).

- [ ] **Step 8: Regenerate the two tracked assets**

```bash
cargo xtask bundle 2>&1 | tail -n 3
node -e "const r=JSON.parse(require('fs').readFileSync('crates/live/assets/antipatterns.json','utf8'));console.log(r.length, r.slice(-11).map(x=>x.id).join(' '))"   # 72 and the eleven tui ids
cargo xtask bundle --check && echo BUNDLE_FRESH
git status --short crates/live/assets   # both files modified
```

`bun run build` will now fail on the README counts (61 vs 72); that is expected until Task 9 and is not fixed here.

- [ ] **Step 9: Commit**

```bash
git add crates/foundation/src/registry.rs crates/detect/tests/rule_pack.rs crates/live/assets/antipatterns.json crates/live/assets/detect-antipatterns-browser.js
git add $(grep -rln "platforms: None" crates --include='*.rs' | grep -v registry.rs)
git commit -m "$(printf 'Registry: platforms field and the eleven terminal source rows\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 3: `checks/terminal.rs` foundations: stacks, signals, comment blanking

**Files:**
- Create: `crates/core/src/checks/terminal.rs`
- Modify: `crates/core/src/checks/mod.rs` (add `pub mod terminal;` after `pub mod text_rules;` on line 46)

**Interfaces:**
- Consumes: `crate::findings::{finding, Finding}` (foundation, re-exported by core); `regex`, `once_cell`, `serde_json` (already core dependencies).
- Produces (used by Tasks 4 to 8):
  - `pub const TERMINAL_EXTENSIONS: &[&str]` = `.rs .go .py .tcss`
  - `pub const SIGNALS_NOTE: &str`
  - `pub enum Stack { Ratatui, Charm, Textual, TextualCss, Ink }` (Copy, Debug, PartialEq, Eq)
  - `pub fn classify_terminal_source(ext: &str, text: &str) -> Option<Stack>` (`ext` lowercase with the dot)
  - `pub struct ProjectSignals { has_tty_guard, has_adaptive_color, has_width_lib, has_icon_toggle, has_plain_flag: bool }` (Copy, Default) with `pub fn absorb(&mut self, text: &str)` and `pub fn collect<'a>(texts: impl IntoIterator<Item = &'a str>) -> ProjectSignals`
  - `pub fn strip_c_comments(text: &str) -> String`, `pub fn strip_python_comments(text: &str) -> String` (line count preserved)
  - `pub fn scan_terminal_source(source: &str, file_path: &str, stack: Stack, signals: Option<&ProjectSignals>) -> Vec<Finding>` (empty until Tasks 4 and 5)
  - private helpers `hit`, `with_note`, `with_count`, and the `re!` macro

- [ ] **Step 1: Create the module with its tests** as `crates/core/src/checks/terminal.rs`:

```rust
//! Terminal source rules (the `tui-` registry rows): pure text scans over TUI
//! and rich-CLI source. `impeccable detect` runs them only on a `terminal`
//! project and only over files that name a terminal framework
//! ([`classify_terminal_source`]); the browser adapters never call this module.
//!
//! Every scan takes comment-blanked text (line count preserved, so line
//! numbers hold) and the [`ProjectSignals`] the walker collected. `None`
//! signals means a single-file scan, and the absence rules then report with
//! [`SIGNALS_NOTE`] instead of deciding.

use crate::findings::{finding, Finding};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

macro_rules! re {
    ($name:ident, $pat:expr) => {
        static $name: Lazy<Regex> = Lazy::new(|| Regex::new(&$pat).expect(stringify!($name)));
    };
}

/// The extensions the walker adds on a terminal project. `.tsx`, `.jsx`,
/// `.ts`, `.js` are already web-scannable and carry Ink components.
pub const TERMINAL_EXTENSIONS: &[&str] = &[".rs", ".go", ".py", ".tcss"];

/// The note an absence rule carries when no project signals were collected.
pub const SIGNALS_NOTE: &str =
    "project signals were not collected; scan the project directory to confirm";

/// The terminal stack a source file belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stack {
    /// Rust: ratatui or crossterm.
    Ratatui,
    /// Go: any charmbracelet import (Bubble Tea, Lip Gloss, Bubbles).
    Charm,
    /// Python: textual or rich.
    Textual,
    /// Textual CSS (`.tcss`), always terminal source.
    TextualCss,
    /// TypeScript or JavaScript that imports ink.
    Ink,
}

re!(PY_TEXTUAL_RE, r"(?m)^[ \t]*(?:from|import)[ \t]+(?:textual|rich)(?:[ \t.]|$)");
re!(INK_IMPORT_RE, r#"(?:from[ \t]+|require\([ \t]*)["']ink(?:-[a-z0-9-]+)?["']"#);

/// Spec section 3 framework table. A file that fails this gets no terminal
/// rules, which keeps Bevy games and Django backends out.
pub fn classify_terminal_source(ext: &str, text: &str) -> Option<Stack> {
    match ext {
        ".rs" if text.contains("ratatui") || text.contains("crossterm") => Some(Stack::Ratatui),
        ".go" if text.contains("charmbracelet") => Some(Stack::Charm),
        ".py" if PY_TEXTUAL_RE.is_match(text) => Some(Stack::Textual),
        ".tcss" => Some(Stack::TextualCss),
        ".tsx" | ".jsx" | ".ts" | ".js" | ".mjs" | ".cjs" if INK_IMPORT_RE.is_match(text) => {
            Some(Stack::Ink)
        }
        _ => None,
    }
}

re!(
    TTY_GUARD_RE,
    r#"isatty|isTTY|is_terminal|IsTerminal|--no-progress|(?:env::var|getenv|Getenv|process\.env\[|os\.environ(?:\.get)?)[ \t]*[\[(]?[ \t]*["']CI["']|process\.env\.CI\b"#
);
re!(ADAPTIVE_COLOR_RE, r"AdaptiveColor|LightDark|HasDarkBackground|ansi_color|NO_COLOR");
re!(
    WIDTH_LIB_RE,
    r"unicode[-_]width|unicode[-_]segmentation|runewidth|uniseg|ansi\.Truncate|string-width|wcwidth|grapheme"
);
re!(
    ICON_TOGGLE_RE,
    r#"--no-icons|["'](?:--)?(?:ascii|nerd|icons)["']|\b(?:ascii|nerd|icons)[ \t]*[:=]"#
);
re!(PLAIN_FLAG_RE, r"--plain|--json|--no-tui|--static|NO_TUI");

/// Spec section 3 "Project signals": booleans collected over every scanned
/// file before the source rules run. Half the rules are "X present and Y
/// absent anywhere in the project"; this is the Y.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProjectSignals {
    pub has_tty_guard: bool,
    pub has_adaptive_color: bool,
    pub has_width_lib: bool,
    pub has_icon_toggle: bool,
    pub has_plain_flag: bool,
}

impl ProjectSignals {
    /// Fold one file's raw text (comments included: a `// NO_COLOR` mention
    /// still tells us the author knows the convention) into the set.
    pub fn absorb(&mut self, text: &str) {
        self.has_tty_guard |= TTY_GUARD_RE.is_match(text);
        self.has_adaptive_color |= ADAPTIVE_COLOR_RE.is_match(text);
        self.has_width_lib |= WIDTH_LIB_RE.is_match(text);
        self.has_icon_toggle |= ICON_TOGGLE_RE.is_match(text);
        self.has_plain_flag |= PLAIN_FLAG_RE.is_match(text);
    }

    pub fn collect<'a>(texts: impl IntoIterator<Item = &'a str>) -> ProjectSignals {
        let mut s = ProjectSignals::default();
        for t in texts {
            s.absorb(t);
        }
        s
    }
}

/// Blank `//` and `/* */` comments in Rust, Go, or Textual CSS, keeping every
/// newline so line numbers hold. String literals (`"..."`, Go backtick raw
/// strings) and Rust char literals are copied verbatim, so `"http://x"` and
/// `'/'` survive.
pub fn strip_c_comments(text: &str) -> String {
    let c: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let n = c.len();
    let mut i = 0;
    while i < n {
        let ch = c[i];
        if ch == '/' && i + 1 < n && c[i + 1] == '/' {
            while i < n && c[i] != '\n' {
                out.push(' ');
                i += 1;
            }
        } else if ch == '/' && i + 1 < n && c[i + 1] == '*' {
            out.push_str("  ");
            i += 2;
            while i < n && !(c[i] == '*' && i + 1 < n && c[i + 1] == '/') {
                out.push(if c[i] == '\n' { '\n' } else { ' ' });
                i += 1;
            }
            if i < n {
                out.push_str("  ");
                i += 2;
            }
        } else if ch == '"' || ch == '`' {
            let q = ch;
            out.push(q);
            i += 1;
            while i < n && c[i] != q {
                if c[i] == '\\' && q == '"' && i + 1 < n {
                    out.push(c[i]);
                    out.push(c[i + 1]);
                    i += 2;
                    continue;
                }
                out.push(c[i]);
                i += 1;
            }
            if i < n {
                out.push(q);
                i += 1;
            }
        } else if ch == '\'' {
            // `'x'` or `'\n'` is a char literal; a lone `'` (a Rust lifetime)
            // is code. Copy the literal through its closing quote.
            let close = if i + 1 < n && c[i + 1] == '\\' { i + 3 } else { i + 2 };
            if close < n && c[close] == '\'' {
                for k in i..=close {
                    out.push(c[k]);
                }
                i = close + 1;
            } else {
                out.push(ch);
                i += 1;
            }
        } else {
            out.push(ch);
            i += 1;
        }
    }
    out
}

/// Blank `#` comments and bare docstrings in Python, keeping every newline.
/// A triple-quoted literal is blanked only when the quote (optionally after a
/// prefix such as `r` or `f`) is the first token on its line: that is a
/// docstring statement. `CSS = """..."""` assignments are kept, because
/// Textual apps carry their stylesheet there and the rules must see it.
pub fn strip_python_comments(text: &str) -> String {
    let c: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let n = c.len();
    let mut i = 0;
    let mut line_start = 0;
    while i < n {
        let ch = c[i];
        if ch == '\n' {
            out.push('\n');
            i += 1;
            line_start = i;
        } else if ch == '#' {
            while i < n && c[i] != '\n' {
                out.push(' ');
                i += 1;
            }
        } else if ch == '"' || ch == '\'' {
            let q = ch;
            let triple = i + 2 < n && c[i + 1] == q && c[i + 2] == q;
            if triple {
                let prefix: String = c[line_start..i].iter().collect();
                let is_docstring = prefix
                    .trim_start()
                    .chars()
                    .all(|p| matches!(p, 'r' | 'R' | 'b' | 'B' | 'f' | 'F' | 'u' | 'U'));
                out.push_str(&q.to_string().repeat(3));
                i += 3;
                while i < n && !(c[i] == q && i + 2 < n && c[i + 1] == q && c[i + 2] == q) {
                    out.push(if is_docstring && c[i] != '\n' { ' ' } else { c[i] });
                    i += 1;
                }
                if i < n {
                    out.push_str(&q.to_string().repeat(3));
                    i += 3;
                }
            } else {
                out.push(q);
                i += 1;
                while i < n && c[i] != q && c[i] != '\n' {
                    if c[i] == '\\' && i + 1 < n {
                        out.push(c[i]);
                        out.push(c[i + 1]);
                        i += 2;
                        continue;
                    }
                    out.push(c[i]);
                    i += 1;
                }
                if i < n && c[i] == q {
                    out.push(q);
                    i += 1;
                }
            }
        } else {
            out.push(ch);
            i += 1;
        }
    }
    out
}

fn hit(id: &str, file_path: &str, lines: &[&str], idx: usize) -> Finding {
    finding(id, file_path, lines[idx].trim(), (idx + 1) as f64)
}

fn with_note(mut f: Finding, note: &str) -> Finding {
    f.extras.insert("note".into(), Value::String(note.into()));
    f
}

fn with_count(mut f: Finding, key: &str, n: usize) -> Finding {
    f.extras.insert(key.into(), Value::from(n as u64));
    f
}

/// Run every terminal rule over one file. `source` is the file text with
/// comments already blanked (Ink files use the detect crate's JS stripper,
/// Python files [`strip_python_comments`], the rest [`strip_c_comments`]).
pub fn scan_terminal_source(
    source: &str,
    file_path: &str,
    stack: Stack,
    signals: Option<&ProjectSignals>,
) -> Vec<Finding> {
    let lines: Vec<&str> = source.split('\n').collect();
    let _ = (&lines, file_path, stack, signals);
    // Rules are appended here by the slop (Task 4) and quality (Task 5) tasks.
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framework_detection_follows_the_spec_table() {
        assert_eq!(classify_terminal_source(".rs", "use ratatui::prelude::*;"), Some(Stack::Ratatui));
        assert_eq!(classify_terminal_source(".rs", "use crossterm::event;"), Some(Stack::Ratatui));
        assert_eq!(classify_terminal_source(".go", "import tea \"github.com/charmbracelet/bubbletea\""), Some(Stack::Charm));
        assert_eq!(classify_terminal_source(".py", "from textual.app import App"), Some(Stack::Textual));
        assert_eq!(classify_terminal_source(".py", "import rich\n"), Some(Stack::Textual));
        assert_eq!(classify_terminal_source(".tcss", "Screen { }"), Some(Stack::TextualCss));
        assert_eq!(classify_terminal_source(".tsx", "import { Box } from 'ink';"), Some(Stack::Ink));
        assert_eq!(classify_terminal_source(".js", "const { render } = require(\"ink\");"), Some(Stack::Ink));
        assert_eq!(classify_terminal_source(".tsx", "import Spinner from 'ink-spinner';"), Some(Stack::Ink));
    }

    #[test]
    fn bevy_game_and_django_backend_are_not_terminal_source() {
        assert_eq!(classify_terminal_source(".rs", "use bevy::prelude::*; let c = Color::Rgb(1,2,3);"), None);
        assert_eq!(classify_terminal_source(".py", "from django.db import models\nprint('x')"), None);
        assert_eq!(classify_terminal_source(".py", "enrich = richness(3)"), None, "rich as a substring is not an import");
        assert_eq!(classify_terminal_source(".go", "package main\nimport \"fmt\""), None);
        assert_eq!(classify_terminal_source(".tsx", "import React from 'react'; // ink is a word"), None);
        assert_eq!(classify_terminal_source(".html", "<b>ratatui</b>"), None);
    }

    #[test]
    fn project_signals_fold_across_files() {
        let s = ProjectSignals::collect(["fn main() {}", "if std::io::stdout().is_terminal() {}", "use unicode_width::UnicodeWidthStr;"]);
        assert!(s.has_tty_guard);
        assert!(s.has_width_lib);
        assert!(!s.has_adaptive_color);
        assert!(!s.has_icon_toggle);
        assert!(!s.has_plain_flag);
        let s = ProjectSignals::collect(["if os.environ.get(\"CI\"): quiet()", "style = lipgloss.AdaptiveColor{}", "flags.Bool(\"no-icons\", ...)  // --no-icons", "--json"]);
        assert!(s.has_tty_guard && s.has_adaptive_color && s.has_icon_toggle && s.has_plain_flag);
    }

    #[test]
    fn c_comment_blanking_keeps_lines_and_strings() {
        let src = "let url = \"http://x\"; // Color::Rgb(1,2,3)\n/* BorderType::Double\n   spans */ let c = '/';\nlet ok = 1;\n";
        let out = strip_c_comments(src);
        assert_eq!(out.matches('\n').count(), src.matches('\n').count());
        assert!(out.contains("\"http://x\""));
        assert!(out.contains("'/'"));
        assert!(!out.contains("Color::Rgb"));
        assert!(!out.contains("BorderType::Double"));
        assert!(out.contains("let ok = 1;"));
        let go = "s := `raw // not a comment`\n// real\n";
        assert!(strip_c_comments(go).contains("raw // not a comment"));
        assert!(!strip_c_comments(go).contains("real"));
        let lifetime = "fn f<'a>(s: &'a str) -> &'a str { s } // BorderType::Double\n";
        assert!(!strip_c_comments(lifetime).contains("BorderType::Double"), "a lifetime tick is not a char literal");
    }

    #[test]
    fn python_blanking_keeps_the_css_block_but_drops_docstrings_and_hashes() {
        let src = "class A(App):\n    \"\"\"A [blink] docstring.\"\"\"\n    CSS = \"\"\"\n    Screen { border: double; }\n    \"\"\"\n    x = 1  # text-style: blink\n    y = \"# not a comment\"\n";
        let out = strip_python_comments(src);
        assert_eq!(out.matches('\n').count(), src.matches('\n').count());
        assert!(!out.contains("[blink] docstring"));
        assert!(out.contains("border: double"));
        assert!(!out.contains("text-style: blink"));
        assert!(out.contains("\"# not a comment\""));
    }

    #[test]
    fn empty_dispatcher_returns_nothing_yet() {
        assert!(scan_terminal_source("x", "a.rs", Stack::Ratatui, None).is_empty());
    }
}
```

- [ ] **Step 2: Register the module.** In `crates/core/src/checks/mod.rs`, after `pub mod text_rules;` add `pub mod terminal;`.

- [ ] **Step 3: Run the tests**

Run: `cargo test -p impeccable-core checks::terminal 2>&1 | grep -E "^test |^test result|panicked"`
Expected: 6 tests pass. If `bevy_game_and_django_backend_are_not_terminal_source` fails on the `richness` line, tighten `PY_TEXTUAL_RE`; do not loosen the test.

- [ ] **Step 4: Make sure the wasm build still compiles** (the module is inside `impeccable-core`, which the bundle links):

Run: `cargo build -p impeccable-wasm --target wasm32-unknown-unknown 2>&1 | tail -n 2` (skip with a ledger note if the target is not installed; `cargo xtask bundle --check` in Task 9 covers it).

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/checks/terminal.rs crates/core/src/checks/mod.rs
git commit -m "$(printf 'core: terminal source-rule foundations (stacks, signals, comment blanking)\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 4: The six slop rules

**Files:**
- Modify: `crates/core/src/checks/terminal.rs` (add the rule functions above `#[cfg(test)]`, fill the dispatcher, add tests)

**Interfaces:**
- Consumes: `hit`, `with_note`, `with_count`, `SIGNALS_NOTE`, `Stack`, `ProjectSignals`, the `re!` macro from Task 3.
- Produces: `scan_terminal_figlet_banner`, `scan_terminal_gradient_title`, `scan_terminal_blink_attribute`, `scan_terminal_emoji_density`, `scan_terminal_double_border`, `scan_terminal_hardcoded_rgb` (all private, called from `scan_terminal_source`); `pub const ADAPTIVE_PRESENT_NOTE`.

- [ ] **Step 1: Write the failing tests** inside `mod tests`:

```rust
    fn ids(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|f| f.antipattern.as_str()).collect()
    }
    fn scan(src: &str, stack: Stack, signals: Option<&ProjectSignals>) -> Vec<Finding> {
        scan_terminal_source(src, "app.rs", stack, signals)
    }
    const ALL: ProjectSignals = ProjectSignals {
        has_tty_guard: true,
        has_adaptive_color: true,
        has_width_lib: true,
        has_icon_toggle: true,
        has_plain_flag: true,
    };
    const NONE: ProjectSignals = ProjectSignals {
        has_tty_guard: false,
        has_adaptive_color: false,
        has_width_lib: false,
        has_icon_toggle: false,
        has_plain_flag: false,
    };

    #[test]
    fn figlet_import_and_hand_drawn_banner_are_flagged() {
        let src = "use figlet_rs::FIGfont;\nlet a = \"███╗   ███╗\";\nlet b = \"████╗ ████║\";\nlet c = \"██╔████╔██║\";\nlet d = \"██║╚██╔╝██║\";\nlet e = 1;\n";
        let f = scan(src, Stack::Ratatui, Some(&ALL));
        let lines: Vec<f64> = f.iter().filter(|f| f.antipattern == "tui-figlet-banner").map(|f| f.line).collect();
        assert_eq!(lines, vec![1.0, 2.0], "{f:?}");
        assert_eq!(f[1].extras.get("bannerLines"), Some(&Value::from(4u64)));
    }

    #[test]
    fn one_box_drawing_line_is_not_a_banner() {
        let src = "let top = \"╔══════╗\";\nlet mid = \"║ text ║\";\nlet ok = render();\n";
        assert!(ids(&scan(src, Stack::Ratatui, Some(&ALL))).is_empty());
    }

    #[test]
    fn gradient_title_is_flagged_and_blend_needs_title_context() {
        let src = "import Gradient from 'ink-gradient';\nconst t = <Gradient name=\"rainbow\">Hi</Gradient>;\n";
        assert_eq!(ids(&scan(src, Stack::Ink, Some(&ALL))), vec!["tui-gradient-title", "tui-gradient-title"]);
        let go = "title := lipgloss.NewStyle().Foreground(lipgloss.Blend1D(0.5, a, b))\nbar := lipgloss.Blend1D(0.2, a, b)\nheaderStyle := lipgloss.NewStyle().\n\tForeground(lipgloss.Blend1D(0.5, a, b))\n";
        let f = scan(go, Stack::Charm, Some(&ALL));
        let lines: Vec<f64> = f.iter().map(|f| f.line).collect();
        assert_eq!(ids(&f), vec!["tui-gradient-title", "tui-gradient-title"]);
        assert_eq!(lines, vec![1.0, 4.0], "line 2 follows a title but is its own statement");
        assert_eq!(ids(&scan("use tui_gradient::gradient;\n", Stack::Ratatui, Some(&ALL))), vec!["tui-gradient-title"]);
    }

    #[test]
    fn blink_is_flagged_in_every_stack_spelling() {
        for (src, stack) in [
            ("Style::default().add_modifier(Modifier::SLOW_BLINK)", Stack::Ratatui),
            ("lipgloss.NewStyle().Blink(true)", Stack::Charm),
            ("console.print(\"[blink]go[/blink]\")", Stack::Textual),
            ("Label { text-style: bold blink; }", Stack::TextualCss),
            ("process.stdout.write(\"\\x1b[5mHi\")", Stack::Ink),
        ] {
            assert_eq!(ids(&scan(src, stack, Some(&ALL))), vec!["tui-blink-attribute"], "{src}");
        }
        assert!(scan("Label { text-style: bold; }", Stack::TextualCss, Some(&ALL)).is_empty());
    }

    #[test]
    fn emoji_density_reports_once_per_file_with_a_count() {
        let many = "let s = [\"🚀\", \"✅\", \"❌\", \"🎉\", \"📦\", \"🔧\", \"🔥\", \"💡\", \"✨\"];\nlet t = 1;\n";
        let f = scan(many, Stack::Ratatui, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-emoji-density"]);
        assert_eq!(f[0].extras.get("emojiCount"), Some(&Value::from(9u64)));
        let canonical = "print(\"🚀 start\")\nprint(\"✅ done\")\nprint(\"❌ fail\")\n";
        assert_eq!(ids(&scan(canonical, Stack::Textual, Some(&ALL))), vec!["tui-emoji-density"]);
        let few = "print(\"🚀 start\")\nprint(\"done\")\n";
        assert!(scan(few, Stack::Textual, Some(&ALL)).is_empty());
    }

    #[test]
    fn double_border_is_flagged_and_single_border_passes() {
        for (src, stack) in [
            ("Block::default().border_type(BorderType::Double)", Stack::Ratatui),
            ("lipgloss.NewStyle().Border(lipgloss.DoubleBorder())", Stack::Charm),
            ("Panel(x, box=box.DOUBLE)", Stack::Textual),
            ("#main { border: double $accent; }", Stack::TextualCss),
            ("<Box borderStyle=\"double\">", Stack::Ink),
        ] {
            assert_eq!(ids(&scan(src, stack, Some(&ALL))), vec!["tui-double-border"], "{src}");
        }
        assert!(scan("Block::default().border_type(BorderType::Rounded)", Stack::Ratatui, Some(&ALL)).is_empty());
        assert!(scan("#main { border: round $accent; }", Stack::TextualCss, Some(&ALL)).is_empty());
    }

    #[test]
    fn hardcoded_truecolor_reports_plainly_without_adaptation_and_with_a_note_otherwise() {
        let src = "let c = Color::Rgb(255, 0, 128);\n";
        let plain = scan(src, Stack::Ratatui, Some(&NONE));
        assert_eq!(ids(&plain), vec!["tui-hardcoded-rgb-no-adapt"]);
        assert!(plain[0].extras.get("note").is_none());
        let adapted = scan(src, Stack::Ratatui, Some(&ALL));
        assert_eq!(adapted[0].extras.get("note"), Some(&Value::String(ADAPTIVE_PRESENT_NOTE.into())));
        let single = scan(src, Stack::Ratatui, None);
        assert_eq!(single[0].extras.get("note"), Some(&Value::String(SIGNALS_NOTE.into())));
        assert!(scan("#main { color: #ff0080; }", Stack::TextualCss, Some(&NONE)).is_empty(), "tcss is excluded");
        for (s, stack) in [
            ("lipgloss.Color(\"#ff0080\")", Stack::Charm),
            ("console.print(\"[#ff0080]x[/]\")", Stack::Textual),
            ("<Text color=\"#ff0080\">x</Text>", Stack::Ink),
        ] {
            assert_eq!(ids(&scan(s, stack, Some(&NONE))), vec!["tui-hardcoded-rgb-no-adapt"], "{s}");
        }
        assert!(scan("let c = Color::Red;", Stack::Ratatui, Some(&NONE)).is_empty());
    }
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p impeccable-core checks::terminal 2>&1 | grep -E "^test result|error\[|panicked" | head`
Expected: compile error (`ADAPTIVE_PRESENT_NOTE` missing) or assertion failures.

- [ ] **Step 3: Add the rules** above `#[cfg(test)]`, and replace the dispatcher body:

```rust
// ─── tui-figlet-banner ──────────────────────────────────────────────────────
re!(
    FIGLET_RE,
    r"(?i)\b(?:pyfiglet|figlet_rs|figlet|go-figure|cfonts|ink-big-text|text2art)\b|from[ \t]+art[ \t]+import"
);
const BLOCK_GLYPHS: &[char] = &['█', '▀', '▄', '╔', '═', '╗', '║', '╚', '╝'];

/// A line that carries a string literal whose visible characters are mostly
/// block glyphs: one row of a hand-drawn banner.
fn is_block_glyph_line(line: &str) -> bool {
    if !(line.contains('"') || line.contains('\'') || line.contains('`')) {
        return false;
    }
    let body: Vec<char> = line
        .chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '"' | '\'' | '`' | ',' | '+' | ';' | '(' | ')' | '[' | ']'))
        .collect();
    if body.len() < 6 {
        return false;
    }
    let glyphs = body.iter().filter(|c| BLOCK_GLYPHS.contains(c)).count();
    glyphs * 10 >= body.len() * 6
}

fn scan_terminal_figlet_banner(lines: &[&str], file_path: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        if FIGLET_RE.is_match(l) {
            out.push(hit("tui-figlet-banner", file_path, lines, i));
        }
    }
    let mut run = 0usize;
    for (i, l) in lines.iter().enumerate() {
        if is_block_glyph_line(l) {
            run += 1;
            if run == 4 {
                out.push(with_count(hit("tui-figlet-banner", file_path, lines, i + 1 - run), "bannerLines", run));
            }
        } else {
            run = 0;
        }
    }
    out.sort_by(|a, b| a.line.partial_cmp(&b.line).unwrap_or(std::cmp::Ordering::Equal));
    out
}

// ─── tui-gradient-title ─────────────────────────────────────────────────────
re!(
    GRADIENT_RE,
    r"gradient-string|ink-gradient|<Gradient\b|rich_gradient|\bGradient\(|\blolcat\b|\bchromacat\b|tui[-_]gradient"
);
re!(BLEND_RE, r"lipgloss\.Blend1D\(|\.Blend\(");
re!(TITLE_WORD_RE, r"(?i)title|header|banner");

fn scan_terminal_gradient_title(lines: &[&str], file_path: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        // A style chain may wrap: `titleStyle := lipgloss.NewStyle().` then
        // `Foreground(lipgloss.Blend1D(...))`. The previous line counts only
        // when it ends mid-expression, so a separate statement after a title
        // line is not a title.
        let continued = i > 0
            && lines[i - 1].trim_end().ends_with(|c: char| matches!(c, '.' | '(' | ',' | '{'));
        let blend_into_title = BLEND_RE.is_match(l)
            && (TITLE_WORD_RE.is_match(l) || (continued && TITLE_WORD_RE.is_match(lines[i - 1])));
        if GRADIENT_RE.is_match(l) || blend_into_title {
            out.push(hit("tui-gradient-title", file_path, lines, i));
        }
    }
    out
}

// ─── tui-blink-attribute ────────────────────────────────────────────────────
re!(
    BLINK_RE,
    r"Modifier::SLOW_BLINK|Modifier::RAPID_BLINK|\.Blink\(|\[blink\]|text-style:[^;\n]*\bblink\b|\\x1b\[5m|\\033\[5m|\\e\[5m|\x1b\[5m"
);

fn scan_terminal_blink_attribute(lines: &[&str], file_path: &str) -> Vec<Finding> {
    lines
        .iter()
        .enumerate()
        .filter(|(_, l)| BLINK_RE.is_match(l))
        .map(|(i, _)| hit("tui-blink-attribute", file_path, lines, i))
        .collect()
}

// ─── tui-emoji-density ──────────────────────────────────────────────────────
const EMOJI_FILE_THRESHOLD: usize = 8;
const CANONICAL_EMOJI_THRESHOLD: usize = 3;
const CANONICAL_EMOJI: &[char] = &['🚀', '✅', '❌', '⚠', '✨', '🎉', '📦', '🔧', '🔥', '💡'];

fn is_emoji(c: char) -> bool {
    matches!(c, '\u{1F300}'..='\u{1FAFF}' | '\u{2600}'..='\u{27BF}')
}

fn scan_terminal_emoji_density(lines: &[&str], file_path: &str) -> Vec<Finding> {
    let mut total = 0usize;
    let mut canonical = 0usize;
    let mut first: Option<usize> = None;
    for (i, l) in lines.iter().enumerate() {
        for c in l.chars().filter(|c| is_emoji(*c)) {
            total += 1;
            if CANONICAL_EMOJI.contains(&c) {
                canonical += 1;
            }
            first.get_or_insert(i);
        }
    }
    match first {
        Some(i) if total >= EMOJI_FILE_THRESHOLD || canonical >= CANONICAL_EMOJI_THRESHOLD => {
            vec![with_count(hit("tui-emoji-density", file_path, lines, i), "emojiCount", total)]
        }
        _ => Vec::new(),
    }
}

// ─── tui-double-border ──────────────────────────────────────────────────────
re!(
    DOUBLE_BORDER_RE,
    r#"BorderType::Double\b|DoubleBorder\(\)|borderStyle=\{?["']double["']|box\.DOUBLE\b|\bborder(?:-(?:top|right|bottom|left))?:[ \t]*double\b"#
);

fn scan_terminal_double_border(lines: &[&str], file_path: &str) -> Vec<Finding> {
    lines
        .iter()
        .enumerate()
        .filter(|(_, l)| DOUBLE_BORDER_RE.is_match(l))
        .map(|(i, _)| hit("tui-double-border", file_path, lines, i))
        .collect()
}

// ─── tui-hardcoded-rgb-no-adapt ─────────────────────────────────────────────
re!(
    RGB_RE,
    r#"Color::Rgb\(|Color::from_u32\(0x|lipgloss\.Color\("#|chalk\.hex\(|\bcolor=["']#|\[#[0-9a-fA-F]{6}\]|\\x1b\[38;2;|\\033\[38;2;|\x1b\[38;2;"#
);
/// The note `tui-hardcoded-rgb-no-adapt` carries when the project does have
/// an adaptive color helper somewhere: the literal may still be themed.
pub const ADAPTIVE_PRESENT_NOTE: &str =
    "an adaptive color helper exists elsewhere in this project; confirm this literal is themed";

fn scan_terminal_hardcoded_rgb(
    lines: &[&str],
    file_path: &str,
    stack: Stack,
    signals: Option<&ProjectSignals>,
) -> Vec<Finding> {
    if stack == Stack::TextualCss {
        // A stylesheet is where theme colors belong; the signal is weak there.
        return Vec::new();
    }
    lines
        .iter()
        .enumerate()
        .filter(|(_, l)| RGB_RE.is_match(l))
        .map(|(i, _)| {
            let f = hit("tui-hardcoded-rgb-no-adapt", file_path, lines, i);
            match signals {
                None => with_note(f, SIGNALS_NOTE),
                Some(s) if s.has_adaptive_color => with_note(f, ADAPTIVE_PRESENT_NOTE),
                Some(_) => f,
            }
        })
        .collect()
}
```

Dispatcher body (replace the placeholder in `scan_terminal_source`):

```rust
    let lines: Vec<&str> = source.split('\n').collect();
    let mut out = Vec::new();
    out.extend(scan_terminal_figlet_banner(&lines, file_path));
    out.extend(scan_terminal_gradient_title(&lines, file_path));
    out.extend(scan_terminal_blink_attribute(&lines, file_path));
    out.extend(scan_terminal_emoji_density(&lines, file_path));
    out.extend(scan_terminal_double_border(&lines, file_path));
    out.extend(scan_terminal_hardcoded_rgb(&lines, file_path, stack, signals));
    // Quality rules follow in Task 5.
    out
```

Delete the `empty_dispatcher_returns_nothing_yet` test.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p impeccable-core checks::terminal 2>&1 | grep -E "^test |^test result|panicked"`
Expected: all pass. A regex that fails to compile panics at first use with its static name; fix the pattern, not the test.

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/checks/terminal.rs
git commit -m "$(printf 'core: the six terminal slop rules\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 5: The five quality rules

**Files:**
- Modify: `crates/core/src/checks/terminal.rs` (rule functions, dispatcher, tests)

**Interfaces:**
- Consumes: Task 3 helpers and Task 4's test helpers (`ids`, `scan`, `ALL`, `NONE`).
- Produces: `scan_terminal_spinner`, `scan_terminal_hardcoded_size`, `scan_terminal_grapheme_truncate`, `scan_terminal_nerd_glyph`, `scan_terminal_print_in_loop` (private). `scan_terminal_source` is complete after this task.

- [ ] **Step 1: Write the failing tests** inside `mod tests`:

```rust
    #[test]
    fn spinner_without_tty_guard_is_flagged_and_a_guard_silences_it() {
        let src = "import Spinner from 'ink-spinner';\n";
        assert_eq!(ids(&scan(src, Stack::Ink, Some(&NONE))), vec!["tui-spinner-no-tty-guard"]);
        assert!(scan(src, Stack::Ink, Some(&ALL)).is_empty());
        assert!(scan("s := spinner.New()\n", Stack::Charm, Some(&NONE)).is_empty(), "the import path is what counts");
        assert_eq!(ids(&scan("import \"github.com/charmbracelet/bubbles/spinner\"\n", Stack::Charm, Some(&NONE))), vec!["tui-spinner-no-tty-guard"]);
    }

    #[test]
    fn hardcoded_terminal_size_is_flagged_outside_test_paths() {
        let src = "let area = Rect::new(0, 0, 80, 24);\nlet width = 80;\nlet rows = 24;\nlet w = term.Width(80)\n";
        let f = scan_terminal_source(src, "src/ui.rs", Stack::Ratatui, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-hardcoded-size"; 4]);
        assert!(scan_terminal_source(src, "tests/ui_test.rs", Stack::Ratatui, Some(&ALL)).is_empty());
        assert!(scan_terminal_source(src, "src/ui_test.rs", Stack::Ratatui, Some(&ALL)).is_empty());
        assert_eq!(scan_terminal_source(src, "tests/fixtures/antipatterns/terminal/x/x.rs", Stack::Ratatui, Some(&ALL)).len(), 4, "a fixture tree under tests/ still scans");
        assert!(scan("let width = 800;\nlet rows = 240;\n", Stack::Ratatui, Some(&ALL)).is_empty());
    }

    #[test]
    fn all_length_layout_is_flagged_once_and_a_flexible_constraint_passes() {
        let rigid = "Layout::vertical([\n    Constraint::Length(3),\n    Constraint::Length(10),\n    Constraint::Length(1),\n])\n";
        let f = scan(rigid, Stack::Ratatui, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-hardcoded-size"]);
        assert_eq!(f[0].line, 2.0);
        assert_eq!(f[0].extras.get("lengthConstraints"), Some(&Value::from(3u64)));
        let flexible = "Layout::vertical([\n    Constraint::Length(3),\n    Constraint::Min(0),\n    Constraint::Length(1),\n])\n";
        assert!(scan(flexible, Stack::Ratatui, Some(&ALL)).is_empty());
    }

    #[test]
    fn truncation_before_an_ellipsis_is_flagged_without_a_width_library() {
        for (src, stack) in [
            ("let short = format!(\"{}...\", &title[..20]);\n", Stack::Ratatui),
            ("let s = title.chars().take(20).collect::<String>();\nlet t = format!(\"{s}…\");\n", Stack::Ratatui),
            ("short := title[:20] + \"...\"\n", Stack::Charm),
            ("const short = title.slice(0, 20) + '...';\n", Stack::Ink),
        ] {
            assert_eq!(ids(&scan(src, stack, Some(&NONE))), vec!["tui-grapheme-unsafe-truncate"], "{src}");
            assert!(scan(src, stack, Some(&ALL)).is_empty(), "a width library silences it: {src}");
        }
        assert!(scan("let head = &buf[..4];\n", Stack::Ratatui, Some(&NONE)).is_empty(), "no ellipsis, no truncation for display");
        assert!(scan("short = title[:20] + '...'\n", Stack::Textual, Some(&NONE)).is_empty(), "Rich and Textual measure for the caller");
    }

    #[test]
    fn nerd_glyph_without_an_icon_toggle_reports_once() {
        let src = "const ICON = \"\u{e0a0}\";\nconst FOLDER = \"\u{f07b}\";\n";
        let f = scan(src, Stack::Ink, Some(&NONE));
        assert_eq!(ids(&f), vec!["tui-nerd-glyph-no-fallback"]);
        assert_eq!(f[0].line, 1.0);
        assert_eq!(f[0].extras.get("glyphCount"), Some(&Value::from(2u64)));
        assert!(scan(src, Stack::Ink, Some(&ALL)).is_empty());
    }

    #[test]
    fn print_inside_a_textual_widget_is_flagged_but_workers_and_module_level_pass() {
        let src = "from textual.app import App\nprint(\"module level is fine\")\n\nclass Demo(App):\n    def on_mount(self) -> None:\n        print(\"corrupts the frame\")\n        self.log(\"fine\")\n\n    @work(thread=True)\n    def fetch(self) -> None:\n        print(\"worker output is allowed\")\n\ndef helper():\n    print(\"outside the class\")\n";
        let f = scan(src, Stack::Textual, Some(&ALL));
        assert_eq!(ids(&f), vec!["tui-print-in-loop"]);
        assert_eq!(f[0].line, 6.0);
    }

    #[test]
    fn console_log_in_an_ink_component_is_flagged_unless_patch_console_is_used() {
        let src = "import { Box } from 'ink';\nconsole.log('debug');\n";
        assert_eq!(ids(&scan(src, Stack::Ink, Some(&ALL))), vec!["tui-print-in-loop"]);
        let patched = "import { Box } from 'ink';\nimport patchConsole from 'patch-console';\nconsole.log('debug');\n";
        assert!(scan(patched, Stack::Ink, Some(&ALL)).is_empty());
    }

    #[test]
    fn single_file_scan_downgrades_absence_rules() {
        let src = "import Spinner from 'ink-spinner';\nconst c = <Text color=\"#ff0080\">{title.slice(0, 8) + '...'}</Text>;\nconst i = \"\u{e0a0}\";\n";
        let f = scan(src, Stack::Ink, None);
        let noted: Vec<&str> = f.iter().filter(|f| f.extras.get("note") == Some(&Value::String(SIGNALS_NOTE.into()))).map(|f| f.antipattern.as_str()).collect();
        for id in ["tui-spinner-no-tty-guard", "tui-hardcoded-rgb-no-adapt", "tui-grapheme-unsafe-truncate", "tui-nerd-glyph-no-fallback"] {
            assert!(noted.contains(&id), "{id} should carry the note: {f:?}");
        }
        for f in &f {
            assert_eq!(f.severity, "advisory", "{}", f.antipattern);
        }
    }
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p impeccable-core checks::terminal 2>&1 | grep -E "^test result|panicked" | head`
Expected: the seven new tests fail (rules not run).

- [ ] **Step 3: Add the rules** above `#[cfg(test)]`:

```rust
// ─── tui-spinner-no-tty-guard ───────────────────────────────────────────────
re!(SPINNER_RE, r"ink-spinner|\bora\(|rich\.spinner|bubbles/spinner|briandowns/spinner");

fn scan_terminal_spinner(lines: &[&str], file_path: &str, signals: Option<&ProjectSignals>) -> Vec<Finding> {
    if matches!(signals, Some(s) if s.has_tty_guard) {
        return Vec::new();
    }
    lines
        .iter()
        .enumerate()
        .filter(|(_, l)| SPINNER_RE.is_match(l))
        .map(|(i, _)| {
            let f = hit("tui-spinner-no-tty-guard", file_path, lines, i);
            if signals.is_none() { with_note(f, SIGNALS_NOTE) } else { f }
        })
        .collect()
}

// ─── tui-hardcoded-size ─────────────────────────────────────────────────────
re!(
    RECT_LITERAL_RE,
    r"Rect::new\([ \t]*\d+[ \t]*,[ \t]*\d+[ \t]*,[ \t]*\d+[ \t]*,[ \t]*\d+[ \t]*\)"
);
re!(SIZE_80_RE, r"(?i)\b(?:width|cols|columns)[ \t]*[:=][ \t]*80\b");
re!(SIZE_24_RE, r"(?i)\b(?:height|rows|lines)[ \t]*[:=][ \t]*24\b");
re!(SIZE_CALL_RE, r"\.Width\(80\)|width=\{80\}|size=\(80,[ \t]*24\)");
re!(CONSTRAINT_LENGTH_RE, r"Constraint::Length\(");
re!(CONSTRAINT_FLEX_RE, r"Constraint::(?:Min|Max|Percentage|Ratio|Fill)\(");

/// Test code pins sizes on purpose (spec: "outside paths containing test").
/// Only the file name and its parent directory are consulted, so a fixture
/// tree under `tests/fixtures/` still scans (planning ruling 7).
fn in_test_path(file_path: &str) -> bool {
    let lower = file_path.to_lowercase().replace('\\', "/");
    lower.rsplit('/').take(2).any(|seg| seg.contains("test"))
}

fn scan_terminal_hardcoded_size(lines: &[&str], file_path: &str) -> Vec<Finding> {
    if in_test_path(file_path) {
        return Vec::new();
    }
    let mut out: Vec<Finding> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| {
            RECT_LITERAL_RE.is_match(l) || SIZE_80_RE.is_match(l) || SIZE_24_RE.is_match(l) || SIZE_CALL_RE.is_match(l)
        })
        .map(|(i, _)| hit("tui-hardcoded-size", file_path, lines, i))
        .collect();
    let lengths: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| CONSTRAINT_LENGTH_RE.is_match(l))
        .map(|(i, _)| i)
        .collect();
    if lengths.len() >= 2 && !lines.iter().any(|l| CONSTRAINT_FLEX_RE.is_match(l)) {
        out.push(with_count(hit("tui-hardcoded-size", file_path, lines, lengths[0]), "lengthConstraints", lengths.len()));
    }
    out.sort_by(|a, b| a.line.partial_cmp(&b.line).unwrap_or(std::cmp::Ordering::Equal));
    out
}

// ─── tui-grapheme-unsafe-truncate ───────────────────────────────────────────
re!(RUST_TRUNC_RE, r"&\w+\[\.\.[ \t]*\w+\]|\.chars\(\)\.take\(");
re!(GO_TRUNC_RE, r"\b\w+\[:\w+\]");
re!(JS_TRUNC_RE, r"\.slice\(0,|\.substring\(0,");
re!(ELLIPSIS_RE, r"\.\.\.|…|\\u\{2026\}|\\u2026");

/// Python is not matched: every `.py` file this module sees imports rich or
/// textual (framework detection), and both measure width for the caller.
fn scan_terminal_grapheme_truncate(
    lines: &[&str],
    file_path: &str,
    stack: Stack,
    signals: Option<&ProjectSignals>,
) -> Vec<Finding> {
    if matches!(signals, Some(s) if s.has_width_lib) {
        return Vec::new();
    }
    let re: &Regex = match stack {
        Stack::Ratatui => &RUST_TRUNC_RE,
        Stack::Charm => &GO_TRUNC_RE,
        Stack::Ink => &JS_TRUNC_RE,
        Stack::Textual | Stack::TextualCss => return Vec::new(),
    };
    let n = lines.len();
    lines
        .iter()
        .enumerate()
        .filter(|(i, l)| re.is_match(l) && (*i..(*i + 3).min(n)).any(|k| ELLIPSIS_RE.is_match(lines[k])))
        .map(|(i, _)| {
            let f = hit("tui-grapheme-unsafe-truncate", file_path, lines, i);
            if signals.is_none() { with_note(f, SIGNALS_NOTE) } else { f }
        })
        .collect()
}

// ─── tui-nerd-glyph-no-fallback ─────────────────────────────────────────────
fn is_private_use(c: char) -> bool {
    matches!(c, '\u{E000}'..='\u{F8FF}' | '\u{F0000}'..='\u{FFFFD}')
}

fn scan_terminal_nerd_glyph(lines: &[&str], file_path: &str, signals: Option<&ProjectSignals>) -> Vec<Finding> {
    if matches!(signals, Some(s) if s.has_icon_toggle) {
        return Vec::new();
    }
    let mut count = 0usize;
    let mut first: Option<usize> = None;
    for (i, l) in lines.iter().enumerate() {
        let here = l.chars().filter(|c| is_private_use(*c)).count();
        if here > 0 {
            count += here;
            first.get_or_insert(i);
        }
    }
    match first {
        Some(i) => {
            let f = with_count(hit("tui-nerd-glyph-no-fallback", file_path, lines, i), "glyphCount", count);
            vec![if signals.is_none() { with_note(f, SIGNALS_NOTE) } else { f }]
        }
        None => Vec::new(),
    }
}

// ─── tui-print-in-loop ──────────────────────────────────────────────────────
re!(
    TEXTUAL_CLASS_RE,
    r"^[ \t]*class[ \t]+\w+\([^)]*\b(?:App|Widget|Screen|ModalScreen|Static|Container)\b"
);
re!(PY_DEF_RE, r"^[ \t]*(?:async[ \t]+)?def[ \t]+\w+");
re!(PY_PRINT_RE, r"(?:^|[^.\w])print\(");
re!(INK_CONSOLE_LOG_RE, r"\bconsole\.log\(");

fn indent_of(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ' || *c == '\t').count()
}

fn scan_terminal_print_in_loop(lines: &[&str], file_path: &str, stack: Stack) -> Vec<Finding> {
    match stack {
        Stack::Ink => {
            if lines.iter().any(|l| l.contains("patchConsole") || l.contains("patch-console")) {
                return Vec::new();
            }
            lines
                .iter()
                .enumerate()
                .filter(|(_, l)| INK_CONSOLE_LOG_RE.is_match(l))
                .map(|(i, _)| hit("tui-print-in-loop", file_path, lines, i))
                .collect()
        }
        Stack::Textual => {
            let n = lines.len();
            let mut out = Vec::new();
            let mut i = 0;
            while i < n {
                if !TEXTUAL_CLASS_RE.is_match(lines[i]) {
                    i += 1;
                    continue;
                }
                let class_indent = indent_of(lines[i]);
                i += 1;
                let mut method_indent: Option<usize> = None;
                let mut in_worker = false;
                while i < n {
                    let l = lines[i];
                    let blank = l.trim().is_empty();
                    if !blank && indent_of(l) <= class_indent {
                        break;
                    }
                    if PY_DEF_RE.is_match(l) {
                        method_indent = Some(indent_of(l));
                        // Decorators sit directly above the def; `@work` marks a worker.
                        in_worker = false;
                        let mut k = i;
                        while k > 0 && lines[k - 1].trim_start().starts_with('@') {
                            k -= 1;
                            if lines[k].contains("work") {
                                in_worker = true;
                            }
                        }
                    } else if !blank
                        && !in_worker
                        && method_indent.map(|m| indent_of(l) > m).unwrap_or(false)
                        && PY_PRINT_RE.is_match(l)
                    {
                        out.push(hit("tui-print-in-loop", file_path, lines, i));
                    }
                    i += 1;
                }
            }
            out
        }
        Stack::Ratatui | Stack::Charm | Stack::TextualCss => Vec::new(),
    }
}
```

Replace the `// Quality rules follow in Task 5.` line in the dispatcher with:

```rust
    out.extend(scan_terminal_spinner(&lines, file_path, signals));
    out.extend(scan_terminal_hardcoded_size(&lines, file_path));
    out.extend(scan_terminal_grapheme_truncate(&lines, file_path, stack, signals));
    out.extend(scan_terminal_nerd_glyph(&lines, file_path, signals));
    out.extend(scan_terminal_print_in_loop(&lines, file_path, stack));
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p impeccable-core checks::terminal 2>&1 | grep -E "^test |^test result|panicked"`
Expected: all pass (19 tests in the module). Then `cargo test --workspace 2>&1 | grep -E "^test result" | grep -v " 0 failed"; echo "expect nothing above"`.

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/checks/terminal.rs
git commit -m "$(printf 'core: the five terminal quality rules\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 6: Text engine and walker: platform-gated terminal scanning

**Files:**
- Modify: `crates/detect/src/detect_text.rs` (`TextOptions` at lines 24-32; `detect_text` at 1402-1574)
- Modify: `crates/detect/src/file_system.rs` (constants at 15-36; `walk_dir_reporting` at 65-104)
- Modify: `crates/detect/src/engines.rs` (`ScanOptions` at 17-30; `Engines` at 89-92)
- Create: `crates/detect/tests/terminal.rs`

**Interfaces:**
- Consumes: `impeccable_core::checks::terminal::{classify_terminal_source, scan_terminal_source, strip_c_comments, strip_python_comments, ProjectSignals, Stack, TERMINAL_EXTENSIONS}`; `impeccable_core::registry::{get_antipattern, rule_runs_on_platform}`.
- Produces: `TextOptions { platform: Option<&'a str>, signals: Option<&'a ProjectSignals>, .. }`; `ScanOptions { platform: Option<String>, signals: Option<Rc<ProjectSignals>>, .. }`; `Engines { platform: Option<&'a dyn Fn(&str) -> Option<String>>, .. }`; `file_system::{TERMINAL_EXTENSIONS, TERMINAL_SKIP_DIRS, has_scannable_extension_for(name, platform), walk_dir_reporting_for(dir, platform, on_read_error)}`. `walk_dir_reporting(dir, cb)` keeps its signature and calls the new function with `None`.

- [ ] **Step 1: Write the failing integration tests** as `crates/detect/tests/terminal.rs`:

```rust
//! The platform gate around the terminal source rules: a web project never
//! sees a `tui-` finding, a terminal project scans terminal source per file,
//! and the walker adds the terminal extensions only on a terminal project.

use impeccable_core::checks::terminal::{ProjectSignals, ADAPTIVE_PRESENT_NOTE};
use impeccable_detect::detect_text::{detect_text, TextOptions};
use impeccable_detect::file_system::{walk_dir_reporting, walk_dir_reporting_for, TERMINAL_SKIP_DIRS};

const RATATUI_DOUBLE: &str = "use ratatui::widgets::{Block, BorderType};\n// BorderType::Double in a comment does not count\nfn ui() -> Block<'static> {\n    Block::default().border_type(BorderType::Double)\n}\n";

fn text(content: &str, path: &str, platform: Option<&str>, signals: Option<&ProjectSignals>) -> Vec<String> {
    detect_text(
        content,
        path,
        &TextOptions { inline_ignores: true, platform, signals, ..Default::default() },
    )
    .into_iter()
    .map(|f| format!("{}@{}", f.antipattern, f.line))
    .collect()
}

#[test]
fn web_project_with_rust_backend_gets_no_terminal_rules() {
    for platform in [None, Some("web"), Some("ios"), Some("adaptive")] {
        assert!(text(RATATUI_DOUBLE, "/app/src/ui.rs", platform, None).is_empty(), "{platform:?}");
    }
}

#[test]
fn terminal_project_flags_rust_ui_once_per_line() {
    assert_eq!(text(RATATUI_DOUBLE, "/app/src/ui.rs", Some("terminal"), None), vec!["tui-double-border@4"]);
}

#[test]
fn hardcoded_truecolor_flagged_only_in_tui_files() {
    let bevy = "use bevy::prelude::*;\nlet c = Color::Rgb(1.0, 0.5, 0.2);\n";
    assert!(text(bevy, "/game/src/main.rs", Some("terminal"), None).is_empty());
    let tui = "use ratatui::style::Color;\nlet c = Color::Rgb(255, 0, 128);\n";
    assert_eq!(text(tui, "/app/src/theme.rs", Some("terminal"), None), vec!["tui-hardcoded-rgb-no-adapt@2"]);
}

#[test]
fn ink_component_gets_terminal_rules_only_on_a_terminal_project() {
    let src = "import { Box, Text } from 'ink';\nexport const Card = () => <Box borderStyle=\"double\"><Text>hi</Text></Box>;\n";
    let ids = text(src, "/cli/src/Card.tsx", Some("terminal"), None);
    assert!(ids.contains(&"tui-double-border@2".to_string()), "{ids:?}");
    assert!(text(src, "/cli/src/Card.tsx", None, None).iter().all(|id| !id.starts_with("tui-")));
}

#[test]
fn terminal_findings_are_waivable_inline() {
    let src = "use ratatui::widgets::BorderType;\n// impeccable-disable-next-line tui-double-border\nlet b = BorderType::Double;\n";
    assert!(text(src, "/app/src/ui.rs", Some("terminal"), None).is_empty());
}

#[test]
fn textual_css_is_always_terminal_source() {
    assert_eq!(text("Screen { border: double $primary; }\n", "/app/app.tcss", Some("terminal"), None), vec!["tui-double-border@1"]);
    assert!(text("Screen { border: double $primary; }\n", "/app/app.tcss", None, None).is_empty());
}

#[test]
fn signals_reach_the_rules_through_text_options() {
    let src = "use ratatui::style::Color;\nlet c = Color::Rgb(1, 2, 3);\n";
    let adaptive = ProjectSignals { has_adaptive_color: true, ..Default::default() };
    let f = detect_text(
        src,
        "/app/src/theme.rs",
        &TextOptions { inline_ignores: true, platform: Some("terminal"), signals: Some(&adaptive), ..Default::default() },
    );
    assert_eq!(f[0].extras.get("note").and_then(|v| v.as_str()), Some(ADAPTIVE_PRESENT_NOTE));
}

fn scratch(name: &str) -> String {
    let dir = std::env::temp_dir().join(format!("impeccable-terminal-walk-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for rel in [
        "src/ui.rs", "src/app.py", "src/main.go", "src/app.tcss", "src/Card.tsx", "src/page.html",
        "target/debug/x.rs", "vendor/lib.go", "node_modules/m/index.js",
    ] {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "").unwrap();
    }
    dir.to_string_lossy().into_owned()
}

fn names(files: Vec<String>, root: &str) -> Vec<String> {
    let mut v: Vec<String> = files.iter().map(|f| f[root.len() + 1..].replace('\\', "/")).collect();
    v.sort();
    v
}

#[test]
fn web_walk_is_unchanged() {
    let root = scratch("web");
    assert_eq!(names(walk_dir_reporting(&root, &mut |_, _| {}), &root), vec!["src/Card.tsx", "src/page.html"]);
    assert_eq!(names(walk_dir_reporting_for(&root, Some("web"), &mut |_, _| {}), &root), vec!["src/Card.tsx", "src/page.html"]);
}

#[test]
fn terminal_walk_adds_the_terminal_extensions_and_skips_build_dirs() {
    let root = scratch("tui");
    assert_eq!(
        names(walk_dir_reporting_for(&root, Some("terminal"), &mut |_, _| {}), &root),
        vec!["src/Card.tsx", "src/app.py", "src/app.tcss", "src/main.go", "src/page.html", "src/ui.rs"]
    );
    assert_eq!(TERMINAL_SKIP_DIRS, &["target", ".venv", "vendor"]);
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p impeccable-detect --test terminal 2>&1 | grep -E "^error|^test result" | head`
Expected: compile errors (`platform`, `signals`, `walk_dir_reporting_for` unknown).

- [ ] **Step 3: `TextOptions` and the terminal branch** in `detect_text.rs`.

Add to the imports at the top of the file:

```rust
use impeccable_core::checks::terminal::{
    classify_terminal_source, scan_terminal_source, strip_c_comments, strip_python_comments,
    ProjectSignals, Stack, TERMINAL_EXTENSIONS,
};
use impeccable_core::registry::{get_antipattern, rule_runs_on_platform};
```

Extend `TextOptions` (after `rule_pack`):

```rust
    /// The resolved platform. `Some("terminal")` turns the terminal source
    /// rules on; anything else, including `None`, is the web rulebook only.
    pub platform: Option<&'a str>,
    /// Project signals the walker collected over a directory target;
    /// `None` in a single-file scan (absence rules then carry a note).
    pub signals: Option<&'a ProjectSignals>,
```

Add this function directly above `pub fn detect_text`:

```rust
/// The terminal half of `detect_text`: per-file framework detection, the
/// language's comment blanking, the `tui-` scans, and the registry platform
/// filter (a `tui-` row never leaves this function on a non-terminal
/// platform, whatever the scans return). Ink files hand in the JS-stripped
/// text the web pipeline already computed so the two halves agree on lines.
fn terminal_findings(
    content: &str,
    file_path: &str,
    ext: &str,
    options: &TextOptions,
    js_stripped: Option<&str>,
) -> Vec<Finding> {
    let Some(stack) = classify_terminal_source(ext, content) else {
        return Vec::new();
    };
    let stripped: String = match (stack, js_stripped) {
        (Stack::Ink, Some(s)) => s.to_string(),
        (Stack::Ink, None) => strip_js_comments(content, ext != ".ts"),
        (Stack::Textual, _) => strip_python_comments(content),
        (Stack::Ratatui | Stack::Charm | Stack::TextualCss, _) => strip_c_comments(content),
    };
    scan_terminal_source(&stripped, file_path, stack, options.signals)
        .into_iter()
        .filter(|f| {
            get_antipattern(&f.antipattern)
                .map(|ap| rule_runs_on_platform(ap, options.platform))
                .unwrap_or(false)
        })
        .collect()
}
```

In `detect_text`, directly after `let ext = ext_from_file_path(file_path);` (line 1405) insert:

```rust
    let terminal_project = options.platform == Some("terminal");
    if TERMINAL_EXTENSIONS.contains(&ext.as_str()) {
        // A terminal-only extension never enters the web pipeline: its
        // matchers read markup and stylesheets. Off a terminal project the
        // walker does not list these files and an explicit file argument
        // scans to nothing.
        if !terminal_project {
            return Vec::new();
        }
        let mut findings = terminal_findings(content, file_path, &ext, options, None);
        if let Some(pack) = options.rule_pack {
            findings.extend(pack.check_text(content, file_path, &ext));
        }
        return if options.inline_ignores {
            apply_inline_ignores(findings, Some(content))
        } else {
            findings
        };
    }
```

Directly above the existing `if let Some(pack) = options.rule_pack {` near line 1565 insert:

```rust
    // Ink components are ordinary web-scannable files that also carry
    // terminal rules on a terminal project; they land after the web findings
    // so web output stays byte-identical everywhere else.
    if terminal_project && JS_SOURCE_EXTS.contains(&ext.as_str()) {
        deduped.extend(terminal_findings(content, file_path, &ext, options, Some(&comment_stripped)));
    }
```

- [ ] **Step 4: The walker** in `file_system.rs`. After `HTML_EXTENSIONS` (line 36) add:

```rust
/// The extensions a terminal project adds to the walk (spec section 3).
pub use impeccable_core::checks::terminal::TERMINAL_EXTENSIONS;
/// Skipped only on a terminal project: Rust build output, Python virtualenvs,
/// Go vendored modules. Web walks keep `SKIP_DIRS` byte-identical; the
/// `detect-config` oracle workspace proves `ignoreFiles` on `src/vendor`.
pub const TERMINAL_SKIP_DIRS: &[&str] = &["target", ".venv", "vendor"];

/// `has_scannable_extension`, plus the terminal extensions when the resolved
/// platform is `terminal`.
pub fn has_scannable_extension_for(filename: &str, platform: Option<&str>) -> bool {
    if has_scannable_extension(filename) {
        return true;
    }
    platform == Some("terminal")
        && TERMINAL_EXTENSIONS
            .contains(&jsp::extname(&impeccable_core::js::to_lower_case(filename)).as_str())
}
```

Rename the existing `walk_dir_reporting` body to `walk_dir_reporting_for` with a `platform: Option<&str>` parameter between `dir` and `on_read_error`, thread `platform` through the recursive call, change the skip test to

```rust
        if SKIP_DIRS.contains(&name.as_str())
            || (platform == Some("terminal") && TERMINAL_SKIP_DIRS.contains(&name.as_str()))
        {
            continue;
        }
```

and the file test to `} else if has_scannable_extension_for(&name, platform) {`. Then add the web-shaped wrapper so every existing caller compiles unchanged:

```rust
/// JS: file-system.mjs#walkDir(dir, onReadError), the web walk.
pub fn walk_dir_reporting(
    dir: &str,
    on_read_error: &mut dyn FnMut(&str, &std::io::Error),
) -> Vec<String> {
    walk_dir_reporting_for(dir, None, on_read_error)
}
```

Keep the doc comment about #711 on `walk_dir_reporting_for`.

- [ ] **Step 5: `ScanOptions` and `Engines`** in `engines.rs`. Add `use impeccable_core::checks::terminal::ProjectSignals;`. Extend `ScanOptions` (after `rule_pack`):

```rust
    /// The resolved platform (`--platform`, else the binary's PRODUCT.md
    /// resolver); `None` is web. Threaded to `TextOptions::platform`.
    pub platform: Option<String>,
    /// Project signals collected over a directory target on a terminal
    /// project; `None` for single-file targets and for web projects.
    pub signals: Option<Rc<ProjectSignals>>,
```

Extend `Engines`:

```rust
    /// Resolves the project's platform for a cwd. The binary reads PRODUCT.md
    /// through `impeccable_context`; `detect` never depends on that crate, so
    /// the lookup arrives here. `None` (no resolver, or no PRODUCT.md) is web.
    pub platform: Option<&'a dyn Fn(&str) -> Option<String>>,
```

Then `cargo build --workspace --all-targets 2>&1 | grep -E "^error" -A 5 | head -40` and add `platform: None,` to every `Engines { ... }` literal the compiler names (`crates/cli/src/main.rs:121` at least; Task 7 replaces that `None` with the real resolver).

- [ ] **Step 6: Run the tests**

Run: `cargo test -p impeccable-detect 2>&1 | grep -E "^test result|panicked|FAILED"`
Expected: all pass, including the existing `rule_pack` tests and the new `terminal` file. Then `cargo test --workspace 2>&1 | grep -E "^test result" | grep -v " 0 failed"; echo "expect nothing above"`.

- [ ] **Step 7: Prove web output is byte-identical** with the oracle before any CLI change:

```bash
cargo build --release -p impeccable 2>&1 | tail -n 1
IMPECCABLE_BIN="$PWD/target/release/impeccable" node --test tests/oracle.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)"
```

Expected: the same pass count as Task 1's baseline. A detect golden that fails here means a web code path changed; fix the code, do not re-record.

- [ ] **Step 8: Commit**

```bash
git add crates/detect/src/detect_text.rs crates/detect/src/file_system.rs crates/detect/src/engines.rs crates/detect/tests/terminal.rs crates/cli/src/main.rs
git commit -m "$(printf 'detect: platform-gated terminal source scanning in the text engine and walker\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 7: `detect --platform`, the PRODUCT.md resolver, and the signals pre-pass

**Files:**
- Modify: `crates/context/src/context.rs` (add `resolve_project_platform` after `extract_platform`, line 1103-1123)
- Modify: `crates/hook/src/hook_lib.rs:339-344` (`resolve_project_platform` delegates to the context function)
- Modify: `crates/detect/src/cli.rs` (imports at 19; `USAGE` at 24-45; `detect_local_file` at 271-304; flag parsing after the `--viewport` loop ending line 487; `base` at 505-513; directory branch at 733-807)
- Modify: `crates/cli/src/main.rs:116-125` (`engines()`)

**Interfaces:**
- Consumes: Task 6's `ScanOptions.platform/signals`, `Engines.platform`, `walk_dir_reporting_for`, `has_scannable_extension`; `ProjectSignals::absorb`; `crate::util::read_text`.
- Produces: `impeccable_context::context::resolve_project_platform(cwd: &str, env: &Env) -> Option<String>`; the `detect` flag `--platform <web|ios|android|adaptive|terminal>` (also `--platform=<value>`); error text `Error: --platform requires one of web, ios, android, adaptive, terminal`; the USAGE line; signals collected once per directory target.

- [ ] **Step 1: Write the failing CLI tests.** `cli.rs` has no test module; add one at the end of the file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::{Engines, MissingHtmlEngine};
    use std::collections::HashMap;

    fn engines(platform: Option<&'static dyn Fn(&str) -> Option<String>>) -> Engines<'static> {
        static HTML: MissingHtmlEngine = MissingHtmlEngine;
        Engines { html: &HTML, url: None, platform }
    }

    fn project(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("impeccable-detect-platform-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("src/ui.rs"), "use ratatui::widgets::BorderType;\nlet b = BorderType::Double;\n").unwrap();
        std::fs::write(dir.join("src/util.rs"), "let c = Color::Rgb(1, 2, 3); // names no framework, never scanned\n").unwrap();
        dir
    }

    fn run(dir: &std::path::Path, args: &[&str], engines: &Engines) -> (i32, String, String) {
        let (mut io, cap) = Io::captured("", dir.to_path_buf(), HashMap::new());
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let code = run_detect(&args, &mut io, engines);
        let out = String::from_utf8(cap.stdout.borrow().clone()).unwrap();
        let err = String::from_utf8(cap.stderr.borrow().clone()).unwrap();
        (code, out, err)
    }

    #[test]
    fn platform_flag_turns_terminal_rules_on_for_a_directory() {
        let dir = project("flag");
        let (code, out, _) = run(&dir, &["--no-config", "--json", "--platform", "terminal", "src"], &engines(None));
        assert_eq!(code, 0, "advisory findings never fail the run");
        assert!(out.contains("\"tui-double-border\""), "{out}");
        assert!(!out.contains("tui-hardcoded-rgb"), "util.rs names no framework: {out}");
        let (_, out, _) = run(&dir, &["--no-config", "--json", "src"], &engines(None));
        assert_eq!(out.trim(), "[]", "web by default: {out}");
    }

    #[test]
    fn resolver_supplies_the_platform_when_no_flag_is_given() {
        let dir = project("resolver");
        fn terminal(_cwd: &str) -> Option<String> {
            Some("terminal".into())
        }
        static RESOLVE: fn(&str) -> Option<String> = terminal;
        let (_, out, _) = run(&dir, &["--no-config", "--json", "src"], &engines(Some(&RESOLVE)));
        assert!(out.contains("\"tui-double-border\""), "{out}");
        let (_, out, _) = run(&dir, &["--no-config", "--json", "--platform=web", "src"], &engines(Some(&RESOLVE)));
        assert_eq!(out.trim(), "[]", "the flag wins over the resolver: {out}");
    }

    #[test]
    fn invalid_platform_value_is_an_error() {
        let dir = project("invalid");
        let (code, _, err) = run(&dir, &["--no-config", "--platform", "nope", "src"], &engines(None));
        assert_eq!(code, 1);
        assert_eq!(err, "Error: --platform requires one of web, ios, android, adaptive, terminal\n");
    }

    #[test]
    fn directory_scan_collects_signals_and_a_single_file_does_not() {
        let dir = project("signals");
        std::fs::write(dir.join("src/theme.rs"), "use ratatui::style::Color;\nlet c = Color::Rgb(9, 9, 9);\n").unwrap();
        std::fs::write(dir.join("src/main.rs"), "use ratatui::prelude::*;\nfn main() { if std::io::stdout().is_terminal() {} }\n").unwrap();
        let (_, out, _) = run(&dir, &["--no-config", "--json", "--platform", "terminal", "src"], &engines(None));
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let rgb = v.as_array().unwrap().iter().find(|f| f["antipattern"] == "tui-hardcoded-rgb-no-adapt").expect("rgb finding");
        assert!(rgb.get("note").is_none(), "a directory scan decides from signals: {rgb}");
        let (_, out, _) = run(&dir, &["--no-config", "--json", "--platform", "terminal", "src/theme.rs"], &engines(None));
        assert!(out.contains("project signals were not collected"), "{out}");
    }
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p impeccable-detect cli::tests 2>&1 | grep -E "^test |^test result|panicked" | head`
Expected: `invalid_platform_value_is_an_error` and the terminal assertions fail (the flag is a target today).

- [ ] **Step 3: The shared resolver.** In `crates/context/src/context.rs`, after `extract_platform`:

```rust
/// JS: resolveProjectPlatform(cwd): `extractPlatform(loadContext(cwd).product)`.
/// The one PRODUCT.md platform lookup the hook and `impeccable detect`
/// share. Only the platform is observable, so it skips loadContext's brief
/// and visual-implementation work. `None` means no PRODUCT.md or no
/// `## Platform`, which every caller treats as web.
pub fn resolve_project_platform(cwd: &str, env: &Env) -> Option<String> {
    let options = crate::target_args::TargetOptions::default();
    let resolved = resolve_context(cwd, &options, env);
    let product = resolved
        .product_path
        .as_deref()
        .and_then(|p| std::fs::read_to_string(p).ok());
    extract_platform(product.as_deref())
}
```

In `crates/hook/src/hook_lib.rs` replace the body of `resolve_project_platform(rt, cwd)` (lines 340-343) with:

```rust
    impeccable_context::context::resolve_project_platform(cwd, &rt.env)
```

and remove the now-unused `TargetOptions` import if the compiler flags it. The hook's behavior is unchanged (same resolution, same `extract_platform`); `cargo test -p impeccable-hook` and the `hook-*` oracle cases prove it in Step 7.

- [ ] **Step 4: The flag, the resolution, and the pre-pass** in `cli.rs`.

Imports: change line 19's `walk_dir_reporting,` to `walk_dir_reporting_for,` and add `has_scannable_extension` to the same `file_system` import list if it is not there; add `use impeccable_core::checks::terminal::ProjectSignals;` and `use std::rc::Rc;`.

USAGE (line 39): directly after the `--no-advisory` line add

```text
  --platform <value>  Scan as web|ios|android|adaptive|terminal instead of reading PRODUCT.md
```

Flag parsing: directly after the `--viewport` loop (after line 487, before `let valid = rule_scopes();`):

```rust
    const PLATFORM_VALUES: &[&str] = &["web", "ios", "android", "adaptive", "terminal"];
    let mut platform_flag: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        let inline = args[i].starts_with("--platform=");
        if args[i] != "--platform" && !inline {
            i += 1;
            continue;
        }
        let value: Option<String> = if inline {
            Some(args[i]["--platform=".len()..].to_string())
        } else {
            args.get(i + 1).cloned()
        };
        match value {
            Some(v) if PLATFORM_VALUES.contains(&v.as_str()) => platform_flag = Some(v),
            _ => {
                io.err("Error: --platform requires one of web, ios, android, adaptive, terminal\n");
                return Err(Exit(1));
            }
        }
        let n = if inline { 1 } else { 2 };
        for _ in 0..n {
            if i < args.len() {
                args.remove(i);
            }
        }
    }
    // The flag wins; else the binary's PRODUCT.md resolver for the process
    // cwd; else web. Resolved once per run, not per target (spec ruling 6).
    let platform: Option<String> =
        platform_flag.or_else(|| engines.platform.and_then(|resolve| resolve(&cwd)));
```

`base` (line 505): add `platform: platform.clone(), signals: None,` after `rule_pack: None,`.

`detect_local_file` `TextOptions` literal (line 297): add `platform: options.platform.as_deref(), signals: options.signals.as_deref(),`.

Directory branch: change the walker call (line 737) to `walk_dir_reporting_for(&resolved, ctx.base.platform.as_deref(), &mut |dir, err| {`. Directly after the `for (dir, message) in walk_failures { ... }` loop add:

```rust
            let terminal_project = ctx.base.platform.as_deref() == Some("terminal");
            if terminal_project {
                // Spec section 3 "Project signals": one pass over every
                // scanned file before the rules run, so the absence rules can
                // decide instead of noting. Read failures surface below when
                // the file is scanned.
                let mut signals = ProjectSignals::default();
                for f in &files {
                    if let Some(text) = crate::util::read_text(f) {
                        signals.absorb(&text);
                    }
                }
                ctx.base.signals = Some(Rc::new(signals));
            }
```

Change the import graph input (line 762) so terminal source never enters the JS import resolver:

```rust
            // The import graph is a web concern; terminal source stays out of it.
            let web_files: Vec<String> = files.iter().filter(|f| has_scannable_extension(f)).cloned().collect();
            let graph = build_import_graph_reporting(&web_files, &mut |file, err| {
```

Directly after the per-file `for file in &files { ... }` loop (after line 807) add `ctx.base.signals = None;` so a later target starts clean.

- [ ] **Step 5: The binary's resolver** in `crates/cli/src/main.rs`. Replace `engines()`:

```rust
/// The engines wired into `impeccable detect`: the static HTML engine
/// (crates/html), the browser engine (crates/browser), and the PRODUCT.md
/// platform resolver `detect` cannot own (it never depends on crates/context;
/// the hook resolves through the same function).
fn engines() -> impeccable_detect::Engines<'static> {
    static HTML: impeccable_html::StaticHtmlEngine = impeccable_html::StaticHtmlEngine {
        // The shipped binary carries the built-in rules only.
        static_rule_pack: None,
    };
    static RESOLVE: fn(&str) -> Option<String> = resolve_platform;
    impeccable_detect::Engines {
        html: &HTML,
        url: Some(url_engine()),
        platform: Some(&RESOLVE),
    }
}

fn resolve_platform(cwd: &str) -> Option<String> {
    let env: std::collections::HashMap<String, String> = std::env::vars().collect();
    impeccable_context::context::resolve_project_platform(cwd, &env)
}
```

- [ ] **Step 6: Run the tests**

Run: `cargo test -p impeccable-detect 2>&1 | grep -E "^test result|panicked|FAILED"` then `cargo test --workspace 2>&1 | grep -E "^test result" | grep -v " 0 failed"; echo "expect nothing above"`.

- [ ] **Step 7: Oracle check before recording anything**

```bash
cargo build --release -p impeccable 2>&1 | tail -n 1
IMPECCABLE_BIN="$PWD/target/release/impeccable" node --test tests/oracle.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)|^✖ "
```

Expected: exactly one new failure, `detect-help` (USAGE gained a line). Every other detect and hook golden must pass unchanged. Do not re-record here; Task 8 records `detect-help` with the new cases.

- [ ] **Step 8: Commit**

```bash
git add crates/context/src/context.rs crates/hook/src/hook_lib.rs crates/detect/src/cli.rs crates/cli/src/main.rs
git commit -m "$(printf 'detect: --platform flag, PRODUCT.md platform resolver, project signals pre-pass\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 8: Fixtures, oracle cases, goldens

**Files:**
- Create: `tests/fixtures/antipatterns/terminal/<rule>/<rule>.<ext>` (one file per stack the rule covers; see the script)
- Create: `tests/oracle/workspaces/detect-terminal-project/{PRODUCT.md,Cargo.toml,package.json,src/main.rs,src/theme.rs,src/util.rs}`
- Modify: `tests/oracle/cases/detect.mjs` (new cases after the framework cases, before `// Flag surface and errors`)
- Modify: `tests/oracle/DELTAS.md` (new section)
- Record: `tests/oracle/golden/detect-terminal-*.json`, `detect-help.json`, `detect-fixture-{json,text}-terminal.json`, and the seven all-fixtures goldens

**Interfaces:**
- Consumes: the `detect --platform` surface from Task 7. Every fixture file names its framework (spec section 3 table) so it passes per-file detection.
- Produces: the fixture tree Task 9's hook case and Task 11's calibration reuse; the workspace `detect-terminal-project`.

- [ ] **Step 1: Create the fixtures.** Each file has a should-flag half and a should-pass half; comments say which. Run this script from the repo root (it only creates files):

```bash
set -e
T=tests/fixtures/antipatterns/terminal
mk() { mkdir -p "$T/$1"; cat > "$T/$1/$1.$2"; }

mk tui-figlet-banner rs <<'F'
use figlet_rs::FIGfont; // flag: import
use ratatui::prelude::*;
const BANNER: [&str; 4] = [ // flag: four block-glyph rows
    "███╗   ███╗██╗   ██╗",
    "████╗ ████║██║   ██║",
    "██╔████╔██║██║   ██║",
    "██║╚██╔╝██║╚██████╔╝",
];
const TITLE: &str = "mu"; // pass: plain text title
F
mk tui-figlet-banner py <<'F'
from textual.app import App
import pyfiglet  # flag
from art import text2art  # flag
TITLE = "mu"  # pass
F
mk tui-figlet-banner tsx <<'F'
import { Text } from 'ink';
import BigText from 'ink-big-text'; // flag
export const Title = () => <Text>mu</Text>; // pass
F

mk tui-gradient-title rs <<'F'
use ratatui::prelude::*;
use tui_gradient::gradient; // flag
fn title() -> Line<'static> { Line::from("mu") } // pass
F
mk tui-gradient-title go <<'F'
package ui
import "github.com/charmbracelet/lipgloss"
var title = lipgloss.NewStyle().Foreground(lipgloss.Blend1D(0.5, a, b)) // flag: blend on a title
var bar = lipgloss.NewStyle().Foreground(lipgloss.Blend1D(0.5, a, b))   // pass: not a title
var plain = lipgloss.NewStyle().Foreground(lipgloss.Color("4"))         // pass
F
mk tui-gradient-title py <<'F'
from rich.console import Console
from rich_gradient import Gradient  # flag
console = Console()
console.print("mu", style="bold")  # pass
F
mk tui-gradient-title tsx <<'F'
import { Text } from 'ink';
import Gradient from 'ink-gradient'; // flag
export const A = () => <Gradient name="rainbow"><Text>mu</Text></Gradient>; // flag
export const B = () => <Text color="cyan">mu</Text>; // pass
F

mk tui-blink-attribute rs <<'F'
use ratatui::style::{Modifier, Style};
fn hot() -> Style { Style::default().add_modifier(Modifier::SLOW_BLINK) } // flag
fn calm() -> Style { Style::default().add_modifier(Modifier::BOLD) } // pass
F
mk tui-blink-attribute go <<'F'
package ui
import "github.com/charmbracelet/lipgloss"
var hot = lipgloss.NewStyle().Blink(true) // flag
var calm = lipgloss.NewStyle().Bold(true) // pass
F
mk tui-blink-attribute py <<'F'
from rich.console import Console
Console().print("[blink]saving[/blink]")  # flag
Console().print("[bold]saved[/bold]")  # pass
F
mk tui-blink-attribute tcss <<'F'
/* Textual CSS */
#alert { text-style: bold blink; } /* flag */
#status { text-style: bold; } /* pass */
F
mk tui-blink-attribute tsx <<'F'
import { Text } from 'ink';
process.stdout.write("\x1b[5mlive\x1b[0m"); // flag
export const A = () => <Text bold>live</Text>; // pass
F

mk tui-emoji-density rs <<'F'
use ratatui::prelude::*;
const STATUS: [&str; 4] = ["🚀 start", "✅ done", "❌ failed", "⚠ retry"]; // flag: canonical set
const PLAIN: [&str; 2] = ["ok", "err"]; // pass
F
mk tui-emoji-density py <<'F'
from rich import print
print("🎉 📦 🔧 🔥 💡 ✨ 🚀 ✅ ❌")  # flag: nine emoji in one file
F
mk tui-emoji-density tsx <<'F'
import { Text } from 'ink';
export const Ok = () => <Text>✅ installed 🚀 ready 🎉</Text>; // flag: three canonical
F

mk tui-double-border rs <<'F'
use ratatui::widgets::{Block, BorderType};
fn framed() -> Block<'static> { Block::default().border_type(BorderType::Double) } // flag
fn quiet() -> Block<'static> { Block::default().border_type(BorderType::Plain) } // pass
F
mk tui-double-border go <<'F'
package ui
import "github.com/charmbracelet/lipgloss"
var framed = lipgloss.NewStyle().Border(lipgloss.DoubleBorder()) // flag
var quiet = lipgloss.NewStyle().Border(lipgloss.NormalBorder())  // pass
F
mk tui-double-border py <<'F'
from rich import box
from rich.panel import Panel
framed = Panel("mu", box=box.DOUBLE)  # flag
quiet = Panel("mu", box=box.SQUARE)  # pass
F
mk tui-double-border tcss <<'F'
#main { border: double $accent; } /* flag */
#side { border: round $accent; } /* pass */
F
mk tui-double-border tsx <<'F'
import { Box } from 'ink';
export const A = () => <Box borderStyle="double" />; // flag
export const B = () => <Box borderStyle="round" />; // pass
F

mk tui-hardcoded-rgb-no-adapt rs <<'F'
use ratatui::style::Color;
const ACCENT: Color = Color::Rgb(255, 0, 128); // flag
const SAFE: Color = Color::Cyan; // pass
F
mk tui-hardcoded-rgb-no-adapt go <<'F'
package ui
import "github.com/charmbracelet/lipgloss"
var accent = lipgloss.Color("#ff0080") // flag
var safe = lipgloss.Color("6")         // pass
F
mk tui-hardcoded-rgb-no-adapt py <<'F'
from rich.console import Console
Console().print("[#ff0080]mu[/]")  # flag
Console().print("[cyan]mu[/]")  # pass
F
mk tui-hardcoded-rgb-no-adapt tsx <<'F'
import { Text } from 'ink';
export const A = () => <Text color="#ff0080">mu</Text>; // flag
export const B = () => <Text color="cyan">mu</Text>; // pass
F

mk tui-spinner-no-tty-guard go <<'F'
package ui
import "github.com/charmbracelet/bubbles/spinner" // flag: no isatty anywhere in this directory
var s = spinner.New()
F
mk tui-spinner-no-tty-guard py <<'F'
from rich.spinner import Spinner  # flag
s = Spinner("dots")
F
mk tui-spinner-no-tty-guard tsx <<'F'
import { Text } from 'ink';
import Spinner from 'ink-spinner'; // flag
export const A = () => <Text><Spinner type="dots" /> working</Text>;
F

mk tui-hardcoded-size rs <<'F'
use ratatui::layout::{Constraint, Layout, Rect};
fn area() -> Rect { Rect::new(0, 0, 80, 24) } // flag
fn rigid() -> Layout { Layout::vertical([Constraint::Length(3), Constraint::Length(10)]) } // flag: all Length
fn live(size: Rect) -> Rect { size } // pass
F
mk tui-hardcoded-size go <<'F'
package ui
import "github.com/charmbracelet/lipgloss"
var box = lipgloss.NewStyle().Width(80) // flag
var width = 80                          // flag
var fluid = lipgloss.NewStyle()         // pass
F
mk tui-hardcoded-size py <<'F'
from textual.app import App
size=(80, 24)  # flag
columns = 100  # pass
F
mk tui-hardcoded-size tsx <<'F'
import { Box } from 'ink';
export const A = () => <Box width={80} />; // flag
export const B = () => <Box flexGrow={1} />; // pass
F

mk tui-grapheme-unsafe-truncate rs <<'F'
use ratatui::prelude::*;
fn short(title: &str) -> String { format!("{}...", &title[..20]) } // flag: no width library in this directory
fn head(buf: &[u8]) -> &[u8] { &buf[..4] } // pass: no ellipsis
F
mk tui-grapheme-unsafe-truncate go <<'F'
package ui
import "github.com/charmbracelet/lipgloss"
func short(title string) string { return title[:20] + "..." } // flag
func head(b []byte) []byte { return b[:4] } // pass
F
mk tui-grapheme-unsafe-truncate tsx <<'F'
import { Text } from 'ink';
export const A = ({ t }) => <Text>{t.slice(0, 20) + '...'}</Text>; // flag
export const B = ({ t }) => <Text>{t}</Text>; // pass
F

mk tui-nerd-glyph-no-fallback rs <<'F'
use ratatui::prelude::*;
const BRANCH: &str = "\u{e0a0} main"; // flag: private-use glyph, no icon toggle in this directory
const PLAIN: &str = "* main"; // pass
F
mk tui-nerd-glyph-no-fallback py <<'F'
from rich import print
print(" main")  # flag
F
mk tui-nerd-glyph-no-fallback tsx <<'F'
import { Text } from 'ink';
export const A = () => <Text>{" main"}</Text>; // flag
F

mk tui-print-in-loop py <<'F'
from textual.app import App
from textual.worker import work

class Demo(App):
    def on_mount(self) -> None:
        print("hello")  # flag
        self.log("hello")  # pass

    @work(thread=True)
    def fetch(self) -> None:
        print("worker output")  # pass

def main() -> None:
    print("outside the app")  # pass
F
mk tui-print-in-loop tsx <<'F'
import { Text } from 'ink';
console.log('debug'); // flag: no patchConsole in this file
export const A = () => <Text>mu</Text>;
F
find $T -type f | wc -l   # expect 39
```

Byte check: the Python nerd-glyph fixture and the Ink one use `` escape sequences that the *language* decodes; the Rust one uses `\u{e0a0}`. Those escapes are not private-use characters in the file bytes, so replace the escaped form with the literal glyph in all three files so the rule (which scans file text) sees it:

```bash
python3 - <<'P'
import pathlib
for p in pathlib.Path('tests/fixtures/antipatterns/terminal/tui-nerd-glyph-no-fallback').iterdir():
    s = p.read_text()
    s = s.replace('\\u{e0a0}', '').replace('\\ue0a0', '')
    p.write_text(s)
P
grep -rl $'\xee\x82\xa0' tests/fixtures/antipatterns/terminal/tui-nerd-glyph-no-fallback | wc -l   # expect 3
```

- [ ] **Step 2: Create the oracle workspace**

```bash
W=tests/oracle/workspaces/detect-terminal-project
mkdir -p $W/src
cat > $W/PRODUCT.md <<'F'
<!-- impeccable:product-schema 2 -->
# Product

## What it is
A fixture TUI the oracle harness uses to pin terminal-platform detection.

## Platform
terminal
F
cat > $W/Cargo.toml <<'F'
[package]
name = "terminal-fixture"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.29"
F
printf '{"name":"terminal-fixture","private":true}\n' > $W/package.json
cat > $W/src/main.rs <<'F'
use ratatui::widgets::{Block, BorderType};

fn frame() -> Block<'static> {
    Block::default().border_type(BorderType::Double)
}

fn main() {
    let _ = frame();
}
F
cat > $W/src/theme.rs <<'F'
use ratatui::style::Color;

pub const ACCENT: Color = Color::Rgb(255, 0, 128);
F
cat > $W/src/util.rs <<'F'
// Names no terminal framework: the per-file gate keeps this out even though
// the token below would match the rule.
pub struct Color;
impl Color {
    pub fn rgb(_: u8, _: u8, _: u8) -> Color { Color }
}
pub const DECOY: &str = "Color::Rgb(1, 2, 3)";
F
```

- [ ] **Step 3: Add the cases** in `tests/oracle/cases/detect.mjs`, as a new block after the last `out.push(...)` call and before the function's `return out;`:

```js
  // Terminal source rules (spec section 3). Each rule's directory is scanned
  // with --platform terminal so its own files supply the project signals;
  // the whole tree without the flag proves a web scan sees no tui- rule.
  const TERMINAL = `<REPO>/tests/fixtures/antipatterns/terminal`;
  const TERMINAL_RULES = fs.readdirSync(path.join(FIXTURES, 'terminal'), { withFileTypes: true })
    .filter((e) => e.isDirectory()).map((e) => e.name).sort();
  for (const rule of TERMINAL_RULES) {
    out.push({ id: `detect-terminal-${rule.replace(/^tui-/, '')}`, verb: 'detect', args: ['--no-config', '--json', '--platform', 'terminal', `${TERMINAL}/${rule}`], isolateHome: false });
  }
  out.push(
    { id: 'detect-terminal-gate-web', verb: 'detect', args: ['--no-config', '--json', TERMINAL], isolateHome: false },
    { id: 'detect-terminal-text-all', verb: 'detect', args: ['--no-config', '--platform', 'terminal', TERMINAL], isolateHome: false },
    { id: 'detect-terminal-single-file-advisory', verb: 'detect', args: ['--no-config', '--json', '--platform', 'terminal', `${TERMINAL}/tui-spinner-no-tty-guard/tui-spinner-no-tty-guard.py`], isolateHome: false },
    { id: 'detect-terminal-platform-invalid', verb: 'detect', args: ['--no-config', '--platform', 'nope', TERMINAL], isolateHome: false },
    { id: 'detect-terminal-product-json', verb: 'detect', workspace: 'detect-terminal-project', args: ['--no-config', '--json', 'src'] },
    { id: 'detect-terminal-product-text', verb: 'detect', workspace: 'detect-terminal-project', args: ['--no-config', 'src'] },
    { id: 'detect-terminal-product-flag-web', verb: 'detect', workspace: 'detect-terminal-project', args: ['--no-config', '--json', '--platform', 'web', 'src'] },
  );
```

- [ ] **Step 4: Record and review**

```bash
cargo build --release -p impeccable 2>&1 | tail -n 1
IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/record.mjs --bin detect- 2>&1 | tail -n 3
git status --short tests/oracle/golden | sort
```

Expected changed or new goldens, and nothing else:
- new: every `detect-terminal-*.json` (18: eleven rules plus seven), `detect-fixture-json-terminal.json`, `detect-fixture-text-terminal.json`
- modified: `detect-help.json` (the USAGE line), and the seven all-fixtures cases (`detect-dir-json-all-fixtures`, `detect-dir-text-all-fixtures`, `detect-dir-quiet-all-fixtures`, `detect-scope-type`, `detect-scope-layout-text`, `detect-scope-both`, `detect-no-advisory-json`, `detect-no-advisory-text`) because the walk now includes the terminal tree's `.tsx` files under web rules.

Any other modified golden is a web regression: fix the code and re-record. Then review by hand:

```bash
node -e '
const fs=require("fs");
for (const f of fs.readdirSync("tests/oracle/golden").filter(f=>f.startsWith("detect-terminal-"))) {
  const g=JSON.parse(fs.readFileSync("tests/oracle/golden/"+f,"utf8"));
  let ids=[]; try { ids=JSON.parse(g.stdout).map(x=>x.antipattern+"@"+x.file.split("/").pop()+":"+x.line); } catch {}
  console.log(f, "exit="+g.exit, ids.join(" ") || g.stdout.trim().split("\n").slice(-2).join(" | "), g.stderr.trim());
}'
```

Check each against its fixture's `flag`/`pass` comments: every flag line reported, no pass line reported, `detect-terminal-gate-web` has no `tui-` id, `detect-terminal-single-file-advisory` carries the note, `detect-terminal-platform-invalid` exits 1 with the error text, `detect-terminal-product-json` reports `main.rs` and `theme.rs` and nothing from `util.rs`, `detect-terminal-product-flag-web` is `[]`. A mismatch is a rule bug: fix in `terminal.rs` (add the case to its unit tests first), rebuild, re-record.

- [ ] **Step 5: Run the oracle**

Run: `IMPECCABLE_BIN="$PWD/target/release/impeccable" node --test tests/oracle.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)|^✖ "`
Expected: baseline pass count plus 20 new cases, zero failures (the `framework-vite` port-8080 flake excepted).

- [ ] **Step 6: DELTAS.md.** Append a section in the style of the existing ones:

```markdown
## Recorded 2026-09-2X: terminal source rules (spec PR 2)

New cases `detect-terminal-*` (one directory scan per `tui-` rule with `--platform terminal`, the web-gate scan of the same tree, the single-file advisory downgrade, the invalid flag value, and the `detect-terminal-project` workspace with and without the flag) pin the eleven terminal rules and the platform gate. `detect-help` gained the `--platform` line. The seven all-fixtures goldens and the auto-enumerated `detect-fixture-{json,text}-terminal` changed only because `tests/fixtures/antipatterns/terminal/` now exists and its `.tsx` files are web-scannable; no web rule's output on an existing fixture changed.
```

Replace `2X` with the day. Confirm the last sentence by diffing one all-fixtures golden: `git diff tests/oracle/golden/detect-dir-json-all-fixtures.json | grep '^[-+]' | grep -v terminal/ | head` must print only the `---`/`+++` header lines.

- [ ] **Step 7: Commit**

```bash
git add tests/fixtures/antipatterns/terminal tests/oracle/workspaces/detect-terminal-project tests/oracle/cases/detect.mjs tests/oracle/golden tests/oracle/DELTAS.md
git commit -m "$(printf 'oracle: terminal source-rule fixtures and goldens\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 9: Hook: platform-aware extension gate and signals in the Stop pass

**Files:**
- Modify: `crates/hook/src/hook_lib.rs` (`ALLOWED_EXTS` at 33-36; `HookScanOptions` struct and its constructor, find with `grep -n "pub struct HookScanOptions\|HookScanOptions {" crates/hook/src/*.rs`; `detector_detect_text` at 1677-1689)
- Modify: `crates/hook/src/hook.rs` (per-edit gate at 214, Stop gate at 731, the `scan` construction before each `detector_detect_text` call at 319 and 768)
- Modify: `crates/hook/src/before_edit.rs` (gate at 766; platform resolution at 794)
- Modify: `tests/oracle/cases/hooks.mjs`; record `tests/oracle/golden/hook-terminal-*.json`, `hook-before-edit-terminal-rs.json`, `hook-web-platform-skips-rs.json`; `tests/oracle/DELTAS.md`

**Interfaces:**
- Consumes: `resolve_project_platform` (Task 7), `TextOptions.platform/signals` and `ScanOptions.platform/signals` (Task 6), `impeccable_detect::file_system::walk_dir_reporting_for`, `ProjectSignals`.
- Produces: `hook_lib::TERMINAL_ALLOWED_EXTS`, `hook_lib::allowed_exts(platform: Option<&str>) -> &'static [&'static str]`; `HookScanOptions { platform: Option<String>, signals: Option<Rc<ProjectSignals>>, .. }`.

- [ ] **Step 1: Write the failing unit test** in `hook_lib.rs`'s test module (find `#[cfg(test)]` in that file; add inside):

```rust
    #[test]
    fn terminal_platform_keeps_hook_active() {
        // Spec: `terminal` is not a native skip, and it admits terminal source.
        assert!(!is_native_platform(Some("terminal")));
        assert!(allowed_exts(Some("terminal")).contains(&".rs"));
        assert!(allowed_exts(Some("terminal")).contains(&".tcss"));
        assert!(allowed_exts(Some("terminal")).contains(&".tsx"), "web extensions stay watched");
        assert_eq!(allowed_exts(None), ALLOWED_EXTS);
        assert_eq!(allowed_exts(Some("web")), ALLOWED_EXTS);
        assert!(!allowed_exts(Some("web")).contains(&".rs"));
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p impeccable-hook terminal_platform_keeps_hook_active 2>&1 | grep -E "^error|^test result" | head -3`
Expected: compile error (`allowed_exts` not found).

- [ ] **Step 3: The extension gate.** In `hook_lib.rs`, after `ACK_EXTS` (line 40):

```rust
/// `ALLOWED_EXTS` plus the terminal source extensions, watched only when the
/// project's platform is `terminal` (spec section 3, "Hook").
pub const TERMINAL_ALLOWED_EXTS: &[&str] = &[
    ".tsx", ".jsx", ".html", ".htm", ".vue", ".svelte", ".astro", ".css", ".scss", ".sass",
    ".less", ".ts", ".js", ".rs", ".go", ".py", ".tcss",
];

/// The extension gate for a resolved platform. Web (and every native value,
/// which the native skip has already handled) keeps `ALLOWED_EXTS`.
pub fn allowed_exts(platform: Option<&str>) -> &'static [&'static str] {
    if platform == Some("terminal") {
        TERMINAL_ALLOWED_EXTS
    } else {
        ALLOWED_EXTS
    }
}
```

Call sites:
- `hook.rs:214` and `hook.rs:731`: `if !allowed_exts(platform.as_deref()).contains(&ext.as_str()) && configured.is_none() {` (`platform` is already in scope from lines 158 and 700).
- `before_edit.rs`: directly after `let config = read_config(&cwd);` (line 755) add `let platform = resolve_project_platform(rt, &cwd);`, change line 766 to use `allowed_exts(platform.as_deref())`, and delete the second `let platform = resolve_project_platform(rt, &cwd);` at line 794 so the native skip reuses the value. The skip order (`extension` before `native-platform`) is unchanged, so no existing hook golden moves.

- [ ] **Step 4: Thread platform and signals into the scan.** In `hook_lib.rs`:
- add `pub platform: Option<String>,` and `pub signals: Option<Rc<ProjectSignals>>,` to `HookScanOptions` (import `impeccable_core::checks::terminal::ProjectSignals` and `std::rc::Rc` if missing), `platform: None, signals: None` in every `HookScanOptions { .. }` literal the compiler names, and `platform: self.platform.clone(), signals: self.signals.clone()` in `to_scan_options()`.
- `detector_detect_text`: `platform: scan.platform.as_deref(), signals: scan.signals.as_deref(),` in its `TextOptions`.

In `hook.rs`, wherever the per-edit path and the Stop path build the `HookScanOptions` they hand to `detector_detect_text` (the bindings feeding lines 319 and 768; make the binding `mut`), set `scan.platform = platform.clone();`. In the Stop path only, directly after that:

```rust
        // Spec section 3, "Hook": the deep pass runs the terminal rules with
        // project signals, collected the same way `detect` collects them.
        if platform.as_deref() == Some("terminal") {
            let mut signals = ProjectSignals::default();
            for f in impeccable_detect::file_system::walk_dir_reporting_for(&project_cwd, Some("terminal"), &mut |_, _| {}) {
                if let Some(text) = safe_read(&f) {
                    signals.absorb(&text);
                }
            }
            scan.signals = Some(Rc::new(signals));
        }
```

In `before_edit.rs`, set `scan.platform = platform.clone();` on the options handed to `detector_detect_text` at line 836 (no signals: the per-edit tiers are the cheap rules, and absence rules carry the note).

- [ ] **Step 5: Build and unit-test**

Run: `cargo test -p impeccable-hook 2>&1 | grep -E "^test result|panicked|FAILED"` then `cargo build --release -p impeccable 2>&1 | tail -n 1`.

- [ ] **Step 6: Oracle cases.** In `tests/oracle/cases/hooks.mjs`, first confirm the config key the advisory filter reads: `grep -n "advisoryRules\|fn read_config" crates/hook/src/hook_lib.rs` (the filter reads `advisoryRules` from the hook section of `.impeccable/config.json`; write the setup below to match what `read_config` parses). Then copy the stdin shape of an existing `hook-before-edit-*` case for the last entry. Add after the per-edit cases:

```js
  // --- terminal platform (spec PR 2): the hook stays on and admits terminal source.
  // Every tui- rule is advisory, so findings reach stdout only when the
  // project includes advisory rules; the audit proves the scan ran either way.
  { id: 'hook-terminal-platform-edit-rs', verb: 'hook', workspace: 'detect-terminal-project', stdin: claudeEdit('src/main.rs'), files: CACHE_FILES },
  {
    id: 'hook-terminal-platform-edit-rs-advisory-included', verb: 'hook', workspace: 'detect-terminal-project', files: CACHE_FILES,
    setup(ws) {
      fs.mkdirSync(`${ws}/.impeccable`, { recursive: true });
      fs.writeFileSync(`${ws}/.impeccable/config.json`, JSON.stringify({ hook: { advisoryRules: 'include' } }) + '\n');
    },
    stdin: claudeEdit('src/main.rs'),
  },
  {
    id: 'hook-terminal-platform-stop', workspace: 'detect-terminal-project', files: CACHE_FILES,
    setup(ws) {
      fs.mkdirSync(`${ws}/.impeccable`, { recursive: true });
      fs.writeFileSync(`${ws}/.impeccable/config.json`, JSON.stringify({ hook: { advisoryRules: 'include' } }) + '\n');
    },
    steps: [
      { verb: 'hook', stdin: claudeEdit('src/theme.rs') },
      { verb: 'hook', stdin: stop() },
    ],
  },
  {
    id: 'hook-web-platform-skips-rs', verb: 'hook', workspace: 'hook-project', files: CACHE_FILES,
    setup(ws) { fs.writeFileSync(`${ws}/src/ui.rs`, 'use ratatui::widgets::BorderType;\nlet b = BorderType::Double;\n'); },
    stdin: claudeEdit('src/ui.rs'),
  },
  { id: 'hook-before-edit-terminal-rs', verb: 'hook-before-edit', workspace: 'detect-terminal-project', stdin: /* the existing before-edit event shape, file_path `${WS}/src/main.rs`, proposed content 'use ratatui::widgets::BorderType;\nlet b = BorderType::Double;\n' */, files: CACHE_FILES },
```

Replace the comment in the last entry with the real object. Record and review:

```bash
IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/record.mjs --bin hook- 2>&1 | tail -n 2
git status --short tests/oracle/golden | grep -v "^??" ; echo "expect no modified hook goldens above"
node -e '
const fs=require("fs");
for (const id of ["hook-terminal-platform-edit-rs","hook-terminal-platform-edit-rs-advisory-included","hook-terminal-platform-stop","hook-web-platform-skips-rs","hook-before-edit-terminal-rs"]) {
  const g=JSON.parse(fs.readFileSync(`tests/oracle/golden/${id}.json`,"utf8"));
  const out=(g.steps?g.steps.map(s=>s.stdout).join("\n"):g.stdout);
  console.log(id, "| tui:", /tui-/.test(out), "| skipped:", JSON.stringify(out.match(/"skipped":"[^"]+"/g)), "| audit files:", Object.keys(g.files||{}).filter(f=>f.includes("impeccable")).join(","));
}'
```

Expected: `hook-terminal-platform-edit-rs` prints no `tui-` (advisory excluded) and no `skipped: extension` or `native-platform` anywhere in stdout or the audit; the `advisory-included` case prints `tui-double-border`; the Stop case's second step carries `tui-hardcoded-rgb-no-adapt` without the signals note (project scanned; `main.rs` has no TTY guard, so the rgb finding is plain); `hook-web-platform-skips-rs` reports `"skipped":"extension"`; `hook-before-edit-terminal-rs` reaches the scan (no `extension` skip). If the audit does not record enough to tell, read the case's `.impeccable/` audit file in `files` and adjust the expectation list in the ledger, not the assertion.

- [ ] **Step 7: Oracle run and DELTAS**

Run: `IMPECCABLE_BIN="$PWD/target/release/impeccable" node --test tests/oracle.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)|^✖ "`. Then append to the PR 2 section of `tests/oracle/DELTAS.md`:

```markdown
Hook: `hook-terminal-platform-*`, `hook-before-edit-terminal-rs`, and `hook-web-platform-skips-rs` pin that a `terminal` PRODUCT.md keeps the hook on, admits `.rs`/`.go`/`.py`/`.tcss`, surfaces `tui-` findings only when `hook.advisoryRules` is `include` (every PR 2 rule is advisory), and that a web project still skips `.rs` at the extension gate. No existing hook golden changed.
```

- [ ] **Step 8: Commit**

```bash
git add crates/hook tests/oracle/cases/hooks.mjs tests/oracle/golden tests/oracle/DELTAS.md
git commit -m "$(printf 'hook: watch terminal source on terminal projects and pass the platform to the text engine\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 10: Documentation, counts, and the full gates

**Files:**
- Modify: `README.md:3,16` and `README.npm.md:3,60` (`61 deterministic detector rules` / `61 deterministic rules` become `72`)
- Modify: `docs/CLI-CONTRACT.md` (args table near line 98; USAGE text near 125; walker sentence at 184; registry list at 249; walker constants at 255; `detectText` at 335; `ALLOWED_EXTS` at 808)
- Modify: `docs/ENGINE.md` (Layout table row for `crates/core`; text-engine order sentence at 174)
- Modify: `CLAUDE.md` (root; the `terminal` bullet at line 55, the web-only paragraph at 63, the rule-authoring section near line 349)
- Modify: `skill/reference/routing.md:19` (the clause `(its terminal rules arrive with the engine that ships them)`)

**Interfaces:** none new. Every statement written here must match code that landed in Tasks 2 to 9; quote ids and messages from the code, not from memory.

- [ ] **Step 1: Counts.** Change the four count phrases to `72` and confirm with the build gate:

```bash
sed -i '' 's/61 deterministic detector rules/72 deterministic detector rules/g; s/61 deterministic rules/72 deterministic rules/g' README.md README.npm.md
bun run build 2>&1 | tail -n 5     # must pass: validateProse, validateSkillProse, generateCounts
```

If `generateCounts` names another file, fix that count too; do not touch `plugin.json` or `marketplace.json` unless the gate names them.

- [ ] **Step 2: `docs/CLI-CONTRACT.md`.**
- Args table: add the row ``| `--platform <value>` | one of `web`, `ios`, `android`, `adaptive`, `terminal` (also `--platform=<value>`); overrides the PRODUCT.md resolver; any other value → stderr `Error: --platform requires one of web, ios, android, adaptive, terminal`, exit 1 |``.
- USAGE block: add the line `  --platform <value>  Scan as web|ios|android|adaptive|terminal instead of reading PRODUCT.md` after `--no-advisory`, matching `crates/detect/src/cli.rs` byte for byte.
- After the args table add a paragraph: ``**Platform**: `--platform` wins; else the binary resolves `impeccable_context::context::resolve_project_platform(cwd, env)` (PRODUCT.md `## Platform` through `extract_platform`) once per run from the process cwd; else web. Only `terminal` changes behavior: the walk adds `TERMINAL_EXTENSIONS = {'.rs','.go','.py','.tcss'}`, skips `TERMINAL_SKIP_DIRS = {'target','.venv','vendor'}`, collects project signals over every listed file before scanning a directory target, and runs the registry rows whose `platforms` names `terminal` over files that pass per-file framework detection (`.rs` names `ratatui`/`crossterm`, `.go` a `charmbracelet` import, `.py` imports `textual`/`rich`, `.tcss` always, JS/TS imports `ink`). Files with a terminal-only extension never enter the web pipeline; Ink files get the web findings first, then the `tui-` findings. On any other platform a terminal-extension file argument scans to `[]`. Single-file targets carry no signals, and the absence rules (`tui-spinner-no-tty-guard`, `tui-hardcoded-rgb-no-adapt`, `tui-grapheme-unsafe-truncate`, `tui-nerd-glyph-no-fallback`) add `"note": "project signals were not collected; scan the project directory to confirm"`; `tui-hardcoded-rgb-no-adapt` adds `"note": "an adaptive color helper exists elsewhere in this project; confirm this literal is themed"` when the project has one. File-level rules add `"emojiCount"`, `"glyphCount"`, `"bannerLines"`, or `"lengthConstraints"`.``
- Line 184: after `files = walkDir(resolved)` note `(the terminal walk when the platform is terminal; the import graph is built over the web-scannable subset only)`.
- Line 249: change `(59 ids, in order)` to `(72 ids, in order)`, append the eleven `tui-` ids in registry order at the end of the list, and add: ``Rows carry `platforms: Option<&[&str]>`; `None` (every web row) runs everywhere a web file is scanned, `Some(["terminal"])` runs only on a terminal platform (`rule_runs_on_platform`). `platforms` is not serialized into findings or `antipatterns.json`.``
- Line 255: after the `SKIP_DIRS` sentence add the `TERMINAL_SKIP_DIRS` and `TERMINAL_EXTENSIONS` sentence (terminal platform only).
- Line 335: append ``Terminal branch (platform `terminal` only): terminal-extension files run `checks::terminal::scan_terminal_source` over comment-blanked text (Rust/Go/tcss: `//` and `/* */`; Python: `#` and bare docstrings; Ink: the JS stripper) then inline ignores; Ink files append the same after the dedupe and before the rule pack.``
- Line 808: after the `ALLOWED_EXTS` set add ``TERMINAL_ALLOWED_EXTS = ALLOWED_EXTS ∪ {'.rs','.go','.py','.tcss'}, used when `resolveProjectPlatform(cwd)` is `terminal` (`allowed_exts(platform)`); the Stop pass on a terminal project collects project signals over the terminal walk of the project root. Advisory findings still follow `advisoryRules`.``

- [ ] **Step 3: `docs/ENGINE.md`.** In the Layout table's `crates/core` row add `checks/terminal.rs` (terminal source rules, platform-gated) to the description. In the "Where each hook runs" text-engine sentence (line 174) add: "On a terminal project the `tui-` scans run after the dedupe and before the pack hook for Ink files, and alone (then inline ignores) for terminal-only extensions."

- [ ] **Step 4: Root `CLAUDE.md`.**
- Line 55, replace `terminal source rules are the engine's job (PR 2 of the terminal spec), and live mode stays web-only` with `the engine's ``tui-`` rules (``crates/core/src/checks/terminal.rs``, registry rows with ``platforms: Some(&["terminal"])``) run over its source, and live mode stays web-only`.
- Line 63 paragraph: after the terminal exception sentence add: "`impeccable detect` resolves the platform from PRODUCT.md (or `--platform`) and, on `terminal`, walks `.rs`/`.go`/`.py`/`.tcss` too, running the terminal rules only over files that name a terminal framework."
- In "Adding or modifying anti-pattern detection rules", add a bullet after the order paragraph: "A terminal rule follows the same order with its fixture under `tests/fixtures/antipatterns/terminal/<rule>/` (one file per stack), its scan in `checks/terminal.rs`, `platforms: Some(&["terminal"])` on its registry row, and a `detect-terminal-<rule>` oracle case (recorded with `--platform terminal`)."

- [ ] **Step 5: `skill/reference/routing.md:19`.** Replace `(its terminal rules arrive with the engine that ships them)` with `(its ``tui-`` rules run on terminal projects)`. Then `bun run build 2>&1 | tail -n 3` (prose gates) and `node --test tests/skill-reference.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)"`. This clause is not Setup text; the skill-behavior suite is not owed (ledger this).

- [ ] **Step 6: Full gates**

```bash
cargo test --workspace 2>&1 | grep -E "^test result" | grep -v " 0 failed"; echo "expect nothing above"
cargo xtask bundle --check && echo BUNDLE_FRESH
cargo build --release -p impeccable 2>&1 | tail -n 1
bun run build 2>&1 | tail -n 3
IMPECCABLE_BIN="$PWD/target/release/impeccable" bun run test 2>&1 | tail -n 15
```

Expected: everything green except the three pre-existing `tests/release.test.mjs` failures on this machine (signed tags) and the port-8080 flake if applicable. Compare pass counts with Task 1's baseline plus the new oracle cases.

- [ ] **Step 7: Commit**

```bash
git add README.md README.npm.md docs/CLI-CONTRACT.md docs/ENGINE.md CLAUDE.md skill/reference/routing.md
git commit -m "$(printf 'Document the terminal source rules, --platform, and the 72-rule count\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 11: Manual false-positive pass over one real project per stack

**Files:**
- Create: `docs/superpowers/research/2026-09-24-terminal-rules-manual-pass.md`
- Possibly modify: `crates/core/src/checks/terminal.rs` (only to fix a rule bug the pass exposes; each fix adds a unit test first and re-records the affected `detect-terminal-*` golden)

**Interfaces:** consumes the release binary from Task 10. Produces the calibration record the spec requires before any promotion (section 6, PR 2 gates) and the two deferred decisions from the spec's Open questions: the emoji threshold (8 per file) and whether `tui-hardcoded-size` exempts `examples/`.

- [ ] **Step 1: Stage the projects** in the session scratchpad (network required; shallow clones):

```bash
S=/private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/manual-pass
mkdir -p "$S" && cd "$S"
git clone --depth 1 https://github.com/ratatui/ratatui.git            # Rust: scan ratatui/examples
git clone --depth 1 https://github.com/charmbracelet/glow.git         # Go: Bubble Tea + Lip Gloss app
git clone --depth 1 https://github.com/Textualize/toolong.git         # Python: Textual app
git clone --depth 1 https://github.com/vadimdemedes/ink-ui.git        # TypeScript: Ink components
ls /Volumes/Containers/musicviz/src/musicviz/tui                      # Python: the fork owner's own Textual app (PRODUCT.md platform web today)
cd /Volumes/Containers/impeccable
```

If a clone fails, substitute another well-known project of the same stack and record the substitution.

- [ ] **Step 2: Scan each with the flag and summarize per rule**

```bash
B="$PWD/target/release/impeccable"
for p in "ratatui/examples" "glow" "toolong/src" "ink-ui/source" ; do
  name=$(echo "$p" | tr '/' '-')
  ( cd "$S/$(dirname "$p")" 2>/dev/null || cd "$S"; "$B" detect --no-config --json --platform terminal "$S/$p" > "$S/$name.json" 2> "$S/$name.err" ; echo "$name exit=$?" )
done
"$B" detect --no-config --json --platform terminal /Volumes/Containers/musicviz/src > "$S/musicviz.json" 2> "$S/musicviz.err"; echo "musicviz exit=$?"
node -e '
const fs=require("fs"); const S=process.argv[1];
for (const f of fs.readdirSync(S).filter(f=>f.endsWith(".json"))) {
  let a=[]; try { a=JSON.parse(fs.readFileSync(`${S}/${f}`,"utf8")); } catch { console.log(f, "unparsable"); continue; }
  const by={}; for (const x of a) by[x.antipattern]=(by[x.antipattern]||0)+1;
  console.log(f, a.length, JSON.stringify(by));
}' "$S"
```

Also time one full scan per project (`time "$B" detect --platform terminal ... >/dev/null`) and record it; a terminal walk that takes more than a few seconds on `ratatui/examples` is a finding.

- [ ] **Step 3: Judge samples.** For every rule with hits, print up to ten findings (file, line, snippet) and read each line in its file. Classify each as true positive, false positive, or arguable, and write the reason in one clause. A false-positive rate above roughly a third on a rule marks it "tighten before promotion"; a rule that produced no hits anywhere is "unmeasured". Where a false positive is a plain bug (a comment not blanked, a pattern that matches a non-UI construct), fix it in `terminal.rs` with a unit test named for the outcome, rebuild, re-record only that rule's `detect-terminal-*` golden, and note the fix in the record.

- [ ] **Step 4: Decide the two open questions** from the evidence: keep or change the emoji threshold constants (`EMOJI_FILE_THRESHOLD`, `CANONICAL_EMOJI_THRESHOLD`), and whether `in_test_path` should also exempt an `examples` segment. Change code only with a unit test and a re-recorded golden; otherwise record "kept" with the reason.

- [ ] **Step 5: Write the record** at `docs/superpowers/research/2026-09-24-terminal-rules-manual-pass.md` with: the projects and commits scanned (`git -C <clone> rev-parse --short HEAD`), scan times, a table `rule | project | hits | sampled | false positives | verdict`, the two decisions, the rule fixes made (if any), and the promotion recommendation for each rule (all stay `advisory` in this PR). No em dashes.

- [ ] **Step 6: Commit**

```bash
git add docs/superpowers/research/2026-09-24-terminal-rules-manual-pass.md crates/core/src/checks/terminal.rs tests/oracle/golden
git commit -m "$(printf 'Record the terminal source-rule manual pass\n\nAI assistance: drafted with Claude Code.')"
```

(Drop paths from `git add` that did not change.)

---

### Task 12: Final review, merge to the fork's main, push

**Files:** none new.

- [ ] **Step 1: Whole-branch review** per subagent-driven-development (final code reviewer on the most capable model). Findings marked Important or Critical get one fix wave and one scoped re-review.

- [ ] **Step 2: Gates once more on the final head**

```bash
cargo test --workspace 2>&1 | grep -E "^test result" | grep -v " 0 failed"; echo "expect nothing above"
cargo xtask bundle --check && echo BUNDLE_FRESH
bun run build 2>&1 | tail -n 2
IMPECCABLE_BIN="$PWD/target/release/impeccable" bun run test 2>&1 | tail -n 8
git status --short   # only the untracked local files (.claude/CLAUDE.md, .gemini/, .agents/, .gitignore edit) may show; never stage them
```

- [ ] **Step 3: Merge and push** (the fork owner pre-authorized merging each PR to `main` as it lands):

```bash
git checkout main && git pull --ff-only origin main
git merge --no-ff terminal-platform-pr2 -m "$(printf 'Merge terminal-platform-pr2: terminal source rules (spec PR 2)\n\nAI assistance: drafted with Claude Code.')"
git push origin main
git log --oneline -3
```

- [ ] **Step 4: Close out.** Ledger completion line; copy the ledger to the session scratchpad; delete the SDD workspace; update the memory file `terminal-platform-project.md` (PR 2 merged, commit id, the manual-pass verdicts, and that `ENGINE_VERSION` needs a bump at release because engine behavior changed); write the journal entry. Draft, for the fork owner to post personally, the upstream issue text proposing the `terminal` platform with PR 1 and PR 2 as evidence; save it at `docs/superpowers/research/2026-09-24-upstream-issue-draft.md` (uncommitted is fine) and never post it.

---

## Self-review notes

- Spec section 3 coverage: platform from `Engines` resolver and `--platform` (Task 7); walker lists (Task 6, ruling 2); `platforms` field and filter (Tasks 2, 6); project signals (Tasks 3, 7, 9); `checks/terminal.rs` with per-language blanking (Tasks 3 to 5, ruling 4); registry rows at the end and `cargo xtask bundle` (Task 2); hook (Task 9, ruling 3); inline ignores (`impeccable-disable` is comment-syntax agnostic already; Task 6 test `terminal_findings_are_waivable_inline` proves `//`); oracle cases including gate-web, single-file advisory, and the hook case (Tasks 8, 9; the spec's `detect-terminal-gate-web` id is kept).
- Spec section 4: all eleven rules (Tasks 4, 5), all advisory, categories as the table says; deferred rules untouched.
- Spec section 6 PR 2 gates: `cargo test --workspace`, `cargo xtask bundle --check`, oracle, `bun run build` with corrected counts, `bun run test`, the manual pass (Tasks 10, 11).
- Not in this PR: `skill/reference/terminal.md`'s verification section (PR 3), runtime rules, `ENGINE_VERSION` (release step). `.claude/CLAUDE.md` is untracked on this machine and is not edited here.
