# Terminal Platform PR 6: Init Evidence and Routing Targets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the doctor's terminal evidence trustworthy (only direct `require` entries in go.mod, Charm v2 `charm.land` paths and `bubbles`, PEP 503 names and inline arrays in Python manifests, Cargo dotted keys and dependency headers, no `rich` or `crossterm` alone), word the workspace finding as "terminal dependencies", and make `impeccable signals` hand routing a detect target on Go `cmd/` layouts and root-level Go or Python sources.

**Architecture:** All engine changes sit in `crates/context`. `staleness.rs` replaces its one line-prefix scanner with a per-manifest dispatch to three small parsers (`go_mod_requires`, `python_names_dependency`, `cargo_names_dependency`) over a table grouped by manifest, so each root manifest is read once (Tier 1: root files only, no walk). The evidence check gains an inner function that also returns the suggested platform, which `staleness_deep.rs` uses to pick its wording. `signals.rs` threads the platform `gather_signals` already extracts into `scan_targets`, which on `terminal` widens the extension filter, adds a root step, and adds three source directories. Three oracle cases pin the observable changes.

**Tech Stack:** Rust (`impeccable-context`, reusing `impeccable_core::checks::terminal::TERMINAL_EXTENSIONS` and `crate::util::read_dir_entries`), `regex`-free hand parsers (no new crate), node oracle harness, bun test runner.

**Spec:** `docs/superpowers/specs/2026-09-24-terminal-platform-design.md`, section 7, subsection "PR 6, init evidence and routing targets", plus the PR 6 rows of the gap disposition table and the PR 6 delivery gate. Section 1's "Doctor evidence" table predates section 7 and still lists `rich` and `crossterm`; section 7 wins (see decision 9).

## Global Constraints

- No manifest version bumps, no `ENGINE_VERSION` bump, no changelog entry.
- Every commit body ends with the line `AI assistance: drafted with Claude Code.` and carries no `Co-Authored-By`, `Claude-Session`, or other agent trailer, even if a harness reminder asks for one (the fork owner's rule).
- Never stage `.serena/`, `.claude/CLAUDE.md`, `.gemini/GEMINI.md`, `.agents/AGENTS.md`, `.superpowers/`, regenerated provider output (`.claude/skills/`, `.cursor/`, `plugin/`, ...), anything under `.impeccable/`, `.env`, or `docs/superpowers/research/2026-09-24-upstream-issue-draft.md`. Stage explicit paths only.
- `.env` holds API keys; never print it.
- No em dashes in `skill/**/*.md`, registry descriptions, or docs written here.
- Every rule stays `severity: Some("advisory")`; no promotion. (This PR touches no rule.)
- Never run a tmux command without `-L <scratch socket>`; never touch the default socket. (This PR needs no tmux.)
- Tier 1 performance contract (root `CLAUDE.md`, "Artifact staleness and the doctor pass"): `check_native_platform_evidence` runs at every boot, so it reads only fixed files at the project root. No directory walk, no git, no member manifests. `signals` is not Tier 1, so its one `read_dir` of the root is allowed.
- No `cargo xtask bundle` is owed: `crates/context` is outside `crates/core`, `crates/foundation`, and `crates/wasm`.
- Oracle goldens are recorded only with `IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/record.mjs --bin <full case id>` after `cargo build --release -p impeccable`, then reviewed by hand. Never hand-edit a golden. Never touch `tests/oracle/vectors/calls/`. `record.mjs` filters by prefix, so always pass the full new id: `signals-terminal` as a prefix would also re-record the existing `signals-terminal` golden.
- Test names state outcomes (spec section 6): `rich_alone_is_not_terminal_evidence`, `go_cmd_layout_on_a_terminal_project_targets_its_source_dirs`.
- Known environment baseline, recorded in Task 1 and not fixed here: `tests/release.test.mjs` fails on this machine (signed tags forced); a local llama-server on port 8080 breaks oracle case `detect-fixture-text-framework-vite`; bun tests can flake on the 5 s timeout under load (they pass in isolation). The RTK proxy summarizes `cargo test` output; read the `test result` line, never grep counts out of a summary.

## Decisions fixed while planning

1. **Per-manifest parsers inside `staleness.rs`, no new crate.** The private `manifest_names_dependency(text, name)` (staleness.rs:79-100, one caller at :228) becomes `manifest_names_dependency(manifest, text, name)`, a `match` on the file name that calls `go_mod_requires`, `python_names_dependency`, or `cargo_names_dependency`. A TOML crate would parse Cargo and pyproject properly but adds a dependency to the binary for three line rules; go.mod is not TOML anyway. Cost if wrong: swapping one function body later.
2. **The evidence table is grouped by manifest and each manifest is read once.** `TERMINAL_EVIDENCE_MANIFESTS` becomes `[(&str, &[(&str, &str)]); 4]` (manifest, then (dependency, reason) rows). Today's loop reads a manifest once per row. A reason is pushed only if it is not already present, so a project carrying `textual` in both `pyproject.toml` and `requirements.txt`, or `bubbletea` under both Charm hosts, says "a textual dependency" once.
3. **Go rows are host-agnostic.** The rows name `bubbletea`, `lipgloss`, `bubbles`; `go_entry_names` accepts each under `github.com/charmbracelet/` or `charm.land/` (`CHARM_MODULE_HOSTS`), with an optional `/v<digits>` suffix, keeping today's reasons ("a bubbletea dependency", "a lipgloss dependency") and adding "a bubbles dependency".
4. **go.mod block states.** `require (` enters a require block; any other `<word> (` enters an "other" block (`replace`, `exclude`, `retract`, and also `tool` and `godebug`, which go.mod allows); a line starting with `)` closes it. Only require-block lines and single-line `require <path> <version>` count. An entry is indirect when the text after its `//` is `indirect` or starts with `indirect;`, which is how `golang.org/x/mod/modfile` reads it. The verb is tokenized up to the first whitespace or `(`, so `require(` works.
5. **Python matching.** After stripping a `#` comment, the line itself and every quoted segment on it (split on `"` and `'`, odd segments) are read as a requirement: a name run that starts alphanumeric and continues over `[A-Za-z0-9._-]`, compared after PEP 503 normalization (`pep503_normalize`: lowercase, each run of `-`, `_`, `.` folded to one `-`), followed after optional spaces by nothing or one of `[ ( < > = ! ~ ; @`. The line form keeps Poetry's `textual = "^0.80"` and requirements lines working; the tail check keeps `description = "Textual app for notes"` out. A comma is deliberately not a tail character: `"Textual, a TUI framework"` in prose would otherwise match. Accepted residuals, recorded here: `keywords = ["textual"]` and a project literally named `textual` still match.
6. **Cargo matching.** A key line matches when, after an optional `"`, it starts with the crate name and the next character (after an optional closing `"`) is whitespace, `=`, or `.`; the `.` admits `ratatui.workspace = true` and `ratatui.version = "0.29"`. A `[` line matches when it ends in `dependencies.ratatui]`, which covers `[dependencies.ratatui]`, `[dev-dependencies.ratatui]`, `[workspace.dependencies.ratatui]`, and `[target.'cfg(unix)'.dependencies.ratatui]`. `#` comments are stripped first. The `require ` and `/vN` handling the old shared scanner applied to Cargo and Python is gone.
7. **Doctor wording through the suggested platform, not text sniffing.** The body of `check_native_platform_evidence` (staleness.rs:179-282) moves into `pub fn native_platform_evidence(..) -> Option<(Finding, &'static str)>` returning the finding and the suggested platform; `check_native_platform_evidence` stays as the wrapper boot calls (staleness.rs:556). `check_workspaces` (staleness_deep.rs:387) calls the inner function and says "carries terminal dependencies" when the suggestion is `terminal`, "carries native build files" otherwise. Mixed mobile and terminal evidence suggests a mobile platform and keeps the build-files wording.
8. **Spec conflict: re-recording `doctor-terminal-evidence-text` and `-json` would change nothing.** Verified: no golden contains "carries native build files"; those two cases run the root-level Tier 1 finding (`platform-native-evidence`), whose sentence this PR does not touch, and their single-line bubbletea `require` still counts. Proposed ruling: they replay unchanged (Task 9 asserts it), and the new wording is pinned by a unit test in `staleness_deep.rs` and a third oracle case, `doctor-monorepo-terminal-evidence` (ctx-monorepo, `apps/b/Cargo.toml` with `ratatui.workspace = true`; apps/b inherits the web PRODUCT.md, so the workspace sentence fires). Reported to the controller.
9. **Spec conflict: section 1's evidence table** still lists `crossterm` and `rich`. Section 7 (later, and the owner's decision) wins; the plan does not edit section 1. Task 9 offers the controller an in-place "(PR 6)" correction of that table if the controller approves it.
10. **Scope addition: `docs/CLI-CONTRACT.md`.** Section 7's PR 6 scope omits it, but the root guide says a verb behavior change updates the contract, and lines 658 (evidence), 682 (`checkWorkspaces`), and 797 (`scan`) describe exactly what changes. Task 9 edits those three lines.
11. **`scan_targets(cwd, git, platform)` order on `terminal`:** git changes (web extensions plus `TERMINAL_EXTENSIONS`), then the root step, then source dirs (`src app components pages public`, then `cmd internal pkg`), then `index.html`, then `has_code`. The root step runs before source dirs because `.` subsumes them: a go.mod with `main.go` at the root and an `internal/` dir would otherwise target only `internal` and miss `main.go`. The root step is: the root holds `Cargo.toml`, `go.mod`, or `pyproject.toml`, and a file with a terminal extension sits directly in the root (one `read_dir_entries(cwd)`, no recursion). It reuses `via: "root"`, because `routing.md:22` enumerates the `via` values and skill text is PR 7's. The existing `signals-terminal` golden (`ctx-terminal`: Cargo.toml plus `src/main.rs`, nothing at the root) stays `["src"]` via `source-dir`.
12. **Spec wording "a flat Python package" is read as modules beside `pyproject.toml`.** A package directory at the root (`mypkg/__init__.py` with no root `.py`) still gets `[]`; so does a Cargo virtual workspace with only `crates/`. Both are recorded as remaining gaps, as is `setup.hasCode` staying `false` for Go and Python projects (it only looks for `package.json` and web dirs). None is in section 7's scope.
13. **The Go layout case gets its own workspace, `ctx-terminal-go`,** with no `package.json`, because a Go project has none. On `ctx-empty` the `package.json` makes `has_code` true, so the golden before this PR would already read `["."]` via `root` and the case would not show the gap routing actually hits. On the new workspace the binary before this PR yields `targets: []`, `via: null`; after it, `["cmd", "internal"]` via `source-dir`.
14. **`IMPECCABLE_NO_STALENESS_CHECK`**: the new cases use `env()` exactly as their siblings do. The pin in the root guide applies to `context` boot cases asserting on other directives; doctor always reports, and signals has no staleness output.

## File structure

| File | Responsibility |
|---|---|
| `crates/context/src/staleness.rs` | grouped `TERMINAL_EVIDENCE_MANIFESTS`, `CHARM_MODULE_HOSTS`, `manifest_names_dependency(manifest, text, name)`, `go_mod_requires`, `go_entry_names`, `python_names_dependency`, `python_requirement_names`, `pep503_normalize`, `cargo_names_dependency`, `native_platform_evidence`; tests in `mod terminal_evidence_tests` |
| `crates/context/src/staleness_deep.rs` | `check_workspaces` wording; one test in `mod tests` |
| `crates/context/src/signals.rs` | `TERMINAL_SOURCE_DIRS`, `TERMINAL_ROOT_MANIFESTS`, `terminal_root_holds_source`, `scan_targets(cwd, git, platform)`, `gather_signals` passes the platform; new `mod tests` |
| `tests/oracle/workspaces/ctx-terminal-go/` | new workspace: `PRODUCT.md`, `go.mod`, `cmd/app/main.go`, `internal/ui/view.go` |
| `tests/oracle/cases/context.mjs` | header line for the workspace; cases `doctor-terminal-evidence-charm-v2`, `doctor-monorepo-terminal-evidence`, `signals-terminal-go-layout` |
| `tests/oracle/golden/` | three new goldens, recorded with `--bin` |
| `docs/CLI-CONTRACT.md` | lines 658, 682, 797 |

---

### Task 1: Branch and baseline

**Files:** none changed.

- [ ] **Step 1: Branch from the fork's main.** PR 6 lands after PRs 4 and 5 (spec section 7 order), so branch from a `main` that contains their merges.

```bash
cd /Volumes/Containers/impeccable
git checkout main && git pull --ff-only origin main
git log --oneline -8   # confirm the PR 4 and PR 5 merge commits are present; if not, stop and ask the controller
git checkout -b terminal-platform-pr6
```

- [ ] **Step 2: Build**

```bash
cargo build --release -p impeccable 2>&1 | tail -n 2
bun --version
```

- [ ] **Step 3: Record the baseline**

```bash
cargo test --workspace 2>&1 | grep -E "^test result|FAILED" | sort | uniq -c
cargo test -p impeccable-context 2>&1 | grep -E "^test result"
IMPECCABLE_BIN="$PWD/target/release/impeccable" bun run test 2>&1 | tail -n 15
for p in doctor- signals- context-terminal; do IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/run.mjs "$p" 2>&1 | tail -n 1; done
node -e "const s=require('fs').readFileSync('docs/CLI-CONTRACT.md','utf8'); console.log('em dashes in contract:', s.split(String.fromCharCode(0x2014)).length-1)"
```

Expected: the known failures only (`tests/release.test.mjs`; `detect-fixture-text-framework-vite` if port 8080 is busy; bun 5 s timeouts under load, which pass when rerun alone). The filtered oracle run passes.

- [ ] **Step 4: No commit.** Record `BASE=$(git rev-parse HEAD)` and the pass counts in the ledger.

---
### Task 2: Weak evidence out, manifests grouped and read once

**Files:**
- Modify: `crates/context/src/staleness.rs` (the table and scanner at lines 61-100, the terminal loop at 226-232, `mod terminal_evidence_tests` at 592-729)

**Interfaces:**
- Produces: `TERMINAL_EVIDENCE_MANIFESTS: [(&str, &[(&str, &str)]); 4]`, `fn manifest_names_dependency(manifest: &str, text: &str, name: &str) -> bool` (private; Tasks 3 to 5 fill its `match`), and the temporary `fn line_names_dependency(text: &str, name: &str) -> bool` (the old scanner, deleted in Task 5).

- [ ] **Step 1: Write the failing tests.** In `mod terminal_evidence_tests`:

(a) In `go_mod_and_python_manifests_count_as_evidence`, delete its third block (the `scratch("req")` block, current lines 637-641, which asserts `rich==13.7.0` alone is evidence). The test keeps its go.mod and pyproject blocks.

(b) Replace the body of `a_prefixed_crate_name_is_not_a_match` (it names `richtext` and `crossterm-winapi`, which stop exercising anything once `rich` and `crossterm` leave the table) with names still in the table:

```rust
    #[test]
    fn a_prefixed_crate_name_is_not_a_match() {
        let root = scratch("prefix");
        write(&root, "Cargo.toml", "[dependencies]\nratatui-macros = \"0.6\"\nratatuix = \"1\"\n");
        write(&root, "requirements.txt", "textual-dev==1.0\ntextualize==1.0\n");
        let f = check_native_platform_evidence(&root, Some("web"), Some(WEB_PRODUCT), Some("PRODUCT.md"));
        assert!(f.is_empty(), "{:?}", f.iter().map(|x| &x.summary).collect::<Vec<_>>());
    }
```

(c) Append before the module's closing `}`:

```rust
    #[test]
    fn rich_alone_is_not_terminal_evidence() {
        let root = scratch("rich-req");
        write(&root, "requirements.txt", "rich==13.7.0\n");
        let f = check_native_platform_evidence(&root, None, None, None);
        assert!(f.is_empty(), "{:?}", f.iter().map(|x| &x.summary).collect::<Vec<_>>());

        let root = scratch("rich-pyproject");
        write(&root, "pyproject.toml", "[project]\ndependencies = [\n  \"rich>=13\",\n]\n");
        let f = check_native_platform_evidence(&root, None, None, None);
        assert!(f.is_empty(), "{:?}", f.iter().map(|x| &x.summary).collect::<Vec<_>>());
    }

    #[test]
    fn crossterm_alone_is_not_terminal_evidence() {
        let root = scratch("crossterm");
        write(&root, "Cargo.toml", "[dependencies]\ncrossterm = \"0.28\"\n");
        let f = check_native_platform_evidence(&root, Some("web"), Some(WEB_PRODUCT), Some("PRODUCT.md"));
        assert!(f.is_empty(), "{:?}", f.iter().map(|x| &x.summary).collect::<Vec<_>>());
    }

    #[test]
    fn a_reason_found_in_two_manifests_is_listed_once() {
        let root = scratch("textual-twice");
        write(&root, "pyproject.toml", "[project]\ndependencies = [\n  \"textual>=0.80\",\n]\n");
        write(&root, "requirements.txt", "textual==0.80.0\n");
        let f = check_native_platform_evidence(&root, None, None, None);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].summary.matches("a textual dependency").count(), 1, "{}", f[0].summary);
    }
```

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test -p impeccable-context terminal_evidence_tests 2>&1 | grep -E "^test |test result"
```

Expected: `rich_alone_is_not_terminal_evidence`, `crossterm_alone_is_not_terminal_evidence`, and `a_reason_found_in_two_manifests_is_listed_once` FAIL (the first two find "a rich dependency" and "a crossterm dependency"; the third lists "a textual dependency and a textual dependency"). The rest pass.

- [ ] **Step 3: Implement.** Replace staleness.rs lines 61-100 (the doc comment above `TERMINAL_EVIDENCE_MANIFESTS`, the constant, and `manifest_names_dependency`) with:

```rust
/// Text manifests at the project root whose dependency entries name a
/// terminal stack: (manifest file, [(dependency, reason)]). Each manifest is
/// read once and parsed by `manifest_names_dependency`. Only the root
/// manifest is read (Tier 1 walks nothing), so Cargo member crates are not
/// seen. `rich` and `crossterm` are absent on purpose: web backends and plain
/// CLIs use both without a full-screen UI, so neither is evidence alone.
const TERMINAL_EVIDENCE_MANIFESTS: [(&str, &[(&str, &str)]); 4] = [
    ("Cargo.toml", &[("ratatui", "a ratatui dependency")]),
    (
        "go.mod",
        &[
            ("github.com/charmbracelet/bubbletea", "a bubbletea dependency"),
            ("github.com/charmbracelet/lipgloss", "a lipgloss dependency"),
        ],
    ),
    ("pyproject.toml", &[("textual", "a textual dependency")]),
    ("requirements.txt", &[("textual", "a textual dependency")]),
];

/// True when `manifest`'s text names the dependency `name`.
fn manifest_names_dependency(_manifest: &str, text: &str, name: &str) -> bool {
    line_names_dependency(text, name)
}

/// The PR 1 line-prefix scanner. Tasks 3 to 5 of the PR 6 plan replace it
/// manifest by manifest; it is deleted once Cargo.toml has its own parser.
fn line_names_dependency(text: &str, name: &str) -> bool {
    text.lines().any(|line| {
        let t = line.trim_start().trim_start_matches(|c| c == '"' || c == '\'');
        let t = t.strip_prefix("require ").map(str::trim_start).unwrap_or(t);
        if !t.starts_with(name) {
            return false;
        }
        let rest = &t[name.len()..];
        let rest = match rest.strip_prefix("/v") {
            Some(after) if after.starts_with(|c: char| c.is_ascii_digit()) => {
                after.trim_start_matches(|c: char| c.is_ascii_digit())
            }
            _ => rest,
        };
        match rest.chars().next() {
            None => true,
            Some(c) => !(c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '/'),
        }
    })
}
```

Then replace the terminal loop in `check_native_platform_evidence` (current lines 226-232, `for (manifest, name, reason) in TERMINAL_EVIDENCE_MANIFESTS { ... }`) with:

```rust
    for (manifest, rows) in TERMINAL_EVIDENCE_MANIFESTS {
        let Some(text) = safe_read(&jsp::join(&[project_root, manifest])) else { continue };
        for &(name, reason) in rows {
            // One reason per dependency even when two manifests (or two
            // module hosts) name it.
            if manifest_names_dependency(manifest, &text, name) && !evidence.iter().any(|e| e.reason == reason) {
                evidence.push(NativeEvidence { platform: "terminal", reason });
            }
        }
    }
```

- [ ] **Step 4: Run to verify they pass**

```bash
cargo test -p impeccable-context terminal_evidence_tests 2>&1 | grep -E "FAILED|test result"
cargo build -p impeccable-context 2>&1 | grep -E "^warning|^error" | head
```

Expected: `test result: ok.` with every `terminal_evidence_tests` test passing; no new warnings.

- [ ] **Step 5: Commit**

```bash
git add crates/context/src/staleness.rs
git commit -m "$(printf 'Stop counting rich or crossterm alone as terminal evidence\n\nGroup the terminal evidence table by manifest so each root manifest is\nread once, and list a reason once when two manifests carry it.\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 3: go.mod: require entries only, Charm v2 paths, bubbles

**Files:**
- Modify: `crates/context/src/staleness.rs` (the go.mod rows, `manifest_names_dependency`, two new functions, tests)

**Interfaces:**
- Produces: `const CHARM_MODULE_HOSTS: [&str; 2]`, `fn go_mod_requires(text: &str, name: &str) -> bool`, `fn go_entry_names(entry: &str, name: &str) -> bool` (all private).
- Consumes: `manifest_names_dependency` from Task 2.

- [ ] **Step 1: Write the failing tests.** Rename and invert `go_major_version_module_path_counts_as_evidence` (current lines 692-699) so the indirect lipgloss line no longer counts:

```rust
    #[test]
    fn go_major_version_path_counts_and_an_indirect_require_does_not() {
        let root = scratch("go-v2");
        write(&root, "go.mod", "module example.com/app\n\ngo 1.22\n\nrequire (\n\tgithub.com/charmbracelet/bubbletea/v2 v2.0.0\n\tgithub.com/charmbracelet/lipgloss/v2 v2.0.0 // indirect\n)\n");
        let f = check_native_platform_evidence(&root, None, Some("# P\n"), Some("PRODUCT.md"));
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("carries a bubbletea dependency."), "{}", f[0].summary);
        assert!(!f[0].summary.contains("lipgloss"), "{}", f[0].summary);
    }
```

Append:

```rust
    #[test]
    fn go_mod_blocks_other_than_require_do_not_count() {
        let root = scratch("go-blocks");
        write(
            &root,
            "go.mod",
            "module example.com/app\n\ngo 1.24\n\nreplace (\n\tgithub.com/charmbracelet/bubbletea => ../fork\n)\n\nexclude (\n\tgithub.com/charmbracelet/lipgloss v0.9.0\n)\n\nretract (\n\tv1.0.1\n)\n\ntool (\n\tcharm.land/bubbles/v2/cmd/demo\n)\n\nrequire github.com/charmbracelet/bubbletea v1.2.0 // indirect\n",
        );
        let f = check_native_platform_evidence(&root, None, Some("# P\n"), Some("PRODUCT.md"));
        assert!(f.is_empty(), "{:?}", f.iter().map(|x| &x.summary).collect::<Vec<_>>());
    }

    #[test]
    fn charm_land_module_paths_and_bubbles_count_as_evidence() {
        let root = scratch("charm-land");
        write(&root, "go.mod", "module example.com/app\n\ngo 1.24\n\nrequire (\n\tcharm.land/bubbletea/v2 v2.0.0\n\tcharm.land/bubbles/v2 v2.0.0\n)\n");
        let f = check_native_platform_evidence(&root, None, Some("# P\n"), Some("PRODUCT.md"));
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("a bubbletea dependency and a bubbles dependency"), "{}", f[0].summary);

        let root = scratch("bubbles-v1");
        write(&root, "go.mod", "module example.com/app\n\ngo 1.22\n\nrequire github.com/charmbracelet/bubbles v0.20.0\n");
        let f = check_native_platform_evidence(&root, None, Some("# P\n"), Some("PRODUCT.md"));
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("a bubbles dependency"), "{}", f[0].summary);
    }

    #[test]
    fn a_module_on_both_charm_hosts_is_listed_once() {
        let root = scratch("both-hosts");
        write(&root, "go.mod", "module example.com/app\n\ngo 1.24\n\nrequire (\n\tgithub.com/charmbracelet/bubbletea v1.3.0\n\tcharm.land/bubbletea/v2 v2.0.0\n)\n");
        let f = check_native_platform_evidence(&root, None, Some("# P\n"), Some("PRODUCT.md"));
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].summary.matches("a bubbletea dependency").count(), 1, "{}", f[0].summary);
    }
```

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test -p impeccable-context terminal_evidence_tests 2>&1 | grep -E "^test |test result"
```

Expected FAIL: `go_major_version_path_counts_and_an_indirect_require_does_not` (lipgloss is listed), `go_mod_blocks_other_than_require_do_not_count` (the `exclude (` lipgloss line and the indirect single-line require match), `charm_land_module_paths_and_bubbles_count_as_evidence` (no `charm.land` or `bubbles` row). `a_module_on_both_charm_hosts_is_listed_once` passes already (only the github line matches today); it guards Step 3's host list against listing bubbletea twice.

- [ ] **Step 3: Implement.** In `TERMINAL_EVIDENCE_MANIFESTS`, replace the `go.mod` entry with:

```rust
    (
        "go.mod",
        &[
            ("bubbletea", "a bubbletea dependency"),
            ("lipgloss", "a lipgloss dependency"),
            ("bubbles", "a bubbles dependency"),
        ],
    ),
```

Change `manifest_names_dependency` to:

```rust
/// True when `manifest`'s text names the dependency `name`.
fn manifest_names_dependency(manifest: &str, text: &str, name: &str) -> bool {
    match manifest {
        "go.mod" => go_mod_requires(text, name),
        _ => line_names_dependency(text, name),
    }
}
```

and add after it:

```rust
/// Charm modules live under both hosts: v1 under GitHub, v2 under the
/// `charm.land` vanity path (`charm.land/bubbletea/v2`).
const CHARM_MODULE_HOSTS: [&str; 2] = ["github.com/charmbracelet/", "charm.land/"];

/// go.mod: true when a `require` entry names the Charm module `name`. A
/// single-line `require <path> <version>` and the lines of a `require (` block
/// count. Nothing inside another block (`replace (`, `exclude (`, `retract (`,
/// `tool (`, `godebug (`) counts, and an `// indirect` entry does not.
fn go_mod_requires(text: &str, name: &str) -> bool {
    // None outside a block; Some(true) inside `require (`; Some(false) inside any other block.
    let mut block: Option<bool> = None;
    for line in text.lines() {
        let t = line.trim();
        if let Some(in_require) = block {
            if t.starts_with(')') {
                block = None;
            } else if in_require && go_entry_names(t, name) {
                return true;
            }
            continue;
        }
        let verb_end = t.find(|c: char| c.is_whitespace() || c == '(').unwrap_or(t.len());
        let (verb, rest) = (&t[..verb_end], t[verb_end..].trim_start());
        if rest.starts_with('(') {
            block = Some(verb == "require");
        } else if verb == "require" && go_entry_names(rest, name) {
            return true;
        }
    }
    false
}

/// One require entry, `<module path> <version> [// comment]`: it names `name`
/// when the path is `<host><name>` or `<host><name>/v<digits>` for a Charm
/// host and the comment does not mark it indirect (`indirect`, or
/// `indirect;` followed by more, as `golang.org/x/mod/modfile` reads it).
fn go_entry_names(entry: &str, name: &str) -> bool {
    if let Some(i) = entry.find("//") {
        let comment = entry[i + 2..].trim();
        if comment == "indirect" || comment.starts_with("indirect;") {
            return false;
        }
    }
    let path = entry.split_whitespace().next().unwrap_or("").trim_matches('"');
    CHARM_MODULE_HOSTS.iter().any(|host| match path.strip_prefix(host).and_then(|p| p.strip_prefix(name)) {
        Some("") => true,
        Some(rest) => rest
            .strip_prefix("/v")
            .map_or(false, |digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())),
        None => false,
    })
}
```

- [ ] **Step 4: Run to verify they pass**

```bash
cargo test -p impeccable-context terminal_evidence_tests 2>&1 | grep -E "FAILED|test result"
```

Expected: `test result: ok.`; `go_mod_single_line_require_counts_as_evidence`, `go_replace_directive_is_not_a_dependency_line`, and the go.mod half of `go_mod_and_python_manifests_count_as_evidence` still pass.

- [ ] **Step 5: Commit**

```bash
git add crates/context/src/staleness.rs
git commit -m "$(printf 'Count only direct go.mod requires as terminal evidence\n\nRead go.mod by block: require entries count, // indirect entries and\nreplace, exclude, retract, tool, and godebug blocks do not. Charm modules\nmatch under github.com/charmbracelet/ and charm.land/, and bubbles joins\nbubbletea and lipgloss.\n\nAI assistance: drafted with Claude Code.')"
```

---
### Task 4: Python manifests: PEP 503 names and quoted strings

**Files:**
- Modify: `crates/context/src/staleness.rs` (`manifest_names_dependency`, three new functions, tests)

**Interfaces:**
- Produces: `fn python_names_dependency(text: &str, name: &str) -> bool`, `fn python_requirement_names(spec: &str, name: &str) -> bool`, `fn pep503_normalize(name: &str) -> String` (all private; the test module imports `pep503_normalize`).

- [ ] **Step 1: Write the failing tests.** Change the module's import line from `use super::check_native_platform_evidence;` to:

```rust
    use super::{check_native_platform_evidence, pep503_normalize};
```

Append:

```rust
    #[test]
    fn pep503_folds_case_and_separator_runs() {
        assert_eq!(pep503_normalize("Textual"), "textual");
        assert_eq!(pep503_normalize("Textual__Dev.-Tools"), "textual-dev-tools");
    }

    #[test]
    fn pyproject_inline_array_and_capitalized_names_count() {
        let root = scratch("py-inline");
        write(&root, "pyproject.toml", "[project]\nname = \"notes\"\ndependencies = [\"Textual>=0.80\", \"rich\"]\n");
        let f = check_native_platform_evidence(&root, None, None, None);
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("carries a textual dependency."), "{}", f[0].summary);

        let root = scratch("req-extras");
        write(&root, "requirements.txt", "-r base.txt\nTextual[syntax] >= 0.80 ; python_version >= \"3.9\"\n");
        let f = check_native_platform_evidence(&root, None, None, None);
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("a textual dependency"), "{}", f[0].summary);
    }

    #[test]
    fn poetry_key_form_still_counts() {
        let root = scratch("poetry");
        write(&root, "pyproject.toml", "[tool.poetry.dependencies]\npython = \"^3.11\"\nTextual = \"^0.80\"\n");
        let f = check_native_platform_evidence(&root, None, None, None);
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("a textual dependency"), "{}", f[0].summary);
    }

    #[test]
    fn prose_and_comments_naming_textual_are_not_dependencies() {
        let root = scratch("py-prose");
        write(
            &root,
            "pyproject.toml",
            "[project]\nname = \"notes\"\ndescription = \"Textual app for managing notes\"\ndependencies = [\"httpx>=0.27\"]  # textual>=0.80 later\n",
        );
        write(&root, "requirements.txt", "# textual==0.80.0\nhttpx==0.27.0\n");
        let f = check_native_platform_evidence(&root, None, None, None);
        assert!(f.is_empty(), "{:?}", f.iter().map(|x| &x.summary).collect::<Vec<_>>());
    }
```

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test -p impeccable-context terminal_evidence_tests 2>&1 | grep -E "^error|^test |test result"
```

Expected: the module fails to compile (`pep503_normalize` is not found in `super`). Once Step 3 lands, before-and-after is: `pyproject_inline_array_and_capitalized_names_count` fails on the old scanner (the array sits on a `dependencies = [` line and `Textual` is capitalized); `poetry_key_form_still_counts` fails on the old scanner (capital `T`); `prose_and_comments_naming_textual_are_not_dependencies` passes on the old scanner and guards the new quoted-string reading against prose and comments.

- [ ] **Step 3: Implement.** Change `manifest_names_dependency` to:

```rust
/// True when `manifest`'s text names the dependency `name`.
fn manifest_names_dependency(manifest: &str, text: &str, name: &str) -> bool {
    match manifest {
        "go.mod" => go_mod_requires(text, name),
        "Cargo.toml" => line_names_dependency(text, name),
        _ => python_names_dependency(text, name),
    }
}
```

and add after `go_entry_names`:

```rust
/// pyproject.toml and requirements.txt: after stripping a `#` comment, the
/// line itself (a requirements line, or a Poetry `textual = "^0.80"` key) and
/// every quoted string on it (a PEP 621 array such as
/// `dependencies = ["Textual>=0.80", "rich"]`) are read as requirements.
fn python_names_dependency(text: &str, name: &str) -> bool {
    text.lines().any(|line| {
        let line = line.split('#').next().unwrap_or("");
        python_requirement_names(line, name)
            || line.split(['"', '\'']).skip(1).step_by(2).any(|s| python_requirement_names(s, name))
    })
}

/// A requirement names the package `name` (already normalized) when its
/// leading name run, PEP 503 normalized, equals it and what follows is a
/// requirement tail: nothing, extras `[`, a version `( < > = ! ~`, a marker
/// `;`, or a URL `@`. `=` also admits a Poetry key. Prose such as
/// "Textual app for notes" fails the tail check. Accepted residuals:
/// `keywords = ["textual"]` and a project named `textual` still match.
fn python_requirement_names(spec: &str, name: &str) -> bool {
    let spec = spec.trim_start();
    if !spec.starts_with(|c: char| c.is_ascii_alphanumeric()) {
        return false;
    }
    let end = spec
        .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')))
        .unwrap_or(spec.len());
    if pep503_normalize(&spec[..end]) != name {
        return false;
    }
    let tail = spec[end..].trim_start();
    tail.is_empty() || tail.starts_with(['[', '(', '<', '>', '=', '!', '~', ';', '@'])
}

/// PEP 503 name normalization: lowercase, each run of `-`, `_`, `.` becomes
/// one `-`.
fn pep503_normalize(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        if matches!(c, '-' | '_' | '.') {
            if !out.ends_with('-') {
                out.push('-');
            }
        } else {
            out.push(c.to_ascii_lowercase());
        }
    }
    out
}
```

- [ ] **Step 4: Run to verify they pass**

```bash
cargo test -p impeccable-context terminal_evidence_tests 2>&1 | grep -E "FAILED|test result"
```

Expected: `test result: ok.`, including `a_prefixed_crate_name_is_not_a_match` (`textual-dev` and `textualize` normalize to other names), `rich_alone_is_not_terminal_evidence`, `a_reason_found_in_two_manifests_is_listed_once`, and the pyproject half of `go_mod_and_python_manifests_count_as_evidence`.

- [ ] **Step 5: Commit**

```bash
git add crates/context/src/staleness.rs
git commit -m "$(printf 'Match Python terminal evidence by PEP 503 name in any quoted string\n\nA pyproject inline array, a capitalized name, and requirement extras\nnow count; prose and comments naming textual do not.\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 5: Cargo.toml: dotted keys and dependency headers

**Files:**
- Modify: `crates/context/src/staleness.rs` (`manifest_names_dependency`, one new function, delete `line_names_dependency`, tests)

**Interfaces:**
- Produces: `fn cargo_names_dependency(text: &str, name: &str) -> bool` (private). Removes `line_names_dependency`.

- [ ] **Step 1: Write the failing tests.** Append:

```rust
    #[test]
    fn cargo_dotted_keys_and_dependency_headers_count() {
        let bodies = [
            "[dependencies]\nratatui.workspace = true\n",
            "[dependencies]\nratatui.version = \"0.29\"\n",
            "[dependencies.ratatui]\nversion = \"0.29\"\n",
            "[target.'cfg(unix)'.dependencies.ratatui]\nversion = \"0.29\"\n",
            "[workspace.dependencies]\n\"ratatui\" = \"0.29\" # tui\n",
        ];
        for (i, body) in bodies.iter().enumerate() {
            let root = scratch(&format!("cargo-form-{i}"));
            write(&root, "Cargo.toml", body);
            let f = check_native_platform_evidence(&root, Some("web"), Some(WEB_PRODUCT), Some("PRODUCT.md"));
            assert_eq!(f.len(), 1, "{body}");
            assert!(f[0].summary.contains("a ratatui dependency"), "{body}: {}", f[0].summary);
        }
    }

    #[test]
    fn ratatui_named_only_in_package_metadata_is_not_a_dependency() {
        let root = scratch("cargo-metadata");
        write(
            &root,
            "Cargo.toml",
            "[package]\nname = \"ratatui-notes\"\ndescription = \"ratatui demo\"\nkeywords = [\"ratatui\"]\n\n[dependencies]\ncrossterm = \"0.28\"\n",
        );
        let f = check_native_platform_evidence(&root, Some("web"), Some(WEB_PRODUCT), Some("PRODUCT.md"));
        assert!(f.is_empty(), "{:?}", f.iter().map(|x| &x.summary).collect::<Vec<_>>());
    }
```

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test -p impeccable-context terminal_evidence_tests 2>&1 | grep -E "^test |test result"
```

Expected: `cargo_dotted_keys_and_dependency_headers_count` FAILS on its first body (`ratatui.workspace`: the old scanner rejects a `.` after the name). `ratatui_named_only_in_package_metadata_is_not_a_dependency` passes today and guards the new key reading.

- [ ] **Step 3: Implement.** Change `manifest_names_dependency` to its final form:

```rust
/// True when `manifest`'s text names the dependency `name`.
fn manifest_names_dependency(manifest: &str, text: &str, name: &str) -> bool {
    match manifest {
        "go.mod" => go_mod_requires(text, name),
        "Cargo.toml" => cargo_names_dependency(text, name),
        _ => python_names_dependency(text, name),
    }
}
```

Delete `line_names_dependency` and its doc comment. Add after `pep503_normalize`:

```rust
/// Cargo.toml: a key line naming the crate (`ratatui = "0.29"`,
/// `ratatui.workspace = true`, `"ratatui" = ...`) or a table header ending in
/// `dependencies.ratatui]` (`[dependencies.ratatui]`,
/// `[target.'cfg(unix)'.dependencies.ratatui]`). `#` comments are ignored.
fn cargo_names_dependency(text: &str, name: &str) -> bool {
    let header_tail = format!("dependencies.{}]", name);
    text.lines().any(|line| {
        let t = line.split('#').next().unwrap_or("").trim();
        if t.starts_with('[') {
            return t.ends_with(&header_tail);
        }
        let key = t.strip_prefix('"').unwrap_or(t);
        match key.strip_prefix(name) {
            Some(rest) => {
                let rest = rest.strip_prefix('"').unwrap_or(rest);
                rest.starts_with(|c: char| c.is_whitespace() || c == '=' || c == '.')
            }
            None => false,
        }
    })
}
```

- [ ] **Step 4: Run to verify they pass**

```bash
cargo test -p impeccable-context terminal_evidence_tests 2>&1 | grep -E "FAILED|test result"
cargo build -p impeccable-context 2>&1 | grep -E "^warning|^error" | head
```

Expected: `test result: ok.` for every test in the module (Tasks 2 to 5 and the PR 1 tests); no dead-code warning (the old scanner is gone).

- [ ] **Step 5: Commit**

```bash
git add crates/context/src/staleness.rs
git commit -m "$(printf 'Match Cargo dotted keys and dependency table headers as evidence\n\nratatui.workspace = true and [dependencies.ratatui] now count. The\nshared line-prefix scanner is gone; each manifest has its own reader.\n\nAI assistance: drafted with Claude Code.')"
```

---
### Task 6: Doctor wording: "carries terminal dependencies"

**Files:**
- Modify: `crates/context/src/staleness.rs` (`check_native_platform_evidence`, lines 178-282)
- Modify: `crates/context/src/staleness_deep.rs` (import at line 7; `check_workspaces` at 387-410; `mod tests` at 436-500)

**Interfaces:**
- Produces: `pub fn native_platform_evidence(project_root: &str, platform: Option<&str>, product: Option<&str>, product_path: Option<&str>) -> Option<(Finding, &'static str)>`, the Tier 1 finding plus the platform it suggests (`terminal`, `ios`, `android`, or `adaptive`). `check_native_platform_evidence` keeps its signature and output; the boot caller at staleness.rs:556 is untouched.

- [ ] **Step 1: Write the failing test.** In `staleness_deep.rs`'s `mod tests`, change `use super::check_hook_installation;` to:

```rust
    use super::{check_hook_installation, check_workspaces};
    use crate::context::TargetCandidate;
```

and append before the module's closing `}`:

```rust
    #[test]
    fn workspace_terminal_evidence_is_worded_as_dependencies_and_mobile_as_build_files() {
        let root = tmp();
        write(&root, "apps/tui/Cargo.toml", "[dependencies]\nratatui = \"0.29\"\n");
        write(&root, "apps/phone/ios/Podfile", "platform :ios, '15.0'\n");
        let candidate = |name: &str| TargetCandidate {
            name: name.to_string(),
            path: format!("apps/{name}"),
            target_example: format!("apps/{name}/src/main.rs"),
            product_status: "missing",
            product_path: None,
            design_status: "missing",
            design_path: None,
        };
        let (findings, _) = check_workspaces(&root, &[candidate("tui"), candidate("phone")]);
        let summary_for = |path: &str| {
            findings
                .iter()
                .find(|f| f.id == "workspace-platform-native-evidence" && f.summary.contains(path))
                .unwrap_or_else(|| panic!("no evidence finding for {path}: {findings:?}"))
                .summary
                .clone()
        };
        let tui = summary_for("`apps/tui`");
        assert!(tui.contains("the workspace itself carries terminal dependencies."), "{tui}");
        assert!(!tui.contains("native build files"), "{tui}");
        let phone = summary_for("`apps/phone`");
        assert!(phone.contains("the workspace itself carries native build files."), "{phone}");
        let _ = std::fs::remove_dir_all(&root);
    }
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test -p impeccable-context workspace_terminal_evidence 2>&1 | grep -E "^test |panicked|test result"
```

Expected: FAIL; the `apps/tui` summary says "the workspace itself carries native build files."

- [ ] **Step 3: Implement the inner function.** In `staleness.rs`, replace the whole of `check_native_platform_evidence` (from its `/// JS: checkNativePlatformEvidence` comment through its closing `}`, current lines 178-282) with the two functions below. The body is today's, with three changes: early exits return `None`, `platforms` and `mobile` are typed `Vec<&'static str>` so the suggestion can leave the function, and the tail returns `Some((finding, suggested))`.

```rust
/// JS: checkNativePlatformEvidence
pub fn check_native_platform_evidence(
    project_root: &str,
    platform: Option<&str>,
    product: Option<&str>,
    product_path: Option<&str>,
) -> Vec<Finding> {
    native_platform_evidence(project_root, platform, product, product_path).map(|(f, _)| f).into_iter().collect()
}

/// The `platform-native-evidence` finding and the platform it suggests
/// (`terminal`, `ios`, `android`, or `adaptive`). `check_workspaces` reads the
/// suggestion to word its own sentence.
pub fn native_platform_evidence(
    project_root: &str,
    platform: Option<&str>,
    product: Option<&str>,
    product_path: Option<&str>,
) -> Option<(Finding, &'static str)> {
    if project_root.is_empty() {
        return None;
    }
    if let Some(p) = platform {
        if !p.is_empty() && p != "web" {
            return None;
        }
    }
    let mut evidence: Vec<NativeEvidence> = Vec::new();
    for (rel, platform, reason) in NATIVE_EVIDENCE_PATHS {
        if exists(&jsp::join(&[project_root, rel])) {
            evidence.push(NativeEvidence { platform, reason });
        }
    }
    if let Some(pkg) = read_json(&jsp::join(&[project_root, "package.json"])) {
        // JS: { ...pkg.dependencies, ...pkg.devDependencies } then deps[name] truthy
        let dep_truthy = |name: &str| -> bool {
            let mut v: Option<&Value> = None;
            if let Some(d) = pkg.get("dependencies").and_then(|d| d.as_object()) {
                if let Some(x) = d.get(name) {
                    v = Some(x);
                }
            }
            if let Some(d) = pkg.get("devDependencies").and_then(|d| d.as_object()) {
                if let Some(x) = d.get(name) {
                    v = Some(x);
                }
            }
            match v {
                None => false,
                Some(x) => js_truthy(x),
            }
        };
        if pkg.is_object() || pkg.is_array() || (!pkg.is_null() && js_truthy(&pkg)) {
            for (name, platform, reason) in NATIVE_EVIDENCE_DEPENDENCIES {
                if dep_truthy(name) {
                    evidence.push(NativeEvidence { platform, reason });
                }
            }
        }
    }
    for (manifest, rows) in TERMINAL_EVIDENCE_MANIFESTS {
        let Some(text) = safe_read(&jsp::join(&[project_root, manifest])) else { continue };
        for &(name, reason) in rows {
            // One reason per dependency even when two manifests (or two
            // module hosts) name it.
            if manifest_names_dependency(manifest, &text, name) && !evidence.iter().any(|e| e.reason == reason) {
                evidence.push(NativeEvidence { platform: "terminal", reason });
            }
        }
    }
    if evidence.is_empty() {
        return None;
    }
    let mut platforms: Vec<&'static str> = Vec::new();
    for e in &evidence {
        if !platforms.contains(&e.platform) {
            platforms.push(e.platform);
        }
    }
    // Mobile evidence outranks terminal evidence: a Flutter app with a Rust
    // helper is still a mobile app. Terminal is suggested only when it is the
    // only kind of evidence present.
    let mobile: Vec<&'static str> = platforms.iter().copied().filter(|p| *p != "terminal").collect();
    let suggested: &'static str = if mobile.is_empty() {
        "terminal"
    } else if mobile.len() > 1 || mobile.contains(&"adaptive") {
        "adaptive"
    } else {
        mobile[0]
    };
    let consequence = if suggested == "terminal" {
        "Web guidance is being applied to a terminal codebase, and the terminal reference never loads."
    } else {
        "Web guidance is being applied to a native codebase, and the iOS and Android references never load."
    };
    let reference = if suggested == "terminal" { "terminal" } else { "native" };
    let declared = if platform == Some("web") {
        "PRODUCT.md declares `## Platform: web`"
    } else if product.map(|p| !p.is_empty()).unwrap_or(false) {
        "PRODUCT.md has no `## Platform` section, so the project resolves to web"
    } else {
        "no PRODUCT.md declares a platform, so the project resolves to web"
    };
    Some((
        finding(
            "platform-native-evidence",
            "PRODUCT.md",
            product_path.map(|s| s.to_string()),
            "mention",
            format!(
                "{}, but the project carries {}. {}",
                declared,
                evidence.iter().map(|e| e.reason).collect::<Vec<_>>().join(" and "),
                consequence
            ),
            format!(
                "Ask the user whether `## Platform` should be `{}`. If it should, write the value and load the matching {} reference before designing.",
                suggested, reference
            ),
        ),
        suggested,
    ))
}
```

Before replacing, diff the current function body against this block by eye (`sed -n '178,282p' crates/context/src/staleness.rs`): anything other than the three listed changes and the Task 2 loop means the file moved since planning; carry the current text over rather than this copy.

- [ ] **Step 4: Implement the wording.** In `staleness_deep.rs`, change line 7 to:

```rust
use crate::staleness::{finding, js_truthy, native_platform_evidence, to_relative, unique_roots, Finding};
```

Replace lines 387-410 (from `let native = check_native_platform_evidence(` through the `for` loop's closing `}`) with:

```rust
        if let Some((entry, suggested)) =
            native_platform_evidence(&workspace_root, platform.as_deref(), product.as_deref(), c.product_path.as_deref())
        {
            let inherited = c.product_status == "inherited";
            // Terminal evidence is a dependency line in a manifest, not a build file.
            let carries = if suggested == "terminal" { "terminal dependencies" } else { "native build files" };
            findings.push(finding(
                "workspace-platform-native-evidence",
                "PRODUCT.md",
                Some(c.product_path.clone().unwrap_or_else(|| format!("{}/PRODUCT.md", c.path))),
                "mention",
                format!(
                    "Workspace `{}` {} that resolves to web, but the workspace itself carries {}. {}",
                    c.path,
                    if inherited { "inherits the repo-root PRODUCT.md" } else { "has a PRODUCT.md" },
                    carries,
                    entry.summary
                ),
                if inherited {
                    format!(
                        "Give `{}` its own PRODUCT.md with the right `## Platform`. An inherited record cannot describe two platforms at once.",
                        c.path
                    )
                } else {
                    entry.fix
                },
            ));
        }
```

- [ ] **Step 5: Run to verify it passes**

```bash
cargo test -p impeccable-context 2>&1 | grep -E "FAILED|test result"
cargo build -p impeccable-context 2>&1 | grep -E "^warning|^error" | head
```

Expected: `test result: ok.`; the new test and every `terminal_evidence_tests` test pass; no warnings.

- [ ] **Step 6: Commit**

```bash
git add crates/context/src/staleness.rs crates/context/src/staleness_deep.rs
git commit -m "$(printf 'Say a workspace carries terminal dependencies, not native build files\n\nThe evidence check now also returns the platform it suggests, and the\ndoctor workspace sweep words terminal evidence as dependencies. Mobile\nevidence keeps the build-files wording.\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 7: Routing targets: `scan_targets` learns the platform

**Files:**
- Modify: `crates/context/src/signals.rs` (imports at lines 3-10; constants and `scan_targets` at 265-320; `gather_signals` at 322-343; a new `mod tests` at the end)

**Interfaces:**
- Consumes: `impeccable_core::checks::terminal::TERMINAL_EXTENSIONS` (`&[&str]`: `.rs .go .py .tcss`, the list the detect walker and the hook already share) and `crate::util::read_dir_entries` (sorted `DirEntry { name, is_dir, is_file }`, symlinks report neither).
- Produces: `fn scan_targets(cwd: &str, git: &Value, platform: Option<&str>) -> Value` (private; one caller), `const TERMINAL_SOURCE_DIRS: [&str; 3]`, `const TERMINAL_ROOT_MANIFESTS: [&str; 3]`, `fn terminal_root_holds_source(cwd: &str) -> bool`.

- [ ] **Step 1: Write the failing tests.** Append to the end of `signals.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::scan_targets;
    use serde_json::{json, Value};

    fn scratch(name: &str) -> String {
        let base = std::env::temp_dir().join(format!("impeccable-scan-targets-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        base.to_string_lossy().into_owned()
    }

    fn write(root: &str, rel: &str, body: &str) {
        let p = std::path::Path::new(root).join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }

    fn no_git() -> Value {
        json!({ "isRepo": false, "changedFiles": [] })
    }

    fn targets_and_via(scan: &Value) -> (Vec<String>, Value) {
        let targets = scan["targets"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
        (targets, scan["via"].clone())
    }

    #[test]
    fn go_cmd_layout_on_a_terminal_project_targets_its_source_dirs() {
        let root = scratch("go-cmd");
        write(&root, "go.mod", "module example.com/app\n\ngo 1.24\n");
        write(&root, "cmd/app/main.go", "package main\n");
        write(&root, "internal/ui/view.go", "package ui\n");
        write(&root, "pkg/style/style.go", "package style\n");
        let (targets, via) = targets_and_via(&scan_targets(&root, &no_git(), Some("terminal")));
        assert_eq!(targets, ["cmd", "internal", "pkg"]);
        assert_eq!(via, "source-dir");
    }

    #[test]
    fn go_module_with_main_beside_go_mod_targets_the_root() {
        let root = scratch("go-root");
        write(&root, "go.mod", "module example.com/app\n\ngo 1.24\n");
        write(&root, "main.go", "package main\n");
        write(&root, "internal/ui/view.go", "package ui\n");
        let (targets, via) = targets_and_via(&scan_targets(&root, &no_git(), Some("terminal")));
        assert_eq!(targets, ["."]);
        assert_eq!(via, "root");
    }

    #[test]
    fn python_module_beside_pyproject_targets_the_root() {
        let root = scratch("py-root");
        write(&root, "pyproject.toml", "[project]\nname = \"notes\"\n");
        write(&root, "app.py", "from textual.app import App\n");
        let (targets, via) = targets_and_via(&scan_targets(&root, &no_git(), Some("terminal")));
        assert_eq!(targets, ["."]);
        assert_eq!(via, "root");
    }

    #[test]
    fn rust_crate_with_src_keeps_its_src_target() {
        let root = scratch("rust-src");
        write(&root, "Cargo.toml", "[dependencies]\nratatui = \"0.29\"\n");
        write(&root, "src/main.rs", "fn main() {}\n");
        let (targets, via) = targets_and_via(&scan_targets(&root, &no_git(), Some("terminal")));
        assert_eq!(targets, ["src"]);
        assert_eq!(via, "source-dir");
    }

    #[test]
    fn source_files_without_a_root_manifest_do_not_target_the_root() {
        let root = scratch("no-manifest");
        write(&root, "main.go", "package main\n");
        let (targets, via) = targets_and_via(&scan_targets(&root, &no_git(), Some("terminal")));
        assert!(targets.is_empty(), "{targets:?}");
        assert_eq!(via, Value::Null);
    }

    #[test]
    fn web_project_ignores_terminal_layouts() {
        let root = scratch("web-go");
        write(&root, "go.mod", "module example.com/app\n\ngo 1.24\n");
        write(&root, "main.go", "package main\n");
        write(&root, "cmd/app/main.go", "package main\n");
        for platform in [None, Some("web")] {
            let (targets, via) = targets_and_via(&scan_targets(&root, &no_git(), platform));
            assert!(targets.is_empty(), "{platform:?}: {targets:?}");
            assert_eq!(via, Value::Null, "{platform:?}");
        }
    }

    #[test]
    fn terminal_git_changes_include_terminal_sources_and_web_changes_do_not() {
        let root = scratch("git-changes");
        write(&root, "src/main.rs", "fn main() {}\n");
        write(&root, "README.md", "x\n");
        let git = json!({ "isRepo": true, "changedFiles": ["src/main.rs", "README.md"] });
        let (targets, via) = targets_and_via(&scan_targets(&root, &git, Some("terminal")));
        assert_eq!(targets, ["src/main.rs"]);
        assert_eq!(via, "git-changes");
        let (targets, via) = targets_and_via(&scan_targets(&root, &git, None));
        assert_eq!(targets, ["src"]);
        assert_eq!(via, "source-dir");
    }
}
```

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test -p impeccable-context signals::tests 2>&1 | grep -E "^error|test result" | head
```

Expected: compile error `this function takes 2 arguments but 3 arguments were supplied` for every `scan_targets` call.

- [ ] **Step 3: Implement.** In the imports, change line 7 and add one line after line 8:

```rust
use crate::util::{exists, js_num, js_trim, json_pretty, opt_string, read_dir_entries, Env};
use impeccable_common::Io;
use impeccable_core::checks::terminal::TERMINAL_EXTENSIONS;
```

After `const SOURCE_DIRS` (line 266) add:

```rust
/// Go's conventional layout directories, read only on a terminal project.
const TERMINAL_SOURCE_DIRS: [&str; 3] = ["cmd", "internal", "pkg"];
/// Root manifests that mark a terminal project whose sources may sit at the root.
const TERMINAL_ROOT_MANIFESTS: [&str; 3] = ["Cargo.toml", "go.mod", "pyproject.toml"];

/// A terminal project whose root holds its manifest beside source files
/// (`main.go` next to `go.mod`, `app.py` next to `pyproject.toml`). One
/// directory listing, no recursion.
fn terminal_root_holds_source(cwd: &str) -> bool {
    TERMINAL_ROOT_MANIFESTS.iter().any(|m| exists(&jsp::join(&[cwd, m])))
        && read_dir_entries(cwd)
            .unwrap_or_default()
            .iter()
            .any(|e| e.is_file && TERMINAL_EXTENSIONS.contains(&jsp::extname(&e.name).to_lowercase().as_str()))
}
```

Replace `scan_targets` (lines 280-320) with:

```rust
fn scan_targets(cwd: &str, git: &Value, platform: Option<&str>) -> Value {
    // A terminal project also scans terminal source, may keep its sources at
    // the root, and may use Go's cmd/internal/pkg layout.
    let terminal = platform == Some("terminal");
    let scannable = |f: &str| {
        let ext = jsp::extname(f).to_lowercase();
        SCANNABLE_EXT.contains(&ext.as_str()) || (terminal && TERMINAL_EXTENSIONS.contains(&ext.as_str()))
    };
    let is_repo = git.get("isRepo").and_then(|v| v.as_bool()).unwrap_or(false);
    let changed: Vec<String> = git
        .get("changedFiles")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    let mut m = Map::new();
    if is_repo && !changed.is_empty() {
        let c: Vec<String> = changed
            .into_iter()
            .filter(|f| scannable(f.as_str()))
            .filter(|f| !is_vendored_path(f))
            .filter(|f| exists(&jsp::join(&[cwd, f])))
            .collect();
        if !c.is_empty() {
            m.insert("targets".into(), Value::Array(c.into_iter().take(50).map(Value::String).collect()));
            m.insert("via".into(), Value::String("git-changes".into()));
            return Value::Object(m);
        }
    }
    // Before the source dirs: `.` covers them, and a go.mod with main.go at
    // the root and an internal/ dir would otherwise lose main.go.
    if terminal && terminal_root_holds_source(cwd) {
        m.insert("targets".into(), Value::Array(vec![Value::String(".".into())]));
        m.insert("via".into(), Value::String("root".into()));
        return Value::Object(m);
    }
    let extra_dirs: &[&str] = if terminal { &TERMINAL_SOURCE_DIRS } else { &[] };
    let dirs: Vec<&str> = SOURCE_DIRS.iter().chain(extra_dirs).copied().filter(|d| exists(&jsp::join(&[cwd, d]))).collect();
    if !dirs.is_empty() {
        m.insert("targets".into(), Value::Array(dirs.into_iter().map(|d| Value::String(d.to_string())).collect()));
        m.insert("via".into(), Value::String("source-dir".into()));
        return Value::Object(m);
    }
    if exists(&jsp::join(&[cwd, "index.html"])) {
        m.insert("targets".into(), Value::Array(vec![Value::String("index.html".into())]));
        m.insert("via".into(), Value::String("html".into()));
        return Value::Object(m);
    }
    if has_code(cwd) {
        m.insert("targets".into(), Value::Array(vec![Value::String(".".into())]));
        m.insert("via".into(), Value::String("root".into()));
        return Value::Object(m);
    }
    m.insert("targets".into(), Value::Array(vec![]));
    m.insert("via".into(), Value::Null);
    Value::Object(m)
}
```

In `gather_signals`, replace line 331 (`setup.insert("platform".into(), opt_string(&extract_platform(ctx.product.as_deref())));`) with:

```rust
    let platform = extract_platform(ctx.product.as_deref());
    setup.insert("platform".into(), opt_string(&platform));
```

and line 335 (`let scan = scan_targets(cwd, &git);`) with:

```rust
    let scan = scan_targets(cwd, &git, platform.as_deref());
```

- [ ] **Step 4: Run to verify they pass**

```bash
cargo test -p impeccable-context 2>&1 | grep -E "FAILED|test result"
cargo build --release -p impeccable 2>&1 | tail -n 2
IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/run.mjs signals- 2>&1 | tail -n 3
```

Expected: `test result: ok.`; every existing `signals-*` golden replays unchanged, including `signals-terminal` (`["src"]` via `source-dir`: `ctx-terminal` has no source file at its root) and `signals-native-ios`.

- [ ] **Step 5: Commit**

```bash
git add crates/context/src/signals.rs
git commit -m "$(printf 'Give routing a detect target on terminal source layouts\n\nscan_targets takes the platform gather_signals already reads. On a\nterminal project it keeps .rs, .go, .py, and .tcss changes, targets the\nroot when Cargo.toml, go.mod, or pyproject.toml sits beside root source\nfiles, and adds cmd, internal, and pkg to the source directories.\n\nAI assistance: drafted with Claude Code.')"
```

---
### Task 8: Oracle cases and goldens

**Files:**
- Create: `tests/oracle/workspaces/ctx-terminal-go/PRODUCT.md`, `go.mod`, `cmd/app/main.go`, `internal/ui/view.go`
- Modify: `tests/oracle/cases/context.mjs` (header comment, three cases)
- Modify: `tests/oracle/DELTAS.md` (one section at the end)
- Record: `tests/oracle/golden/doctor-terminal-evidence-charm-v2.json`, `doctor-monorepo-terminal-evidence.json`, `signals-terminal-go-layout.json`

**Interfaces:**
- Consumes: the release binary built from Tasks 2 to 7.

- [ ] **Step 1: Create the workspace.** Four hand-written input files (not goldens):

`tests/oracle/workspaces/ctx-terminal-go/PRODUCT.md`:

```markdown
# Oracle Fixture Product

<!-- impeccable:product-schema 1 -->

## Platform

terminal

## Positioning
A fixture Bubble Tea app in Go's cmd/ layout that the oracle harness uses to pin routing targets.
```

`tests/oracle/workspaces/ctx-terminal-go/go.mod`:

```
module example.com/ctxterminalgo

go 1.24

require charm.land/bubbletea/v2 v2.0.0
```

`tests/oracle/workspaces/ctx-terminal-go/cmd/app/main.go`:

```go
package main

import "example.com/ctxterminalgo/internal/ui"

func main() {
	ui.Run()
}
```

`tests/oracle/workspaces/ctx-terminal-go/internal/ui/view.go`:

```go
package ui

import tea "charm.land/bubbletea/v2"

// Run starts the fixture program.
func Run() {
	_ = tea.NewProgram(nil)
}
```

There is deliberately no `package.json` (decision 13) and no `.go` file at the root.

- [ ] **Step 2: Add the cases.** In `tests/oracle/cases/context.mjs`:

(a) In the header comment's workspace list, after the line ` *   ctx-legacy         unstamped PRODUCT.md with ## Register, DESIGN.json sidecar v1, bad config, orphan brief`, add:

```js
 *   ctx-terminal-go    PRODUCT.md `## Platform` terminal, go.mod + cmd/ + internal/, no package.json
```

(b) Immediately after the `doctor-terminal-evidence-json` case line, add:

```js
  // Charm v2 module paths count; an `// indirect` require and a `replace (` block do not.
  {
    id: 'doctor-terminal-evidence-charm-v2', verb: 'doctor', workspace: 'ctx-product-only',
    setup: (ws) => write(ws, 'go.mod', 'module example.com/app\n\ngo 1.24\n\nrequire (\n\tcharm.land/bubbletea/v2 v2.0.0\n\tcharm.land/lipgloss/v2 v2.0.0\n\tcharm.land/bubbles/v2 v2.0.0 // indirect\n)\n\nreplace (\n\tgithub.com/charmbracelet/bubbles => ../bubbles-fork\n)\n'),
    env: env(),
  },
  // The workspace sweep words terminal evidence as dependencies, not build files.
  {
    id: 'doctor-monorepo-terminal-evidence', verb: 'doctor', workspace: 'ctx-monorepo',
    setup: (ws) => write(ws, 'apps/b/Cargo.toml', '[package]\nname = "b"\nversion = "0.1.0"\n\n[dependencies]\nratatui.workspace = true\n'),
    args: ['--json'], env: env(),
  },
```

(c) Immediately after the `signals-terminal` case line, add:

```js
  { id: 'signals-terminal-go-layout', verb: 'context-signals', workspace: 'ctx-terminal-go', env: env() },
```

- [ ] **Step 3: Run the new cases to verify they fail**

```bash
for id in doctor-terminal-evidence-charm-v2 doctor-monorepo-terminal-evidence signals-terminal-go-layout; do
  IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/run.mjs "$id" 2>&1 | grep -E "^\?\?|missing goldens"
done
```

Expected: each prints `?? <id>: no golden (run record.mjs)` and a summary with `1 missing goldens`. (On the Task 1 binary the three would have recorded: no finding for the charm.land go.mod, "carries native build files" for `apps/b`, and `"targets": []` with `"via": null` for the Go layout.)

- [ ] **Step 4: Record the three goldens, one full id each** (never a shorter prefix; `signals-terminal` would also overwrite the existing golden):

```bash
cargo build --release -p impeccable 2>&1 | tail -n 1
for id in doctor-terminal-evidence-charm-v2 doctor-monorepo-terminal-evidence signals-terminal-go-layout; do
  IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/record.mjs --bin "$id"
done
git status --short tests/oracle/golden
```

Expected: `recorded <id> (exit 0, ...)` three times, and `git status` lists exactly three new goldens and no modified one.

- [ ] **Step 5: Review each golden by hand**

```bash
for id in doctor-terminal-evidence-charm-v2 doctor-monorepo-terminal-evidence signals-terminal-go-layout; do
  node -e "const g=require('./tests/oracle/golden/$id.json'); console.log('== $id exit', g.exit); console.log(g.stdout)"
done
```

Check, and stop to diagnose if any line differs:
- `doctor-terminal-evidence-charm-v2`: exit 0; one `platform-native-evidence` finding whose summary reads `PRODUCT.md declares \`## Platform: web\`, but the project carries a bubbletea dependency and a lipgloss dependency. Web guidance is being applied to a terminal codebase, and the terminal reference never loads.` No "bubbles" (indirect) and nothing from the `replace (` block; the fix names `terminal`.
- `doctor-monorepo-terminal-evidence`: exit 0; `findings` holds `workspace-platform-native-evidence` (path `PRODUCT.md`) with summary ``Workspace `apps/b` inherits the repo-root PRODUCT.md that resolves to web, but the workspace itself carries terminal dependencies. PRODUCT.md declares `## Platform: web`, but the project carries a ratatui dependency. Web guidance is being applied to a terminal codebase, and the terminal reference never loads.`` and fix ``Give `apps/b` its own PRODUCT.md with the right `## Platform`. An inherited record cannot describe two platforms at once.``, then the unchanged `workspace-context-inherited` finding from `doctor-monorepo-json`; the `workspaces` array matches `doctor-monorepo-json`'s. The `ratatui.workspace = true` line also pins decision 6 end to end.
- `signals-terminal-go-layout`: exit 0; `setup.platform` is `"terminal"`, `setup.hasCode` is `false` (decision 12's recorded gap), `git.isRepo` is `false`, `devServer` is `<DEV_SERVER_PROBE>`, and `scan` is `{"targets": ["cmd", "internal"], "via": "source-dir"}`.

- [ ] **Step 6: Record the delta note.** Keep it prose: `run.mjs` (`loadAcceptedDeltas`) treats any line starting with ``- `<id>` `` as an accepted delta and stops failing that case, so no line may start that way. Append to `tests/oracle/DELTAS.md`:

```markdown

## Recorded 2026-09-25: terminal evidence and routing targets (spec PR 6)

New cases, recorded from the binary with `--bin` and reviewed by hand. `doctor-terminal-evidence-charm-v2` pins Charm v2 `charm.land` module paths as evidence, and pins that an `// indirect` require (`charm.land/bubbles/v2`) and a `replace (` block do not count; the summary lists bubbletea and lipgloss only. `doctor-monorepo-terminal-evidence` adds `apps/b/Cargo.toml` (`ratatui.workspace = true`) to `ctx-monorepo`: the workspace sweep now says the workspace "carries terminal dependencies" where it said "native build files" for every kind of evidence. `signals-terminal-go-layout` runs `signals` on the new `ctx-terminal-go` workspace (a terminal PRODUCT.md, `go.mod`, `cmd/app/main.go`, `internal/ui/view.go`, no `package.json`): `scan` reads `["cmd", "internal"]` via `source-dir`, where the binary before this change returned `[]`.

No existing golden changed. `doctor-terminal-evidence-text` and `-json` were not re-recorded: they report the root-level `platform-native-evidence` finding, whose wording this change does not touch, and their single-line `require github.com/charmbracelet/bubbletea` still counts. `signals-terminal` still reads `["src"]` via `source-dir` (`ctx-terminal` has no source file at its root), and `context-terminal-evidence-web` still reports its ratatui dependency.
```

- [ ] **Step 7: Run the whole oracle**

```bash
IMPECCABLE_BIN="$PWD/target/release/impeccable" bun run test:oracle 2>&1 | tail -n 6
git status --short tests/oracle
```

Expected: every case passes except `detect-fixture-text-framework-vite` when port 8080 is busy (baseline). `git status` shows only the three new goldens, the new workspace, `context.mjs`, and `DELTAS.md`.

- [ ] **Step 8: Commit**

```bash
git add tests/oracle/workspaces/ctx-terminal-go tests/oracle/cases/context.mjs tests/oracle/DELTAS.md \
  tests/oracle/golden/doctor-terminal-evidence-charm-v2.json \
  tests/oracle/golden/doctor-monorepo-terminal-evidence.json \
  tests/oracle/golden/signals-terminal-go-layout.json
git commit -m "$(printf 'Pin Charm v2 evidence, workspace wording, and Go routing targets\n\nThree oracle cases recorded from the binary and reviewed by hand:\ndoctor-terminal-evidence-charm-v2, doctor-monorepo-terminal-evidence,\nand signals-terminal-go-layout on a new ctx-terminal-go workspace.\n\nAI assistance: drafted with Claude Code.')"
```

---

### Task 9: Contract, full gates

**Files:**
- Modify: `docs/CLI-CONTRACT.md` (the evidence sentence on line 658, the `checkWorkspaces` line 682, the `scan` sentence on line 797)

- [ ] **Step 1: Evidence sentence (line 658).** Replace this exact text:

```
Text manifests: `Cargo.toml` `ratatui`/`crossterm`, `go.mod` `github.com/charmbracelet/bubbletea`/`lipgloss`, `pyproject.toml` and `requirements.txt` `textual`/`rich` (all terminal; a line matches when it starts with the name after trimming whitespace and a leading quote and the next char is not `[A-Za-z0-9_./-]`; a leading `require ` token is skipped first, and a Go major-version suffix `/v<digits>` directly after the name is consumed before the next-char check).
```

with:

```
Text manifests, each read once from the project root (member manifests are never read), all terminal, each reason listed once: `Cargo.toml` `ratatui` (a key line whose key, optionally quoted, is the name followed by whitespace, `=`, or `.`, so `ratatui.workspace = true` counts; or a `[` line ending in `dependencies.ratatui]`; `#` comments ignored); `go.mod` `bubbletea`, `lipgloss`, `bubbles`, each under `github.com/charmbracelet/` or `charm.land/` with an optional `/v<digits>` suffix (only `require` entries count: a single-line `require <path> <version>` or a line inside a `require (` block; an entry whose `//` comment is `indirect` or starts `indirect;` does not, and nothing inside any other block such as `replace (`, `exclude (`, `retract (`, `tool (` counts); `pyproject.toml` and `requirements.txt` `textual` (after stripping a `#` comment, the line and every quoted string on it are read as a requirement: a name run starting alphanumeric over `[A-Za-z0-9._-]`, compared after PEP 503 normalization, followed after optional spaces by nothing or one of `[ ( < > = ! ~ ; @`). `rich` and `crossterm` are not evidence.
```

- [ ] **Step 2: Workspace line (line 682).** Replace ``native evidence per workspace -> `workspace-platform-native-evidence` (mention);`` with:

```
native evidence per workspace -> `workspace-platform-native-evidence` (mention; summary ``Workspace `<path>` <inherits the repo-root PRODUCT.md | has a PRODUCT.md> that resolves to web, but the workspace itself carries <terminal dependencies | native build files>. <evidence summary>``, terminal dependencies when the suggested platform is `terminal`; fix: inherited -> ``Give `<path>` its own PRODUCT.md with the right `## Platform`. An inherited record cannot describe two platforms at once.``, else the evidence fix);
```

- [ ] **Step 3: Scan sentence (line 797).** Replace the text from `` `scan`: changed files with ext in`` through ``else `[]`, null.`` with:

```
`scan` (terminal = `platform` is `terminal`): changed files with ext in `.html .htm .css .scss .jsx .tsx .js .ts .vue .svelte .astro` (plus `.rs .go .py .tcss` on terminal), not under a hidden/`node_modules`/`dist`/`build`/`__pycache__` dir segment (hidden `.vitepress`, `.vuepress`, `.storybook` allowed), existing -> `{targets (max 50), via:'git-changes'}`; else on terminal, when the root holds `Cargo.toml`, `go.mod`, or `pyproject.toml` and a file with a terminal extension sits directly in the root -> `['.']`, `'root'`; else existing of `src app components pages public` (then `cmd internal pkg` on terminal) -> `'source-dir'`; else `index.html` -> `'html'`; else hasCode -> `['.']`, `'root'`; else `[]`, null.
```

Before editing, `sed -n '658p;682p;797p' docs/CLI-CONTRACT.md` and confirm each old text is present verbatim; if PR 4 or PR 5 moved the lines, search by the quoted text instead of the line number.

- [ ] **Step 4: Full gates**

```bash
cargo test --workspace 2>&1 | grep -E "^test result|FAILED" | sort | uniq -c
cargo build --release -p impeccable 2>&1 | tail -n 1
IMPECCABLE_BIN="$PWD/target/release/impeccable" bun run test 2>&1 | tail -n 15
node -e "const s=require('fs').readFileSync('docs/CLI-CONTRACT.md','utf8'); console.log('em dashes in contract:', s.split(String.fromCharCode(0x2014)).length-1)"
git diff main --stat
```

Expected: every `test result` ok; `bun run test` shows only the Task 1 baseline failures (rerun a bun 5 s timeout alone before calling it baseline); the em dash count equals the Task 1 count (the contract's existing `#### path <em dash> title` headings; this task adds none); the diff lists only `crates/context/src/{staleness,staleness_deep,signals}.rs`, `docs/CLI-CONTRACT.md`, and the Task 8 oracle files.

- [ ] **Step 5: Commit**

```bash
git add docs/CLI-CONTRACT.md
git commit -m "$(printf 'Document PR 6 evidence matching, workspace wording, and scan targets\n\nAI assistance: drafted with Claude Code.')"
```

- [x] **Step 6: Spec corrections.** Done by the controller while committing this plan (decisions 8 and 9 approved): section 7's PR 6 Tests paragraph and section 1's evidence table already say this. Nothing to do.

---

### Task 10: Final review, merge to the fork's main, push

**Files:** none new.

- [ ] **Step 1: Whole-branch review** per subagent-driven-development (final code reviewer on the most capable model), pointed at `git diff main...terminal-platform-pr6` and this plan's decisions. Ask it specifically to probe the three parsers with inputs the unit tests do not hold. (The planner compiled the Task 3 to 5 parser bodies standalone and confirmed every test input above plus `require(` without a space, a Cargo inline table `ratatui = { version = "0.29" }`, and `textual @ https://x/y.whl`; the reviewer should look past those.) Findings marked Important or Critical get one fix wave and one scoped re-review.

- [ ] **Step 2: Gates once more on the final head**

```bash
cargo test --workspace 2>&1 | tail -n 3
cargo build --release -p impeccable 2>&1 | tail -n 1
IMPECCABLE_BIN="$PWD/target/release/impeccable" bun run test 2>&1 | tail -n 8
git status --short   # only the untracked local files may show; never stage them
```

- [ ] **Step 3: Merge and push** (the fork owner pre-authorized merging each PR to `main` as it lands):

```bash
git checkout main && git pull --ff-only origin main
git merge --no-ff terminal-platform-pr6 -m "$(printf 'Merge terminal-platform-pr6: init evidence and routing targets (spec PR 6)\n\nAI assistance: drafted with Claude Code.')"
git push origin main
git log --oneline -3
```

- [ ] **Step 4: Close out.** Ledger completion line; update the memory file `terminal-platform-project.md` (PR 6 merged, commit id, the rulings on decisions 8 and 9, the recorded gaps from decision 12, and that `ENGINE_VERSION` still needs a bump at release); write the journal entry. Never post the upstream issue draft.

---

## Self-review notes

- Spec section 7, PR 6 bullets: go.mod require-only, `// indirect`, other blocks, inverted indirect test (Task 3); Charm `charm.land` paths and `bubbles` (Task 3); PEP 503 and every quoted string (Task 4); Cargo dotted keys and `dependencies.ratatui]` headers, member crates unread (Task 5, and the table doc comment in Task 2); `rich` and `crossterm` removed, `rich` test inverted, `crossterm` pinned (Task 2); doctor wording (Task 6); routing targets: platform passed from `gather_signals`, terminal extensions, `cmd internal pkg`, root with a manifest beside top-level source (Task 7).
- Spec tests: a unit test per matcher change (Tasks 2 to 5), one per `scan_targets` layout (Task 7), the two named oracle cases plus `doctor-monorepo-terminal-evidence` (Task 8). The `doctor-terminal-evidence-*` re-record is replaced by a replay (decision 8).
- Delivery gate: `cargo test --workspace`, the oracle cases, `bun run test` (Tasks 8 to 10). No `cargo xtask bundle` is owed.
- Type consistency: `manifest_names_dependency(manifest, text, name)` in Tasks 2 to 5; `native_platform_evidence(..) -> Option<(Finding, &'static str)>` in Task 6 only; `scan_targets(cwd, git, platform)` in Task 7 only; `TERMINAL_EVIDENCE_MANIFESTS: [(&str, &[(&str, &str)]); 4]` from Task 2 on.
- Not in this PR: skill text (`routing.md`'s `via` list, `doctor.md`'s evidence wording) is PR 7's; `hasCode` for Go and Python projects, a root package directory, and Cargo virtual workspaces stay recorded gaps (decision 12); concept-seed forwarding stays left out (spec).
