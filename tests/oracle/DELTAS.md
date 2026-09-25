# Accepted deltas

Cases listed here differ from their JS golden on purpose. Each entry names the
case id, what differs, and why it is an improvement. Nothing gets on this list
without review.

Format: `- \`<case-id>\`: <what differs> (<why>)`

## Recorded 2026-08-17: the engine names its own commands

The JS scripts printed their own file names in usage lines, directives, and
the hook manifests they wrote. The binary prints the verb (`impeccable doctor`)
or the launcher path (`"<scripts>/impeccable" hook`). Each case below was
re-recorded from the engine after a line-level review confirmed the only change
is that wording; behavior, exit codes, and every other byte are unchanged.

- `doctor-help`, `doctor-help-short`: `Usage: node doctor.mjs …` is now `Usage: impeccable doctor [--json] [--fix] [--target <path>]`.
- `doctor-legacy-text`: the closing hint reads `Run \`<self> doctor --fix\``.
- `pin-usage-no-args`, `pin-usage-one-arg`: `Usage: impeccable pin <pin|unpin> <command>`.
- `surface-brief-usage`, `surface-brief-unknown`, `surface-brief-write-usage`: usage lines name `impeccable surface-brief`.
- `critique-usage`, `critique-unknown`: usage lines name `impeccable critique-storage`.
- `context-monorepo-target-missing`: MONOREPO_TARGET_REQUIRED says `impeccable context ran without --target`.
- `hadmin-on`, `hadmin-on-twice`, `hadmin-off-then-status`, `hadmin-on-repairs-existing-manifest`, `hadmin-on-malformed-manifest-backup`: `hooks on` writes manifests that run the launcher (`"<scripts>/impeccable" hook`, Cursor `hook-before-edit`) instead of `node "<scripts>/hook.mjs"`.
- `hook-session-fresh-then-pending-then-stop`, `hook-session-two-sessions`, `hbe-denial-downgrade-after-6`: the short footer names `impeccable hooks ignore-value`.
- `live-help`, `live-accept-help`, `live-inject-help`, `live-insert-help`, `live-server-help`, `live-resume-help`, `live-commit-help`, `live-discard-help`, `live-complete-help`, `live-complete-no-id`: usage text names `impeccable live*` verbs.
- `live-server-already-running`, `live-daemon-server-status-poll-complete`, `live-status-empty`, `live-status-generating`, `live-status-many-sessions`, `live-status-stale-server-json`, `live-status-legacy-sessions-dir`, `live-status-from-subdir`, `live-status-manual-apply`, `live-resume-manual-apply`, `live-status-mount-failed`, `live-resume-mount-failed`, `live-resume-generating`, `live-resume-by-id`, `live-resume-first-active-sorted`, `live-resume-accept-requested`, `live-resume-carbonize-required`: recovery hints and next-command lines spell `<self> live-poll` / `live-server` / `live-complete` / `live-commit-manual-edits` instead of the `.mjs` names.

## Recorded 2026-08-17: live-inject adds `'wasm-unsafe-eval'` to a CSP meta script-src

The detector the live overlay loads from the helper origin is a WebAssembly
module in the engine (its `docs/WASM-BUNDLE.md`); a `script-src` that names the
origin but not `'wasm-unsafe-eval'` still refuses to compile it. The JS
`patchCspMeta` predates the wasm bundle and appended only the origin.

- `live-inject-csp-meta-no-connect-src`: the patched `<meta http-equiv="Content-Security-Policy">` reads `script-src 'self' http://localhost:8412 'wasm-unsafe-eval'` (was `script-src 'self' http://localhost:8412`). The `data-impeccable-csp-original` marker, the `connect-src` and `img-src` additions, idempotence, and the revert on unpatch are unchanged. `live-inject-vite-csp-meta` and `live-inject-next-jsx` carry meta tags the patch does not touch, so their goldens did not move.

## Recorded 2026-08-31: detector-engine ports landed, gap goldens restored

The section previously here pinned the gap between main's post-freeze detector
fixes and the engine. Those fixes are now ported (engine repo commits:
`c0aa75f` oklch in visual-contrast/neon-text, upstream 1b7da15b #592;
`5cdeec8` color-mix nested hex, upstream 54440319 #578; the 1D grid fix,
upstream a236137b #615, rode along in `9046e8f` via a concurrent staging race;
`6d36231` comment stripping for regex matchers, upstream 067665cc #589 +
ddb60993 + ba873f75 + 9a7d0fbc; `33aef88` root-relative linked stylesheets,
upstream 2b88aa52 #652 + daae1d41; `6d0ecf1` URL userinfo redaction with
origin-scoped basic auth, upstream d5873ff8 + d690349d #657; `09f8ae7` inert
exact ignore-value refusal, upstream be87f5eb #662; `20c8347` the
comp-fidelity rules organic-clip-path and buried-raster, upstream 58561610).
The affected goldens were re-recorded from the fixed engine and each json
fixture golden was byte-verified against the last JS engine state in history
(`db1462b9^`, which carries both main's drift and the comp-fidelity rules):

- Moved to post-fix behavior: `detect-fixture-json-codex-grid-1d-pass-html`,
  `detect-fixture-text-codex-grid-1d-pass-html` (no finding, exit 0),
  `detect-fixture-json-organic-clip-path-html`,
  `detect-fixture-text-organic-clip-path-html`,
  `detect-fixture-json-buried-raster-html`,
  `detect-fixture-text-buried-raster-html` (the new rules fire),
  `detect-fixture-json-glow-html`, `detect-fixture-text-glow-html` (glow's
  `.photo-opaque-grad` column now carries its intended buried-raster finding),
  and the sweeps `detect-dir-json-all-fixtures`, `detect-dir-text-all-fixtures`,
  `detect-dir-quiet-all-fixtures`, `detect-no-advisory-json`,
  `detect-no-advisory-text`.
- Unchanged on re-record (already matched the fixed JS in the static engine):
  `detect-fixture-json-color-html`, `detect-fixture-text-color-html`,
  `detect-fixture-json-oklch-neon-text-html`,
  `detect-fixture-text-oklch-neon-text-html` (the oklch and color-mix fixes
  observably change the browser-side visual-contrast path, which these static
  scans do not exercise), `detect-scope-type`, `detect-scope-both`.

The frozen call vectors for `checkHtmlPatterns`
(`tests/oracle/vectors/calls/rules.checks/checkHtmlPatterns.jsonl`) were
re-recorded the same way: args untouched, results replayed through the
`db1462b9^` JS (14 of 101 moved: the comp-fidelity scans and the
comment-stripping/inline-fragment fixes to `enclosingCssSelector`). No case in
this section is an accepted delta any more; the engine matches the final JS.

## Recorded 2026-08-31: main's Aug 17-31 verb fixes ported to the engine, goldens re-recorded

The goldens below froze pre-fix behavior. Each fix landed on main in JS and
was ported to the engine; the cases were re-recorded from the binary and
reviewed line by line, so they now pin the fixed behavior.

- `hook-session-fresh-then-pending-then-stop`, `hook-session-two-sessions`: the Stop deep pass syncs the remembered set to the live scan, including findings the per-edit pass already surfaced, so a second Stop with nothing new is silent and a fixed-then-reintroduced finding fires again (upstream 3c442af7).
- `hadmin-on`, `hadmin-on-twice`, `hadmin-off-then-status`, `hadmin-on-repairs-existing-manifest`, `hadmin-on-malformed-manifest-backup`: the Claude manifests `hooks on` writes match on `Edit|Write` and the description names the current tools; Claude Code folded multi-edit behavior into Edit (upstream 7d5c60d2).
- `live-commit-mock-unreported-file-change`: the rollback-failure results share one constructor, which moved `unreportedFiles` and `notes` after `pageUrl` in the emitted JSON (upstream 1f2c3f9d).

## Recorded 2026-08-31: main's Sep-1 verb fixes ported after the rust-swap rebase

Five more fixes landed on main in JS between the swap branch and its rebase.
Each was ported to the engine and the affected goldens re-recorded from the
binary after a line-level review; the engine's output was also diffed
byte-for-byte against the upstream JS on the same inputs before recording.

- `critique-usage`, `critique-unknown`: the usage line now lists the new `close` subcommand (upstream 5211bdf4, #660).
- `critique-latest-existing`: `latest` applies the #660 identity/freshness path: a legacy snapshot carrying no fingerprint for a concrete local target is closed and `latest` exits 2 instead of printing the stale body (upstream 5211bdf4, #660).
- `critique-write-then-read`: `write` stamps `target_identity`/`target_fingerprint`/`target_path`, uses a fixed-width `~NNNN` collision suffix when two snapshots share a UTC second, `latest` freshness-closes the read snapshot, and `trend` now surfaces the `closed` flag and identity fields (upstream 5211bdf4, #660).
- `critique-write-monorepo-child`: `write` stamps the resolved `target_identity`, and a `latest` run from a sibling app resolves to a different identity so it exits 2 rather than returning the neighbor's backlog (upstream 5211bdf4, #660).
- `detect-fixture-json-overused-font-html`, `detect-fixture-text-overused-font-html`: new fixture added on the swap branch; overused-font primary selection now skips only the CSS generics, so a system stack keeps its system face as primary and later web-font fallbacks like Roboto no longer flag (upstream 2cfd6076, #678).
- `detect-dir-json-all-fixtures`, `detect-dir-text-all-fixtures`, `detect-dir-quiet-all-fixtures`, `detect-scope-type`, `detect-scope-both`, `detect-no-advisory-json`, `detect-no-advisory-text`: the directory sweep picks up the new overused-font fixture and the #678 primary-face change (upstream 2cfd6076, #678).

## Recorded 2026-08-31: E8 hook-manifest self-heal on upgrade

Two new cases pin the fix for triage E8 (the v3-to-launcher upgrade path). The
JS `automaticHookMode` counted any hook command naming the skill as an active
hook, including the JS-era `node .../hook.mjs` form. After a skill update the
`.mjs` script no longer exists, so that manifest points at a dead command yet
still suppressed `MANUAL_DETECTOR_REQUIRED`, leaving the detector dark. The
engine now treats a manifest that names ONLY the `.mjs` form as not an active
launcher hook, so the manual detector fallback fires until install/update
repairs the manifest to the launcher form. The launcher form still counts as
active exactly as before. No existing golden moved: every other `context` case
runs under the `source` provider, whose manifest list is empty, so none of them
scan a hook manifest.

- `context-stale-hook-manifest`: a `.claude/settings.local.json` naming `node "${CLAUDE_PROJECT_DIR}/.claude/skills/impeccable/scripts/hook.mjs"` under the `claude-code` provider emits `MANUAL_DETECTOR_REQUIRED` (the stale marker no longer counts as active).
- `context-launcher-hook-active`: the same manifest in the launcher form (`"…/impeccable" hook`) suppresses `MANUAL_DETECTOR_REQUIRED`, confirming the launcher marker is still recognized as active.

## Recorded 2026-09-01: the harness stages workspaces at their real path

Two goldens were re-recorded after `stageWorkspace` started returning the
realpath of the staged directory. macOS's tmpdir is a symlink (`/var` ->
`/private/var`), and the old goldens carried that artifact rather than the
verbs' behavior; Linux, where the two paths are the same, never reproduced
them. The binary's output is unchanged; the input the harness fed it is.

- `context-dir-override`: `productPath` is `elsewhere/PRODUCT.md`, the plain relative path, instead of `../../../../../../..<WS>/elsewhere/PRODUCT.md` (a relative path from the symlinked cwd to the resolved one).
- `live-accept-source-locked`: the accept now reports `source_locked`, which is what the case is named for. The staged lock named the file under the symlinked path, so the verb never matched it against its own resolved path and the old golden recorded a successful accept.

`context-lowercase-product-name` runs only on case-insensitive hosts
(`platforms: ['darwin', 'win32']` in the case): `product.md` is found through
the canonical name there and through the fallback scan elsewhere, both right.

## Recorded 2026-09-03: #710 resolves an explicit target at its own git boundary

Upstream `672ca296` (#710) scopes an explicit `--target` to its own repository.
A route-shaped target that begins with `/` is an absolute path outside the
workspace, so route cases that used to resolve inside the fixture now resolve
against the filesystem root. Every case below was re-recorded after confirming
`origin/main`'s `context.mjs` / `surface-brief.mjs` produce the same stdout and
the same exit code for the same run.

- `context-full-target-route`, `surface-brief-path-slash`, `surface-brief-path-outside`, `surface-brief-read-route`: stdout and exit code match origin/main byte for byte; nothing here is a delta beyond the upstream change itself.
- `surface-brief-write-route`: the write now fails on both engines (exit 1) because `/.impeccable/surfaces` is not writable. Node reports `ENOENT: no such file or directory, mkdir '/.impeccable/surfaces'`; the engine reports the failed write as `No such file or directory (os error 2)`. Same failure, different wording for an unwritable filesystem root.

## Recorded 2026-09-03: the OpenCode pinned command names the launcher

Upstream `9736a9f6` (#483) makes `pin` write an OpenCode slash-command bridge
whose body tells the agent to run `node <skill-base-dir>/scripts/context.mjs`.
The engine names its own command everywhere else the launcher replaced a
script path (see the 2026-08-17 section above), so the bridge says
`<skill-base-dir>/scripts/impeccable context` instead. Nothing else in the
file, the file set, or the printed lines differs from the JS.

- `pin-opencode-project`, `pin-opencode-user-scope`, `pin-opencode-skips-foreign-command`, `pin-opencode-then-unpin`, `pin-opencode-unpin-skips-foreign`.


## Recorded 2026-09-04: `--version` follows the npm package to 4.0.0

The npm shim answers `--version` / `-v` itself from its own `package.json`
(docs/CLI-CONTRACT.md), so the number users see tracks the package they
installed. The binary's `CLI_VERSION` moves from `3.6.0` to `4.0.0` with the
CLI 4.0.0 release; it is what the binary prints when run directly.

- `cli-version`.


## Recorded 2026-09-24: `terminal` platform value

`terminal` joins the platform values. The boot WARNING for an unrecognized
value now lists it, so the one golden that prints that WARNING moved.

- `context-bad-platform`: the WARNING's valid-values sentence reads ``Valid values are `web`, `ios`, `android`, `adaptive` (cross-platform, ships both), or `terminal` (a TUI or rich CLI)`` and the fix clause says `native or terminal-based`. Nothing else in the output changed.
- `seed-platform-invalid`: the rejection reads `concept-seed: --platform must be one of web, ios, android, terminal`. Exit code and everything else unchanged.


## Recorded 2026-09-24: terminal source rules (spec PR 2)

New cases `detect-terminal-*` (one directory scan per `tui-` rule with `--platform terminal`, the web-gate scan of the same tree, the single-file advisory downgrade, the invalid flag value, and the `detect-terminal-project` workspace with and without the flag) pin the eleven terminal rules and the platform gate. `detect-help` gained the `--platform` line. The auto-enumerated `detect-fixture-{json,text}-terminal` are new because `tests/fixtures/antipatterns/terminal/` now exists. The seven all-fixtures goldens did not change: the new tree's `.tsx` files are web-scannable, but none of their terse Ink snippets trips an existing web rule (no CSS-in-JS block, no HTML), so the corpus-wide walk produces byte-identical output; no web rule's output on an existing fixture changed.

Final review: an explicit terminal-extension file argument off a terminal project now falls through to the web pipeline, as it did before PR 2, instead of scanning to `[]` (spec section 3; the extension gate belongs to the walker, not to a named file). New case `detect-terminal-explicit-py-web` scans `tests/fixtures/antipatterns/terminal/web-tell/app.py` (a Textual import plus a CSS string with a side-tab border) with no flag and records the `side-tab` finding at L3. The `detect-terminal-<rule>` enumeration now takes only `tui-` directories so `web-tell/` does not become a fake rule case. No existing golden changed: the web walk does not list `.py`, so the `detect-fixture-{json,text}-terminal` and all-fixtures scans never see `app.py`, and on `--platform terminal` the file classifies as Textual and trips no `tui-` rule.

Two fixture-review fixes landed alongside the goldens: `tui-hardcoded-size`'s "two `Constraint::Length` calls on one line" case was silently dropped by the old line-based dedup (fixed in `crates/core/src/checks/terminal.rs`, `scan_terminal_hardcoded_size`, plus a new unit test), and the `tui-spinner-no-tty-guard` Go fixture's own comment (`// flag: no isatty anywhere in this directory`) contained the literal trigger word for `TTY_GUARD_RE`, poisoning the directory's project signals and silencing all three of that rule's fixtures; the comment was reworded, no rule change needed.

## Recorded 2026-09-24: hook platform-aware extension gate and signals (spec PR 2, task 9)

New cases `hook-terminal-platform-*`, `hook-before-edit-terminal-rs`, and `hook-web-platform-skips-rs` pin that a `terminal` PRODUCT.md keeps the hook on and admits `.rs`/`.go`/`.py`/`.tcss`, that `tui-` findings surface only when `hook.advisoryRules` is `include` (every PR 2 rule is advisory), and that a web project still skips `.rs` at the extension gate. No existing hook golden changed.

`hook-terminal-platform-edit-rs` and `hook-web-platform-skips-rs` both set `env: { IMPECCABLE_HOOK_LOG: '<WS>/.impeccable/audit.ndjson' }` (same pattern as the precedent `hook-audit-log` case) so their `files` output makes the two code paths observably different, instead of both recording `{stdout: "", files: {}}` as they did before review round 1. `impeccable hook`'s `RunResult::stdout` is still empty on both an advisory-excluded clean pass and an extension skip — the `.impeccable/audit.ndjson` file is what tells them apart. `hook-terminal-platform-edit-rs`'s audit line carries `"ext":".rs","editCount":1,"findings":1,"freshFindings":0,"emitted":false,"skipped":"non-ui-ack"`: the `.rs` file reached the scan, `tui-double-border` fired (`findings:1`) and was advisory-filtered (`freshFindings:0`), and the file was never skipped at the extension gate. `hook-web-platform-skips-rs`'s audit line carries only `"ext":".rs","skipped":"extension"` — no `findings` key at all, because the extension gate short-circuited before any scan ran. `durationMs` and `ts` are normalized by the recorder (`<MS>`/`<ISO>`), so neither golden is timing-flaky. The gate is additionally pinned at the Rust level by the `terminal_platform_keeps_hook_active` unit test in `crates/hook/src/hook_lib.rs` (`allowed_exts(Some("terminal"))` contains `.rs`/`.tcss`/still contains `.tsx`; `allowed_exts(Some("web"))` excludes `.rs` and equals `ALLOWED_EXTS`), and end-to-end by `hook-terminal-platform-edit-rs-advisory-included`, which shows the `tui-double-border` finding actually reaching stdout (`src/main.rs`). `hook-before-edit-terminal-rs` is unchanged and still symmetric: `hook-before-edit` never writes an audit log at all (`write_audit_log` is only called from `hook.rs`'s `run()`), so its `{"permission":"allow"}` output is consistent with either "reached the scan, advisory-filtered" or "extension-skipped"; the gate is pinned by the same unit test, and this golden only pins that the verb does not crash or deny on terminal source.

Review round 1 also fixed `stop_baseline::capture` (`crates/hook/src/stop_baseline.rs`): it scanned the pre-edit preimage with `HookScanOptions::default()` (`platform: None`) unconditionally, so a terminal project's baseline always came back with zero `tui-*` findings and `stop_baseline::classify` then labeled every pre-existing `tui-*` Stop finding `[new]` or attribution-unknown. `capture` now takes a `platform: Option<&str>` parameter (threaded from the already-resolved `platform` binding at its one call site in `hook.rs`, not re-resolved) and scans the preimage through a new `scan_original` helper that carries it into `HookScanOptions`. `terminal_baseline_counts_preexisting_tui_findings` (new unit test in `stop_baseline.rs`) pins that a `tui-double-border` hit in the preimage counts as 1 with `platform: Some("terminal")` and 0 with `platform: None`; confirmed the test fails (0 vs 1) against the pre-fix code before landing the fix. No oracle golden changed from this half of the fix: no existing `hook-stop-baseline-*` case runs on a terminal-platform workspace, and the terminal oracle cases added in round 0 don't exercise a second session touching the same file twice (which is what would make a pre-existing-vs-new baseline distinction observable in stdout).

`hook-terminal-platform-stop`'s second step (`stop()`) records an empty stdout, not the `tui-hardcoded-rgb-no-adapt` finding the task brief's illustrative expectation named. Traced to source: `.impeccable/config.json` sets `hook.advisoryRules: include` in `setup()`, before either step runs, so the *first* step (`claudeEdit('src/theme.rs')`, a PostToolUse per-edit call) already surfaces `tui-hardcoded-rgb-no-adapt` at L3 and remembers it in the session cache (`"findings":["tui-hardcoded-rgb-no-adapt:3"]`). The Stop step's `dedupe_against_cache` then correctly treats the identical finding as already-shown and reports nothing new — the same dedup behavior every other multi-step `hook`/`stop` oracle case relies on (e.g. `hook-session-fresh-then-pending-then-stop`). Separately, `scan_terminal_hardcoded_rgb` (`crates/core/src/checks/terminal.rs`) fires on a literal `Color::Rgb(...)` regardless of whether `signals` is `Some` or `None` — signals only pick which advisory `note` lands in `extras` (`SIGNALS_NOTE` vs `ADAPTIVE_PRESENT_NOTE` vs no note), and the hook's ack renderer never surfaces `extras.note`, so the per-edit pass (no signals threaded) and the Stop pass (project signals threaded) would have rendered byte-identical finding text even had the Stop pass not deduped it away. The golden is correct; the brief's illustrative expectation assumed the Stop pass would be the first surface of the finding, which does not hold once `advisoryRules: include` is active from the first edit.

Final review: `hook-terminal-platform-stop` now sets `IMPECCABLE_HOOK_LOG`, so its golden gains `.impeccable/audit.ndjson`; the Stop line carries `"scannedFiles":1,"unknownFindings":1`, which shows the `.rs` file was scanned at Stop rather than skipped (stdout is still empty for the dedup reason above). New case `hook-terminal-platform-stop-fresh` seeds the session cache with `src/theme.rs` touched and no remembered findings, then sends only a Stop event: the Stop pass is the first surface, and stdout carries `L3 [tui-hardcoded-rgb-no-adapt] [attribution unknown]`. The Stop pass now walks for project signals only when a touched file is terminal source, and filters that walk through `ignoreFiles` like `detect` does (unit tests `stop_signals_skip_ignored_files` and `stop_signals_owed_only_to_terminal_source` in `crates/hook/src/hook.rs`). `context-terminal` moved because `skill/reference/terminal.md` gained one paragraph saying the hook drops advisory `tui-` findings by default; nothing else in its output changed.

## Recorded 2026-09-24: tmux engine (spec PR 3)

New cases `detect-tmux-*` and `detect-palette-invalid` pin the five tmux flags (`--tmux`, `--tmux-capture`, `--tmux-sizes`, `--tmux-settle`, `--palette`) and the seven `tui-rt-*` runtime rules through saved captures under `tests/fixtures/terminal-captures/`, with no tmux binary involved. One golden per fixture (`detect-tmux-capture-<name>`), plus the text renderer (`detect-tmux-capture-text`), the light palette (`detect-tmux-capture-light-palette`), a capture combined with an HTML file target (`detect-tmux-capture-with-files`), a missing capture file (`detect-tmux-capture-missing`), a missing tmux binary (`detect-tmux-missing-binary`), and one validation case per flag (`detect-tmux-flag-no-target`, `detect-tmux-capture-no-path`, `detect-tmux-sizes-invalid`, `detect-tmux-settle-invalid`, `detect-palette-invalid`).

`detect-help` moved for the new usage lines: the Options block gained `--tmux <target>`, `--tmux-capture <f>`, `--tmux-sizes <list>`, `--tmux-settle <ms>`, and `--palette <name>`; the Detection modes block gained a `tmux panes` line; and the Examples block gained `impeccable detect --tmux app:0.0 --tmux-sizes 80x24,120x40,40x24`. No other golden changed: the runtime rules run only behind `--tmux` / `--tmux-capture`, so every pre-existing scan (fixture, directory, hook, framework) is untouched.

The missing-binary case sets `PATH` to `/nonexistent` and unsets `IMPECCABLE_TMUX` (`env: { PATH: '/nonexistent', IMPECCABLE_TMUX: null }`). Its golden reads exactly:

```
Error: tmux 3.2 or newer is required for --tmux and was not found on PATH. Install tmux, or point IMPECCABLE_TMUX at the executable.
```

The other four validation goldens read exactly:

```
Error: --tmux requires a tmux target, e.g. --tmux app:0.0
Error: --tmux-capture requires a path to a saved capture
Error: --tmux-sizes requires comma-separated WxH values, e.g. --tmux-sizes 80x24,120x40,40x24
Error: --tmux-settle requires a whole number of milliseconds
Error: --palette requires dark or light
```

`detect-tmux-capture-missing` reads exactly:

```
Error: cannot scan <REPO>/tests/fixtures/terminal-captures/does-not-exist.txt: ENOENT: no such file or directory, open '<REPO>/tests/fixtures/terminal-captures/does-not-exist.txt'
```

`detect-tmux-capture-with-files` exits 2, not 0: `tests/fixtures/antipatterns/blinking-cursor.html` carries a real (non-advisory) `pulsing-dot` finding, so the pre-existing exit-code contract (2 means primary findings present, `crates/detect/src/cli.rs`) applies same as any other scan with a primary finding. The HTML finding is listed first and the tmux advisory finding second, matching target order.

Fixture note: `width-drift.txt` uses a 60-column pane, not 30. On a pane under 60 columns, `tui-rt-collapse-narrow`'s unconditional "a row wider than the pane" check (spec section 4) co-fires with `tui-rt-width-drift`'s wide-glyph overflow check on the same row, because both rules read the same cell count and neither excludes the other. Both rule implementations match their spec table text as written; this is not a rule defect. Widening the pane to 60 columns keeps `tui-rt-collapse-narrow` out of scope (`frames: capture frames under 60 columns`) while preserving both `tui-rt-width-drift` messages, so the fixture pins exactly `row measures 61 cells on a 60-column pane` and `border ends at column 58 while other rows end at 60`.

## Recorded 2026-09-25: runtime engine hardening (spec PR 4)

Three new captures come from the fixture loop in `cases/detect.mjs`: `detect-tmux-capture-collapse-narrow-prose` (a 40-column frame whose only overflow row is joined prose; `[]`), `detect-tmux-capture-width-drift-narrow` (a 40-column frame whose overflow is one emoji; one `tui-rt-width-drift`, no `tui-rt-collapse-narrow`), and `detect-tmux-capture-no-key-hints-pager` (less's bare `:` prompt with the trailing spaces `-J` keeps; `[]`).

Case `detect-tmux-capture-spinner-never-rests` moved: the fixture gained a second recapture (frame 0 `⠋`, 700 ms `⠙`, 1000 ms `⠋`). The snippet is now `spinner glyph ⠋ became ⠙ between captures with no input` and the registry description says "across captures taken within one second".

Cases `detect-tmux-capture-low-contrast` and `detect-tmux-capture-light-palette` moved: each finding gains `paletteRelative` after `cellCount`, true when the foreground or background is the terminal default or one of the 16 named colors. The spec's PR 4 Tests paragraph does not list these two; the change is the new extra only.

No other golden moved: the width rules, the overflow verdict, and the key-hint exemptions leave the PR 3 fixtures' findings as they were.

## Recorded 2026-09-25: source rules tightening (spec PR 5)

Six `detect-terminal-*` goldens were re-recorded from the binary and reviewed by hand. `tui-print-in-loop` is Ink-only, so its Textual fixture line no longer reports, and its description now reads "`console.log` in an Ink app that sets `patchConsole: false` writes over the rendered frame. Leave `patchConsole` on, or log to a file." `tui-spinner-no-tty-guard` fires only on the Bubbles spinner, so its Rich and Ink fixtures became pass files and `detect-terminal-single-file-advisory` now scans the Go fixture, which still reports with the signals note. `tui-hardcoded-rgb-no-adapt` gained a chained chalk literal (flagged) and a computed `chalk.hex` (passes) in its Ink fixture. `tui-hardcoded-size` gained two pass files, one that measures the terminal and one that uses ratatui's canvas; each also holds one line that still fires, so the exemption is visible in the golden. Its `.rs` fixture gained two one-`Length` layouts that per-call counting leaves alone. `detect-terminal-text-all` carries the same changes and still ends at 44 advisory notes. `detect-terminal-emoji-density` did not move: the eight text-presentation dingbats added to its Rust fixture leave `emojiCount` at 4. No web golden changed; the new `.tsx` lines trip no web rule.
