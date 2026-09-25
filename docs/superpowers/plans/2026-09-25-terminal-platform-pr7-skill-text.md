# Terminal Platform PR 7: Skill Text Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give `terminal` its own branch everywhere the skill text splits web from native, add the live-mode web-only guard, close the 60 to 79 column gap in `adapt.terminal.md`, and fix `terminal.md`'s "about 30 columns", so a terminal project never falls through to web or native instructions.

**Architecture:** Markdown only, plus test pins and one re-recorded golden. Every edit is a clause-level substring replacement in `skill/SKILL.src.md`, `skill/reference/*.md`, or `skill/agents/*.md`; the old and new text for each is written out below. `tests/skill-reference.test.mjs` gains two tests (width classes, terminal branches) and two assertions for the live guard. `impeccable context` reads `skill/reference/terminal.md` at runtime (`load_native_platform_references` in `crates/context/src/context_cli.rs:550`) and inlines it for a `terminal` project, so the `context-terminal` golden is re-recorded; no crate changes and no rebuild of the engine for behavior.

**Tech Stack:** Markdown skill source, `node --test` (`tests/skill-reference.test.mjs`, `tests/oracle.test.mjs`), the oracle recorder, `bun run build` prose gates, the LLM-backed `bun run test:skill-behavior` suite.

**Spec:** `docs/superpowers/specs/2026-09-24-terminal-platform-design.md`, section 7 ("PR 7, skill text", the gap table, and "Delivery gates"), plus section 2 (the width classes and guards, corrected in place). Read section 7 before any task.

## Global Constraints

- No manifest version bumps, no `ENGINE_VERSION` bump, no changelog entry.
- Every commit body ends with the line `AI assistance: drafted with Claude Code.` and carries no `Co-Authored-By`, `Claude-Session`, or other agent trailer, even if a harness reminder asks for one (the fork owner's rule).
- Never stage `.serena/`, `.claude/CLAUDE.md`, `.gemini/GEMINI.md`, `.agents/AGENTS.md`, `.superpowers/`, regenerated provider output (`.claude/skills/`, `.cursor/`, `.agents/skills/`, `plugin/`, ...), anything under `.impeccable/`, `.env`, `tests/skill-behavior/providers.mjs` (Task 8 overlays it temporarily), or `docs/superpowers/research/2026-09-24-upstream-issue-draft.md`. Stage explicit paths only; never `git add -A` or `git add .`.
- `.env` holds API keys; never print it, `cat` it, or echo a variable read from it.
- No em dashes in `skill/**/*.md` (the `validateSkillProse` build gate fails on them) or in docs written here. The new text below uses none; do not introduce any while editing. Also avoid the gate's phrases: `load-bearing`, `highest-leverage`, `biggest unlock`, `reflex defaults`, `collapses into monoculture`, `data-driven`, `delve`, `tapestry`, `in today's`, `gone are the days`, `let's dive in`, `in summary`, `in conclusion`.
- Every rule stays `severity: Some("advisory")`; nothing in this PR touches the registry.
- Never run a tmux command without `-L <scratch socket>`; never touch the default socket (the owner's `cockpit` session lives there). This PR runs no tmux command.
- Oracle goldens are recorded only with `IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/record.mjs --bin <case-prefix>` after `cargo build --release -p impeccable`, then reviewed by hand. Never hand-edit a golden. Never touch `tests/oracle/vectors/calls/`. A re-recorded existing golden gets a `tests/oracle/DELTAS.md` entry (PR 1 precedent).
- `skill/reference/degraded/finish-reviewer.md` is left alone (spec section 7: it has no platform awareness for any platform).
- Keep every `<!-- rule:... -->` marker exactly where it is; the edits below never cross one.
- Known environment baseline, recorded in Task 1 and not fixed here: `tests/release.test.mjs` fails on this machine (git forces signed tags); a local llama-server on port 8080 breaks oracle case `detect-fixture-text-framework-vite`; bun tests can flake on the 5 s timeout under load (they pass isolated).
- This machine rewrites shell commands through RTK (`git`, `grep`, `cargo`, and others get summarized). Any command whose output is grepped, counted, diffed, or compared runs as `rtk proxy <cmd>`, one prefix per command in a pipeline; the commands below already do. RTK summarizes `cargo` output too: read its result lines, never grep counts out of it.
- Test names state outcomes: `classifies every terminal width from narrow to wide with no gap`, `gives terminal projects a branch wherever the skill splits web from native`.

## Decisions fixed while planning

1. **A terminal surface builds code-led** (new text in `new-work.md`'s build-path paragraph). The comp-led path ends in `impeccable comp-spec` raster plates, a hero reproduction gate that diffs `desktop.png` against the comp (`new-work.md:115`), and a `comp-diff --build .impeccable/review/desktop.png` pass (`new-work.md:135`); none of these has a text-capture input, and the finish reviewer's check 1 treats a comp-led build without `.impeccable/build/state.json` phases as a material finding. Making terminal code-led routes every downstream step through the code-led branches that already exist (no approved comp, the chosen decision comp as the critique reference, no `state.json`, no `report.json`). The payload carries `buildPath: { "value": "code" }` with no `toggle`; `crates/context/src/serve_question.rs:942` reads a missing toggle as `false`, and `question_page.rs` then renders no switch, so the user cannot flip into a path that cannot finish. Decision comps still render for the decision page, framed per the spec's "landscape frame on an 80x24 grid". **Spec gap reported:** section 7 says to treat a comp as a landscape frame on an 80x24 grid but does not say what the comp-led pipeline does with it; this ruling fills that. Cost if wrong: one sentence in `new-work.md`.
2. **`visualize.md:11` is left alone.** It carries the same portrait/landscape comp split as `new-work.md:51`, but `visualize.md` is loaded only on a comp-led build (its own line 3), and decision 1 makes terminal code-led, so a terminal run never reads it. **Spec gap reported:** section 7's file list is incomplete by this one site; the ruling makes the omission harmless.
3. **Terminal evidence is text captures with fixed names**, used identically in `new-work.md` (step 7) and the finish reviewer's input contract and check 0: `.impeccable/review/80x24.txt`, `120x40.txt`, `40x24.txt` (from `tmux capture-pane -p -e -J` at each size), `no-color.txt` (a run with `NO_COLOR=1`), and `piped.txt` (what the app writes when stdout is piped through `cat`). Check 0 gets a terminal validity test: not empty, no more rows than the height its name claims, the app rather than a shell prompt; `piped.txt` is evidence whatever it holds.
4. **Terminal runs `impeccable detect` once even when a hook is active.** `terminal.md:64` says the hook drops advisory findings and every `tui-` rule is advisory, so `new-work.md:139`'s "where this harness runs no design hook" gate would hide every terminal finding. The terminal branch runs `detect --json <source directory> --tmux <target> --tmux-sizes 80x24,120x40,40x24` (the combined form `audit.terminal.md:3` already documents). `polish.md:95`'s "never add another detector pass" gains the matching exception so the two files agree.
5. **Finish-reviewer `:32` and `:45` are left alone.** "The world's native devices" there means the chosen visual world's devices, not a platform; the explorer note flagged `:32` as a false positive. Other `native` grep hits checked and found non-platform: `new-work.md:55/:91/:97/:111/:129`, `operate.md:52`, `audit.md:15`, `clarify.md:51`, `distill.md:108`, `onboard.md:19`, `adapt.md:197`, `live.md:309`, `animate.md:78`, `critique.md:84/:211`, `hooks.md:7`, `overdrive.md:51`, `generate.md:34`, `asset-producer.md:41`. `new-work.md:115` is the comp-led hero gate that decision 1 routes terminal around.
6. **`adapt.terminal.md:21` is reworded, not given a fourth size.** Its "captures all three classes" becomes false once Compact exists. The matrix stays `80x24,120x40,40x24` everywhere (terminal.md, audit, adapt, the spec); the sentence names the three classes it covers and suggests adding a size such as `70x24` only when the compact class needs its own evidence.
7. **`doctor.md:47` matches PR 6's engine wording** ("carries terminal dependencies"; spec section 7, PR 6, "Doctor wording"). The finding id stays `workspace-platform-native-evidence` (`crates/context/src/staleness_deep.rs:391`).
8. **Layout and typeset bullets use `**Terminal:**`**, matching the plain `**Native:**` label those two files use (animate.md's parenthesized `**Terminal (`terminal`):**` form matches animate's own `**Native (...)**`). Neither file needs a "do not apply the web tooling below" clause: neither has a web-tooling section (one `rem` mention in `typeset.md:46`).
9. **The skill-behavior run needs a provider overlay.** Repo-root `.env` sets `IMPECCABLE_SKILL_BEHAVIOR_MODELS` to `gpt-5.6-terra` plus OpenRouter slugs (`anthropic/claude-sonnet-5`, `google/gemini-3.7-flash`); main's `tests/skill-behavior/providers.mjs` has no `/` branch, so `detectProvider` throws at load. The only direct key is `OPENAI_API_KEY`. Task 8 overlays the one file from the unmerged branch `skill-behavior-openrouter` into the working tree (`git show skill-behavior-openrouter:tests/skill-behavior/providers.mjs > ...`), runs, and restores it with `git checkout --` before anything else. That branch's file equals main's plus the OpenRouter provider (verified with `git diff main skill-behavior-openrouter -- tests/skill-behavior/providers.mjs`). **Never check out or merge that branch:** its tree predates PR 2 and PR 3 and its diff against main deletes both. The evidence names its configuration: terra through the OpenAI SDK, sonnet and gemini through OpenRouter's chat-completions path (not the README's native-SDK cells).
10. **The red set for judging the run** is the 2026-09-24 sweep on main-equivalent text (scratchpad `skill-behavior.log`), not the stale README table: terra fails 16 (missing product context) and 17; sonnet (OpenRouter) fails 16 (missing), 17, and 19 (denied-launcher planning case, the documented #744 gap); gemini-3.7-flash (OpenRouter) fails 9, 11, 16 (both cases), and 17. Terra's 16 (existing project) also failed on the main control run, so it is flaky. A regression is a scenario red on this branch and green on a detached-main control with the same model and the same provider path. No scenario uses a `terminal` fixture, so the run proves only that web and native routing did not regress.
11. **`context-terminal` is the only golden that moves.** `grep -rl "about 30 columns" tests/` finds only `tests/oracle/golden/context-terminal.json`; `audit.terminal.md` and `adapt.terminal.md` are never inlined. The recorder prefix `context-terminal` also matches `context-terminal-evidence-web`, which must come back byte-identical.
12. **The run is recorded in `tests/skill-behavior/README.md`.** The root `CLAUDE.md` asks that a change to Setup text update that README in the same change, and the README keeps dated run records. One short dated section, no change to its scenario or baseline tables (no scenario is added). This is one file beyond section 7's scope list; cost if unwanted: drop one commit.
13. **init's build-path question skips terminal.** `init.md:112` asks comp-first or code-first whenever image generation exists; decision 1 makes the answer moot for a terminal project, so the sentence gains a `terminal` exception rather than recording a value nothing reads.
14. **No other file pins the replaced text.** A fixed-string search of `tests/`, `scripts/`, and `crates/` for every replaced clause (`desktop web;`, `dropped frames on native;`, `never add another detector pass.`, `Verifying the build section. Determine`, `on a native platform)`, `native-platform guidance`, `Above 120 columns`, `the native reference above`, `non-runnable projects`) found nothing besides the `context-terminal` golden. Opt-in suites whose triggers match these paths (`scripts/test-suites.mjs`): `skill-behavior` runs in Task 8; `skill-workflow` (full billed builds, `^skill\//`) is skipped, since spec section 7 names only `test:skill-behavior` and that lane's README says to keep it manually scoped; `live-e2e` is not owed (no `skill/scripts/live-browser*` change, and `live.md` gains only a guard line).

## File structure

| File | Change |
|---|---|
| `skill/SKILL.src.md` | line 17 verify-round terminal case; line 21 "platform guidance for a native or terminal project" |
| `skill/reference/polish.md` | lines 22, 89, 92 terminal cases; line 95 detector exception |
| `skill/reference/new-work.md` | line 51 comp frame; line 53 terminal is code-led; line 135 capture matrix; line 139 detect and capture names; line 141 reviewer packet |
| `skill/reference/layout.md`, `typeset.md` | one `**Terminal:**` bullet after line 9 |
| `skill/reference/doctor.md` | line 47 names terminal dependencies and `terminal.md` |
| `skill/reference/init.md` | line 112 build-path question skips terminal; line 118 live skip; line 131 "the platform reference above" |
| `skill/reference/live.md` | web-only guard after line 1 |
| `skill/reference/adapt.terminal.md` | Compact row; line 21 reworded; line 24 "At 120 columns and above" |
| `skill/reference/terminal.md` | line 61 "40 columns" |
| `skill/agents/impeccable-finish-reviewer.md` | line 25 capture names and terminal packet; line 29 required set and terminal validity |
| `skill/agents/impeccable-asset-producer.md` | line 25 terminal comp frame |
| `tests/skill-reference.test.mjs` | two new tests; two assertions in the routing test |
| `tests/oracle/golden/context-terminal.json` | re-recorded |
| `tests/oracle/DELTAS.md` | one entry for the re-recorded golden |

---

### Task 1: Branch and baseline

**Files:** none changed.

- [ ] **Step 1: Confirm PRs 4, 5, and 6 are on the fork's main, then branch**

PR 7 goes last so its skill-behavior run sees the finished engine (spec section 7).

```bash
cd /Volumes/Containers/impeccable
git checkout main && git pull --ff-only origin main
rtk proxy git log --oneline --merges main | rtk proxy grep -E "terminal-platform-pr[456]"   # expect three lines: pr4, pr5, pr6
git checkout -b terminal-platform-pr7
git log --oneline -1
```

If any of the three merges is missing, stop and report to the controller; do not start PR 7 on an unfinished engine.

- [ ] **Step 2: Build the engine and confirm the tooling**

```bash
export SCRATCH="<the session scratchpad directory the harness names>"   # used by Tasks 2 and 8; never a path inside the repo
mkdir -p "$SCRATCH"
cargo build --release -p impeccable 2>&1 | tail -n 2
ls -l target/release/impeccable
bun --version
which claude   # plugin-e2e needs the claude CLI; without it that suite skips
```

- [ ] **Step 3: Record the baseline**

```bash
node --test tests/skill-reference.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)"          # expect fail 0
IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/run.mjs context-terminal 2>&1 | tail -n 3   # expect no XX lines
bun run build 2>&1 | tail -n 3
IMPECCABLE_BIN="$PWD/target/release/impeccable" bun run test 2>&1 | tail -n 15
rtk proxy grep -rl "about 30 columns" tests/ skill/   # expect skill/reference/terminal.md and tests/oracle/golden/context-terminal.json only
git status --short   # after the build: only the untracked local files; never stage them
```

Known pre-existing failures on this machine, recorded and not fixed: `tests/release.test.mjs` (signed tags forced), oracle case `detect-fixture-text-framework-vite` when a llama-server holds port 8080, bun 5 s timeouts under load (rerun the file alone to confirm). If `bun run build` leaves tracked files modified, record which ones and restore them (`git checkout -- <path>`) before Task 2; they are not this PR's.

- [ ] **Step 4: No commit.** Shell variables do not survive between tool calls, so re-export `SCRATCH` and `BASE` in any later command that uses them. Record `BASE=$(git rev-parse HEAD)`, the pass and fail counts, and any failure outside the known list in the ledger.

---
### Task 2: Width classes, the 40-column probe, and the `context-terminal` golden

**Files:**
- Modify: `tests/skill-reference.test.mjs` (new test after the audit-skeleton test, which ends at line 138)
- Modify: `skill/reference/adapt.terminal.md` (lines 17, 21, 24)
- Modify: `skill/reference/terminal.md` (line 61)
- Re-record: `tests/oracle/golden/context-terminal.json`
- Modify: `tests/oracle/DELTAS.md` (append one entry)

**Interfaces:** none (markdown and test pins).

- [ ] **Step 1: Write the failing test**

Insert this test in `tests/skill-reference.test.mjs` directly after the closing `});` of `it('keeps the audit report skeleton identical across the web, native, and terminal variants', ...)` (line 138) and before `it('routes terminal projects to the terminal references and keeps detect available to them', ...)`. The existing pins at lines 134 and 135 (`/under 60 columns/`, `/120 columns and above/`) stay.

```js
  it('classifies every terminal width from narrow to wide with no gap', () => {
    // The engine's narrow cutoff (NARROW_COLUMNS = 60 in crates/terminal)
    // and the adapt table must agree, and no width between 40 and 120
    // columns may fall outside a class.
    const adapt = readFileSync(join(ROOT, 'skill/reference/adapt.terminal.md'), 'utf-8').replace(/\r\n?/g, '\n');
    const terminal = readFileSync(join(ROOT, 'skill/reference/terminal.md'), 'utf-8').replace(/\r\n?/g, '\n');
    const rows = adapt.split('\n').filter((l) => /^\| (Narrow|Compact|Standard|Wide) \|/.test(l));
    assert.deepEqual(rows.map((l) => l.split('|')[2].trim()), [
      'under 60 columns',
      '60 to 79 columns',
      '80 to 119 columns',
      '120 columns and above',
    ]);
    assert.match(adapt, /At 120 columns and above, text blocks stay near 80 to 100 cells/);
    assert.doesNotMatch(adapt, /Above 120 columns/);
    assert.doesNotMatch(adapt, /all three classes/);
    // The matrix probes 40 columns: every capture command passes 40x24.
    assert.match(terminal, /80x24, 120x40, and 40 columns;/);
    assert.doesNotMatch(terminal, /about 30 columns/);
  });
```

- [ ] **Step 2: Run it and watch it fail**

```bash
node --test --test-name-pattern="classifies every terminal width" tests/skill-reference.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)|AssertionError|Expected|actual" | head -n 12
```

Expected: `ℹ fail 1`, the `deepEqual` failing because the Compact row is missing (three rows, not four).

- [ ] **Step 3: Edit `skill/reference/adapt.terminal.md`**

Replace line 17 plus the newline after it (the Narrow row) so the Compact row follows it. Old:

```markdown
| Narrow | under 60 columns | Which single region survives, how the footer collapses to one line, which labels abbreviate |
```

New (two rows):

```markdown
| Narrow | under 60 columns | Which single region survives, how the footer collapses to one line, which labels abbreviate |
| Compact | 60 to 79 columns | Which secondary region folds away or stacks, how many footer keys still fit, and whether nested borders flatten to one level |
```

Line 21. Old substring:

```markdown
captures all three classes in one run and reports what collapses below 60 columns (`tui-rt-collapse-narrow`) and what drifts on emoji or CJK (`tui-rt-width-drift`).
```

New:

```markdown
captures the narrow, standard, and wide classes in one run and reports what collapses below 60 columns (`tui-rt-collapse-narrow`) and what drifts on emoji or CJK (`tui-rt-width-drift`). Add a size such as `70x24` to `--tmux-sizes` when the compact class needs its own evidence.
```

Line 24. Old substring:

```markdown
- **Cap line length.** Above 120 columns, text blocks
```

New:

```markdown
- **Cap line length.** At 120 columns and above, text blocks
```

- [ ] **Step 4: Edit `skill/reference/terminal.md` line 61**

Old substring:

```markdown
**Run the matrix.** 80x24, 120x40, and about 30 columns;
```

New:

```markdown
**Run the matrix.** 80x24, 120x40, and 40 columns;
```

The rest of the line, including `<!-- rule:terminal-verify-matrix -->`, is unchanged.

- [ ] **Step 5: Run the test and watch it pass**

```bash
node --test tests/skill-reference.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)"
```

Expected: `ℹ fail 0` (all prior tests plus the new one pass).

- [ ] **Step 6: Watch the oracle catch the inlined file**

```bash
IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/run.mjs context-terminal 2>&1 | grep -E "^(XX|\?\?) " | head
```

Expected: one `XX context-terminal` line (stdout differs), and no line for `context-terminal-evidence-web`.

- [ ] **Step 7: Re-record and review the golden by hand**

```bash
cp tests/oracle/golden/context-terminal.json $SCRATCH/context-terminal.before.json
IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/record.mjs --bin context-terminal
rtk proxy git diff --stat tests/oracle/golden/
node -e "
const a=require(process.env.SCRATCH+'/context-terminal.before.json'), b=require('./tests/oracle/golden/context-terminal.json');
const same = a.stdout.replace('80x24, 120x40, and about 30 columns;','80x24, 120x40, and 40 columns;') === b.stdout;
console.log('stdout differs only by the matrix clause:', same);
console.log('stderr/exit/files unchanged:', a.stderr===b.stderr && a.exit===b.exit && JSON.stringify(a.files)===JSON.stringify(b.files));
"
rm $SCRATCH/context-terminal.before.json
```

Expected: `git diff --stat` lists exactly one file, `tests/oracle/golden/context-terminal.json` (the prefix also re-records `context-terminal-evidence-web`, which must come back byte-identical); both `node` lines print `true`. If either prints `false` or a second golden moved, stop and diagnose before going on.

- [ ] **Step 8: Add the DELTAS entry**

Append to the end of `tests/oracle/DELTAS.md`, with the execution date from `date +%F` in the heading:

```markdown


## Recorded <date +%F>: terminal.md's size matrix (spec PR 7)

`impeccable context` inlines `skill/reference/terminal.md` for a `terminal`
project, so the edit to its Run the matrix bullet moved one golden.

The `context-terminal` golden's matrix now reads `80x24, 120x40, and 40
columns` where it read `80x24, 120x40, and about 30 columns`. Nothing else
in the output changed. The golden was re-recorded, so the case passes; it is
deliberately not written as a `` - `id` `` bullet, because `tests/oracle/run.mjs`
(`loadAcceptedDeltas`) treats every line starting with a backticked id as an
accepted delta and would stop reporting future differences on the case.
```

- [ ] **Step 9: Confirm the oracle is green for the prefix**

```bash
IMPECCABLE_BIN="$PWD/target/release/impeccable" node tests/oracle/run.mjs context-terminal 2>&1 | tail -n 2
```

Expected: no `XX` or `??` lines; the summary counts both cases as passing.

- [ ] **Step 10: Commit**

```bash
git add tests/skill-reference.test.mjs skill/reference/adapt.terminal.md skill/reference/terminal.md tests/oracle/golden/context-terminal.json tests/oracle/DELTAS.md
git commit -F - <<'EOF'
Close the 60 to 79 column gap in the terminal width classes

adapt.terminal.md gains a Compact row (60 to 79 columns) and says "At
120 columns and above" to match its table; its --tmux-sizes sentence
now names the three classes the matrix covers. terminal.md's matrix
probes 40 columns, the size every capture command already passes.
context-terminal is re-recorded because it inlines terminal.md.

AI assistance: drafted with Claude Code.
EOF
```

---

### Task 3: The live-mode web-only guard

**Files:**
- Modify: `tests/skill-reference.test.mjs` (the routing test, lines 163 to 164)
- Modify: `skill/reference/live.md` (after line 1)

**Interfaces:** none.

- [ ] **Step 1: Write the failing assertions**

In `it('routes terminal projects to the terminal references and keeps detect available to them', ...)`, replace these two lines (163 and 164 before Task 2's insertion; find them by content):

```js
    const generate = readFileSync(join(ROOT, 'skill/reference/generate.md'), 'utf-8');
    assert.match(generate, /on `ios` \/ `android` \/ `adaptive` \/ `terminal` projects, decline this command/);
```

with:

```js
    const generate = readFileSync(join(ROOT, 'skill/reference/generate.md'), 'utf-8');
    assert.match(generate, /on `ios` \/ `android` \/ `adaptive` \/ `terminal` projects, decline this command/);

    // live.md carries the same guard as generate.md, near the top so it is
    // read before the setup steps.
    const liveHead = readFileSync(join(ROOT, 'skill/reference/live.md'), 'utf-8').replace(/\r\n?/g, '\n').split('\n').slice(0, 5).join('\n');
    assert.match(liveHead, /\*\*Web only\.\*\* Live mode's browser overlay has no native or terminal equivalent/);
    assert.match(liveHead, /on `ios` \/ `android` \/ `adaptive` \/ `terminal` projects, decline this command/);
```

- [ ] **Step 2: Run it and watch it fail**

```bash
node --test --test-name-pattern="routes terminal projects" tests/skill-reference.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)"
```

Expected: `ℹ fail 1`.

- [ ] **Step 3: Add the guard to `skill/reference/live.md`**

Old (lines 1 to 3):

```markdown
Interactive live variant mode: select elements in the browser, pick a design action, and get AI-generated HTML+CSS variants hot-swapped via the dev server's HMR.

## Prerequisites
```

New:

```markdown
Interactive live variant mode: select elements in the browser, pick a design action, and get AI-generated HTML+CSS variants hot-swapped via the dev server's HMR.

**Web only.** Live mode's browser overlay has no native or terminal equivalent; on `ios` / `android` / `adaptive` / `terminal` projects, decline this command and offer `bolder` or `quieter` on the source instead.

## Prerequisites
```

The guard line is byte-identical to `skill/reference/generate.md:5`.

- [ ] **Step 4: Run the test and watch it pass**

```bash
node --test tests/skill-reference.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)"
diff <(sed -n 5p skill/reference/generate.md) <(sed -n 3p skill/reference/live.md) && echo GUARD_MATCHES
```

Expected: `ℹ fail 0` and `GUARD_MATCHES`.

- [ ] **Step 5: Commit**

```bash
git add tests/skill-reference.test.mjs skill/reference/live.md
git commit -F - <<'EOF'
Guard live mode as web-only like generate

live.md now opens with the same web-only line generate.md carries, so
a terminal or native project that reaches live directly is turned back
before the setup steps.

AI assistance: drafted with Claude Code.
EOF
```

---
### Task 4: Terminal branches in SKILL.src.md, polish, layout, typeset, doctor, and init

**Files:**
- Modify: `tests/skill-reference.test.mjs` (new test at the end of the `describe` block)
- Modify: `skill/SKILL.src.md` (lines 17, 21)
- Modify: `skill/reference/polish.md` (lines 22, 89, 92, 95)
- Modify: `skill/reference/layout.md`, `skill/reference/typeset.md` (after line 9)
- Modify: `skill/reference/doctor.md` (line 47)
- Modify: `skill/reference/init.md` (lines 112, 118, 131)

**Interfaces:** the test's `branches` table, keyed by repo-relative path; Tasks 5 and 6 add entries to it.

- [ ] **Step 1: Write the failing test**

Append this test inside the `describe('skill reference authoring contracts', ...)` block, after the routing test's closing `});` and before the final `});` of the file:

```js
  it('gives terminal projects a branch wherever the skill splits web from native', () => {
    // Each file that names a web case and a native case must also name the
    // terminal case, or a terminal project falls through to the wrong one.
    const read = (p) => readFileSync(join(ROOT, p), 'utf-8').replace(/\r\n?/g, '\n');
    const branches = {
      'skill/SKILL.src.md': [
        /the shipped device classes on a native platform; the tmux capture matrix on a terminal\)/,
        /platform guidance for a native or terminal project when applicable/,
      ],
      'skill/reference/polish.md': [
        /on `terminal`, the tmux capture matrix from \[terminal\.md\]\(terminal\.md\)'s Verifying the build section/,
        /on terminal, the 80x24, 120x40, and 40x24 captures, a live resize/,
        /on terminal, a first frame within 100 ms, a terminal left as found after quit and after `Ctrl-C`/,
        /never add another detector pass beyond the one terminal run above/,
      ],
      'skill/reference/layout.md': [/^- \*\*Terminal:\*\* follow the Layout & structure section of \[terminal\.md\]\(terminal\.md\)/m],
      'skill/reference/typeset.md': [/^- \*\*Terminal:\*\* follow the Typography section of \[terminal\.md\]\(terminal\.md\)/m],
      'skill/reference/doctor.md': [/native build files or terminal dependencies/, /\[android\.md\]\(android\.md\), or \[terminal\.md\]\(terminal\.md\)/],
      'skill/reference/init.md': [
        /skip native, terminal, or non-runnable projects/,
        /the platform reference above is the one thing/,
        /the platform is not `terminal` \(a terminal surface always builds code-led/,
      ],
    };
    for (const [file, patterns] of Object.entries(branches)) {
      const text = read(file);
      for (const pattern of patterns) assert.match(text, pattern, `${file} has no terminal branch matching ${pattern}`);
    }
    assert.doesNotMatch(read('skill/reference/init.md'), /the native reference above/);
    assert.doesNotMatch(read('skill/SKILL.src.md'), /native-platform guidance/);
  });
```

- [ ] **Step 2: Run it and watch it fail**

```bash
node --test --test-name-pattern="branch wherever the skill splits" tests/skill-reference.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)|has no terminal branch" | head -n 4
```

Expected: `ℹ fail 1`, the message naming `skill/SKILL.src.md` (the first entry).

- [ ] **Step 3: Edit `skill/SKILL.src.md`**

Line 17 (verify rounds). Old substring:

```markdown
(desktop and mobile together on the web; the shipped device classes on a native platform)
```

New:

```markdown
(desktop and mobile together on the web; the shipped device classes on a native platform; the tmux capture matrix on a terminal)
```

Line 21 (Setup step 1). Old substring:

```markdown
and native-platform guidance when applicable;
```

New:

```markdown
and platform guidance for a native or terminal project when applicable;
```

- [ ] **Step 4: Edit `skill/reference/polish.md`**

Line 22 (representative sizes). Old substring:

```markdown
captured per the platform reference's Verifying the build section. Determine:
```

New:

```markdown
captured per the platform reference's Verifying the build section; on `terminal`, the tmux capture matrix from [terminal.md](terminal.md)'s Verifying the build section (80x24, 120x40, and 40x24, plus one run with `NO_COLOR=1` and one with stdout piped through `cat`). Determine:
```

Line 89 (size classes). Old substring:

```markdown
phone and tablet size classes in both supported orientations on native;
```

New:

```markdown
phone and tablet size classes in both supported orientations on native; on terminal, the 80x24, 120x40, and 40x24 captures, a live resize, and the width classes in [adapt.terminal.md](adapt.terminal.md);
```

Line 92 (runtime checks). Old substring:

```markdown
supported OS versions, runtime warnings, and dropped frames on native;
```

New:

```markdown
supported OS versions, runtime warnings, and dropped frames on native; on terminal, a first frame within 100 ms, a terminal left as found after quit and after `Ctrl-C`, and one `{{scripts_path}}/impeccable detect --json <source directory> --tmux <target> --tmux-sizes 80x24,120x40,40x24` run, because the hook drops the advisory `tui-` findings; that run is also the scan a `MANUAL_DETECTOR_REQUIRED` directive asks for, and it takes the source directory so the project signals are collected;
```

Line 95 (detector discipline). Old substring:

```markdown
never add another detector pass.
```

New:

```markdown
never add another detector pass beyond the one terminal run above.
```

- [ ] **Step 5: Add the Terminal bullets to `layout.md` and `typeset.md`**

`skill/reference/layout.md`: after line 9, `- **Native:** follow [ios.md](ios.md) or [android.md](android.md) for navigation, insets, adaptation, and touch targets.`, insert a new line:

```markdown
- **Terminal:** follow the Layout & structure section of [terminal.md](terminal.md) for constraint layout, resize, and borders, and decide each width class in [adapt.terminal.md](adapt.terminal.md).
```

`skill/reference/typeset.md`: after line 9, `- **Native:** follow [ios.md](ios.md) or [android.md](android.md), including platform scaling and accessibility behavior.`, insert a new line:

```markdown
- **Terminal:** follow the Typography section of [terminal.md](terminal.md): one monospace face, bold, dim, and reverse as the whole scale, widths measured in cells, no figlet banners.
```

- [ ] **Step 6: Edit `skill/reference/doctor.md` line 47**

Old substring:

```markdown
a workspace carrying native build files while inheriting a root record that resolves to web gets web guidance for its whole life and never loads [ios.md](ios.md) or [android.md](android.md).
```

New:

```markdown
a workspace carrying native build files or terminal dependencies while inheriting a root record that resolves to web gets web guidance for its whole life and never loads [ios.md](ios.md), [android.md](android.md), or [terminal.md](terminal.md).
```

- [ ] **Step 7: Edit `skill/reference/init.md`**

Line 118. Old substring `skip native or non-runnable projects`; new `skip native, terminal, or non-runnable projects`. Live mode is web-only, so a terminal project gets no live config.

Line 131. Old substring `the native reference above is the one thing`; new `the platform reference above is the one thing`. Line 104 already loads `terminal.md` for a terminal platform; this sentence now covers it.

Line 112 (decision 13). Old substring:

```markdown
When image generation is available and no `buildPath` is recorded yet, ask once how new surfaces should be built.
```

New:

```markdown
When image generation is available, the platform is not `terminal` (a terminal surface always builds code-led; see [new-work.md](new-work.md)), and no `buildPath` is recorded yet, ask once how new surfaces should be built.
```

- [ ] **Step 8: Run the tests and the prose gate**

```bash
node --test tests/skill-reference.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)"
bun run build 2>&1 | tail -n 3
git status --short
```

Expected: `ℹ fail 0`; the build finishes with no `validateSkillProse` error; `git status` shows only the seven files above as modified (plus the untracked local files). If the build touched tracked generated output, restore it with `git checkout -- <path>`; never stage it.

- [ ] **Step 9: Commit**

```bash
git add tests/skill-reference.test.mjs skill/SKILL.src.md skill/reference/polish.md skill/reference/layout.md skill/reference/typeset.md skill/reference/doctor.md skill/reference/init.md
git commit -F - <<'EOF'
Give terminal its own branch in setup, polish, layout, and typeset

SKILL.md's verify rounds and Setup step name the terminal case. polish
verifies with the tmux capture matrix, a live resize, the first-frame
and restore checks, and one advisory detect run; layout and typeset get
a Terminal bullet beside Native. doctor's workspace evidence finding
names terminal dependencies and terminal.md, and init skips live config
for terminal projects and does not ask the build-path question there.

AI assistance: drafted with Claude Code.
EOF
```

---

### Task 5: Terminal branches in new-work

**Files:**
- Modify: `tests/skill-reference.test.mjs` (one entry in the Task 4 `branches` table)
- Modify: `skill/reference/new-work.md` (lines 51, 53, 135, 139, 141)

**Interfaces:** the capture names fixed in decision 3: `80x24.txt`, `120x40.txt`, `40x24.txt`, `no-color.txt`, `piped.txt` under `.impeccable/review/`. Task 6 uses the same names.

- [ ] **Step 1: Write the failing entry**

Add this entry to the `branches` object in `it('gives terminal projects a branch wherever the skill splits web from native', ...)`, after the `'skill/SKILL.src.md'` entry:

```js
      'skill/reference/new-work.md': [
        /landscape for desktop web, and a landscape frame on an 80x24 grid of monospace cells for a terminal surface/,
        /A terminal surface is code-led too, image generation or not/,
        /`buildPath: \{ "value": "code" \}` and no toggle/,
        /on `terminal`, the tmux capture matrix from \[terminal\.md\]\(terminal\.md\)/,
        /A terminal project runs it once here, hook or no hook/,
        /`80x24\.txt`, `120x40\.txt`, and `40x24\.txt` from `tmux capture-pane -p -e -J`/,
        /on a terminal platform \[terminal\.md\]\(terminal\.md\) plus the terminal detect findings/,
      ],
```

- [ ] **Step 2: Run it and watch it fail**

```bash
node --test --test-name-pattern="branch wherever the skill splits" tests/skill-reference.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)|has no terminal branch" | head -n 4
```

Expected: `ℹ fail 1`, naming `skill/reference/new-work.md`.

- [ ] **Step 3: Line 51, the comp frame**

Old substring:

```markdown
portrait at device viewport for a native app or mobile-first surface, landscape for desktop web;
```

New:

```markdown
portrait at device viewport for a native app or mobile-first surface, landscape for desktop web, and a landscape frame on an 80x24 grid of monospace cells for a terminal surface;
```

- [ ] **Step 4: Line 53, terminal is code-led (decision 1)**

Old substring:

```markdown
Without image generation there is no toggle and no choice: code-led is the only path, stated in one line rather than asked.
```

New:

```markdown
Without image generation there is no toggle and no choice: code-led is the only path, stated in one line rather than asked. A terminal surface is code-led too, image generation or not: the comp-led path ends in raster plates and a pixel hero gate that a terminal cannot draw. Author its payloads with `buildPath: { "value": "code" }` and no toggle; the decision comps still render for the page, and the chosen one rides at the finish review as the critique reference.
```

- [ ] **Step 5: Line 135, the inspection sizes**

Old substring:

```markdown
captured from the simulator or emulator the way the platform reference's Verifying the build section describes.
```

New:

```markdown
captured from the simulator or emulator the way the platform reference's Verifying the build section describes; on `terminal`, the tmux capture matrix from [terminal.md](terminal.md) (80x24, 120x40, and 40x24, plus one run with `NO_COLOR=1` and one with stdout piped through `cat`).
```

- [ ] **Step 6: Line 139, the detect step and the capture names (decisions 3 and 4)**

First old substring:

```markdown
A native platform skips the detector entirely: it reads HTML and CSS and has no verdict on native code, so the reviewer's floor check is the only slop gate and the input packet says so.
```

New:

```markdown
A native platform skips the detector entirely: it reads HTML and CSS and has no verdict on native code, so the reviewer's floor check is the only slop gate and the input packet says so. A terminal project runs it once here, hook or no hook, because every `tui-` rule is advisory and the hook drops advisory findings: with the app running in tmux, run `{{scripts_path}}/impeccable detect --json <source directory> --tmux <target> --tmux-sizes 80x24,120x40,40x24`, confirm each finding against the source, fix what is mechanical, and pass the rest to the reviewer. That run is also the scan a `MANUAL_DETECTOR_REQUIRED` directive asks for; it takes the source directory, not the changed files, so the project signals are collected.
```

Second old substring (same line, later):

```markdown
on native, one per device class, such as `phone.png` and `tablet.png`, suffixed per OS on adaptive)
```

New:

```markdown
on native, one per device class, such as `phone.png` and `tablet.png`, suffixed per OS on adaptive; on terminal, the text captures `80x24.txt`, `120x40.txt`, and `40x24.txt` from `tmux capture-pane -p -e -J`, plus `no-color.txt` from an 80x24 run with `NO_COLOR=1` and `piped.txt` holding what the app writes when stdout is piped through `cat`)
```

- [ ] **Step 7: Line 141, the reviewer packet**

Old substring:

```markdown
plus one line saying no detector ran, so the reviewer judges in the platform's conventions rather than the web's.
```

New:

```markdown
plus one line saying no detector ran, and on a terminal platform [terminal.md](terminal.md) plus the terminal detect findings, so the reviewer judges in the platform's conventions rather than the web's.
```

- [ ] **Step 8: Run the tests and the prose gate**

```bash
node --test tests/skill-reference.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)"
bun run build 2>&1 | tail -n 3
```

Expected: `ℹ fail 0`; no `validateSkillProse` error. The existing `keeps direction contracts in development-only surface briefs` test reads `new-work.md`'s section 5 and must still pass (none of the edits touch it).

- [ ] **Step 9: Commit**

```bash
git add tests/skill-reference.test.mjs skill/reference/new-work.md
git commit -F - <<'EOF'
Route terminal builds through new-work's code-led path

A terminal surface comps as a landscape frame on an 80x24 grid and
builds code-led, since the comp-led path ends in raster plates and a
pixel hero gate. Step 7 inspects the tmux capture matrix, runs detect
once over source and pane (the hook drops advisory findings), names
the text captures, and hands the reviewer terminal.md.

AI assistance: drafted with Claude Code.
EOF
```

---
### Task 6: Terminal branches in the shipped agents

**Files:**
- Modify: `tests/skill-reference.test.mjs` (two entries in the `branches` table)
- Modify: `skill/agents/impeccable-finish-reviewer.md` (lines 25, 29)
- Modify: `skill/agents/impeccable-asset-producer.md` (line 25)

**Interfaces:** the capture names from Task 5. Lines 32 and 45 of the reviewer stay as they are (decision 5).

- [ ] **Step 1: Write the failing entries**

Add to the `branches` object, after the `'skill/reference/init.md'` entry:

```js
      'skill/agents/impeccable-finish-reviewer.md': [
        /terminal: the text captures `80x24\.txt`, `120x40\.txt`, `40x24\.txt`, `no-color\.txt`, and `piped\.txt`/,
        /On a terminal \(`terminal`\) build the packet adds `reference\/terminal\.md` and the terminal detect findings/,
        /terminal: the three sizes plus `no-color\.txt` and `piped\.txt`/,
        /A terminal text capture is valid when it is not empty/,
      ],
      'skill/agents/impeccable-asset-producer.md': [/A terminal surface is a landscape frame drawn on an 80x24 grid of monospace cells/],
```

- [ ] **Step 2: Run it and watch it fail**

```bash
node --test --test-name-pattern="branch wherever the skill splits" tests/skill-reference.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)|has no terminal branch" | head -n 4
```

Expected: `ℹ fail 1`, naming `skill/agents/impeccable-finish-reviewer.md`.

- [ ] **Step 3: Edit the reviewer's input contract (line 25)**

First old substring:

```markdown
(web: `desktop.png` and `mobile.png`; native: device-class names such as `phone.png` and `tablet.png`, suffixed per OS on adaptive)
```

New:

```markdown
(web: `desktop.png` and `mobile.png`; native: device-class names such as `phone.png` and `tablet.png`, suffixed per OS on adaptive; terminal: the text captures `80x24.txt`, `120x40.txt`, `40x24.txt`, `no-color.txt`, and `piped.txt`)
```

Second old substring (same line, end of the native sentence):

```markdown
and know your floor check is the build's only slop gate.
```

New:

```markdown
and know your floor check is the build's only slop gate. On a terminal (`terminal`) build the packet adds `reference/terminal.md` and the terminal detect findings: read the terminal reference alongside the craft floor, judge every check in the terminal's conventions (the user's 16 colors, cells, the keys the footer names), read color from the escape sequences the captures keep, and hold each `tui-` finding as advisory until a capture or the source confirms it. A terminal build is code-led, so its decision comp is a critique reference.
```

- [ ] **Step 4: Edit check 0 (line 29)**

First old substring:

```markdown
(web: `desktop.png` and `mobile.png`; native: one capture per shipped device class)
```

New:

```markdown
(web: `desktop.png` and `mobile.png`; native: one capture per shipped device class; terminal: the three sizes plus `no-color.txt` and `piped.txt`)
```

Second old substring:

```markdown
dimensions that make sense for the named viewport.
```

New:

```markdown
dimensions that make sense for the named viewport. A terminal text capture is valid when it is not empty, holds no more rows than the height its name claims (24 for `no-color.txt`), and shows the app rather than a shell prompt; `piped.txt` holds whatever the app wrote to the pipe, and that output is the evidence, error or not.
```

- [ ] **Step 5: Edit the asset producer (line 25)**

Old substring:

```markdown
A native app or mobile-first surface is a portrait frame at its device viewport, never a landscape default.
```

New:

```markdown
A native app or mobile-first surface is a portrait frame at its device viewport, never a landscape default. A terminal surface is a landscape frame drawn on an 80x24 grid of monospace cells.
```

- [ ] **Step 6: Run the tests, the prose gate, and the plugin loader E2E**

```bash
node --test tests/skill-reference.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)"
bun run build 2>&1 | tail -n 3
bun run test:plugin-e2e 2>&1 | tail -n 6
git status --short
```

Expected: `ℹ fail 0`; the build passes (`validateSkillProse` scans `skill/agents/*.md` too); the plugin E2E passes (it installs the committed `plugin/`, which this PR does not regenerate, so it proves the loader contract is unchanged; the agent text itself reaches `plugin/agents/` only through `sync-generated-output.yml` after merge). If the `claude` CLI is missing the suite skips; record that as a skip, not a pass. `git status` shows only the three files above as modified.

- [ ] **Step 7: Commit**

```bash
git add tests/skill-reference.test.mjs skill/agents/impeccable-finish-reviewer.md skill/agents/impeccable-asset-producer.md
git commit -F - <<'EOF'
Teach the finish reviewer and asset producer the terminal case

The reviewer's input contract names the five text captures and the
terminal packet (terminal.md plus advisory detect findings), and check
0 gets a validity test for text captures. The asset producer frames a
terminal decision comp as landscape on an 80x24 grid.

AI assistance: drafted with Claude Code.
EOF
```

---

### Task 7: Full gates

**Files:** none changed.

- [ ] **Step 1: Build and the default suite against the local engine**

```bash
bun run build 2>&1 | tail -n 3
IMPECCABLE_BIN="$PWD/target/release/impeccable" bun run test 2>&1 | tail -n 15
git status --short
```

Expected: the build passes; `bun run test` shows no failure outside Task 1's recorded baseline (the known list: `tests/release.test.mjs`, the port-8080 oracle case, bun timeouts that pass when the file is rerun alone). The oracle suite passes with the re-recorded `context-terminal` golden. `git status` shows no tracked change (every edit is committed) apart from the untracked local files; if the build rewrote tracked generated output, restore it with `git checkout -- <path>` and never stage it.

- [ ] **Step 2: Confirm the diff is exactly this PR's**

```bash
rtk proxy git diff --stat "$BASE"..HEAD
rtk proxy git diff "$BASE"..HEAD -- skill/ | rtk proxy grep -c $'\u2014'   # expect 0 (no em dash added)
```

Expected file list: `skill/SKILL.src.md`, `skill/reference/{polish,new-work,layout,typeset,doctor,init,live,adapt.terminal,terminal}.md`, `skill/agents/impeccable-{finish-reviewer,asset-producer}.md`, `tests/skill-reference.test.mjs`, `tests/oracle/golden/context-terminal.json`, `tests/oracle/DELTAS.md`. Nothing under `plugin/`, `.claude/`, `.cursor/`, `.agents/`, `crates/`, or `skill/reference/degraded/`.

---
### Task 8: The skill-behavior run against the baseline

**Files:**
- Temporarily overlaid, never staged: `tests/skill-behavior/providers.mjs` (decision 9)
- Modify: `tests/skill-behavior/README.md` (one dated result section, decision 12)

**Interfaces:** none. This task bills the OpenAI and OpenRouter keys in `.env` (a full sweep was about 25 minutes and a few dollars on 2026-09-24). It runs once; a rerun is only for a scenario that Step 4 flags.

- [ ] **Step 1: Overlay the OpenRouter provider and confirm the overlay is only that**

```bash
rtk proxy git diff main skill-behavior-openrouter -- tests/skill-behavior/providers.mjs | rtk proxy grep -E '^[-+][^-+]'
git show skill-behavior-openrouter:tests/skill-behavior/providers.mjs > tests/skill-behavior/providers.mjs
git status --short tests/skill-behavior/
```

Expected: the diff shows only the `createOpenAI` import change, the `openrouter` row in `PROVIDERS`, the `/` branch in `detectProvider`, and the `openrouter` block in `getModel` (no deletions beyond the one import line). `git status` shows ` M tests/skill-behavior/providers.mjs`. Never check out or merge `skill-behavior-openrouter` itself.

- [ ] **Step 2: Run the suite once, in the background**

The model list is given on the command line so the run does not depend on the `.env` value (`loadEnv` in `providers.mjs` never overwrites a variable already set). Keys load from `.env` without being printed.

```bash
IMPECCABLE_BIN="$PWD/target/release/impeccable" \
IMPECCABLE_SKILL_BEHAVIOR_MODELS=gpt-5.6-terra,anthropic/claude-sonnet-5,google/gemini-3.7-flash \
  bun run test:skill-behavior > "$SCRATCH/pr7-skill-behavior.log" 2>&1
```

Run it with `run_in_background` and wait on it with a Monitor until-loop on the log's final `ℹ duration_ms` line (the suite's wall clock cap is 30 minutes, `wallClockMs: 1_800_000` in `scripts/test-suites.mjs`). Do not run anything else that bills keys meanwhile.

- [ ] **Step 3: Restore the provider file immediately**

```bash
git checkout -- tests/skill-behavior/providers.mjs
git status --short tests/skill-behavior/   # expect nothing
```

- [ ] **Step 4: Tabulate the failures per model and compare with the red set**

```bash
node -e "
const lines = require('fs').readFileSync(process.argv[1], 'utf8').split('\n');
let pending = [];
for (const l of lines) {
  const m = l.match(/^  ✖ (scenario [^(]+)/);
  if (m) { pending.push(m[1].trim()); continue; }
  const end = l.match(/^[✖✔] skill behavior :: (\S+)/);
  if (end) { console.log(end[1] + ': ' + (pending.length ? pending.join(' | ') : 'all pass')); pending = []; }
}
const t = lines.filter((l) => /^ℹ (tests|pass|fail|skipped) /.test(l));
console.log(t.join('  '));
" "$SCRATCH/pr7-skill-behavior.log"
```

Judge against decision 10's red set (the 2026-09-24 sweep, same models and provider paths):

| Model (path) | Red on main already |
|---|---|
| gpt-5.6-terra (OpenAI SDK) | 16 (missing product context), 17; 16 (existing project) is flaky |
| anthropic/claude-sonnet-5 (OpenRouter) | 16 (missing product context), 17, 19 (denied launcher, planning-only) |
| google/gemini-3.7-flash (OpenRouter) | 9, 11, 16 (both cases), 17 |

A failure inside that set is baseline. A skip line (`skipped`, then the unset key name) means that family was not measured; say so, never count it as a pass. For every failure outside the set, go to Step 5.

- [ ] **Step 5: Separate flake from regression (only for failures outside the red set)**

First rerun the failing scenario once on this branch, same model, with the overlay reapplied (Step 1's `git show` line) and the trace on:

```bash
IMPECCABLE_BIN="$PWD/target/release/impeccable" IMPECCABLE_SKILL_BEHAVIOR_MODELS=<model> IMPECCABLE_SKILL_BEHAVIOR_VERBOSE=1 \
  node --test --test-timeout=600000 --test-name-pattern="scenario <N>" tests/skill-behavior/scenarios.test.mjs > "$SCRATCH/pr7-s<N>-branch.log" 2>&1
git checkout -- tests/skill-behavior/providers.mjs
```

If it passes, record it as flake. If it fails again, run the same scenario on a detached-main control:

```bash
git worktree add --detach "$SCRATCH/wt-main" main
ln -s "$PWD/node_modules" "$SCRATCH/wt-main/node_modules"
git show skill-behavior-openrouter:tests/skill-behavior/providers.mjs > "$SCRATCH/wt-main/tests/skill-behavior/providers.mjs"
( set -a; . ./.env; set +a
  cd "$SCRATCH/wt-main" && IMPECCABLE_BIN=/Volumes/Containers/impeccable/target/release/impeccable \
  IMPECCABLE_SKILL_BEHAVIOR_MODELS=<model> IMPECCABLE_SKILL_BEHAVIOR_VERBOSE=1 \
  node --test --test-timeout=600000 --test-name-pattern="scenario <N>" tests/skill-behavior/scenarios.test.mjs ) > "$SCRATCH/pr7-s<N>-main.log" 2>&1
git worktree remove --force "$SCRATCH/wt-main"
```

The subshell sources `.env` into its own environment without printing it; the worktree's harness then finds the keys already set. Red on the branch and green on main is a regression: read both traces (the README's rule: an assertion reports the property it checks, not the reason it failed), find the edited sentence the trace turns on, fix it in a new commit on this branch, and rerun that one scenario. Red on both is baseline the 2026-09-24 sweep did not catch; record it.

- [ ] **Step 6: Record the run in the suite README**

Append a dated section at the end of `tests/skill-behavior/README.md` (the file's convention for run records), with the date from `date +%F` and the numbers from Step 4 filled in:

```markdown

## Terminal skill text (spec PR 7, <date +%F>)

One sweep after the terminal branches landed in `SKILL.src.md` Setup and
verify rounds, `polish.md`, `new-work.md`, `layout.md`, `typeset.md`,
`doctor.md`, `init.md`, `live.md`, and the finish reviewer and asset
producer. terra ran through the OpenAI SDK; sonnet and gemini ran through
OpenRouter's chat-completions path (the unmerged `skill-behavior-openrouter`
provider), so their cells are not this README's native-SDK cells.

Result: <P> of <T> passed. Every failure was already red on main in the
2026-09-24 sweep (<list per model>)<, or: plus <scenario> on <model>, which
also failed on a detached-main control>. No scenario uses a `terminal`
fixture, so this run shows web and native routing did not regress; it does
not measure terminal routing.
```

Replace every `<...>` with the measured value before committing; the section must contain no placeholder. Keep it free of em dashes.

- [ ] **Step 7: Commit the README record**

```bash
git status --short tests/skill-behavior/   # only README.md; providers.mjs must be clean
git add tests/skill-behavior/README.md
git commit -F - <<'EOF'
Record the skill-behavior sweep after the terminal skill text

One run over the three families against the 2026-09-24 red set, with
the provider path each family used.

AI assistance: drafted with Claude Code.
EOF
```

---

### Task 9: Final review, merge to the fork's main, push

**Files:** none new.

- [ ] **Step 1: Whole-branch review** per subagent-driven-development (final code reviewer on the most capable model), briefed with spec section 7's PR 7 subsection, this plan's decisions 1 to 14, and `docs/STYLE.md`. The reviewer reads every edited sentence in place for voice and for agreement with `terminal.md`, `audit.terminal.md`, and `adapt.terminal.md`. Findings marked Important or Critical get one fix wave and one scoped re-review; a fix to `SKILL.src.md` Setup text reruns only the scenarios its trace could reach (Task 8, Step 5 commands).

- [ ] **Step 2: Gates once more on the final head**

```bash
node --test tests/skill-reference.test.mjs 2>&1 | grep -E "^ℹ (pass|fail)"
bun run build 2>&1 | tail -n 2
IMPECCABLE_BIN="$PWD/target/release/impeccable" bun run test 2>&1 | tail -n 8
git status --short   # only the untracked local files may show; never stage them
```

- [ ] **Step 3: Merge and push** (the fork owner pre-authorized merging each PR to `main` as it lands):

```bash
git checkout main && git pull --ff-only origin main
git merge --no-ff terminal-platform-pr7 -m "$(printf 'Merge terminal-platform-pr7: skill text (spec PR 7)\n\nAI assistance: drafted with Claude Code.')"
git push origin main
git log --oneline -3
```

- [ ] **Step 4: Close out.** Ledger completion line; copy the ledger to the session scratchpad; delete the SDD workspace; update the memory file `terminal-platform-project.md` (PR 7 merged with its commit id; the skill-behavior result and provider paths; terminal builds are code-led; the spec gaps in decisions 1 and 2; `ENGINE_VERSION` still needs a bump at release, and the skill version bump covers PR 7's text); write the journal entry. Update the untracked upstream issue draft `docs/superpowers/research/2026-09-24-upstream-issue-draft.md` with a line for the skill-text PR and its merge commit. Never post it.

---

## Self-review notes

- Spec section 7, PR 7 coverage: terminal branches in `SKILL.src.md` (verify rounds, platform guidance; Task 4), `polish.md` (sizes, size classes, runtime checks; Task 4), `new-work.md` (comp aspect, sizes, detect, reviewer packet; Task 5), `layout.md` and `typeset.md` Terminal bullets (Task 4), `doctor.md` (Task 4), `init.md` (live skip, reference sentence; Task 4), both agents (Task 6); the live guard (Task 3); the Compact row, "At 120 columns and above", and "40 columns" (Task 2); `degraded/finish-reviewer.md` untouched (Global Constraints). The terminal case verifies with the 80x24, 120x40, 40x24 matrix plus `NO_COLOR=1` and a `cat` pipe (Tasks 4, 5, 6), says detect runs on terminal source (Tasks 4, 5), and frames a comp as landscape on an 80x24 grid (Tasks 5, 6).
- Tests: the Compact row and widths (Task 2), the live guard (Task 3), one terminal branch per edited file (Tasks 4 to 6: SKILL.src.md, polish, new-work, layout, typeset, doctor, init, both agents; live.md in Task 3; adapt.terminal.md and terminal.md in Task 2). `context-terminal` re-recorded with a DELTAS entry (Task 2). One skill-behavior run against the baseline (Task 8).
- Delivery gates: `bun run build` (Tasks 4 to 7, 9), `bun run test` (Tasks 7, 9), `test:skill-behavior` (Task 8), `test:plugin-e2e` for the agents (Task 6, also in the default suite).
- Consistency: the five capture names are spelled the same in `new-work.md` and the reviewer; the detect command `detect --json <source directory> --tmux <target> --tmux-sizes 80x24,120x40,40x24` is the same in `polish.md`, `new-work.md`, and `audit.terminal.md:3`; the matrix is `80x24, 120x40, 40x24` everywhere.
- Not in this PR: `visualize.md` (decision 2), reviewer lines 32 and 45 (decision 5), `audit.terminal.md` (its 60 and 120 are right per spec section 2), any crate, any version bump.
