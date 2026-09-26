# Wording Experiment Stage 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Measure whether rewriting the stage-1 skill text into non-violent-communication form (observation, visitor need, request), with every constraint kept, changes instruction following, the tone and thoroughness of critique and audit reports, and the design quality of built pages, against an unchanged arm and an equal-effort paraphrase arm; then hand the owner a report that evaluates the spec's decision rule mechanically.

**Architecture:** Three git worktrees (`original`, `nvc`, `paraphrase`) on the session scratchpad, branched from one recorded `main` commit. Experiment code lives in `tests/wording-experiment/` on a `wording-experiment` branch of the main checkout and drives each arm's own `tests/skill-behavior/harness.mjs` (lanes 2 and 3, in process) or its own `scenarios.test.mjs` (lane 1, as a `node --test` child with a TAP reporter). Every paid call is appended to a JSONL store keyed by (lane, arm, model, item) and priced into a JSONL cost ledger with a $40 hard cap. A blinded Gemini judge scores reports and page pairs; `report.mjs` aggregates into the research report.

**Tech Stack:** Node 24 (`node:test`, `node:util` `parseArgs`), Vercel AI SDK 7 (`generateText` via the harness, `generateObject` for the judge, `MockLanguageModelV3` from `ai/test` for stubs), `@ai-sdk/openai` (direct OpenAI and OpenRouter chat), `zod` 4, Playwright 1.63 Chromium, the engine binary `target/release/impeccable` (`detect --json`, `concept-seed`), git worktrees.

**Spec:** docs/superpowers/specs/2026-09-26-wording-experiment-design.md

## Global Constraints

- Every commit body ends with the line `AI assistance: drafted with Claude Code.` and carries no `Co-Authored-By`, `Claude-Session`, or other agent trailer, even if a harness reminder asks for one. Commit with `git commit -m "<subject>" -m "<body>" -m "AI assistance: drafted with Claude Code."`.
- Stage explicit paths only; never `git add -A` or `git add .`. Never stage `.serena/`, `.claude/CLAUDE.md`, `.gemini/GEMINI.md`, `.agents/AGENTS.md`, `.superpowers/`, `.impeccable/`, `.env`, `docs/superpowers/research/2026-09-24-upstream-issue-draft.md`, or regenerated provider output (`.claude/skills/`, `.cursor/`, `.agents/skills/`, `plugin/`, `dist/`, `build/`).
- `.env` holds API keys: never print, `cat`, or echo it or a variable read from it. Listing names is allowed: `rtk proxy grep -o '^[A-Z_]*=' .env`.
- No em dashes in any doc, skill text, or code comment written here. In skill text also avoid the `validateSkillProse` phrases: `load-bearing`, `highest-leverage`, `biggest unlock`, `reflex defaults`, `collapses into monoculture`, `data-driven`, `delve`, `tapestry`, `in today's`, `gone are the days`, `let's dive in`, `in summary`, `in conclusion`.
- No version bumps (`package.json`, `.claude-plugin/*.json`, `extension/manifest.json`), no changelog entries, no `ENGINE_VERSION` change.
- The experiment is opt-in: nothing it adds runs in the default `bun run test`. Its unit tests run with `node --test tests/wording-experiment/*.test.mjs` (equivalently `node scripts/run-tests.mjs wording-experiment`); see Decision 15.
- Billed steps run one at a time, only in the tasks marked BILLED, only after the Task 13 dry run passes, and stop at the $40 cap. A cap stop (exit code 3) ends the task; report it to the owner and never raise the cap yourself.
- Never run tmux. Never touch `tests/oracle/vectors/calls/` or any oracle golden.
- Arm worktrees live under `/private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/wording-arms/` (not on the repo's SD-card volume), with `node_modules` symlinked from the main checkout. Results live under `.../scratchpad/wording-results/` (dry runs under `.../scratchpad/wording-results-dry/`). A different session may override the scratchpad root with `WORDING_SCRATCH=<dir>`; the arms and results then move with it.
- This machine rewrites shell commands through RTK, and `grep` is aliased to ripgrep. Any command whose output is grepped, counted, diffed, or compared runs as `rtk proxy <cmd>`, one prefix per command in a pipeline. The commands below already do.
- Never merge an arm branch, and never merge `wording-experiment` into `main`. The one merge this plan makes is the Task 1 prerequisite, which the spec requires on `main` before the arms branch.
- Known environment baseline, not fixed here: `tests/release.test.mjs` fails on this machine (git forces signed tags); bun tests can flake on the 5 s default timeout under load (they pass isolated; the gates below pass `--timeout 60000`).
- Test names state outcomes (`a skipped scenario fails the sweep instead of counting as a pass`), never function names.

---

## Decisions fixed while planning

1. **The "shared design laws section" is a line-anchored span.** `skill/SKILL.src.md` has no section by that name; its headings are `## Setup`, `## How to design`, `## Modes`, `## Commands`. The editable text is the preamble after the frontmatter (the intro paragraph and "Core principles", lines 12 to 17 at base) plus the `## How to design` section. `## Setup`, `## Modes`, `## Commands`, the routing list, and the Pin, Hooks, Doctor, and drift paragraphs stay byte-identical, because Setup and routing text is what lane 1 measures and changing it would confound the compliance result with a protocol edit. Cost if wrong: the Modes paragraph (four sentences, almost no prohibitive or judgmental wording) is left for stage 2.
2. **The paraphrase arm follows the spec's own example.** It keeps the prohibitive form ("Do not", "Never", "No") of every prohibitive sentence, replaces judgmental labels ("slop", "lazy", "generic") with neutral descriptors exactly as the example turns "generic, lazy defaults: they are slop" into "stock framework defaults; they read as unconsidered", and adds no observation or visitor-need clause. Both rewritten arms therefore end with zero judgmental-lexicon words, and nvc versus paraphrase isolates framing (prohibition versus observation, need, request). Cost if wrong: a vocabulary-only effect shows up in both-versus-original rather than in nvc-versus-paraphrase; the report's attribution sentence states this.
3. **Frozen spans are byte-identical in every arm:** YAML frontmatter, every Markdown heading line, every table row (lines starting with `|`), every fenced code block, every provider block (`<codex>`, `<gemini>`, and the other `PROVIDER_BLOCK_TAGS` tags, indented or not), and everything in `SKILL.src.md` outside Decision 1's span. The audit skeleton test pins `audit.md`'s headings against the untouched native and terminal variants; critique headings are the report template, so freezing them keeps report structure comparable across arms. Provider blocks are compiled away by the harness (`compileProviderBlocks(content, [])`), so no lane ever sees them; rewriting them would spend effort that nothing measures. Cost if wrong: stage 2 rewrites them.
4. **"The same sentences" is mechanical.** `invariants.mjs` splits the editable text into sentences and classifies each as P (prohibitive), J (judgmental), or N (neutral) with a fixed lexicon plus two context rules (every bullet under craft-floor's `## Refuse` is P; every bullet after audit's `**NEVER**:` line is P). Both rewritten arms must keep every N sentence verbatim and must change every P and J sentence. Cost if wrong: an implementer who wants to smooth a neighboring neutral sentence cannot; the gate names the sentence to restore.
5. **Rewrite gates beyond the spec's four:** nvc keeps at most 25% of the original's prohibitive-lexicon count; paraphrase keeps at least 70%; both have zero judgmental-lexicon words; the rewrite degree (word-level changes from `git diff --no-index --word-diff=porcelain`, divided by the words on both sides) of paraphrase lies within 25% (relative) of nvc's for each file where nvc's degree is at least 0.08, and within 0.02 (absolute) below that. Cost if wrong: an arm is rejected that a human would accept; the implementer adjusts wording, not the gate.
6. **Pinned strings are enforced by running the tests that pin them**, not by parsing their regexes: `node --test tests/skill-reference.test.mjs` (spec gate) plus `bun test --timeout 60000 tests/build.test.js`, which pins `material_fixes` in the finish reviewer and its frontmatter description. The pinned strings inside editable text are also listed in Task 5 so the implementer keeps them verbatim.
7. **The harness gains one additive field.** `runTurn` already returns `usage: result.totalUsage ?? result.usage`, a sum over every provider request in the turn. `gpt-5.6-terra` bills prompts above 272,000 tokens at a higher rate per request, which a sum cannot resolve, so `runTurn` also returns `stepUsages` (one usage per request) and the completed trace file records it. Nothing reads it in the existing suite; its behavior is unchanged. Cost if wrong: none measurable; the field is ignored where unused.
8. **Lane 1 runs the scenario file directly**, as `node --test --test-reporter=tap --test-timeout=240000 tests/skill-behavior/scenarios.test.mjs` in the arm worktree with `IMPECCABLE_SKILL_BEHAVIOR_MODELS=<one model>`, instead of `bun run test:skill-behavior`. Same file, same 240000 ms per-test cap the suite registry sets; one sweep per (arm, model) keeps the cap check between scenarios, avoids `run-tests.mjs`'s 30-minute wall clock killing a three-model sweep, and gives per-scenario resume through `--test-skip-pattern`. The run-tests leak guard is not needed: the scenarios start no live servers. Cost if wrong: none; the scenario file and timeouts are identical.
9. **Keys come from the main checkout.** `run.mjs` imports `tests/skill-behavior/providers.mjs` from the main checkout, whose `loadEnv()` fills `process.env` from the main `.env`; worktree children inherit it (a worktree has no `.env`, and its own `loadEnv()` is then a no-op). `.env` defines `IMPECCABLE_SKILL_BEHAVIOR_MODELS`, and values already in `process.env` win, so every lane-1 child gets it set explicitly. No `.env` is copied or linked into a worktree.
10. **The engine binary is exported, never discovered.** `findEngineBinary()` resolves from the importing file's repo root; a worktree has neither `skill/scripts/bin/` nor `target/release/`, so `ENGINE_BIN` would be null and every scenario would skip. `run.mjs` sets `process.env.IMPECCABLE_BIN=/Volumes/Containers/impeccable/target/release/impeccable` before any dynamic import of an arm harness (it is read at module load) and every lane-1 child inherits it.
11. **The node_modules symlink is excluded, not committed.** `.gitignore`'s `node_modules/` matches directories only, so a `node_modules` symlink would show as untracked and fail the clean-tree check. `arms.mjs` appends `/node_modules` to the repository's shared `info/exclude` (`git rev-parse --git-common-dir`), which also leaves the main checkout unaffected (its `node_modules` is already ignored).
12. **A skipped scenario fails the lane.** A skip means a missing key or binary, not a pass; the sweep throws after it ends and records nothing for the skipped names, so the fixed rerun picks them up.
13. **Provider errors are recognized from the TAP diagnostic.** A failure whose YAML block contains `LLM behavior turn failed before completing` (the harness's own wrapper message) is a provider error: retried once in a second child, then recorded `missing`. Any other failure is a red scenario.
14. **Regression rule.** A candidate is a scenario red in `nvc` or `paraphrase` and green in `original` for the same model; it is rerun once in that arm (item `<name>::rerun`); a regression is red again. `original` is never rerun. The lane runs reruns only for models whose three sweeps are complete.
15. **Unit tests run under `node --test`**, like `tests/skill-behavior-harness.test.mjs`: they spawn node children and use `node:test`. They are registered as a new opt-in suite `wording-experiment` in `scripts/test-suites.mjs`, because `tests/test-suites.test.mjs` (default suite) fails on any unregistered `*.test.mjs`. Every task that adds a test file adds it to that suite's `files` list in the same commit.
16. **Judge model: `google/gemini-3.7-flash`.** OpenRouter lists no current-generation Gemini Pro (`google/gemini-3.1-pro-preview` is an older generation and a preview); `google/gemini-3.8-flash` is the same tier at the same price, not a stronger one. The spec names 3.7-flash as the default, and Gemini generates nothing in lanes 2 and 3.
17. **Pair scoring.** Each judged order scores 1 (preferred), 0.5 (tie), or 0; a pair's score is the mean of its two orders. Win rate is the mean pair score; its 95% interval is the Wilson score interval on (sum of pair scores, pairs judged). A pair is lost only when both orders prefer the other page. "Better design win rate" means the Wilson lower bound exceeds 0.5 (17 or more of 24); an interval that contains 0.5 is reported as "no detectable difference".
18. **"Loses no more than 16 of 24 pairs"** is `lost <= floor(16 * judged / 24)` when pairs are missing.
19. **Fewer labels and fewer meta-comments** mean a strict decrease in judge-confirmed counts summed over the arm's 20 reports.
20. **Defect stop:** the mean paired difference (nvc minus original) over the 20 (page, command, model) cells is at most -1.0. Its 95% interval uses Student's t with n - 1 degrees of freedom.
21. **A build with no `index.html`** loses its pair in both orders without a judge call; if both sides have none, the pair is a tie. These are stored as `auto` verdicts and counted in the report.
22. **Builds are seeded identically across arms.** `IMPECCABLE_CONCEPT_SEED` and `IMPECCABLE_PALETTE_SEED` are fixed per brief, the concept roll uses the local catalog `tests/fixtures/concept-catalog`, and telemetry is off (`DO_NOT_TRACK=1`, `IMPECCABLE_NO_TELEMETRY=1`). Dice variance would otherwise swamp 24 pairs. Task 9 tests that a seed reproduces its roll.
23. **Budget is gated by pilots, not trusted.** The spec's compliance row ("3 arms x 78 scenarios x 3 models") double counts: 78 already is 26 scenarios times 3 models, so lane 1 is 234 scenario runs plus reruns. Lanes 2 and 3 run full multi-step turns whose context grows every step, and prompt caching through OpenRouter's chat endpoint cannot be assumed, so the $7 and $16 estimates are unverified. Tasks 14 and 15 measure pilots and project; when a projection exceeds the remaining budget the executor stops and brings the numbers to the owner. The executor never cuts scope on its own.
24. **The cap binds before a call starts.** No billed call or scenario starts once `spent + reserve > $40`, where the reserve is `max($1.50, largest single charge so far)`. In lane 1 the check runs after every scenario result and kills the sweep's process group.
25. **Unpriced calls are counted, never priced at zero.** A turn that throws has no usage; the ledger records it as unpriced and the report states the count, so the cost figure is a floor.
26. **`run.mjs` is written last among the code tasks (Task 12)** so it never dispatches to a module that does not exist yet; `invariants.mjs` and `arms.mjs` carry their own small CLIs for the tasks that need them earlier.
27. **Branching.** Task 1 is committed on `wording-openrouter-prereq` and merged `--no-ff` into `main` (the spec requires it on `main` before the arms). Tasks 2 to 16 commit on `wording-experiment`, cut from that merged `main`. The arms branch from the same merged `main` commit, recorded in `wording-arms/base.json`. Rewrites commit on the arm branches inside their worktrees.
28. **Lane 2 stores what the user sees.** Critique ends on `ask_user_question` (its Hard Invariants put the question last), so the turn stops at the first question and the stored `userText` is every step's text plus the asked questions, not `result.text`. No subagent tool exists in the harness, so every critique carries the degraded banner in every arm; that is constant across arms.
29. **Every lane-2 page ships a `PRODUCT.md`** so the init gate never enters lane 2.
30. **Chromium.** `playwright` 1.63 expects Chromium revision 1243; the local cache holds 1208. Task 9 checks with a real launch and installs Chromium (free) when it fails.
31. **Screenshots** are full page, clipped at 3000 px tall, at viewports 1440x900 (desktop) and 390x844 (mobile).
32. **The contact sheet** is a JPEG that Playwright renders from an HTML grid of the desktop screenshots, captioned with the unblinded arms and the pair verdicts. Only the JPEG is committed; screenshots stay in the scratchpad.
33. **`defects.json` records which engine rule, if any, flags each planted defect** (measured 2026-09-26 with `target/release/impeccable detect --json`), so recall splits into detector-assisted and judgment-only. A wording effect on the model's own eye shows in the judgment-only half.
34. **Store and ledger survive crashes.** Both are append-only JSONL; a truncated last line is skipped with a stderr warning and the file is newline-terminated before the next append, so the item runs again. A cap stop never writes a `missing` record, so resume retries it.
35. **A systematic failure never becomes a permanent `missing` record.** Resume skips every recorded key, so a wrong model slug or a rejected request shape recorded as `missing` could never be retried. When the first item of a (lane, arm, model) group fails on the first try and on the retry, the lane throws and records nothing (`assertNotSystematic`); later failures in a group that already has a success are recorded `missing`, as the spec requires. Lane 1 applies the same rule to its retry phase. Tasks 14 and 15 also pilot every lane, the judge included, before the full runs.
36. **Critique stops at its closing question, not its first.** A model may ask something before writing the report (a scope check); stopping there would store a report-less turn that scores zero defects and looks like lower recall. The lane stops only when a question arrives after a report heading (`Priority Issues`, `Design Health Score`, or `Audit Health Score`) is in the step texts; earlier questions are answered by the harness's simulated user. Each record keeps `questions` and `reportSeen`, and the report prints them per arm.
37. **Lanes 2 and 3 switch off run-to-run noise from the engine:** `IMPECCABLE_NO_UPDATE_CHECK=1` (an `UPDATE_AVAILABLE` directive tells the model to ask the user about updating) and `IMPECCABLE_NO_STALENESS_CHECK=1`, the same in every arm. Lane 1 is left exactly as the suite runs it, because some scenarios test the update path themselves.
38. **`detect` runs with `--platform web`** in the fixture test and in the builds lane, so a `PRODUCT.md` above the target can never switch its rule set.

## File structure

| Path | Action | Responsibility |
|---|---|---|
| `tests/skill-behavior/providers.mjs` | Modify (Task 1) | OpenRouter provider (exact diff from `skill-behavior-openrouter`) |
| `tests/skill-behavior/harness.mjs` | Modify (Task 1) | `stepUsages` in `runTurn`'s return and the completed trace |
| `tests/skill-behavior-harness.test.mjs` | Modify (Task 1) | OpenRouter routing and per-request usage tests |
| `scripts/test-suites.mjs` | Modify (Task 2, extended by 3, 4, 7, 8, 9, 10, 11, 12) | `wording-experiment` opt-in suite |
| `tests/wording-experiment/config.mjs` | Create (Task 2) | paths, arms, stage-1 files, lane models, judge, cap |
| `tests/wording-experiment/store.mjs` | Create (Task 2) | append-only JSONL store, one-retry helper |
| `tests/wording-experiment/cost.mjs` | Create (Task 2) | price table, per-request pricing, ledger, `CapExceeded` |
| `tests/wording-experiment/stub-model.mjs` | Create (Task 2) | stub models for tone, build, and judge dry runs |
| `tests/wording-experiment/store.test.mjs`, `cost.test.mjs` | Create (Task 2) | unit tests |
| `tests/wording-experiment/invariants.mjs` | Create (Task 3) | token extraction, frozen spans, sentence inventory, gates, effort comparison, CLI |
| `tests/wording-experiment/invariants.test.mjs` | Create (Task 3) | unit tests on synthetic files |
| `tests/wording-experiment/arms.mjs` | Create (Task 4) | create and tear down arm branches and worktrees, CLI |
| `tests/wording-experiment/arms.test.mjs` | Create (Task 4) | temp-repo tests |
| `skill/SKILL.src.md`, `skill/reference/{craft-floor,critique,audit}.md`, `skill/agents/impeccable-finish-reviewer.md` | Modify on `wording-arm-nvc` (Task 5) and `wording-arm-paraphrase` (Task 6) only | the rewrites |
| `tests/wording-experiment/lanes/compliance.mjs` | Create (Task 7) | TAP parsing, sweeps, retries, reruns, trace pricing |
| `tests/wording-experiment/fixtures/stub-compliance.fixture.mjs` | Create (Task 7) | stub scenario file for dry runs (not a `*.test.mjs`) |
| `tests/wording-experiment/compliance.test.mjs` | Create (Task 7) | unit tests |
| `tests/wording-experiment/fixtures/pages/{landing,dashboard,form,docs,portfolio}.html` | Create (Task 8) | lane 2 pages |
| `tests/wording-experiment/fixtures/defects.json` | Create (Task 8) | planted-defect ground truth and page PRODUCT.md text |
| `tests/wording-experiment/lanes/tone.mjs` | Create (Task 8) | lane 2 driver |
| `tests/wording-experiment/fixtures.test.mjs`, `tone.test.mjs` | Create (Task 8, extended in 9) | fixture and lane tests |
| `tests/wording-experiment/fixtures/briefs.json` | Create (Task 9) | twelve briefs with PRODUCT.md and seeds |
| `tests/wording-experiment/lanes/builds.mjs` | Create (Task 9) | lane 3 driver, screenshots, detect |
| `tests/wording-experiment/builds.test.mjs` | Create (Task 9) | lane tests |
| `tests/wording-experiment/judge.mjs`, `judge.test.mjs` | Create (Task 10) | blinding, prompts, schemas, judge lane |
| `tests/wording-experiment/report.mjs`, `report.test.mjs` | Create (Task 11) | statistics, decision rule, report, contact sheet |
| `tests/wording-experiment/run.mjs`, `run.test.mjs` | Create (Task 12) | CLI entry point |
| `docs/superpowers/research/<date>-wording-experiment/{report.md,results.jsonl,contact-sheet.jpg}` | Create (Task 16) | committed outputs |

---

## Task 1: Prerequisite on `main`: OpenRouter provider and per-request usage

**Files:**
- Modify: `tests/skill-behavior/providers.mjs` (apply the exact diff `git diff main skill-behavior-openrouter -- tests/skill-behavior/providers.mjs`; never check out or merge that branch, its tree predates months of work)
- Modify: `tests/skill-behavior/harness.mjs` (two additive lines)
- Test: `tests/skill-behavior-harness.test.mjs`

**Interfaces:**
- Produces: `detectProvider(modelId)` returns `'openrouter'` for any id containing `/`; `getModel('<vendor>/<model>')` returns `createOpenAI({ baseURL: 'https://openrouter.ai/api/v1', apiKey: process.env.OPENROUTER_API_KEY, name: 'openrouter' }).chat(modelId)`; `PROVIDERS.openrouter = { envKey: 'OPENROUTER_API_KEY', label: 'OpenRouter' }`.
- Produces: `runTurn(...)` resolves to its existing object plus `stepUsages: LanguageModelUsage[]` (one entry per provider request, in order); the completed trace file written under `IMPECCABLE_SKILL_BEHAVIOR_TRACE_DIR` gains the same `stepUsages` field.
- Consumes: nothing new.

- [ ] **Step 1: Branch from `main`**

```bash
cd /Volumes/Containers/impeccable
git checkout main
git checkout -b wording-openrouter-prereq
```

- [ ] **Step 2: Write the two failing tests**

Add to `tests/skill-behavior-harness.test.mjs`. Change the import block's line 6 and add a providers import after it:

```js
import { prepareWorkspace, cleanupWorkspace, makeTools, runTurn, fileLoaded, SKILL_BODY } from './skill-behavior/harness.mjs';
import { detectProvider, getModel, getProviderOptions, PROVIDERS } from './skill-behavior/providers.mjs';
```

Append at the end of the file:

```js
it('OpenRouter slugs route through OpenRouter, never a direct provider', () => {
  assert.equal(detectProvider('anthropic/claude-sonnet-5'), 'openrouter');
  assert.equal(detectProvider('google/gemini-3.7-flash'), 'openrouter');
  assert.equal(detectProvider('claude-sonnet-5'), 'anthropic');
  assert.equal(detectProvider('gpt-5.6-terra'), 'openai');
  const model = getModel('anthropic/claude-sonnet-5');
  assert.equal(model.modelId, 'anthropic/claude-sonnet-5');
  assert.match(model.provider, /^openrouter/);
  assert.equal(getProviderOptions('anthropic/claude-sonnet-5'), undefined);
  assert.equal(PROVIDERS.openrouter.envKey, 'OPENROUTER_API_KEY');
});

it('every provider request in a turn reports its own token usage', async () => {
  const workspace = prepareWorkspace();
  try {
    let call = 0;
    const model = new MockLanguageModelV3({ modelId: 'claude-sonnet-5', doGenerate: async () => {
      call++;
      if (call === 1) {
        return {
          content: [{ type: 'tool-call', toolCallId: 'list', toolName: 'list', input: '{}' }],
          finishReason: { unified: 'tool-calls', raw: 'tool-calls' },
          usage: { inputTokens: { total: 7 }, outputTokens: { total: 3 } }, warnings: [],
        };
      }
      return {
        content: [{ type: 'text', text: 'done' }],
        finishReason: { unified: 'stop', raw: 'stop' },
        usage: { inputTokens: { total: 11 }, outputTokens: { total: 5 } }, warnings: [],
      };
    } });
    const result = await runTurn({ workspace, model, userPrompt: 'List the workspace, then stop.', maxSteps: 4 });
    assert.deepEqual(result.stepUsages.map((usage) => [usage.inputTokens, usage.outputTokens]), [[7, 3], [11, 5]]);
    assert.equal(result.usage.inputTokens, 18);
  } finally {
    cleanupWorkspace(workspace);
  }
});
```

- [ ] **Step 3: Run them and watch them fail**

Run: `node --test --test-name-pattern="OpenRouter slugs|its own token usage" tests/skill-behavior-harness.test.mjs`
Expected: 2 failing. The first with `Unsupported model id: "anthropic/claude-sonnet-5"`; the second with `TypeError: Cannot read properties of undefined (reading 'map')`.

- [ ] **Step 4: Apply the provider diff exactly**

```bash
cd /Volumes/Containers/impeccable
rtk proxy git diff main skill-behavior-openrouter -- tests/skill-behavior/providers.mjs | git apply
rtk proxy git diff --stat -- tests/skill-behavior/providers.mjs
```

Expected: `1 file changed, 15 insertions(+), 1 deletion(-)`. The resulting hunks are: the import becomes `import { createOpenAI, openai } from '@ai-sdk/openai';`; `PROVIDERS` gains `openrouter: { envKey: 'OPENROUTER_API_KEY', label: 'OpenRouter' },`; `detectProvider` starts with the comment `// OpenRouter slugs carry a vendor prefix (\`anthropic/claude-sonnet-5\`);` / `// no direct provider id contains a slash.` and `if (modelId.includes('/')) return 'openrouter';`; `getModel` gains the `openrouter` branch returning `openrouter.chat(modelId)` before the final `throw`.

- [ ] **Step 5: Add `stepUsages` to the harness**

In `tests/skill-behavior/harness.mjs`, replace:

```js
  saveTrace({ status: 'completed', responseMessages,
    outcome, finishReason: result.finishReason, steps: result.steps.length, usage: result.totalUsage ?? result.usage });
```

with:

```js
  const stepUsages = result.steps.map((step) => step.usage);
  saveTrace({ status: 'completed', responseMessages,
    outcome, finishReason: result.finishReason, steps: result.steps.length, usage: result.totalUsage ?? result.usage, stepUsages });
```

and replace:

```js
    usage: result.totalUsage ?? result.usage,
    responseMessages,
  };
```

with:

```js
    usage: result.totalUsage ?? result.usage,
    stepUsages,
    responseMessages,
  };
```

- [ ] **Step 6: Run the harness tests green**

Run: `node --test tests/skill-behavior-harness.test.mjs`
Expected: every test passes, including the two new ones (`# fail 0`).

- [ ] **Step 7: Confirm the existing suites behave identically**

Run: `node --test tests/test-suites.test.mjs && node scripts/run-tests.mjs core`
Expected: `tests/test-suites.test.mjs` passes. `core` passes apart from the known baseline (`tests/release.test.mjs` fails on signed tags); a bun file that times out at 5 s is rerun alone with `bun test --timeout 60000 <file>` and must pass. Any other failure: stop and diagnose before committing.

- [ ] **Step 8: Commit and merge into `main`**

```bash
cd /Volumes/Containers/impeccable
git add tests/skill-behavior/providers.mjs tests/skill-behavior/harness.mjs tests/skill-behavior-harness.test.mjs
git commit -m "skill-behavior: OpenRouter provider and per-request usage" -m "Brings the OpenRouter provider from skill-behavior-openrouter onto main (vendor-prefixed slugs route through OpenRouter's chat endpoint) and has runTurn report each provider request's usage, so a per-request price rule can be applied. Prerequisite for the wording experiment; existing scenarios are unchanged." -m "AI assistance: drafted with Claude Code."
git checkout main
git merge --no-ff wording-openrouter-prereq -m "Merge wording-openrouter-prereq: OpenRouter provider and per-request usage" -m "AI assistance: drafted with Claude Code."
rtk proxy git rev-parse main
git checkout -b wording-experiment
```

Expected: the merge succeeds without conflicts; record the printed `main` hash in your checkpoint (Task 4 uses it as the arm base).

**Checkpoint:** `main` carries the OpenRouter provider and `stepUsages`; the working branch is `wording-experiment`; nothing else changed.

---

## Task 2: Experiment core: suite registration, config, store, cost, stubs

**Files:**
- Modify: `scripts/test-suites.mjs`
- Create: `tests/wording-experiment/config.mjs`, `tests/wording-experiment/store.mjs`, `tests/wording-experiment/cost.mjs`, `tests/wording-experiment/stub-model.mjs`
- Test: `tests/wording-experiment/store.test.mjs`, `tests/wording-experiment/cost.test.mjs`

**Interfaces:**
- Produces (`config.mjs`): `EXPERIMENT_DIR`, `REPO_ROOT`, `SCRATCH`, `ARMS_ROOT`, `RESULTS_ROOT`, `DRY_RESULTS_ROOT`, `ARMS = ['original','nvc','paraphrase']`, `armBranch(arm)`, `armDir(arm, root?)`, `STAGE1_FILES` (5 paths), `LANE_MODELS = { compliance, tone, builds }`, `JUDGE_MODEL`, `COST_CAP_USD = 40`, `ENGINE_BIN`, `CATALOG_DIR`, `COMPARISONS` (`[['nvc','paraphrase'],['nvc','original'],['paraphrase','original']]`), `modelSlug(modelId)`.
- Produces (`store.mjs`): `recordKey(record) -> string`, `openStore(file) -> { file, malformed, has(record), append(record), all(filter) }`, `withOneRetry(attempt, { isFatal }) -> { ok, value?, retried?, error?, firstError? }`, `assertNotSystematic(store, { lane, arm, model }, error)` (throws when that group has no `ok` record yet). A record is `{ lane, arm, model, item, status: 'ok'|'missing', data, ts }`.
- Produces (`cost.mjs`): `PRICES`, `PRICES_FETCHED`, `MIN_RESERVE_USD = 1.5`, `class CapExceeded`, `class UsageMissing`, `requestCost(priceId, usage) -> usd`, `turnCost(priceId, usages) -> usd`, `openLedger(file, cap) -> { file, cap, spent(), unpriced(), reserve(), assertCanSpend(next), charge({ lane, arm, model, item, priceId, usages }) -> usd, chargeUnpriced({ lane, arm, model, item, reason }) }`, `chargeTurn(ledger, key, priceId, usages) -> usd | null`.
- Produces (`stub-model.mjs`): `stubModel(kind: 'tone'|'build'|'judge') -> MockLanguageModelV3` (modelId `'stub'`), `STUB_REPORT`, `STUB_BUILD_HTML`, `stubVerdict(promptText)`.

- [ ] **Step 1: Register the opt-in suite**

In `scripts/test-suites.mjs`, replace:

```js
  'skill-workflow',
  'live-svelte-adapter-deepseek',
];
```

with:

```js
  'skill-workflow',
  'live-svelte-adapter-deepseek',
  'wording-experiment',
];
```

and replace the end of the `SUITES` object:

```js
        files: ['tests/live-svelte-adapter-deepseek.test.mjs'],
      },
    ],
  },
};
```

with:

```js
        files: ['tests/live-svelte-adapter-deepseek.test.mjs'],
      },
    ],
  },
  'wording-experiment': {
    description: 'Wording experiment unit tests: stub models, no network, no billed calls.',
    optIn: true,
    triggers: [
      ...COMMON_INFRA_PATTERNS,
      /^tests\/wording-experiment\//,
    ],
    commands: [{
      runner: 'node',
      timeoutMs: 600000,
      files: [
        'tests/wording-experiment/store.test.mjs',
        'tests/wording-experiment/cost.test.mjs',
      ],
    }],
  },
};
```

Later tasks append their test files to this `files` list in the same commit that adds them.

- [ ] **Step 2: Write `config.mjs`**

```js
// Paths, arms, models, and limits shared by every wording-experiment module.
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const EXPERIMENT_DIR = path.dirname(fileURLToPath(import.meta.url));
export const REPO_ROOT = path.resolve(EXPERIMENT_DIR, '..', '..');
export const SCRATCH = process.env.WORDING_SCRATCH
  || '/private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad';
export const ARMS_ROOT = path.join(SCRATCH, 'wording-arms');
export const RESULTS_ROOT = path.join(SCRATCH, 'wording-results');
export const DRY_RESULTS_ROOT = path.join(SCRATCH, 'wording-results-dry');

export const ARMS = ['original', 'nvc', 'paraphrase'];
export const armBranch = (arm) => `wording-arm-${arm}`;
export const armDir = (arm, root = ARMS_ROOT) => path.join(root, arm);

export const STAGE1_FILES = [
  'skill/SKILL.src.md',
  'skill/reference/craft-floor.md',
  'skill/reference/critique.md',
  'skill/reference/audit.md',
  'skill/agents/impeccable-finish-reviewer.md',
];

export const LANE_MODELS = {
  compliance: ['gpt-5.6-terra', 'anthropic/claude-sonnet-5', 'google/gemini-3.7-flash'],
  tone: ['anthropic/claude-sonnet-5', 'gpt-5.6-terra'],
  builds: ['anthropic/claude-sonnet-5', 'gpt-5.6-terra'],
};
export const JUDGE_MODEL = 'google/gemini-3.7-flash';
export const COST_CAP_USD = 40;

// Worktrees carry no engine binary; every lane uses the main checkout's build.
export const ENGINE_BIN = process.env.IMPECCABLE_BIN || path.join(REPO_ROOT, 'target', 'release', 'impeccable');
export const CATALOG_DIR = path.join(REPO_ROOT, 'tests', 'fixtures', 'concept-catalog');

export const COMPARISONS = [['nvc', 'paraphrase'], ['nvc', 'original'], ['paraphrase', 'original']];
export const modelSlug = (modelId) => modelId.replace(/[^a-z0-9.-]+/gi, '_');
```

- [ ] **Step 3: Write the failing store and cost tests**

`tests/wording-experiment/store.test.mjs`:

```js
import { it } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { assertNotSystematic, openStore, recordKey, withOneRetry } from './store.mjs';

const tempFile = () => path.join(fs.mkdtempSync(path.join(os.tmpdir(), 'wording-store-')), 'store.jsonl');
const key = { lane: 'tone', arm: 'nvc', model: 'gpt-5.6-terra', item: 'landing/critique' };

it('a rerun skips work already recorded', () => {
  const file = tempFile();
  openStore(file).append({ ...key, status: 'ok', data: { userText: 'report' } });
  const reopened = openStore(file);
  assert.equal(reopened.has(key), true);
  assert.equal(reopened.has({ ...key, arm: 'paraphrase' }), false);
  assert.equal(reopened.all({ lane: 'tone', arm: 'nvc' })[0].data.userText, 'report');
});

it('a line cut short by a crash is dropped and later appends stay readable', () => {
  const file = tempFile();
  openStore(file).append({ ...key, status: 'ok', data: {} });
  fs.appendFileSync(file, '{"lane":"tone","arm":"nvc","mod');
  const store = openStore(file);
  assert.equal(store.malformed, 1);
  store.append({ ...key, item: 'landing/audit', status: 'ok', data: {} });
  const again = openStore(file);
  assert.equal(again.all().length, 2);
  assert.equal(again.malformed, 1);
});

it('the same item is never written twice', () => {
  const store = openStore(tempFile());
  store.append({ ...key, status: 'ok', data: {} });
  assert.throws(() => store.append({ ...key, status: 'missing', data: {} }), /already holds/);
});

it('a key with an empty field is refused', () => {
  assert.throws(() => recordKey({ ...key, model: '' }), /model/);
});

it('a provider error is retried once, then reported as missing', async () => {
  let calls = 0;
  const flaky = await withOneRetry(async () => {
    calls++;
    if (calls === 1) throw new Error('socket closed');
    return 'ok';
  });
  assert.deepEqual([flaky.ok, flaky.value, flaky.retried], [true, 'ok', true]);
  const dead = await withOneRetry(async () => { throw new Error('502 bad gateway'); });
  assert.equal(dead.ok, false);
  assert.match(dead.error, /502/);
  assert.match(dead.firstError, /502/);
});

it('a failure before any success stops the lane instead of becoming a permanent missing record', () => {
  const store = openStore(tempFile());
  assert.throws(() => assertNotSystematic(store, key, 'bad model id'), /nothing was recorded/);
  store.append({ ...key, item: 'landing/audit', status: 'ok', data: {} });
  assert.doesNotThrow(() => assertNotSystematic(store, key, 'socket closed'));
});

it('a cost-cap stop is never retried', async () => {
  let calls = 0;
  const cap = Object.assign(new Error('cap'), { name: 'CapExceeded' });
  await assert.rejects(
    withOneRetry(async () => { calls++; throw cap; }, { isFatal: (error) => error.name === 'CapExceeded' }),
    /cap/,
  );
  assert.equal(calls, 1);
});
```

`tests/wording-experiment/cost.test.mjs`:

```js
import { it } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { CapExceeded, PRICES, chargeTurn, openLedger, requestCost, turnCost } from './cost.mjs';
import { JUDGE_MODEL, LANE_MODELS } from './config.mjs';

const usage = (input, output, cacheRead = 0) => ({
  inputTokens: input,
  outputTokens: output,
  inputTokenDetails: { noCacheTokens: input - cacheRead, cacheReadTokens: cacheRead, cacheWriteTokens: 0 },
});
const ledgerFile = () => path.join(fs.mkdtempSync(path.join(os.tmpdir(), 'wording-cost-')), 'cost.jsonl');
const close = (actual, expected) => assert.ok(Math.abs(actual - expected) < 1e-9, `${actual} != ${expected}`);
const key = { lane: 'builds', arm: 'nvc', model: 'stub', item: 'x' };

it('a turn is priced request by request from the published table, cache reads at their own rate', () => {
  close(requestCost('anthropic/claude-sonnet-5', usage(1_000_000, 50_000, 100_000)), 1.8 + 0.02 + 0.5);
  close(turnCost('google/gemini-3.7-flash', [usage(100_000, 10_000), usage(200_000, 20_000)]),
    (300_000 * 0.75 + 30_000 * 3.75) / 1e6);
});

it('a terra request above 272k prompt tokens bills at the long-context rate', () => {
  close(requestCost('gpt-5.6-terra', usage(272_000, 0)), (272_000 * 2) / 1e6);
  close(requestCost('gpt-5.6-terra', usage(300_000, 1_000)), (300_000 * 4 + 1_000 * 18) / 1e6);
});

it('an unknown model is refused rather than priced at zero', () => {
  assert.throws(() => requestCost('openai/gpt-7', usage(1, 1)), /no price/);
});

it('missing usage is an error, never a free call', () => {
  assert.throws(() => requestCost('stub', { inputTokens: undefined, outputTokens: 3 }), { name: 'UsageMissing' });
  assert.throws(() => turnCost('stub', []), { name: 'UsageMissing' });
});

it('every lane model and the judge have a price', () => {
  for (const model of new Set([...Object.values(LANE_MODELS).flat(), JUDGE_MODEL])) assert.ok(PRICES[model], model);
});

it('the running total survives a restart and unpriced calls are counted', () => {
  const file = ledgerFile();
  const ledger = openLedger(file, 40);
  ledger.charge({ ...key, item: 'a', priceId: 'stub', usages: [usage(1_000_000, 0)] });
  assert.equal(chargeTurn(ledger, { ...key, item: 'b' }, 'stub', [{}]), null);
  const reopened = openLedger(file, 40);
  close(reopened.spent(), 1);
  assert.equal(reopened.unpriced(), 1);
});

it('no call starts once the next one could cross the cap', () => {
  const ledger = openLedger(ledgerFile(), 40);
  for (let i = 0; i < 64; i++) ledger.charge({ ...key, item: `a${i}`, priceId: 'stub', usages: [usage(600_000, 0)] });
  ledger.assertCanSpend('call 65');
  ledger.charge({ ...key, item: 'a64', priceId: 'stub', usages: [usage(600_000, 0)] });
  assert.throws(() => ledger.assertCanSpend('call 66'),
    (error) => error instanceof CapExceeded && error.name === 'CapExceeded' && /cap \$40/.test(error.message));
});

it('one large turn raises the reserve for the next call', () => {
  const ledger = openLedger(ledgerFile(), 40);
  ledger.charge({ ...key, item: 'big', priceId: 'stub', usages: [usage(5_000_000, 0)] });
  for (let i = 0; i < 26; i++) ledger.charge({ ...key, item: `s${i}`, priceId: 'stub', usages: [usage(1_000_000, 0)] });
  ledger.assertCanSpend('at 31 spent with a 5 dollar reserve');
  ledger.charge({ ...key, item: 'last', priceId: 'stub', usages: [usage(5_000_000, 0)] });
  assert.throws(() => ledger.assertCanSpend('at 36 spent'), { name: 'CapExceeded' });
});
```

- [ ] **Step 4: Run them and watch them fail**

Run: `node --test tests/wording-experiment/store.test.mjs tests/wording-experiment/cost.test.mjs`
Expected: both files fail at import with `ERR_MODULE_NOT_FOUND` for `./store.mjs` and `./cost.mjs`.

- [ ] **Step 5: Write `store.mjs`**

```js
// Append-only JSONL store keyed by (lane, arm, model, item). A key already present is never paid for again.
import fs from 'node:fs';
import path from 'node:path';

const KEY_FIELDS = ['lane', 'arm', 'model', 'item'];

export function recordKey(record) {
  for (const field of KEY_FIELDS) {
    if (typeof record?.[field] !== 'string' || !record[field]) {
      throw new Error(`store key field "${field}" must be a non-empty string`);
    }
  }
  return JSON.stringify(KEY_FIELDS.map((field) => record[field]));
}

export function openStore(file) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  const records = [];
  const keys = new Set();
  let malformed = 0;
  if (fs.existsSync(file)) {
    const text = fs.readFileSync(file, 'utf8');
    for (const line of text.split('\n')) {
      if (!line.trim()) continue;
      let record;
      try {
        record = JSON.parse(line);
        recordKey(record);
      } catch {
        malformed++;
        continue;
      }
      records.push(record);
      keys.add(recordKey(record));
    }
    // A crash can leave a partial last line; terminate it so the next append starts clean.
    if (text && !text.endsWith('\n')) fs.appendFileSync(file, '\n');
  }
  if (malformed) process.stderr.write(`store ${file}: skipped ${malformed} malformed line(s); those items run again\n`);
  return {
    file,
    malformed,
    has(record) {
      return keys.has(recordKey(record));
    },
    append(record) {
      const key = recordKey(record);
      if (keys.has(key)) throw new Error(`store already holds ${key}`);
      const full = { ...record, ts: new Date().toISOString() };
      fs.appendFileSync(file, `${JSON.stringify(full)}\n`);
      records.push(full);
      keys.add(key);
      return full;
    },
    all(filter = {}) {
      return records.filter((record) => Object.entries(filter).every(([field, value]) => record[field] === value));
    },
  };
}

// Decision 35: a lane's first item failing twice is usually systematic (key, model id, request shape). Stop
// instead of recording it missing, because a missing record is never retried.
export function assertNotSystematic(store, { lane, arm, model }, error) {
  if (!store.all({ lane, arm, model, status: 'ok' }).length) {
    throw new Error(`${lane} ${arm} ${model}: the first item failed on the first try and on the retry (${error}); nothing was recorded. A failure this early is usually systematic: fix it and rerun.`);
  }
}

// The spec's provider-error rule: one retry, then the caller records the item as missing.
export async function withOneRetry(attempt, { isFatal = () => false } = {}) {
  let firstError;
  try {
    return { ok: true, value: await attempt(1), retried: false };
  } catch (error) {
    if (isFatal(error)) throw error;
    firstError = String(error?.message ?? error);
  }
  try {
    return { ok: true, value: await attempt(2), retried: true, firstError };
  } catch (error) {
    if (isFatal(error)) throw error;
    return { ok: false, error: String(error?.message ?? error), firstError };
  }
}
```

- [ ] **Step 6: Write `cost.mjs`**

```js
// Per-request pricing and the cost ledger. The cap binds before a call starts, never after.
import fs from 'node:fs';
import path from 'node:path';

// USD per million tokens, from https://openrouter.ai/api/v1/models, fetched 2026-09-26.
// gpt-5.6-terra is called on the direct OpenAI API; OpenRouter lists openai/gpt-5.6-terra at the
// provider's rates, including the higher rate for a request whose prompt exceeds 272,000 tokens.
export const PRICES_FETCHED = '2026-09-26';
export const PRICES = {
  'gpt-5.6-terra': {
    input: 2, output: 12, cacheRead: 0.2, cacheWrite: 2.5,
    longPromptTokens: 272_000,
    long: { input: 4, output: 18, cacheRead: 0.4, cacheWrite: 5 },
  },
  'anthropic/claude-sonnet-5': { input: 2, output: 10, cacheRead: 0.2, cacheWrite: 2.5 },
  'google/gemini-3.7-flash': { input: 0.75, output: 3.75, cacheRead: 0.075, cacheWrite: 0.0417 },
  // Dry runs only: a nonzero price so the ledger and the cap logic are exercised.
  stub: { input: 1, output: 1, cacheRead: 0, cacheWrite: 0 },
};
export const MIN_RESERVE_USD = 1.5;

export class CapExceeded extends Error {
  constructor(spent, reserve, cap, next) {
    super(`cost cap reached: $${spent.toFixed(2)} spent, $${reserve.toFixed(2)} reserved for the next call, cap $${cap}; stopped before ${next}`);
    this.name = 'CapExceeded';
  }
}

export class UsageMissing extends Error {
  constructor(detail) {
    super(`usage missing: ${detail}`);
    this.name = 'UsageMissing';
  }
}

export function requestCost(priceId, usage) {
  const price = PRICES[priceId];
  if (!price) throw new Error(`no price for model "${priceId}"; add it to PRICES with its source and date`);
  const input = usage?.inputTokens;
  const output = usage?.outputTokens;
  if (!Number.isFinite(input) || !Number.isFinite(output)) {
    throw new UsageMissing(`a ${priceId} request reported no token counts`);
  }
  const details = usage.inputTokenDetails ?? {};
  const cacheRead = details.cacheReadTokens ?? 0;
  const cacheWrite = details.cacheWriteTokens ?? 0;
  const noCache = details.noCacheTokens ?? Math.max(0, input - cacheRead - cacheWrite);
  const rates = price.longPromptTokens && input > price.longPromptTokens ? price.long : price;
  return (noCache * rates.input + cacheRead * rates.cacheRead + cacheWrite * rates.cacheWrite + output * rates.output) / 1e6;
}

export function turnCost(priceId, usages) {
  if (!Array.isArray(usages) || usages.length === 0) throw new UsageMissing(`a ${priceId} turn has no per-request usage`);
  return usages.reduce((sum, usage) => sum + requestCost(priceId, usage), 0);
}

export function openLedger(file, cap) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  let spent = 0;
  let unpriced = 0;
  let largest = 0;
  if (fs.existsSync(file)) {
    const text = fs.readFileSync(file, 'utf8');
    for (const line of text.split('\n')) {
      if (!line.trim()) continue;
      let entry;
      try {
        entry = JSON.parse(line);
      } catch {
        process.stderr.write(`ledger ${file}: skipped a malformed line\n`);
        continue;
      }
      if (entry.unpriced) unpriced++;
      else {
        spent += entry.usd;
        largest = Math.max(largest, entry.usd);
      }
    }
    if (text && !text.endsWith('\n')) fs.appendFileSync(file, '\n');
  }
  const write = (entry) => fs.appendFileSync(file, `${JSON.stringify({ ...entry, ts: new Date().toISOString() })}\n`);
  const reserve = () => Math.max(MIN_RESERVE_USD, largest);
  return {
    file,
    cap,
    spent: () => spent,
    unpriced: () => unpriced,
    reserve,
    assertCanSpend(next) {
      if (spent + reserve() > cap) throw new CapExceeded(spent, reserve(), cap, next);
    },
    charge({ lane, arm, model, item, priceId, usages }) {
      const usd = turnCost(priceId, usages);
      const sum = (pick) => usages.reduce((total, usage) => total + (pick(usage) ?? 0), 0);
      write({
        lane, arm, model, item, priceId, usd, requests: usages.length,
        inputTokens: sum((usage) => usage.inputTokens),
        outputTokens: sum((usage) => usage.outputTokens),
        cacheReadTokens: sum((usage) => usage.inputTokenDetails?.cacheReadTokens),
      });
      spent += usd;
      largest = Math.max(largest, usd);
      return usd;
    },
    chargeUnpriced({ lane, arm, model, item, reason }) {
      write({ lane, arm, model, item, unpriced: true, reason: String(reason).slice(0, 500) });
      unpriced++;
    },
  };
}

// Charge a finished turn; a turn whose provider reported no usage is counted, never priced at zero.
export function chargeTurn(ledger, key, priceId, usages) {
  try {
    return ledger.charge({ ...key, priceId, usages });
  } catch (error) {
    if (error.name !== 'UsageMissing') throw error;
    ledger.chargeUnpriced({ ...key, reason: error.message });
    return null;
  }
}
```

- [ ] **Step 7: Write `stub-model.mjs`**

```js
// Canned models for dry runs: same call shapes as the real lanes, no network, no cost beyond the stub price.
import { MockLanguageModelV3 } from 'ai/test';

const USAGE = { inputTokens: { total: 1000 }, outputTokens: { total: 200 } };
const finish = (unified) => ({ unified, raw: unified });

export const STUB_REPORT = [
  'DEGRADED: single-context (stub run)',
  'Priority issues:',
  '1. The .hero gradient (#667eea to #764ba2) reads as a stock template; draw the color from the product\'s world.',
  '2. The .fine-print text (#b8b8b8 on white) fails 4.5:1 contrast; darken it to #5f6b73.',
  '3. The feature cards look lazy: three identical boxes with emoji icons.',
].join('\n');

export const STUB_BUILD_HTML = '<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Stub build</title><style>body{margin:0;font-family:Georgia,serif}main{padding:48px}h1{font-size:48px;background:linear-gradient(90deg,#f6d365,#fda085);-webkit-background-clip:text;background-clip:text;color:transparent}</style></head><body><main><h1>Stub build</h1><p>Dry-run page written by the stub model.</p></main></body></html>';

function promptText(prompt) {
  return prompt.flatMap((message) => (typeof message.content === 'string'
    ? [message.content]
    : message.content.filter((part) => part.type === 'text').map((part) => part.text))).join('\n');
}

export function stubVerdict(text) {
  if (text.includes('PAIRWISE')) {
    return {
      preference: 'first',
      reason: 'stub verdict',
      craft: { first: { contrast: 2, spacing: 2, hierarchy: 2 }, second: { contrast: 1, spacing: 1, hierarchy: 1 } },
      stock_default_look: { first: 2, second: 4 },
    };
  }
  const ids = [...text.matchAll(/^DEFECT ([a-z0-9-]+):/gm)].map((match) => match[1]);
  const hits = [...text.matchAll(/^LEXICON HIT (\d+):/gm)].map((match) => Number(match[1]));
  return {
    defects: ids.map((id, index) => ({ id, found: index % 2 === 0, evidence: index % 2 === 0 ? 'stub evidence' : '' })),
    findings_total: 3,
    findings_specific: 2,
    labels: hits.map((hit) => ({ hit, aimed_at_work_or_author: true })),
    meta_comments: [],
  };
}

export function stubModel(kind) {
  return new MockLanguageModelV3({
    modelId: 'stub',
    doGenerate: async ({ prompt }) => {
      const text = promptText(prompt);
      if (kind === 'tone') {
        const content = [{ type: 'text', text: STUB_REPORT }];
        const critique = promptText(prompt.filter((message) => message.role === 'user')).includes('/impeccable critique');
        if (critique) {
          content.push({ type: 'tool-call', toolCallId: 'question', toolName: 'ask_user_question',
            input: JSON.stringify({ questions: [{ question: 'Which issue first?', options: [{ label: 'Contrast' }, { label: 'Copy' }] }] }) });
        }
        return { content, finishReason: finish(critique ? 'tool-calls' : 'stop'), usage: USAGE, warnings: [] };
      }
      if (kind === 'build') {
        if (prompt.some((message) => message.role === 'tool')) {
          return { content: [{ type: 'text', text: 'Built index.html. Assumptions: stub run.' }], finishReason: finish('stop'), usage: USAGE, warnings: [] };
        }
        return {
          content: [{ type: 'tool-call', toolCallId: 'write', toolName: 'write', input: JSON.stringify({ path: 'index.html', contents: STUB_BUILD_HTML }) }],
          finishReason: finish('tool-calls'), usage: USAGE, warnings: [],
        };
      }
      if (kind === 'judge') {
        return { content: [{ type: 'text', text: JSON.stringify(stubVerdict(text)) }], finishReason: finish('stop'), usage: USAGE, warnings: [] };
      }
      throw new Error(`unknown stub kind "${kind}"`);
    },
  });
}
```

The stub report deliberately contains `lazy` so the dry run exercises the label path. The stub models are exercised by the lane tests in Tasks 8 to 10.

- [ ] **Step 8: Run green**

Run: `node --test tests/wording-experiment/store.test.mjs tests/wording-experiment/cost.test.mjs tests/test-suites.test.mjs tests/ci-test-plan.test.mjs`
Expected: all pass (`# fail 0`); `tests/ci-test-plan.test.mjs` is in the default core suite and also reads the suite registry. `node scripts/run-tests.mjs --list` shows `wording-experiment (opt-in)`.

- [ ] **Step 9: Commit**

```bash
cd /Volumes/Containers/impeccable
git add scripts/test-suites.mjs tests/wording-experiment/config.mjs tests/wording-experiment/store.mjs tests/wording-experiment/cost.mjs tests/wording-experiment/stub-model.mjs tests/wording-experiment/store.test.mjs tests/wording-experiment/cost.test.mjs
git commit -m "wording-experiment: store, cost ledger, stubs, opt-in suite" -m "Append-only JSONL store keyed by lane, arm, model, and item with a one-retry helper; per-request pricing from the OpenRouter table fetched 2026-09-26 with a ledger that refuses a call once spend plus reserve would pass the \$40 cap; stub models for dry runs. Registered as the opt-in wording-experiment suite." -m "AI assistance: drafted with Claude Code."
```

**Checkpoint:** core modules exist and are tested; nothing billed; default suite unaffected (the new suite is opt-in).

---

## Task 3: `invariants.mjs`: constraint parity, frozen spans, sentence inventory, gates

**Files:**
- Create: `tests/wording-experiment/invariants.mjs`
- Test: `tests/wording-experiment/invariants.test.mjs`
- Modify: `scripts/test-suites.mjs` (append `'tests/wording-experiment/invariants.test.mjs'` to the `wording-experiment` files list)

**Interfaces:**
- Consumes: `config.mjs` (`ARMS`, `ARMS_ROOT`, `ENGINE_BIN`, `REPO_ROOT`, `RESULTS_ROOT`, `STAGE1_FILES`, `armDir`), `PROVIDER_BLOCK_TAGS` from `scripts/lib/utils.js`, `readBase(root?)` from `arms.mjs` (Task 4; imported lazily inside the CLI only, so Task 3's unit tests do not need it).
- Produces: `SKILL_FILE`, `extractTokens(text) -> Set<string>`, `splitRegions(file, text) -> { frozen: string[], free: { line, context }[] }`, `freeText(file, text) -> string`, `wordCount(file, text) -> number`, `sentences(file, text) -> { text, cls: 'P'|'J'|'N' }[]`, `classify(sentence, context)`, `prohibitiveCount(text)`, `judgmentalCount(text)`, `rewriteDegree(original, rewritten) -> number`, `checkFile({ file, original, rewritten, arm }) -> { checks, metrics }`, `runInvariants({ arm, base, dir?, repo?, gates? }) -> { arm, base, head, clean, passed, checks, perFile, at }`, `compareEffort({ base, repo?, nvcDir?, paraphraseDir? }) -> { rows, passed }`, `requirePassingInvariants({ arms, out, armDirs })` (throws with the command to run).
- CLI: `node tests/wording-experiment/invariants.mjs [arm ...] [--out <dir>] [--no-gates]` writes `<out>/invariants/<arm>.json` (only when gates ran) and prints one line per check; `--compare-effort` prints per-file rewrite degrees and exits 1 outside the band; `--inventory <file>` prints the base commit's charged (P and J) sentences of one stage-1 file, numbered.

- [ ] **Step 1: Write the failing tests**

`tests/wording-experiment/invariants.test.mjs`:

```js
import { it } from 'node:test';
import assert from 'node:assert/strict';
import {
  checkFile, extractTokens, judgmentalCount, prohibitiveCount, rewriteDegree, sentences, splitRegions,
} from './invariants.mjs';

const FILE = 'skill/reference/craft-floor.md';
const CRAFT = [
  '# Craft floor',
  '',
  'Load this after the direction is settled. <!-- rule:skill-craft-floor -->',
  '',
  '## Verify',
  '',
  '- **Contrast:** body text ≥4.5:1, large text ≥3:1. On colored surfaces never use gray. <!-- rule:skill-color-verify-contrast -->',
  '- **Type:** body measure 65–75ch, display max 6rem, tracking floor -0.04em.',
  '',
  '## Refuse',
  '',
  '- Gradient text. Emphasis comes from weight or size. <!-- rule:skill-ban-gradient-text -->',
  '- Cards are the lazy container; nested cards are always wrong.',
  '',
  '<codex>',
  '- Card radii stay at 12–16px.',
  '</codex>',
  '',
  '| # | Score |',
  '|---|---|',
  '| 1 | **??/20** |',
  '',
  '```bash',
  '{{scripts_path}}/impeccable detect --json index.html',
  '```',
  '',
  'Touch targets under 44x44px, a `0.01ms` kill, a 0.72 gate, a **/40** total, and `80x24.txt` captures.',
].join('\n');

const NVC = CRAFT
  .replace('On colored surfaces never use gray.', 'Gray text on a colored surface goes muddy for the reader; tint it from the surface hue.')
  .replace('- Gradient text. Emphasis comes from weight or size.', '- Gradient text blurs emphasis for the reader; carry it with weight or size.')
  .replace('- Cards are the lazy container; nested cards are always wrong.', '- Identical cards as structure hide what matters; shape the page from its content and keep cards unnested.');

const PARAPHRASE = CRAFT
  .replace('On colored surfaces never use gray.', 'Never use gray on colored surfaces.')
  .replace('- Gradient text. Emphasis comes from weight or size.', '- Do not use gradient text; put emphasis in weight or size.')
  .replace('- Cards are the lazy container; nested cards are always wrong.', '- Do not build the structure from cards, a default container; never nest them.');

const byId = (result) => Object.fromEntries(result.checks.map((check) => [check.id, check]));

it('thresholds, identifiers, paths, placeholders, and rule markers become tokens the rewrite must keep', () => {
  const tokens = extractTokens(CRAFT);
  for (const token of ['4.5:1', '3:1', '65–75ch', '6rem', '-0.04em', '12–16px', '44x44px', '0.01ms', '0.72', '/40',
    '??/20', '80x24.txt', '<!-- rule:skill-craft-floor -->', '{{scripts_path}}']) {
    assert.ok(tokens.has(token), `missing token ${token}`);
  }
  assert.ok(extractTokens('Lead with `⚠️ DEGRADED: single-context (<reason>)` or `Questions skipped: <reason>`.')
    .has('Questions skipped: <reason>'));
});

it('a rewrite that drops a threshold fails constraint parity', () => {
  const result = checkFile({ file: FILE, original: CRAFT, rewritten: NVC.replace('≥3:1', 'a lower ratio'), arm: 'nvc' });
  assert.equal(byId(result)['constraint-parity'].pass, false);
  assert.match(byId(result)['constraint-parity'].detail, /3:1/);
});

it('a rewrite that grows the file past ten percent fails length parity', () => {
  const longer = NVC.replace('Load this after the direction is settled.', `Load this after the direction is settled. ${'More words here. '.repeat(30)}`);
  assert.equal(byId(checkFile({ file: FILE, original: CRAFT, rewritten: longer, arm: 'nvc' }))['length-parity'].pass, false);
});

it('an em dash anywhere in an arm file fails', () => {
  const dashed = NVC.replace('Load this after', 'Load this \u2014 after');
  assert.equal(byId(checkFile({ file: FILE, original: CRAFT, rewritten: dashed, arm: 'nvc' }))['no-em-dash'].pass, false);
});

it('headings, table rows, code fences, and provider blocks must survive byte for byte', () => {
  for (const [from, to] of [['## Verify', '## Check'], ['| 1 | **??/20** |', '| 1 | **??/20 total** |'],
    ['- Card radii stay at 12–16px.', '- Keep card radii at 12–16px.'], ['--json index.html', '--json  index.html']]) {
    const result = checkFile({ file: FILE, original: CRAFT, rewritten: NVC.replace(from, to), arm: 'nvc' });
    assert.equal(byId(result)['frozen-spans'].pass, false, `${from} -> ${to} should break a frozen span`);
  }
  assert.equal(byId(checkFile({ file: FILE, original: CRAFT, rewritten: NVC, arm: 'nvc' }))['frozen-spans'].pass, true);
});

it('outside the design-laws span SKILL.src.md stays byte-identical', () => {
  const skill = ['---', 'name: impeccable', '---', '', 'Intro. Never hedge.', '', '## Setup', '', '1. Run the loader once; do not rerun it.',
    '', '## How to design', '', '- **The brief wins.** Redirecting a clear brief toward your taste is failure.', '', '## Modes', '', '- Persuade: act.'].join('\n');
  const free = splitRegions('skill/SKILL.src.md', skill).free.map((entry) => entry.line);
  assert.deepEqual(free.filter(Boolean), ['Intro. Never hedge.', '- **The brief wins.** Redirecting a clear brief toward your taste is failure.']);
  const setupEdited = skill.replace('do not rerun it', 'run it once');
  assert.equal(byId(checkFile({ file: 'skill/SKILL.src.md', original: skill, rewritten: setupEdited, arm: 'nvc' }))['frozen-spans'].pass, false);
  const introEdited = skill.replace('Intro. Never hedge.', 'Intro. Commit to each call.');
  assert.equal(byId(checkFile({ file: 'skill/SKILL.src.md', original: skill, rewritten: introEdited, arm: 'nvc' }))['frozen-spans'].pass, true);
});

it('sentences are classed by lexicon and by the refuse and never-list context', () => {
  const classes = Object.fromEntries(sentences(FILE, CRAFT).map((sentence) => [sentence.text, sentence.cls]));
  assert.equal(classes['Emphasis comes from weight or size.'], 'P');
  assert.equal(classes['- Cards are the lazy container; nested cards are always wrong.'], 'J');
  assert.equal(classes['Load this after the direction is settled.'], 'N');
  const audit = sentences('skill/reference/audit.md', 'Intro line.\n\n**NEVER**:\n- Report issues without explaining impact\n\nAfter the list.');
  assert.deepEqual(audit.map((sentence) => sentence.cls), ['N', 'P', 'P', 'N']);
});

it('the nvc arm reframes prohibitions and drops labels while keeping neutral sentences verbatim', () => {
  const checks = byId(checkFile({ file: FILE, original: CRAFT, rewritten: NVC, arm: 'nvc' }));
  for (const id of ['constraint-parity', 'frozen-spans', 'neutral-kept', 'charged-rewritten', 'no-judgmental-labels', 'prohibitions-reframed', 'no-em-dash']) {
    assert.equal(checks[id].pass, true, `${id}: ${checks[id].detail}`);
  }
  const touchedNeutral = NVC.replace('Load this after the direction is settled.', 'Load this once the direction is settled.');
  assert.equal(byId(checkFile({ file: FILE, original: CRAFT, rewritten: touchedNeutral, arm: 'nvc' }))['neutral-kept'].pass, false);
});

it('the paraphrase arm keeps its prohibitions but not the labels', () => {
  const checks = byId(checkFile({ file: FILE, original: CRAFT, rewritten: PARAPHRASE, arm: 'paraphrase' }));
  for (const id of ['charged-rewritten', 'no-judgmental-labels', 'prohibitions-kept']) assert.equal(checks[id].pass, true, `${id}: ${checks[id].detail}`);
  const labelled = PARAPHRASE.replace('a default container', 'the lazy container');
  assert.equal(byId(checkFile({ file: FILE, original: CRAFT, rewritten: labelled, arm: 'paraphrase' }))['no-judgmental-labels'].pass, false);
  assert.ok(prohibitiveCount('Never nest. Do NOT skip. No hedging.') >= 3);
  assert.equal(judgmentalCount('the cheapest signal, a regenerated page'), 0);
});

it('rewrite degree grows with the share of changed words', () => {
  assert.equal(rewriteDegree('alpha beta gamma delta\n', 'alpha beta gamma delta\n'), 0);
  const degree = rewriteDegree('alpha beta gamma delta\n', 'alpha omega gamma delta\n');
  assert.ok(Math.abs(degree - 0.25) < 1e-9, String(degree));
});
```

- [ ] **Step 2: Run and watch them fail**

Run: `node --test tests/wording-experiment/invariants.test.mjs`
Expected: fails at import with `ERR_MODULE_NOT_FOUND` for `./invariants.mjs`.

- [ ] **Step 3: Write `invariants.mjs`**

```js
// Pre-run arm checks. A failure here blocks every billed lane.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync, spawnSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
import { parseArgs } from 'node:util';
import { PROVIDER_BLOCK_TAGS } from '../../scripts/lib/utils.js';
import { ARMS, ENGINE_BIN, REPO_ROOT, RESULTS_ROOT, STAGE1_FILES, armDir } from './config.mjs';

export const SKILL_FILE = 'skill/SKILL.src.md';
// Decision 1: the preamble (section null) and "How to design" are editable; every other section is frozen.
const SKILL_EDITABLE_SECTIONS = new Set([null, '## How to design']);
const EM_DASH = /\u2014|&mdash;|&#8212;|&#x2014;/i;
const MAX_BUFFER = 64 * 1024 * 1024;

const TOKEN_PATTERNS = [
  /<!--\s*rule:[^>]*?-->/g,
  /\{\{\w+\}\}/g,
  /\d+(?:\.\d+)?:1\b/g,
  /\d+x\d+(?:px)?/g,
  /-?\d+(?:\.\d+)?(?:[\u2013-]\d+(?:\.\d+)?)?(?:px|rem|em|ch|ms)\b/g,
  /\b\d+\.\d+\b/g,
  /(?:\?\?)?\/\d{2}\b/g,
  /\d+(?:\.\d+)?%/g,
  /(?:\.?[\w-]+\/)*[\w-]+\.(?:md|json|png|txt|html|mjs|js|css|jpg)\b/g,
];

export function extractTokens(text) {
  const tokens = new Set();
  for (const match of text.matchAll(/`([^`\n]+)`/g)) tokens.add(match[1]);
  const unticked = text.replace(/`[^`\n]+`/g, ' ');
  for (const pattern of TOKEN_PATTERNS) for (const match of unticked.matchAll(pattern)) tokens.add(match[0]);
  return tokens;
}

export function prohibitiveCount(text) {
  return (text.match(/\b(?:never|do not|don't|must not|not permitted|no brief earns|ban|banned|bans|refuse)\b/gi) ?? []).length
    + (text.match(/\bNOT\b/g) ?? []).length
    + (text.match(/(?:^|[.;:!?]\s+)No\s/gm) ?? []).length;
}

const JUDGMENTAL = /\b(?:slop|sloppy|lazy|generic|amateur|amateurish|cheap|costume|lifeless|bland|timid|tacky|soulless|cookie-cutter|machine-made)\b|\b(?:is|are) (?:a )?failure\b|\balways wrong\b/gi;
export function judgmentalCount(text) {
  return (text.match(JUDGMENTAL) ?? []).length;
}

export function splitRegions(file, text) {
  const lines = text.split('\n');
  const frozen = [];
  const free = [];
  let index = 0;
  if (lines[0] === '---') {
    const end = lines.indexOf('---', 1);
    frozen.push(lines.slice(0, end + 1).join('\n'));
    index = end + 1;
  }
  let section = null;
  let fence = null;
  let block = null;
  let blockTag = null;
  let neverList = false;
  for (; index < lines.length; index++) {
    const line = lines[index];
    if (block) {
      block.push(line);
      if (new RegExp(`^[ \\t]*</${blockTag}>[ \\t]*$`).test(line)) {
        frozen.push(block.join('\n'));
        block = null;
      }
      continue;
    }
    if (fence) {
      fence.push(line);
      if (/^\s*(```|~~~)/.test(line)) {
        frozen.push(fence.join('\n'));
        fence = null;
      }
      continue;
    }
    const open = line.match(/^[ \t]*<([a-z][a-z0-9-]*)>[ \t]*$/);
    if (open && PROVIDER_BLOCK_TAGS.has(open[1])) {
      block = [line];
      blockTag = open[1];
      continue;
    }
    if (/^\s*(```|~~~)/.test(line)) {
      fence = [line];
      continue;
    }
    if (/^#{1,6} /.test(line)) {
      if (/^## /.test(line)) section = line;
      neverList = false;
      frozen.push(line);
      continue;
    }
    if (/^\s*\|/.test(line)) {
      frozen.push(line);
      continue;
    }
    if (file === SKILL_FILE && !SKILL_EDITABLE_SECTIONS.has(section)) {
      frozen.push(line);
      continue;
    }
    if (!line.trim()) neverList = false;
    const bullet = /^\s*- /.test(line);
    const context = section === '## Refuse' && bullet ? 'refuse' : neverList && bullet ? 'never-list' : null;
    free.push({ line, context });
    if (/^\*\*NEVER\*\*:?$/.test(line.trim())) neverList = true;
  }
  if (block || fence) throw new Error(`${file}: unterminated ${block ? `<${blockTag}> block` : 'code fence'}`);
  return { frozen, free };
}

const words = (text) => text.split(/\s+/).filter(Boolean).length;
const normalize = (text) => text.replace(/<!--.*?-->/g, '').replace(/\s+/g, ' ').trim();

export function freeText(file, text) {
  return splitRegions(file, text).free.map((entry) => entry.line).join('\n');
}

// Decision 1: SKILL.src.md is measured over its editable span, every other file over the whole file.
export function wordCount(file, text) {
  return words(file === SKILL_FILE ? freeText(file, text) : text);
}

export function classify(sentence, context) {
  if (judgmentalCount(sentence) > 0) return 'J';
  if (context === 'refuse' || context === 'never-list' || prohibitiveCount(sentence) > 0) return 'P';
  return 'N';
}

export function sentences(file, text) {
  const out = [];
  for (const { line, context } of splitRegions(file, text).free) {
    const clean = line.replace(/<!--.*?-->/g, '').trim();
    if (!clean) continue;
    for (const piece of clean.split(/(?<=[.!?])\s+(?=[A-Z*`"(\[])/)) {
      const sentence = piece.trim();
      if (sentence) out.push({ text: sentence, cls: classify(sentence, context) });
    }
  }
  return out;
}

export function rewriteDegree(original, rewritten) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'wording-degree-'));
  try {
    const before = path.join(dir, 'before.txt');
    const after = path.join(dir, 'after.txt');
    fs.writeFileSync(before, original);
    fs.writeFileSync(after, rewritten);
    const diff = spawnSync('git', ['diff', '--no-index', '--no-color', '--word-diff=porcelain', '--unified=0', before, after], { encoding: 'utf8', maxBuffer: MAX_BUFFER });
    if (diff.status !== 0 && diff.status !== 1) throw new Error(`git diff --no-index failed: ${diff.stderr}`);
    let removed = 0;
    let added = 0;
    let inHunk = false;
    for (const line of diff.stdout.split('\n')) {
      if (line.startsWith('@@')) {
        inHunk = true;
        continue;
      }
      if (!inHunk) continue;
      if (line.startsWith('-')) removed += words(line.slice(1));
      else if (line.startsWith('+')) added += words(line.slice(1));
    }
    const total = words(original) + words(rewritten);
    return total ? (removed + added) / total : 0;
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

function firstDifference(before, after) {
  const index = before.findIndex((span, i) => span !== after[i]);
  const at = index === -1 ? Math.min(before.length, after.length) : index;
  return JSON.stringify((before[at] ?? after[at] ?? '').slice(0, 160));
}

export function checkFile({ file, original, rewritten, arm }) {
  const checks = [];
  const add = (id, pass, detail) => checks.push({ id, file, pass, detail });

  const tokens = [...extractTokens(original)];
  const missing = tokens.filter((token) => !rewritten.includes(token));
  add('constraint-parity', missing.length === 0,
    missing.length ? `missing: ${missing.slice(0, 20).join(' | ')}` : `${tokens.length} tokens present`);

  const before = splitRegions(file, original);
  const after = splitRegions(file, rewritten);
  const frozenSame = before.frozen.length === after.frozen.length && before.frozen.every((span, i) => span === after.frozen[i]);
  add('frozen-spans', frozenSame,
    frozenSame ? `${before.frozen.length} spans identical` : `first difference: ${firstDifference(before.frozen, after.frozen)}`);

  const origWords = wordCount(file, original);
  const armWords = wordCount(file, rewritten);
  const ratio = armWords / origWords;
  add('length-parity', ratio >= 0.9 && ratio <= 1.1, `${armWords} words vs ${origWords} (${(ratio * 100).toFixed(1)}%)`);
  add('no-em-dash', !EM_DASH.test(rewritten), EM_DASH.test(rewritten) ? 'em dash present' : 'none');

  const beforeFree = freeText(file, original);
  const afterFree = freeText(file, rewritten);
  const metrics = {
    origWords, armWords,
    degree: rewriteDegree(beforeFree, afterFree),
    prohibitive: prohibitiveCount(afterFree), origProhibitive: prohibitiveCount(beforeFree),
    judgmental: judgmentalCount(afterFree), origJudgmental: judgmentalCount(beforeFree),
  };

  if (arm === 'nvc' || arm === 'paraphrase') {
    const inventory = sentences(file, original);
    const armText = normalize(afterFree);
    const droppedNeutral = inventory.filter((sentence) => sentence.cls === 'N' && !armText.includes(normalize(sentence.text)));
    const unchangedCharged = inventory.filter((sentence) => sentence.cls !== 'N' && armText.includes(normalize(sentence.text)));
    const charged = inventory.filter((sentence) => sentence.cls !== 'N').length;
    add('neutral-kept', droppedNeutral.length === 0, droppedNeutral.length
      ? `restore verbatim: ${droppedNeutral.slice(0, 5).map((sentence) => JSON.stringify(sentence.text)).join(' ')}`
      : 'every neutral sentence verbatim');
    add('charged-rewritten', unchangedCharged.length === 0, unchangedCharged.length
      ? `still verbatim: ${unchangedCharged.slice(0, 5).map((sentence) => JSON.stringify(sentence.text)).join(' ')}`
      : `${charged} charged sentences rewritten`);
    add('no-judgmental-labels', metrics.judgmental === 0, `${metrics.judgmental} judgmental words (original ${metrics.origJudgmental})`);
    if (arm === 'nvc') {
      const limit = Math.floor(metrics.origProhibitive * 0.25);
      add('prohibitions-reframed', metrics.prohibitive <= limit, `${metrics.prohibitive} prohibitive (original ${metrics.origProhibitive}, at most ${limit})`);
    } else {
      const floor = Math.ceil(metrics.origProhibitive * 0.7);
      add('prohibitions-kept', metrics.prohibitive >= floor, `${metrics.prohibitive} prohibitive (original ${metrics.origProhibitive}, at least ${floor})`);
    }
  }
  return { checks, metrics };
}

function gate(command, args, cwd) {
  const result = spawnSync(command, args, {
    cwd, encoding: 'utf8', timeout: 900_000, maxBuffer: MAX_BUFFER,
    env: { ...process.env, IMPECCABLE_BIN: ENGINE_BIN },
  });
  const tail = `${result.stdout ?? ''}\n${result.stderr ?? ''}`.trim().slice(-1500);
  return {
    id: `gate: ${command} ${args.join(' ')}`,
    pass: result.status === 0,
    detail: result.status === 0 ? 'exit 0' : `exit ${result.status}${result.signal ? ` (${result.signal})` : ''}: ${tail}`,
  };
}

const gitIn = (dir) => (...args) => execFileSync('git', ['-C', dir, ...args], { encoding: 'utf8', maxBuffer: MAX_BUFFER }).trim();

// The original text of a stage-1 file at the arm base, byte for byte (no trimming).
function showOriginal(repo, base, file) {
  return execFileSync('git', ['-C', repo, 'show', `${base}:${file}`], { encoding: 'utf8', maxBuffer: MAX_BUFFER });
}

export function runInvariants({ arm, dir = armDir(arm), repo = REPO_ROOT, base, gates = true }) {
  const git = gitIn(dir);
  const checks = [];
  const perFile = {};
  const changed = [...new Set([
    ...git('diff', '--name-only', base).split('\n'),
    ...git('ls-files', '--others', '--exclude-standard').split('\n'),
  ].filter(Boolean))];
  const outside = changed.filter((file) => !STAGE1_FILES.includes(file));
  checks.push({
    id: 'diff-scope',
    pass: outside.length === 0 && (arm !== 'original' || changed.length === 0),
    detail: `changed since ${base.slice(0, 10)}: ${changed.join(', ') || 'none'}`,
  });
  for (const file of STAGE1_FILES) {
    const original = showOriginal(repo, base, file);
    const rewritten = fs.readFileSync(path.join(dir, file), 'utf8');
    const result = checkFile({ file, original, rewritten, arm });
    checks.push(...result.checks);
    perFile[file] = result.metrics;
  }
  if (gates) {
    checks.push(gate('bun', ['run', 'build'], dir));
    checks.push(gate('node', ['--test', 'tests/skill-reference.test.mjs'], dir));
    checks.push(gate('bun', ['test', '--timeout', '60000', 'tests/build.test.js'], dir));
  }
  return {
    arm, base, head: git('rev-parse', 'HEAD'), clean: git('status', '--porcelain') === '',
    passed: checks.every((check) => check.pass), checks, perFile, at: new Date().toISOString(),
  };
}

// Decision 5: paraphrase's rewrite degree sits within 25% (relative) of nvc's, or within 0.02 when nvc's is under 0.08.
export function compareEffort({ repo = REPO_ROOT, base, nvcDir = armDir('nvc'), paraphraseDir = armDir('paraphrase') }) {
  const rows = STAGE1_FILES.map((file) => {
    const original = freeText(file, showOriginal(repo, base, file));
    const nvc = rewriteDegree(original, freeText(file, fs.readFileSync(path.join(nvcDir, file), 'utf8')));
    const paraphrase = rewriteDegree(original, freeText(file, fs.readFileSync(path.join(paraphraseDir, file), 'utf8')));
    const pass = nvc >= 0.08 ? Math.abs(paraphrase - nvc) <= 0.25 * nvc : Math.abs(paraphrase - nvc) <= 0.02;
    return { file, nvc, paraphrase, pass };
  });
  return { rows, passed: rows.every((row) => row.pass) };
}

export function requirePassingInvariants({ arms, out, armDirs }) {
  for (const arm of arms) {
    const file = path.join(out, 'invariants', `${arm}.json`);
    const command = `node tests/wording-experiment/invariants.mjs ${arm}${out === RESULTS_ROOT ? '' : ` --out ${out}`}`;
    if (!fs.existsSync(file)) throw new Error(`invariants have not run for ${arm}; run: ${command}`);
    const record = JSON.parse(fs.readFileSync(file, 'utf8'));
    const git = gitIn(armDirs[arm]);
    if (!record.passed) throw new Error(`invariants failed for ${arm}; fix the arm and run: ${command}`);
    if (record.head !== git('rev-parse', 'HEAD')) throw new Error(`${arm} moved since its invariants passed; run: ${command}`);
    if (git('status', '--porcelain') !== '') throw new Error(`${arm} has uncommitted changes; commit them, then run: ${command}`);
  }
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  const { values, positionals } = parseArgs({
    allowPositionals: true,
    options: {
      out: { type: 'string', default: RESULTS_ROOT },
      'no-gates': { type: 'boolean', default: false },
      'compare-effort': { type: 'boolean', default: false },
      inventory: { type: 'string' },
    },
  });
  const { readBase } = await import('./arms.mjs');
  const { base } = readBase();
  if (values.inventory) {
    if (!STAGE1_FILES.includes(values.inventory)) throw new Error(`--inventory takes one of: ${STAGE1_FILES.join(', ')}`);
    const inventory = sentences(values.inventory, showOriginal(REPO_ROOT, base, values.inventory));
    inventory.forEach((sentence, index) => {
      if (sentence.cls !== 'N') console.log(`${String(index + 1).padStart(4)} ${sentence.cls} ${sentence.text}`);
    });
  } else if (values['compare-effort']) {
    const result = compareEffort({ base });
    for (const row of result.rows) {
      console.log(`${row.pass ? 'PASS' : 'FAIL'} ${row.file} nvc=${row.nvc.toFixed(3)} paraphrase=${row.paraphrase.toFixed(3)}`);
    }
    process.exitCode = result.passed ? 0 : 1;
  } else {
    let failed = false;
    for (const arm of positionals.length ? positionals : ARMS) {
      if (!ARMS.includes(arm)) throw new Error(`unknown arm "${arm}"; expected one of ${ARMS.join(', ')}`);
      const result = runInvariants({ arm, base, gates: !values['no-gates'] });
      if (!values['no-gates']) {
        fs.mkdirSync(path.join(values.out, 'invariants'), { recursive: true });
        fs.writeFileSync(path.join(values.out, 'invariants', `${arm}.json`), `${JSON.stringify(result, null, 2)}\n`);
      }
      for (const check of result.checks) {
        console.log(`${check.pass ? 'PASS' : 'FAIL'} ${arm} ${check.id}${check.file ? ` ${check.file}` : ''}: ${check.detail}`);
      }
      console.log(`${arm}: ${result.passed ? 'PASSED' : 'FAILED'} at ${result.head}${result.clean ? '' : ' (uncommitted changes)'}`);
      failed ||= !result.passed;
    }
    process.exitCode = failed ? 1 : 0;
  }
}
```

- [ ] **Step 4: Run green**

Run: `node --test tests/wording-experiment/invariants.test.mjs`
Expected: 10 tests pass. If `the nvc arm reframes...` fails on `length-parity` it is not asserted there; any assertion that does fail names its check id and detail: fix the code, not the fixture strings, unless the fixture contradicts Decisions 3 to 5.

- [ ] **Step 5: Confirm provider blocks never reach a lane**

Run:

```bash
cd /Volumes/Containers/impeccable
node --input-type=module -e "import fs from 'node:fs'; const h = await import('./tests/skill-behavior/harness.mjs'); const w = h.prepareWorkspace(); const t = fs.readFileSync(w + '/.claude/skills/impeccable/reference/critique.md', 'utf8'); console.log('codex block text staged:', t.includes('Codex sub-agent gate')); h.cleanupWorkspace(w);"
```

Expected: `codex block text staged: false` (Decision 3's premise). If it prints `true`, stop: the frozen-block decision is wrong and must go back to the owner.

- [ ] **Step 6: Register the test file and commit**

Append `'tests/wording-experiment/invariants.test.mjs',` to the `wording-experiment` `files` list in `scripts/test-suites.mjs`, then:

```bash
cd /Volumes/Containers/impeccable
node --test tests/test-suites.test.mjs
git add scripts/test-suites.mjs tests/wording-experiment/invariants.mjs tests/wording-experiment/invariants.test.mjs
git commit -m "wording-experiment: rewrite invariants" -m "Checks each arm against the base: every threshold, identifier, path, placeholder, and rule marker kept; frontmatter, headings, tables, code fences, provider blocks, and SKILL.src.md outside the design-laws span byte-identical; word count within 10%; no em dash; neutral sentences verbatim and charged sentences rewritten; framing counts per arm; build, skill-reference, and build.test.js gates. Also compares the two rewrites' effort." -m "AI assistance: drafted with Claude Code."
```

**Checkpoint:** invariants are unit-tested on synthetic files; the CLI needs `arms.mjs` (Task 4) before it can run against real arms.

---

## Task 4: `arms.mjs`: arm branches and worktrees

**Files:**
- Create: `tests/wording-experiment/arms.mjs`
- Test: `tests/wording-experiment/arms.test.mjs`
- Modify: `scripts/test-suites.mjs` (append `'tests/wording-experiment/arms.test.mjs'`)

**Interfaces:**
- Consumes: `config.mjs` (`ARMS`, `ARMS_ROOT`, `REPO_ROOT`, `armBranch`).
- Produces: `createArms({ repo?, root?, base?, arms? }) -> { base, root, dirs: { [arm]: dir } }` (idempotent; refuses a different base while `base.json` exists), `readBase(root?) -> { base, createdAt }`, `teardownArms({ repo?, root?, arms? })` (removes worktrees, keeps branches, removes `base.json`).
- CLI: `node tests/wording-experiment/arms.mjs create|teardown`.

- [ ] **Step 1: Write the failing tests**

`tests/wording-experiment/arms.test.mjs`:

```js
import { it } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { createArms, readBase, teardownArms } from './arms.mjs';

function tempRepo() {
  const repo = fs.mkdtempSync(path.join(os.tmpdir(), 'wording-arms-repo-'));
  const git = (...args) => execFileSync('git', ['-C', repo, '-c', 'user.name=Test', '-c', 'user.email=test@example.com',
    '-c', 'commit.gpgsign=false', ...args], { encoding: 'utf8' }).trim();
  git('init', '-q', '-b', 'main');
  fs.writeFileSync(path.join(repo, '.gitignore'), 'node_modules/\n');
  fs.writeFileSync(path.join(repo, 'skill.md'), 'text\n');
  git('add', '.gitignore', 'skill.md');
  git('commit', '-q', '-m', 'base');
  fs.mkdirSync(path.join(repo, 'node_modules', 'pkg'), { recursive: true });
  return { repo, git, root: fs.mkdtempSync(path.join(os.tmpdir(), 'wording-arms-root-')) };
}

it('three arms branch from one recorded commit, each with linked node_modules and a clean tree', () => {
  const { repo, git, root } = tempRepo();
  const result = createArms({ repo, root });
  const base = git('rev-parse', 'main');
  assert.equal(result.base, base);
  assert.equal(readBase(root).base, base);
  for (const arm of ['original', 'nvc', 'paraphrase']) {
    const dir = path.join(root, arm);
    assert.equal(git('rev-parse', `wording-arm-${arm}`), base);
    assert.equal(fs.realpathSync(path.join(dir, 'node_modules')), fs.realpathSync(path.join(repo, 'node_modules')));
    assert.equal(execFileSync('git', ['-C', dir, 'status', '--porcelain'], { encoding: 'utf8' }).trim(), '');
  }
});

it('creating the arms again changes nothing', () => {
  const { repo, git, root } = tempRepo();
  createArms({ repo, root });
  const before = git('worktree', 'list', '--porcelain');
  createArms({ repo, root });
  assert.equal(git('worktree', 'list', '--porcelain'), before);
});

it('arms from a different base are refused while the recorded ones exist', () => {
  const { repo, git, root } = tempRepo();
  createArms({ repo, root });
  fs.writeFileSync(path.join(repo, 'skill.md'), 'changed\n');
  git('commit', '-q', '-am', 'moved');
  assert.throws(() => createArms({ repo, root, base: git('rev-parse', 'main') }), /refusing to mix/);
});

it('teardown removes the worktrees but keeps the branches and the shared node_modules', () => {
  const { repo, git, root } = tempRepo();
  createArms({ repo, root });
  teardownArms({ repo, root });
  for (const arm of ['original', 'nvc', 'paraphrase']) {
    assert.equal(fs.existsSync(path.join(root, arm)), false);
    assert.ok(git('rev-parse', `wording-arm-${arm}`));
  }
  assert.ok(fs.existsSync(path.join(repo, 'node_modules', 'pkg')));
  assert.equal(fs.existsSync(path.join(root, 'base.json')), false);
});
```

- [ ] **Step 2: Run and watch them fail**

Run: `node --test tests/wording-experiment/arms.test.mjs`
Expected: fails at import with `ERR_MODULE_NOT_FOUND` for `./arms.mjs`.

- [ ] **Step 3: Write `arms.mjs`**

```js
// Arm branches and worktrees: one recorded base commit, three branches, node_modules linked from the main checkout.
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
import { ARMS, ARMS_ROOT, REPO_ROOT, armBranch } from './config.mjs';

const git = (repo, ...args) => execFileSync('git', ['-C', repo, ...args], { encoding: 'utf8' }).trim();

function branchExists(repo, branch) {
  try {
    git(repo, 'rev-parse', '--verify', '--quiet', `refs/heads/${branch}`);
    return true;
  } catch {
    return false;
  }
}

// Decision 11: `node_modules/` in .gitignore matches directories only, so the symlink is excluded here.
function excludeNodeModulesLink(repo) {
  const common = path.resolve(repo, git(repo, 'rev-parse', '--git-common-dir'));
  const exclude = path.join(common, 'info', 'exclude');
  fs.mkdirSync(path.dirname(exclude), { recursive: true });
  const current = fs.existsSync(exclude) ? fs.readFileSync(exclude, 'utf8') : '';
  if (!current.split('\n').includes('/node_modules')) {
    fs.appendFileSync(exclude, `${current && !current.endsWith('\n') ? '\n' : ''}/node_modules\n`);
  }
}

export function readBase(root = ARMS_ROOT) {
  const file = path.join(root, 'base.json');
  if (!fs.existsSync(file)) throw new Error(`no arms recorded under ${root}; run: node tests/wording-experiment/arms.mjs create`);
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

export function createArms({ repo = REPO_ROOT, root = ARMS_ROOT, base, arms = ARMS } = {}) {
  const resolvedBase = base ?? git(repo, 'rev-parse', 'main');
  fs.mkdirSync(root, { recursive: true });
  const baseFile = path.join(root, 'base.json');
  if (fs.existsSync(baseFile)) {
    const recorded = readBase(root).base;
    if (recorded !== resolvedBase) {
      throw new Error(`arms under ${root} come from ${recorded}; refusing to mix them with ${resolvedBase}. Tear down first.`);
    }
  } else {
    fs.writeFileSync(baseFile, `${JSON.stringify({ base: resolvedBase, createdAt: new Date().toISOString() }, null, 2)}\n`);
  }
  excludeNodeModulesLink(repo);
  const dirs = {};
  for (const arm of arms) {
    const branch = armBranch(arm);
    if (!branchExists(repo, branch)) git(repo, 'branch', branch, resolvedBase);
    try {
      git(repo, 'merge-base', '--is-ancestor', resolvedBase, branch);
    } catch {
      throw new Error(`${branch} does not descend from ${resolvedBase}`);
    }
    const dir = path.join(root, arm);
    if (!fs.existsSync(dir)) git(repo, 'worktree', 'add', '--quiet', dir, branch);
    const link = path.join(dir, 'node_modules');
    if (!fs.existsSync(link)) fs.symlinkSync(path.join(repo, 'node_modules'), link, 'dir');
    dirs[arm] = dir;
  }
  return { base: resolvedBase, root, dirs };
}

export function teardownArms({ repo = REPO_ROOT, root = ARMS_ROOT, arms = ARMS } = {}) {
  for (const arm of arms) {
    const dir = path.join(root, arm);
    if (fs.existsSync(dir)) git(repo, 'worktree', 'remove', '--force', dir);
  }
  git(repo, 'worktree', 'prune');
  fs.rmSync(path.join(root, 'base.json'), { force: true });
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  const action = process.argv[2];
  if (action === 'create') console.log(JSON.stringify(createArms(), null, 2));
  else if (action === 'teardown') {
    teardownArms();
    console.log('arm worktrees removed; the wording-arm-* branches are kept');
  } else {
    console.error('usage: node tests/wording-experiment/arms.mjs create|teardown');
    process.exitCode = 1;
  }
}
```

- [ ] **Step 4: Run green**

Run: `node --test tests/wording-experiment/arms.test.mjs tests/wording-experiment/invariants.test.mjs`
Expected: all pass.

- [ ] **Step 5: Create the real arms from the merged `main`**

```bash
cd /Volumes/Containers/impeccable
rtk proxy git rev-parse main
node tests/wording-experiment/arms.mjs create
rtk proxy git worktree list
```

Expected: the printed `base` equals the Task 1 merge hash; `git worktree list` shows the three `.../scratchpad/wording-arms/{original,nvc,paraphrase}` worktrees on `wording-arm-*`.

- [ ] **Step 6: Run the control: the unchanged arm passes every invariant**

Run: `node tests/wording-experiment/invariants.mjs original`
Expected: every line `PASS`, ending `original: PASSED at <base hash>` with no `(uncommitted changes)` suffix (the gates must not dirty the worktree, or every billed lane would later refuse to start); the file `.../scratchpad/wording-results/invariants/original.json` exists. This takes minutes (it builds). A gate that fails here fails on unchanged text: diagnose the environment (bun, engine binary, node_modules link) before any rewrite, because the same failure would block every arm.

- [ ] **Step 7: Register and commit**

Append `'tests/wording-experiment/arms.test.mjs',` to the suite's `files` list, then:

```bash
cd /Volumes/Containers/impeccable
node --test tests/test-suites.test.mjs
git add scripts/test-suites.mjs tests/wording-experiment/arms.mjs tests/wording-experiment/arms.test.mjs
git commit -m "wording-experiment: arm branches and worktrees" -m "Creates wording-arm-original, -nvc, and -paraphrase from one recorded main commit as worktrees on the scratchpad with node_modules linked from the main checkout; idempotent, refuses a second base, and tears down worktrees while keeping the branches." -m "AI assistance: drafted with Claude Code."
```

**Checkpoint:** three worktrees exist from the recorded base; the original arm passes every invariant (the control). Record the base hash and the arm paths.

---

## Task 5: The nvc rewrite (branch `wording-arm-nvc`)

The implementer writes the prose. This task fixes the scope, the form, the hard limits, and the checks.

**Files (inside the worktree `.../scratchpad/wording-arms/nvc/`, branch `wording-arm-nvc`):**
- Modify: `skill/SKILL.src.md` (only the preamble lines after the frontmatter and the `## How to design` section; Decision 1)
- Modify: `skill/reference/craft-floor.md`, `skill/reference/critique.md`, `skill/reference/audit.md`, `skill/agents/impeccable-finish-reviewer.md`
- Test: `node tests/wording-experiment/invariants.mjs nvc` (run from the main checkout)

**Interfaces:**
- Consumes: the base text of the five files; `invariants.mjs --inventory <file>` for the list of sentences to rewrite.
- Produces: one commit on `wording-arm-nvc` touching only the five files, passing every invariant; `.../wording-results/invariants/nvc.json` with `passed: true` at that commit.

**Scope (what may change).** Only sentences the inventory classes P (prohibitive) or J (judgmental), in editable text. Everything frozen by Decision 3 stays byte-identical: frontmatter, headings, table rows, fenced code, `<codex>` and `<gemini>` blocks, rule markers, and all of `SKILL.src.md` outside the preamble and `## How to design`. Every N sentence stays verbatim, including the neighbors of a sentence you rewrite. `new-work.md`, `live.md`, and every other file stay untouched.

The lexicon also marks some descriptive sentences in critique's Reference Material, such as the persona profile "Never used this type of product." or the example finding "No visible help." They describe a persona or a failure, not an instruction to the model: reword them minimally with the same meaning ("Has not used this type of product before.", "Help is nowhere visible."), in both rewritten arms.

**The NVC form.** Each P or J sentence becomes, in one or two sentences:
1. an observation: what the page, the report, or the run does, stated as fact, with no label ("A zero-offset colored halo reads as decoration", not "is lazy");
2. the need it fails: the visitor's (for design rules), the user's (for report and protocol rules), or the parent's (for the finish reviewer's contract), named concretely ("visitors cannot tell what matters", "the user cannot act on it", "the parent ships an unreviewed viewport");
3. a request: a positive imperative that names the action to take ("Shape the structure from the content", "Name the element and the change").

Compress: fold observation and need into one clause where they share a subject. Keep each rewritten sentence within 130% of the original's word count, and each file within 10% of its original length (the `length-parity` gate; `SKILL.src.md` is measured over its editable span). Use no word from the judgmental lexicon (`slop`, `sloppy`, `lazy`, `generic`, `amateur`, `amateurish`, `cheap`, `costume`, `lifeless`, `bland`, `timid`, `tacky`, `soulless`, `cookie-cutter`, `machine-made`, "is failure", "always wrong") and keep at most a quarter of the original prohibitive-lexicon count (`never`, `do not`, `don't`, `must not`, `not permitted`, `no brief earns`, `ban`, `refuse`, capital `NOT`, a sentence-initial `No`).

**Absolutes stay absolute.** NVC changes the framing, never the strength. For every hard limit below, the rewrite keeps the constraint firm with a strength word (`only`, `always`, `every`, `must`, `exactly`, `required`) and adds the reason as the need. A hard limit may never become a preference, a default, "usually", "consider", or "try to". Hard limits found in the five files (quote-anchored so the implementer can find each):

`skill/SKILL.src.md` (editable span):
- H1 "The deliverable must be complete (except assets the user must provide)."
- H2 Bounded verification: "confirm with at most one more round, and stop polishing" (the pinned clause "the shipped device classes on a native platform; the tmux capture matrix on a terminal)" stays verbatim).
- H3 "The brief wins": honor pinned aesthetics, eras, materials, fonts, and palettes; "Redirecting a clear brief toward your taste is failure."
- H4 "Ask before replacing factual copy or adding claims."
- H5 "Never split the difference into polish on the discarded look."

`skill/reference/craft-floor.md`:
- H6 Contrast floors `≥4.5:1` and `≥3:1`; secondary text on colored surfaces is tinted, "never gray".
- H7 Type floor: `65–75ch`, `display max 6rem`, `tracking floor -0.04em`.
- H8 The eyebrow: "This one is a ban, not a default: no brief earns it back."
- H9 "nested cards are always wrong."
- H10 "A pinned brief or the committed visual world overrides anything here; your own habit does not."
- H11 System display face: "the closest installed font is a failure, not a fallback" (the finish reviewer treats Refuse items as material fixes).
The rest of the Refuse list is, by its own preamble, a set of category defaults the brief can earn; keep that condition ("the brief's own words can earn any of them") intact and reframe each item as observation, need, request.

`skill/reference/critique.md`:
- H12 Assessments A and B are both required and run as isolated sub-agents whenever a sub-agent tool exists; inline only when none exists or the user declined.
- H13 A degraded run leads with the literal banner `⚠️ DEGRADED: single-context (<reason>)`.
- H14 Assessment A finishes before detector findings enter the synthesis.
- H15 A skipped detector is a failed run unless `impeccable detect` is missing or crashes after a real attempt.
- H16 A local server started for visualization is stopped before final reporting unless the user asks to keep it.
- H17 No claim of a user-visible overlay unless script injection succeeded and the detector ran in the page.
- H18 The question is the last thing in the response; nothing follows it.
- H19 Targeted questions or the literal `Questions skipped: <reason>` line; skipping only below 3 Priority Issues.
- H20 Never hand-write a slug.
- H21 `.impeccable/critique/ignore.md` is the only prior-run input.
- H22 Never reuse an existing browser tab.
- H23 Never print `/40` over a partial heuristic set.
- H24 Do not show the helper's JSON output.
- H25 Only recommend commands from `{{available_commands}}`.
- H26 Do not invent audience details for personas.
- H27 "Don't soften criticism." Keep the strength: every problem is stated at its real severity, because the developer needs an honest reading to ship.

`skill/reference/audit.md`:
- H28 "Don't fix issues; document them for other commands to address."
- H29 Web only: native and terminal projects switch to their variant now.
- H30 Only recommend commands from `{{available_commands}}`; end with `{{command_prefix}}impeccable polish` when fixes were recommended.
- H31 The `**NEVER**:` list: every issue explains its impact, every recommendation is specific, positive findings are reported, issues are prioritized, and no false positive is reported unverified.

`skill/agents/impeccable-finish-reviewer.md`:
- H32 "You edit nothing; the parent applies your fixes."
- H33 No browser: never render, screenshot, start a server, or open a page.
- H34 A missing or invalid capture forces `disposition: recapture`; never a partial review and never a matrix built on malformed evidence.
- H35 Read only the provided inputs plus the craft floor, never another skill reference; stop reading by roughly the tenth turn.
- H36 The four disposition words are the whole vocabulary; never invent another; the word is derived, never felt.
- H37 The parent reports the disposition verbatim and has no authority to soften it; calibrate against the comp and the quality bar, never the visible effort.
- H38 Do not run a second detector pass.
- H39 Exactly five sections (or the single `recapture` section), at most eight `material_fixes`, no praise, no summary prose.
- H40 A banned element is a material fix even when it matches the comp.
- H41 Unresolved or partial findings never recompute to `ship`; re-read the exact screenshot paths; a round-stamped filename you invent points at nothing.
- H42 A screenshot filename is never invented; `.impeccable/review/` is where to look when the brief names none.

**Pinned strings that must survive verbatim inside editable text** (the gates fail otherwise): `SKILL.src.md` "the shipped device classes on a native platform; the tmux capture matrix on a terminal)"; `critique.md` "records an exact content fingerprint"; `audit.md` "**Broken touch interaction**", "what stayed untested", "Exercise the gesture when a browser tool can synthesize touch", "Terminal projects (`terminal`) route to [audit.terminal.md]"; `impeccable-finish-reviewer.md` "terminal: the text captures `80x24.txt`, `120x40.txt`, `40x24.txt`, `no-color.txt`, and `piped.txt`", "On a terminal (`terminal`) build the packet adds `reference/terminal.md` and the terminal detect findings", "terminal: the three sizes plus `no-color.txt` and `piped.txt`", "A terminal text capture is valid when it is not empty", and every `material_fixes`.

**Worked examples (original, then nvc):**

- The spec's own example: "Never ship generic, lazy defaults: they are slop."
  nvc: "When a page keeps the framework's stock defaults, visitors read it as unconsidered and trust it less. Choose each default on purpose."
- craft-floor Refuse, J: "Same-size cards of icon plus heading plus text as the page structure. Cards are the lazy container; nested cards are always wrong."
  nvc: "Same-size cards of icon plus heading plus text as the page structure leave visitors scanning identical boxes for what matters. Shape the structure from the content, and keep every card unnested: nesting always hides the hierarchy." (H9 stays absolute: "every", "always".)
- craft-floor Refuse, P by context: "Gradient text. Emphasis comes from weight or size."
  nvc: "Gradient text blurs emphasis and costs legibility. Carry emphasis with weight or size."
- SKILL.src.md H5: "Never split the difference into polish on the discarded look."
  nvc: "Polish on the discarded look hands the user back the world they chose to replace; a redesign always replaces it whole."
- finish reviewer H33: "You have no browser. Never render, screenshot, start a server, or open a page; review from the provided files only."
  nvc: "You have no browser, and a render or server started here would show a page the parent never captured; review from the provided files only, with no rendering, screenshots, servers, or opened pages."
- audit H31 item: "Provide generic recommendations (be specific and actionable)"
  nvc: "Name the element and the change in every recommendation, so the user can act on it"

- [ ] **Step 1: Print the inventories**

```bash
cd /Volumes/Containers/impeccable
for f in skill/SKILL.src.md skill/reference/craft-floor.md skill/reference/critique.md skill/reference/audit.md skill/agents/impeccable-finish-reviewer.md; do
  echo "== $f"; node tests/wording-experiment/invariants.mjs --inventory "$f"
done
```

Expected: numbered P and J sentences per file (on the base text, roughly: SKILL.src.md 4, craft-floor.md about 25 once the Refuse context counts, critique.md about 31, audit.md about 11, finish reviewer about 22). Keep this list open; it is exactly the set of sentences to rewrite, in both rewritten arms.

- [ ] **Step 2: Rewrite, file by file, in the nvc worktree**

Edit the files under `.../scratchpad/wording-arms/nvc/` only. For each inventory sentence: apply the NVC form; if it is a hard limit (H1 to H42), keep the strength word and add the reason; keep every token in it (numbers with units, backticked text, paths, placeholders, rule markers). Leave N sentences, frozen spans, and punctuation around them exactly as they are. No em dash anywhere.

- [ ] **Step 3: Iterate on the fast checks**

Run: `node tests/wording-experiment/invariants.mjs nvc --no-gates`
Expected at the end: every line `PASS` (`constraint-parity`, `frozen-spans`, `length-parity`, `no-em-dash`, `neutral-kept`, `charged-rewritten`, `no-judgmental-labels`, `prohibitions-reframed` for each file, and `diff-scope`). A `FAIL` names the file and what to restore; fix the text and rerun. Do not edit `invariants.mjs` to make a check pass; a check that looks wrong goes to the owner.

- [ ] **Step 4: Self-review the hard limits**

For H1 to H42, read the rewritten sentence beside the original and confirm: the same core instruction, the same scope, a strength word, a stated reason. Write the pairs (H number, original, rewrite) into the commit body's second paragraph only if a limit needed judgment; otherwise the commit body says "H1 to H42 kept absolute".

- [ ] **Step 5: Commit on the arm branch, then run every gate**

```bash
ARM=/private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/wording-arms/nvc
git -C "$ARM" add skill/SKILL.src.md skill/reference/craft-floor.md skill/reference/critique.md skill/reference/audit.md skill/agents/impeccable-finish-reviewer.md
git -C "$ARM" commit -m "wording arm nvc: observation, need, request in the stage-1 text" -m "Rewrites every prohibitive and judgmental sentence of the stage-1 files into observation, the need it fails, and a concrete request. Neutral sentences, frozen spans, and every threshold, identifier, path, and marker are unchanged; H1 to H42 kept absolute." -m "AI assistance: drafted with Claude Code."
cd /Volumes/Containers/impeccable
node tests/wording-experiment/invariants.mjs nvc
```

Expected: every line `PASS`, ending `nvc: PASSED at <new head>`. A gate failure (`bun run build` prose gate, `skill-reference`, `build.test.js`) prints its tail: fix in the worktree, `git -C "$ARM" commit --amend --no-edit` the same explicit paths, and rerun until `PASSED`.

**Checkpoint:** `wording-arm-nvc` has one commit touching only the five files; `invariants/nvc.json` records `passed: true` at its head.

---

## Task 6: The paraphrase rewrite (branch `wording-arm-paraphrase`) and the effort review

**Files (inside `.../scratchpad/wording-arms/paraphrase/`, branch `wording-arm-paraphrase`):** the same five files and the same editable scope as Task 5.

**Interfaces:**
- Consumes: the same inventories as Task 5; the committed nvc arm for the effort comparison.
- Produces: one commit on `wording-arm-paraphrase`, `.../wording-results/invariants/paraphrase.json` with `passed: true`, and `invariants.mjs --compare-effort` passing.

**The paraphrase form (Decision 2).** Rewrite exactly the inventory's P and J sentences, to a comparable degree, and nothing else:
- keep every prohibition a prohibition: a sentence that says "never", "do not", "no", or sits in the Refuse or NEVER list stays a negated imperative ("Do not", "Never", "No"), with at least 70% of the original prohibitive count per file;
- replace each judgmental label with a neutral descriptor, as the spec's example does ("slop" becomes "unconsidered", "lazy container" becomes "default container", "generic" becomes "unspecific");
- add no observation of the effect on a visitor and no need clause; do not turn a prohibition into a positive request;
- change wording and clause order about as much as the nvc arm did (Step 3 measures it);
- the same hard limits H1 to H42 stay exactly as firm; the same pinned strings stay verbatim; the same length, token, frozen-span, and em-dash rules apply.

**Worked examples (original, then paraphrase):**

- The spec's own example: "Never ship generic, lazy defaults: they are slop."
  paraphrase: "Do not ship stock framework defaults; they read as unconsidered."
- "Same-size cards of icon plus heading plus text as the page structure. Cards are the lazy container; nested cards are always wrong."
  paraphrase: "Do not build the page structure from same-size cards of icon, heading, and text. Cards are a default container; never nest one card inside another."
- "Gradient text. Emphasis comes from weight or size."
  paraphrase: "No gradient-filled text. Put emphasis in weight or size instead."
- "Never split the difference into polish on the discarded look."
  paraphrase: "Never compromise by polishing the discarded look."
- "You have no browser. Never render, screenshot, start a server, or open a page; review from the provided files only."
  paraphrase: "No browser is available to you. Do not render, screenshot, start a server, or open a page; use only the provided files."
- "Provide generic recommendations (be specific and actionable)"
  paraphrase: "Give unspecific recommendations (each one must be specific and actionable)"

- [ ] **Step 1: Rewrite in the paraphrase worktree, iterating on the fast checks**

Run after each file: `node tests/wording-experiment/invariants.mjs paraphrase --no-gates`
Expected at the end: every line `PASS`, with `prohibitions-kept` in place of `prohibitions-reframed`.

- [ ] **Step 2: Commit on the arm branch, then run every gate**

```bash
ARM=/private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/wording-arms/paraphrase
git -C "$ARM" add skill/SKILL.src.md skill/reference/craft-floor.md skill/reference/critique.md skill/reference/audit.md skill/agents/impeccable-finish-reviewer.md
git -C "$ARM" commit -m "wording arm paraphrase: same sentences reworded, prohibitions kept" -m "Rewords the same prohibitive and judgmental sentences of the stage-1 files to a comparable degree, keeping every prohibition in prohibitive form and replacing judgmental labels with neutral descriptors. Neutral sentences, frozen spans, and every threshold, identifier, path, and marker are unchanged; H1 to H42 kept absolute." -m "AI assistance: drafted with Claude Code."
cd /Volumes/Containers/impeccable
node tests/wording-experiment/invariants.mjs paraphrase
```

Expected: `paraphrase: PASSED at <new head>`.

- [ ] **Step 3: Measure comparable effort**

Run: `node tests/wording-experiment/invariants.mjs --compare-effort`
Expected: five `PASS` lines, e.g. `PASS skill/reference/critique.md nvc=0.071 paraphrase=0.066`. A `FAIL` means one arm rewrote a file much more than the other: bring the lighter arm's rewrites closer (more of each sentence reworded) or the heavier arm's closer, amend that arm, rerun its full invariants, then rerun this step.

- [ ] **Step 4: Independent review of both arms**

Dispatch a fresh reviewer (a subagent that drafted neither arm) with: the base hash from `wording-arms/base.json`; both worktree paths; the outputs of Step 3 and of both full invariant runs; the H1 to H42 list and the pinned strings from Task 5; and this checklist, answered per item with PASS or the file, sentence, and fix:
1. Every hard limit is exactly as firm in both arms as in the original, and the nvc arm states a reason.
2. Fifteen randomly chosen inventory sentences per arm keep the same core instruction as the original.
3. The paraphrase arm adds no observation or visitor-need clause and turns no prohibition into a positive request.
4. The nvc arm has no leftover prohibition phrased as a prohibition apart from those it needs for a hard limit.
5. Neither arm's prose would fail `docs/STYLE.md`'s structural checks more than the original does.
6. The effort band passes, and the reviewer's own read agrees the rewrites are of comparable depth.

Expected: PASS on all six. Fix any listed sentence in its arm, amend, rerun that arm's full invariants and Step 3, and send the fixed items back to the same reviewer until it returns PASS.

**Checkpoint:** both rewritten arms are committed, pass every invariant, pass the effort band, and have an independent reviewer's PASS. Record both arm heads.

---

## Task 7: Lane 1 driver: `lanes/compliance.mjs`

Split out of the brief's item 2 because the compliance lane needs its own TAP parsing, retry, and rerun logic, and Task 14 bills it.

**Files:**
- Create: `tests/wording-experiment/lanes/compliance.mjs`
- Create: `tests/wording-experiment/fixtures/stub-compliance.fixture.mjs` (named without `.test.` so `findTestFiles` never registers it)
- Test: `tests/wording-experiment/compliance.test.mjs`
- Modify: `scripts/test-suites.mjs` (append `'tests/wording-experiment/compliance.test.mjs'`)

**Interfaces:**
- Consumes: `config.mjs` (`ARMS`, `ENGINE_BIN`, `EXPERIMENT_DIR`, `modelSlug`), a store and a ledger from Task 2, each arm's `tests/skill-behavior/scenarios.test.mjs` (real) or the stub fixture (dry run), the harness's completed trace files (`{ model, status, usage, stepUsages }`).
- Produces: `SCENARIO_FILE`, `STUB_SCENARIO_FILE`, `PER_TEST_TIMEOUT_MS = 240000`, `SWEEP_MARKER = '__sweep__'`, `RERUN_SUFFIX = '::rerun'`, `escapeRegExp(text)`, `exactNamePattern(name)`, `class TapReader`, `parseTap(text) -> { name, pass, skipped, providerError, diagnostic }[]`, `regressionCandidates(store, arm, model) -> string[]`, `runComplianceLane({ arms, armDirs, models, store, ledger, out, dryRun })`.
- Store records: `{ lane: 'compliance', arm, model, item: <scenario name> | '<name>::rerun' | '__sweep__', status: 'ok'|'missing', data: { phase, pass, diagnostic? } | { scenarios } }`.

- [ ] **Step 1: Write the stub scenario file**

`tests/wording-experiment/fixtures/stub-compliance.fixture.mjs`:

```js
// Stub scenario file for dry runs of the compliance lane; lanes/compliance.mjs runs it with node --test.
import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';

const model = process.env.IMPECCABLE_SKILL_BEHAVIOR_MODELS;
const arm = process.env.WORDING_STUB_ARM;
const phase = process.env.WORDING_STUB_PHASE;
const usage = { inputTokens: 1000, outputTokens: 200, inputTokenDetails: { noCacheTokens: 1000, cacheReadTokens: 0, cacheWriteTokens: 0 } };

function trace(status) {
  const dir = process.env.IMPECCABLE_SKILL_BEHAVIOR_TRACE_DIR;
  fs.mkdirSync(dir, { recursive: true });
  const body = status === 'completed'
    ? { model: 'stub', status, usage, stepUsages: [usage] }
    : { model: 'stub', status, error: 'stub socket closed' };
  fs.writeFileSync(path.join(dir, `stub-${crypto.randomUUID()}.json`), JSON.stringify(body));
}

// WORDING_STUB_BROKEN=1 simulates a provider outage on every scenario and every phase.
function outage() {
  if (process.env.WORDING_STUB_BROKEN !== '1') return;
  trace('failed');
  throw new Error('LLM behavior turn failed before completing: stub outage');
}

describe(`skill behavior :: ${model}`, () => {
  it('scenario 1: routes the same in every arm', () => {
    outage();
    trace('completed');
  });
  it('scenario 2: regresses in nvc (#2)', () => {
    outage();
    trace('completed');
    assert.notEqual(arm, 'nvc', 'stub regression that survives the rerun');
  });
  it('scenario 3: fails once in paraphrase', () => {
    outage();
    trace('completed');
    assert.ok(!(arm === 'paraphrase' && phase === 'sweep'), 'stub failure that clears on the rerun');
  });
  it('scenario 4: provider error once in original', () => {
    outage();
    if (arm === 'original' && phase === 'sweep') {
      trace('failed');
      throw new Error('LLM behavior turn failed before completing: stub socket closed');
    }
    trace('completed');
  });
  it('scenario 5: skipped only when the environment says so', { skip: process.env.WORDING_STUB_SKIP === '1' }, () => {
    outage();
    trace('completed');
  });
});
```

- [ ] **Step 2: Write the failing tests**

`tests/wording-experiment/compliance.test.mjs`:

```js
import { it } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { ARMS } from './config.mjs';
import { openLedger } from './cost.mjs';
import { openStore } from './store.mjs';
import { exactNamePattern, parseTap, regressionCandidates, runComplianceLane } from './lanes/compliance.mjs';

const SAMPLE_TAP = [
  'TAP version 13',
  '# Subtest: skill behavior :: gpt-5.6-terra',
  '    # Subtest: scenario 7: sub-command routing (`/impeccable audit` loads audit.md)',
  '    ok 1 - scenario 7: sub-command routing (`/impeccable audit` loads audit.md)',
  '      ---',
  '      duration_ms: 12',
  '      ...',
  '    # Subtest: scenario 2: regresses (\\#2)',
  '    not ok 2 - scenario 2: regresses (\\#2)',
  '      ---',
  '      error: |-',
  '        agent should load audit.md',
  '      ...',
  '    # Subtest: scenario 3: provider',
  '    not ok 3 - scenario 3: provider',
  '      ---',
  '      error: |-',
  '        LLM behavior turn failed before completing: socket hang up',
  '      ...',
  '    ok 4 - skipped: OPENROUTER_API_KEY is unset # SKIP',
  '    1..4',
  'not ok 1 - skill behavior :: gpt-5.6-terra',
].join('\n');

function setup(cap = 40) {
  const out = fs.mkdtempSync(path.join(os.tmpdir(), 'wording-compliance-'));
  const armDirs = Object.fromEntries(ARMS.map((arm) => {
    const dir = path.join(out, 'arms', arm);
    fs.mkdirSync(dir, { recursive: true });
    return [arm, dir];
  }));
  return { out, armDirs, store: openStore(path.join(out, 'store.jsonl')), ledger: openLedger(path.join(out, 'cost.jsonl'), cap) };
}
const record = (store, arm, item) => store.all({ lane: 'compliance', arm, model: 'stub-model', item })[0];

it('scenario results are read from nested node TAP, escapes and diagnostics included', () => {
  const results = parseTap(SAMPLE_TAP);
  assert.deepEqual(results.map((result) => [result.name, result.pass, result.skipped, result.providerError]), [
    ['scenario 7: sub-command routing (`/impeccable audit` loads audit.md)', true, false, false],
    ['scenario 2: regresses (#2)', false, false, false],
    ['scenario 3: provider', false, false, true],
    ['skipped: OPENROUTER_API_KEY is unset', true, true, false],
  ]);
  assert.match(results[1].diagnostic, /agent should load audit\.md/);
});

it('an exact-name pattern selects its scenario and no scenario that merely shares a prefix', () => {
  const pattern = new RegExp(exactNamePattern('scenario 1: a (b)'));
  assert.equal(pattern.test('skill behavior :: m scenario 1: a (b)'), true);
  assert.equal(pattern.test('scenario 1: a (b)'), true);
  assert.equal(pattern.test('scenario 1: a (b) and more'), false);
  assert.equal(pattern.test('scenario 11: a (b)'), false);
});

it('a sweep records every scenario, retries a provider error once, reruns candidates, and prices every trace', async () => {
  const { out, armDirs, store, ledger } = setup();
  await runComplianceLane({ arms: ARMS, armDirs, models: ['stub-model'], store, ledger, out, dryRun: true });
  const retried = record(store, 'original', 'scenario 4: provider error once in original');
  assert.deepEqual([retried.data.phase, retried.data.pass], ['retry', true]);
  assert.equal(record(store, 'nvc', 'scenario 2: regresses in nvc (#2)').data.pass, false);
  assert.deepEqual(regressionCandidates(store, 'nvc', 'stub-model'), ['scenario 2: regresses in nvc (#2)']);
  assert.equal(record(store, 'nvc', 'scenario 2: regresses in nvc (#2)::rerun').data.pass, false);
  assert.equal(record(store, 'paraphrase', 'scenario 3: fails once in paraphrase::rerun').data.pass, true);
  for (const arm of ARMS) assert.equal(record(store, arm, '__sweep__').data.scenarios, 5);
  assert.ok(Math.abs(ledger.spent() - 17 * 0.0012) < 1e-9, String(ledger.spent()));
  assert.equal(ledger.unpriced(), 1);
});

it('running the lane again pays for nothing already recorded', async () => {
  const { out, armDirs, store, ledger } = setup();
  await runComplianceLane({ arms: ARMS, armDirs, models: ['stub-model'], store, ledger, out, dryRun: true });
  const records = store.all().length;
  const spent = ledger.spent();
  await runComplianceLane({ arms: ARMS, armDirs, models: ['stub-model'], store, ledger, out, dryRun: true });
  assert.equal(store.all().length, records);
  assert.equal(ledger.spent(), spent);
});

it('a skipped scenario fails the sweep instead of counting as a pass', async () => {
  const { out, armDirs, store, ledger } = setup();
  process.env.WORDING_STUB_SKIP = '1';
  try {
    await assert.rejects(runComplianceLane({ arms: ['original'], armDirs, models: ['stub-model'], store, ledger, out, dryRun: true }), /skipped/);
  } finally {
    delete process.env.WORDING_STUB_SKIP;
  }
  assert.equal(record(store, 'original', 'scenario 5: skipped only when the environment says so'), undefined);
  assert.equal(record(store, 'original', '__sweep__'), undefined);
});

it('a provider outage from the first scenario stops the sweep with nothing recorded', async () => {
  const { out, armDirs, store, ledger } = setup();
  process.env.WORDING_STUB_BROKEN = '1';
  try {
    await assert.rejects(runComplianceLane({ arms: ['original'], armDirs, models: ['stub-model'], store, ledger, out, dryRun: true }), /nothing was recorded/);
  } finally {
    delete process.env.WORDING_STUB_BROKEN;
  }
  assert.equal(store.all().length, 0);
});

it('the cost cap stops a sweep between scenarios and leaves the sweep open for resume', async () => {
  const { out, armDirs, store, ledger } = setup(1.5 + 0.0012 * 1.5);
  await assert.rejects(runComplianceLane({ arms: ['original'], armDirs, models: ['stub-model'], store, ledger, out, dryRun: true }), { name: 'CapExceeded' });
  assert.ok(store.all().length < 5, `recorded ${store.all().length}`);
  assert.equal(record(store, 'original', '__sweep__'), undefined);
  assert.equal(store.all({ status: 'missing' }).length, 0);
});
```

- [ ] **Step 3: Run and watch them fail**

Run: `node --test tests/wording-experiment/compliance.test.mjs`
Expected: fails at import with `ERR_MODULE_NOT_FOUND` for `./lanes/compliance.mjs`.

- [ ] **Step 4: Write `lanes/compliance.mjs`**

```js
// Lane 1: each arm's own scenarios.test.mjs, one model per sweep, read from node's TAP reporter.
import fs from 'node:fs';
import path from 'node:path';
import { spawn } from 'node:child_process';
import { ARMS, ENGINE_BIN, EXPERIMENT_DIR, modelSlug } from '../config.mjs';
import { assertNotSystematic } from '../store.mjs';

export const SCENARIO_FILE = 'tests/skill-behavior/scenarios.test.mjs';
export const STUB_SCENARIO_FILE = path.join(EXPERIMENT_DIR, 'fixtures', 'stub-compliance.fixture.mjs');
export const PER_TEST_TIMEOUT_MS = 240_000;
export const SWEEP_MARKER = '__sweep__';
export const RERUN_SUFFIX = '::rerun';
// The harness wraps every provider failure in this message (tests/skill-behavior/harness.mjs, runTurn).
const PROVIDER_ERROR = /LLM behavior turn failed before completing/;

export const escapeRegExp = (text) => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
export const exactNamePattern = (name) => `(?:^| )${escapeRegExp(name)}$`;
const isScenarioItem = (item) => item !== SWEEP_MARKER && !item.endsWith(RERUN_SUFFIX);

// Streams node's TAP output; a scenario is a result line indented four spaces (one describe level).
export class TapReader {
  constructor(onResult) {
    this.onResult = onResult;
    this.buffer = '';
    this.pending = null;
  }

  push(chunk) {
    this.buffer += chunk;
    const lines = this.buffer.split('\n');
    this.buffer = lines.pop();
    for (const line of lines) this.line(line);
  }

  end() {
    if (this.buffer) this.line(this.buffer);
    this.buffer = '';
    this.flush();
  }

  line(line) {
    if (this.pending && /^ {6}/.test(line)) {
      this.pending.diagnostic.push(line);
      return;
    }
    this.flush();
    const match = line.match(/^ {4}(not ok|ok) \d+ - (.*?)(?: # (SKIP|TODO)\b.*)?$/);
    if (match) {
      this.pending = { name: match[2].replace(/\\(.)/g, '$1'), pass: match[1] === 'ok', skipped: match[3] === 'SKIP', diagnostic: [] };
    }
  }

  flush() {
    if (!this.pending) return;
    const { diagnostic, ...result } = this.pending;
    this.pending = null;
    const text = diagnostic.join('\n');
    this.onResult({ ...result, providerError: !result.pass && PROVIDER_ERROR.test(text), diagnostic: text.slice(0, 4000) });
  }
}

export function parseTap(text) {
  const results = [];
  const reader = new TapReader((result) => results.push(result));
  reader.push(text);
  reader.end();
  return results;
}

function runNodeTest({ cwd, file, env, patterns, onResult }) {
  return new Promise((resolve, reject) => {
    const child = spawn(process.execPath,
      ['--test', '--test-reporter=tap', `--test-timeout=${PER_TEST_TIMEOUT_MS}`, ...patterns, file],
      { cwd, env, detached: true, stdio: ['ignore', 'pipe', 'pipe'] });
    let stderr = '';
    let stopError = null;
    const reader = new TapReader((result) => {
      if (stopError) return;
      try {
        onResult(result);
      } catch (error) {
        stopError = error;
        try { process.kill(-child.pid, 'SIGTERM'); } catch { /* the group already exited */ }
      }
    });
    child.stdout.setEncoding('utf8').on('data', (chunk) => reader.push(chunk));
    child.stderr.setEncoding('utf8').on('data', (chunk) => { stderr = (stderr + chunk).slice(-8000); });
    child.on('error', reject);
    child.on('close', (code, signal) => {
      reader.end();
      if (stopError) reject(stopError);
      else resolve({ code, signal, stderr });
    });
  });
}

// Completed harness traces carry usage; failed or interrupted ones are counted as unpriced.
function chargeNewTraces({ traceDir, seen, ledger, key, final = false }) {
  if (!fs.existsSync(traceDir)) return;
  for (const name of fs.readdirSync(traceDir).sort()) {
    if (!name.endsWith('.json') || seen.has(name)) continue;
    let trace;
    try {
      trace = JSON.parse(fs.readFileSync(path.join(traceDir, name), 'utf8'));
    } catch {
      if (final) {
        seen.add(name);
        ledger.chargeUnpriced({ ...key, reason: `unreadable trace ${name}` });
      }
      continue;
    }
    if (trace.status === 'in-progress' && !final) continue;
    seen.add(name);
    if (trace.status !== 'completed') {
      ledger.chargeUnpriced({ ...key, reason: `turn ${trace.status}: ${String(trace.error ?? '').slice(0, 200)}` });
      continue;
    }
    try {
      ledger.charge({ ...key, priceId: trace.model, usages: trace.stepUsages ?? [trace.usage] });
    } catch (error) {
      if (error.name !== 'UsageMissing') throw error;
      ledger.chargeUnpriced({ ...key, reason: error.message });
    }
  }
}

async function runPhase({ arm, cwd, model, phase, names = null, skip = [], store, ledger, out, dryRun }) {
  ledger.assertCanSpend(`compliance ${arm} ${model} (${phase})`);
  const traceDir = path.join(out, 'traces', 'compliance', arm, modelSlug(model), `${phase}-${Date.now()}`);
  const env = {
    ...process.env,
    IMPECCABLE_BIN: ENGINE_BIN,
    IMPECCABLE_SKILL_BEHAVIOR_MODELS: model,
    IMPECCABLE_SKILL_BEHAVIOR_TRACE_DIR: traceDir,
    WORDING_STUB_ARM: arm,
    WORDING_STUB_PHASE: phase,
  };
  const patterns = names
    ? names.map((name) => `--test-name-pattern=${exactNamePattern(name)}`)
    : skip.map((name) => `--test-skip-pattern=${exactNamePattern(name)}`);
  const rerun = phase.startsWith('rerun');
  const retry = phase.endsWith('retry');
  const seen = new Set();
  const skipped = [];
  const providerErrors = [];
  let parsed = 0;
  const { code, signal, stderr } = await runNodeTest({
    cwd, env, patterns,
    file: dryRun ? STUB_SCENARIO_FILE : SCENARIO_FILE,
    onResult: (result) => {
      parsed++;
      const item = rerun ? `${result.name}${RERUN_SUFFIX}` : result.name;
      const key = { lane: 'compliance', arm, model, item };
      chargeNewTraces({ traceDir, seen, ledger, key });
      if (result.skipped) {
        // Skips we asked for (resume, or a name filter) are expected; any other skip is a missing key or binary.
        const expected = names ? !names.includes(result.name) : skip.includes(result.name);
        if (!expected) skipped.push(result.name);
        return;
      }
      if (names && !names.includes(result.name)) return;
      if (result.providerError && !retry) providerErrors.push(result.name);
      else if (result.providerError) {
        assertNotSystematic(store, { lane: 'compliance', arm, model }, 'provider error'); // Decision 35
        store.append({ ...key, status: 'missing', data: { phase, error: 'provider error on the first try and on the retry', diagnostic: result.diagnostic } });
      } else {
        store.append({ ...key, status: 'ok', data: { phase, pass: result.pass, ...(result.pass ? {} : { diagnostic: result.diagnostic }) } });
      }
      ledger.assertCanSpend(`the compliance scenario after "${result.name}" (${arm}, ${model})`);
    },
  });
  chargeNewTraces({ traceDir, seen, ledger, key: { lane: 'compliance', arm, model, item: `${phase}:unattributed` }, final: true });
  if (skipped.length) {
    throw new Error(`compliance ${arm} ${model} (${phase}): ${skipped.length} scenario(s) skipped, first "${skipped[0]}". A skip means a missing key or engine binary; nothing was recorded for them. Fix the environment and rerun.`);
  }
  if (parsed === 0) {
    throw new Error(`compliance ${arm} ${model} (${phase}): no TAP results (exit ${code}${signal ? `, ${signal}` : ''}). stderr tail: ${stderr.slice(-1500)}`);
  }
  return providerErrors;
}

async function sweepArmModel({ arm, cwd, model, store, ledger, out, dryRun }) {
  if (store.has({ lane: 'compliance', arm, model, item: SWEEP_MARKER })) return;
  const done = store.all({ lane: 'compliance', arm, model }).map((record) => record.item).filter(isScenarioItem);
  const errors = await runPhase({ arm, cwd, model, phase: 'sweep', skip: done, store, ledger, out, dryRun });
  if (errors.length) await runPhase({ arm, cwd, model, phase: 'retry', names: errors, store, ledger, out, dryRun });
  const scenarios = store.all({ lane: 'compliance', arm, model }).filter((record) => isScenarioItem(record.item)).length;
  store.append({ lane: 'compliance', arm, model, item: SWEEP_MARKER, status: 'ok', data: { scenarios } });
}

// Decision 14: red in the rewritten arm, green in original, same model.
export function regressionCandidates(store, arm, model) {
  const originalPass = new Map(store.all({ lane: 'compliance', arm: 'original', model, status: 'ok' })
    .filter((record) => isScenarioItem(record.item))
    .map((record) => [record.item, record.data.pass]));
  return store.all({ lane: 'compliance', arm, model, status: 'ok' })
    .filter((record) => isScenarioItem(record.item) && record.data.pass === false && originalPass.get(record.item) === true)
    .map((record) => record.item);
}

export async function runComplianceLane({ arms, armDirs, models, store, ledger, out, dryRun = false }) {
  for (const arm of arms) {
    for (const model of models) await sweepArmModel({ arm, cwd: armDirs[arm], model, store, ledger, out, dryRun });
  }
  for (const model of models) {
    if (!ARMS.every((arm) => store.has({ lane: 'compliance', arm, model, item: SWEEP_MARKER }))) continue;
    for (const arm of ['nvc', 'paraphrase']) {
      const names = regressionCandidates(store, arm, model)
        .filter((name) => !store.has({ lane: 'compliance', arm, model, item: `${name}${RERUN_SUFFIX}` }));
      if (!names.length) continue;
      const errors = await runPhase({ arm, cwd: armDirs[arm], model, phase: 'rerun', names, store, ledger, out, dryRun });
      if (errors.length) await runPhase({ arm, cwd: armDirs[arm], model, phase: 'rerun-retry', names: errors, store, ledger, out, dryRun });
    }
  }
}
```

- [ ] **Step 5: Run green**

Run: `node --test tests/wording-experiment/compliance.test.mjs`
Expected: 7 tests pass. The third to seventh spawn real `node --test` children against the stub file, so they also prove the TAP shape assumed by `TapReader` (four-space scenario lines, `\#` escapes). If the third fails on a result name or indentation, print the child's raw TAP (`node --test --test-reporter=tap tests/wording-experiment/fixtures/stub-compliance.fixture.mjs`) and fix the reader to the observed format, not the other way round.

- [ ] **Step 6: Register and commit**

Append `'tests/wording-experiment/compliance.test.mjs',` to the suite's `files` list, then:

```bash
cd /Volumes/Containers/impeccable
node --test tests/test-suites.test.mjs
git add scripts/test-suites.mjs tests/wording-experiment/lanes/compliance.mjs tests/wording-experiment/fixtures/stub-compliance.fixture.mjs tests/wording-experiment/compliance.test.mjs
git commit -m "wording-experiment: compliance lane" -m "Runs each arm's own scenarios.test.mjs one model at a time with the TAP reporter, records every scenario, retries a provider error once before recording it missing, fails loudly on any unexpected skip, prices each completed trace, stops between scenarios at the cost cap, resumes with skip patterns, and reruns nvc and paraphrase scenarios that are red where original is green." -m "AI assistance: drafted with Claude Code."
```

**Checkpoint:** lane 1 is proven against the stub, including resume, skip, retry, rerun, and cap paths.

---

## Task 8: Lane 2 fixtures and the tone lane

**Files:**
- Create: `tests/wording-experiment/fixtures/pages/landing.html`, `dashboard.html`, `form.html`, `docs.html`, `portfolio.html`
- Create: `tests/wording-experiment/fixtures/defects.json`
- Create: `tests/wording-experiment/lanes/tone.mjs`
- Test: `tests/wording-experiment/fixtures.test.mjs`, `tests/wording-experiment/tone.test.mjs`
- Modify: `scripts/test-suites.mjs` (append both test files)

**Interfaces:**
- Consumes: an arm's `tests/skill-behavior/harness.mjs` (`prepareWorkspace`, `cleanupWorkspace`, `runTurn`, `summarizeTrace`), `withOneRetry`, `chargeTurn`, a store, a ledger, `modelFor(modelId, kind) -> LanguageModel`.
- Produces: `defects.json` with schema `{ version: 1, pages: [{ id, file, mode, productMd, defects: [{ id, description, selector, evidence, keywords: string[], detector: string | null }] }] }` where `evidence` is a literal substring of the page source that carries the defect and `detector` is the engine rule id that flags it (measured 2026-09-26) or `null`.
- Produces (`lanes/tone.mjs`): `TONE_COMMANDS = ['critique', 'audit']`, `TONE_MAX_STEPS = 30`, `TONE_TIMEOUT_MS = 600000`, `TONE_ENVIRONMENT`, `TONE_ENV`, `REPORT_MARKER`, `loadDefects()`, `loadArmHarness(armDir)`, `userFacingText(trace) -> string`, `compactTrace(harness, trace)`, `runToneLane({ arms, armDirs, models, store, ledger, dryRun, modelFor, limit })`.
- Store records: `{ lane: 'tone', arm, model, item: '<pageId>/<command>', status, data: { userText, finalText, outcome, steps, questions, reportSeen, trace, usd, retried } }`. `questions` counts every `ask_user_question` call; `reportSeen` is true when a report heading matched `REPORT_MARKER`. `report.mjs` reads both.

The pages plant 29 defects (6, 6, 6, 5, 6). The landing page also has an unplanted h1-to-h3 skip; the judge scores planted defects only. The detector column was measured by running `target/release/impeccable detect --json` on each page: 12 of the 29 are detector-visible, 17 need the model's own eye.

- [ ] **Step 1: Write the five pages exactly as below**

`tests/wording-experiment/fixtures/pages/landing.html`:

```html
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Tidewell: plan sails around the tide</title>
<style>
  body { margin: 0; font-family: Georgia, 'Times New Roman', serif; color: #1d2a33; background: #ffffff; }
  .hero { padding: 96px 24px 80px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: #ffffff; text-align: center; }
  .hero h1 { font-size: 64px; margin: 0 0 16px; background: linear-gradient(90deg, #f6d365, #fda085); -webkit-background-clip: text; background-clip: text; color: transparent; }
  .hero .lede { max-width: 40rem; margin: 0 auto 32px; font-size: 20px; line-height: 1.5; }
  .cta { display: inline-block; padding: 14px 28px; border-radius: 6px; background: #1d2a33; color: #ffffff; text-decoration: none; font-weight: 700; }
  .cta:focus { outline: none; }
  .features { display: grid; grid-template-columns: repeat(3, 1fr); gap: 32px; max-width: 64rem; margin: 64px auto; padding: 0 24px; }
  .feature { padding: 24px; border: 1px solid #e3e8ec; border-radius: 12px; }
  .feature .icon { font-size: 32px; }
  .feature h3 { margin: 12px 0 8px; font-size: 20px; }
  .feature p { margin: 0; line-height: 1.6; }
  .fine-print { max-width: 64rem; margin: 0 auto 48px; padding: 0 24px; font-size: 14px; color: #b8b8b8; }
</style>
</head>
<body>
  <header class="hero">
    <h1>Sail with the tide, not against it</h1>
    <p class="lede">Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.</p>
    <a class="cta" href="#download">Download Tidewell</a>
  </header>
  <main>
    <section class="features" aria-label="Features">
      <article class="feature">
        <span class="icon">🌊</span>
        <h3>Tide windows</h3>
        <p>See the hours when the channel carries enough water for your keel, drawn from local harbor tables.</p>
      </article>
      <article class="feature">
        <span class="icon">⛵</span>
        <h3>Passage timing</h3>
        <p>Tidewell suggests a departure time that rides the ebb out and the flood home.</p>
      </article>
      <article class="feature">
        <span class="icon">📍</span>
        <h3>Harbor notes</h3>
        <p>Keep your own notes on moorings, fuel docks, and the shoals the chart does not show.</p>
      </article>
    </section>
    <p class="fine-print">Tide predictions come from public harbor tables and are not a substitute for local knowledge or official notices to mariners.</p>
  </main>
</body>
</html>
```

`tests/wording-experiment/fixtures/pages/dashboard.html`:

```html
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Berthboard: tonight's berths</title>
<style>
  body { margin: 0; font-family: 'Helvetica Neue', Arial, sans-serif; color: #202124; background: #f8f9fa; }
  .top { display: flex; align-items: center; justify-content: space-between; padding: 16px 24px; background: #ffffff; border-bottom: 1px solid #dadce0; }
  .top h1 { margin: 0; font-size: 20px; }
  .refresh { padding: 8px 12px; background: #1a73e8; color: #ffffff; border-radius: 4px; cursor: pointer; }
  .wrap { padding: 24px; }
  #filter { width: 280px; padding: 8px; border: 1px solid #dadce0; border-radius: 4px; }
  .alert { margin: 16px 0; padding: 12px 16px; background: #ffffff; border-left: 4px solid #e53935; }
  table { width: 100%; border-collapse: collapse; background: #ffffff; margin-top: 16px; }
  thead th { text-align: left; background: #f1f3f4; color: #9aa0a6; font-size: 12px; padding: 8px; }
  td { padding: 2px 4px; line-height: 1.1; font-size: 13px; border-bottom: 1px solid #f1f3f4; }
  .row-action { display: inline-block; width: 20px; height: 20px; padding: 0; border: 0; background: #e8eaed; border-radius: 3px; font-size: 11px; }
</style>
</head>
<body>
  <header class="top">
    <h1>Berthboard</h1>
    <div class="refresh" onclick="location.reload()">Refresh</div>
  </header>
  <main class="wrap">
    <input id="filter" type="text" placeholder="Filter vessels">
    <div class="alert">Berth 4 conflict: MV Kestrel and MV Orla both assigned 02:10 to 03:40.</div>
    <table>
      <thead>
        <tr><th>Vessel</th><th>Berth</th><th>ETA</th><th>ETD</th><th>Status</th><th>Actions</th></tr>
      </thead>
      <tbody>
        <tr><td>MV Kestrel</td><td>4</td><td>02:10</td><td>03:40</td><td>Conflict</td><td><button class="row-action" title="Reassign">↻</button> <button class="row-action" title="Hold">⏸</button></td></tr>
        <tr><td>MV Orla</td><td>4</td><td>02:25</td><td>04:00</td><td>Conflict</td><td><button class="row-action" title="Reassign">↻</button> <button class="row-action" title="Hold">⏸</button></td></tr>
        <tr><td>Tug Brannock</td><td>1</td><td>01:50</td><td>02:20</td><td>Docked</td><td><button class="row-action" title="Reassign">↻</button> <button class="row-action" title="Hold">⏸</button></td></tr>
        <tr><td>Ferry Saoirse</td><td>2</td><td>03:05</td><td>03:35</td><td>Inbound</td><td><button class="row-action" title="Reassign">↻</button> <button class="row-action" title="Hold">⏸</button></td></tr>
        <tr><td>MV Halden</td><td>6</td><td>04:15</td><td>06:00</td><td>Scheduled</td><td><button class="row-action" title="Reassign">↻</button> <button class="row-action" title="Hold">⏸</button></td></tr>
      </tbody>
    </table>
  </main>
</body>
</html>
```

`tests/wording-experiment/fixtures/pages/form.html`:

```html
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Lantern Street Market: vendor permit</title>
<style>
  body { margin: 0; font-family: Verdana, sans-serif; color: #222222; background: #fbfaf7; }
  main { max-width: 36rem; margin: 48px auto; padding: 0 24px; }
  h1 { font-size: 28px; margin: 0 0 8px; }
  .intro { margin: 0 0 32px; line-height: 1.6; }
  input, select { display: block; width: 100%; box-sizing: border-box; margin: 0 0 16px; padding: 12px; border: 1px solid #c9c4b8; border-radius: 4px; font: inherit; }
  input:focus, select:focus { outline: none; }
  input::placeholder { color: #cccccc; }
  .required { border-color: #d93025; }
  .error { margin: -8px 0 16px; color: #d93025; font-size: 13px; }
  button { padding: 12px 24px; border: 0; border-radius: 4px; background: #3b5d3a; color: #ffffff; font: inherit; }
</style>
</head>
<body>
  <main>
    <h1>Vendor permit</h1>
    <p class="intro">Apply for a stall at the Saturday night market. Permits are reviewed within five working days.</p>
    <form>
      <input id="applicant-name" class="required" type="text" placeholder="Full name">
      <input id="applicant-email" class="required" type="email" placeholder="Email">
      <div class="error">Error: invalid input</div>
      <select id="stall-type">
        <option>Food</option>
        <option>Crafts</option>
        <option>Produce</option>
      </select>
      <input id="stall-size" type="text" placeholder="Stall size">
      <button type="submit">Submit</button>
    </form>
  </main>
</body>
</html>
```

`tests/wording-experiment/fixtures/pages/docs.html`:

```html
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>driftctl: getting started</title>
<style>
  body { margin: 0; font-family: 'Segoe UI', Roboto, sans-serif; color: #24292f; background: #ffffff; }
  .content { max-width: none; padding: 32px 48px; }
  h1 { font-size: 36px; margin: 0 0 24px; }
  h4.section { font-size: 22px; margin: 32px 0 12px; }
  p { line-height: 1.15; font-size: 16px; margin: 0 0 12px; }
  code, pre { font-family: Menlo, Consolas, monospace; background: #f5f5f5; color: #a0a0a0; }
  pre { padding: 12px 16px; border-radius: 6px; }
</style>
</head>
<body>
  <main class="content">
    <h1>Getting started with driftctl</h1>
    <p>driftctl compares the infrastructure your code declares with the infrastructure that is actually running, and reports every resource that has drifted from its declaration since the last apply, so an operator can decide whether to reconcile the live resource back to the code or to update the code so that it matches what the team changed by hand during an incident.</p>
    <h4 class="section" id="install">Install</h4>
    <p>Download the binary for your platform and put it on your PATH. Check the install with the version command.</p>
    <pre><code>driftctl version</code></pre>
    <h4 class="section" id="first-scan">Run a first scan</h4>
    <p>Point driftctl at your state file and run a scan. The first scan reads every resource the state declares and compares it with the provider's live view of the same resource.</p>
    <pre><code>driftctl scan --from tfstate://terraform.tfstate</code></pre>
    <h4 class="section" id="configuration">Configuration</h4>
    <p>TODO: write this section.</p>
  </main>
</body>
</html>
```

`tests/wording-experiment/fixtures/pages/portfolio.html`:

```html
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Mara Quell: coastal photographs</title>
<style>
  body { margin: 0; font-family: 'Gill Sans', 'Trebuchet MS', sans-serif; color: #1b1b1b; background: #ffffff; }
  header { padding: 64px 32px 32px; }
  h1 { font-family: Impact, 'Arial Black', sans-serif; font-size: 72px; margin: 0; letter-spacing: 1px; }
  .eyebrow { display: block; font-size: 12px; letter-spacing: 3px; text-transform: uppercase; margin: 0 0 8px; }
  .gallery { display: grid; grid-template-columns: repeat(3, 1fr); gap: 24px; padding: 32px; }
  .card { border: 2px solid #000000; box-shadow: 6px 6px 0 #000000; background: #ffffff; }
  .gallery img { display: block; width: 100%; height: 280px; object-fit: cover; transition: transform 0.4s; }
  .gallery img:hover { transform: scale(1.1); }
  .caption { padding: 12px 16px; font-size: 14px; color: #aaaaaa; }
</style>
</head>
<body>
  <header>
    <span class="eyebrow">Portfolio</span>
    <h1>Mara Quell</h1>
  </header>
  <main>
    <section>
      <span class="eyebrow">Selected work</span>
      <h2 style="padding: 0 32px; margin: 0;">Salt and slate, 2019 to 2025</h2>
      <div class="gallery">
        <figure class="card" style="margin:0"><img src="photos/breakwater.jpg"><figcaption class="caption">Breakwater at low water, Porthleven</figcaption></figure>
        <figure class="card" style="margin:0"><img src="photos/slate-quay.jpg"><figcaption class="caption">Slate quay after rain, Port Isaac</figcaption></figure>
        <figure class="card" style="margin:0"><img src="photos/net-lofts.jpg"><figcaption class="caption">Net lofts in winter light, Mousehole</figcaption></figure>
      </div>
    </section>
  </main>
</body>
</html>
```

- [ ] **Step 2: Write `tests/wording-experiment/fixtures/defects.json` exactly as below**

```json
{
  "version": 1,
  "pages": [
    {
      "id": "landing",
      "file": "pages/landing.html",
      "mode": "Persuade",
      "productMd": "# Tidewell\n\n<!-- impeccable:product-schema 1 -->\n\n## Platform\nweb\n\n## Users\nWeekend sailors on tidal coasts who plan short passages around harbor bars and shallow channels, usually the evening before, on a phone.\n\n## Product Purpose\nTidewell turns local tide tables into sailing windows: when the channel carries enough water for a given keel, and when to leave so the ebb carries the boat out and the flood brings it home.\n\n## Positioning\nA passage planner built around the tide rather than the weather forecast.\n\n## Brand Commitments\nPlain, seamanlike language. No invented user counts, ratings, or testimonials.\n",
      "defects": [
        {
          "id": "landing-stock-gradient",
          "description": "The hero background is the stock purple-to-violet gradient (#667eea to #764ba2), unrelated to the tide product.",
          "selector": ".hero",
          "evidence": "linear-gradient(135deg, #667eea 0%, #764ba2 100%)",
          "keywords": [
            "gradient",
            "purple",
            "violet",
            "#667eea",
            "#764ba2",
            "stock",
            "template"
          ],
          "detector": "ai-color-palette"
        },
        {
          "id": "landing-placeholder-copy",
          "description": "The hero lede is Lorem ipsum placeholder text.",
          "selector": ".hero .lede",
          "evidence": "Lorem ipsum dolor sit amet",
          "keywords": [
            "lorem",
            "ipsum",
            "placeholder",
            "filler",
            "lede"
          ],
          "detector": null
        },
        {
          "id": "landing-low-contrast",
          "description": "The fine print is #b8b8b8 on white, about 2:1, below 4.5:1.",
          "selector": ".fine-print",
          "evidence": "color: #b8b8b8",
          "keywords": [
            "contrast",
            "#b8b8b8",
            "fine print",
            "disclaimer",
            "gray"
          ],
          "detector": "low-contrast"
        },
        {
          "id": "landing-missing-focus",
          "description": "The call to action removes its focus outline and shows no replacement.",
          "selector": ".cta",
          "evidence": ".cta:focus { outline: none; }",
          "keywords": [
            "focus",
            "outline",
            "keyboard",
            "focus ring",
            "focus-visible"
          ],
          "detector": null
        },
        {
          "id": "landing-gradient-text",
          "description": "The hero heading is gradient-filled text via background-clip.",
          "selector": ".hero h1",
          "evidence": "background-clip: text",
          "keywords": [
            "gradient text",
            "background-clip",
            "heading",
            "h1"
          ],
          "detector": "gradient-text"
        },
        {
          "id": "landing-emoji-icons",
          "description": "The feature icons are emoji glyphs rather than a drawn icon set.",
          "selector": ".feature .icon",
          "evidence": "<span class=\"icon\">🌊</span>",
          "keywords": [
            "emoji",
            "icon",
            "glyph",
            "wave"
          ],
          "detector": null
        }
      ]
    },
    {
      "id": "dashboard",
      "file": "pages/dashboard.html",
      "mode": "Operate",
      "productMd": "# Berthboard\n\n<!-- impeccable:product-schema 1 -->\n\n## Platform\nweb\n\n## Users\nNight-shift harbor dispatchers who assign berths to arriving vessels from a dim control room with constant radio traffic.\n\n## Product Purpose\nShow tonight's berth assignments, surface conflicts before a vessel arrives, and let the dispatcher reassign or hold a vessel in one action.\n\n## Operating Context\nEight-hour shifts on a desktop monitor at arm's length, glanced at between radio calls; some dispatchers use a touch screen at the berth desk.\n",
      "defects": [
        {
          "id": "dashboard-low-contrast-header",
          "description": "The table header text #9aa0a6 on #f1f3f4 is about 2.4:1.",
          "selector": "thead th",
          "evidence": "color: #9aa0a6",
          "keywords": [
            "contrast",
            "header",
            "#9aa0a6",
            "column labels",
            "th"
          ],
          "detector": "low-contrast"
        },
        {
          "id": "dashboard-cramped-cells",
          "description": "Table cells use 2px 4px padding and a 1.1 line height, so rows are cramped for scanning.",
          "selector": "td",
          "evidence": "padding: 2px 4px; line-height: 1.1",
          "keywords": [
            "padding",
            "cramped",
            "dense",
            "row height",
            "line-height",
            "spacing"
          ],
          "detector": null
        },
        {
          "id": "dashboard-unlabeled-filter",
          "description": "The filter input has only a placeholder, no label.",
          "selector": "#filter",
          "evidence": "<input id=\"filter\" type=\"text\" placeholder=\"Filter vessels\">",
          "keywords": [
            "label",
            "placeholder",
            "filter",
            "aria-label"
          ],
          "detector": null
        },
        {
          "id": "dashboard-small-targets",
          "description": "Row action buttons are 20x20px, under a 44x44px touch target, and carry only a glyph and a title.",
          "selector": ".row-action",
          "evidence": "width: 20px; height: 20px",
          "keywords": [
            "touch target",
            "20px",
            "small",
            "44",
            "row action",
            "button size"
          ],
          "detector": null
        },
        {
          "id": "dashboard-side-stripe",
          "description": "The conflict alert uses a thick colored left border stripe.",
          "selector": ".alert",
          "evidence": "border-left: 4px solid #e53935",
          "keywords": [
            "border-left",
            "stripe",
            "side border",
            "alert"
          ],
          "detector": "side-tab"
        },
        {
          "id": "dashboard-div-button",
          "description": "Refresh is a div with onclick: not focusable, no button role, no keyboard activation.",
          "selector": ".refresh",
          "evidence": "<div class=\"refresh\" onclick=\"location.reload()\">",
          "keywords": [
            "div",
            "button",
            "keyboard",
            "role",
            "onclick",
            "focusable"
          ],
          "detector": null
        }
      ]
    },
    {
      "id": "form",
      "file": "pages/form.html",
      "mode": "Operate",
      "productMd": "# Lantern Street Market permits\n\n<!-- impeccable:product-schema 1 -->\n\n## Platform\nweb\n\n## Users\nSmall food and craft vendors applying for a Saturday night market stall, often on a phone between other work.\n\n## Product Purpose\nCollect a complete permit application on the first try so the market office can approve it within five working days.\n\n## Capabilities and Constraints\nThe office needs a name, an email, a stall type, and a stall size; nothing else.\n",
      "defects": [
        {
          "id": "form-placeholder-labels",
          "description": "The name and email inputs use placeholders as their only labels.",
          "selector": "#applicant-name, #applicant-email",
          "evidence": "placeholder=\"Full name\"",
          "keywords": [
            "label",
            "placeholder",
            "name",
            "email"
          ],
          "detector": null
        },
        {
          "id": "form-vague-error",
          "description": "The error says only \"Error: invalid input\", naming neither the field nor the fix.",
          "selector": ".error",
          "evidence": "Error: invalid input",
          "keywords": [
            "error",
            "invalid input",
            "message",
            "vague",
            "which field"
          ],
          "detector": null
        },
        {
          "id": "form-no-focus",
          "description": "Inputs and the select remove the focus outline and show no replacement.",
          "selector": "input, select",
          "evidence": "input:focus, select:focus { outline: none; }",
          "keywords": [
            "focus",
            "outline",
            "keyboard"
          ],
          "detector": null
        },
        {
          "id": "form-placeholder-contrast",
          "description": "Placeholder text #cccccc on white is about 1.6:1.",
          "selector": "input::placeholder",
          "evidence": "color: #cccccc",
          "keywords": [
            "placeholder",
            "contrast",
            "#cccccc",
            "light gray"
          ],
          "detector": "low-contrast"
        },
        {
          "id": "form-color-only-required",
          "description": "Required fields are marked only by a red border color, with no text or symbol.",
          "selector": ".required",
          "evidence": ".required { border-color: #d93025; }",
          "keywords": [
            "required",
            "color",
            "red border",
            "asterisk",
            "indicator"
          ],
          "detector": null
        },
        {
          "id": "form-generic-submit",
          "description": "The submit button says \"Submit\" instead of naming the action (apply for a permit).",
          "selector": "button[type=submit]",
          "evidence": "<button type=\"submit\">Submit</button>",
          "keywords": [
            "submit",
            "button label",
            "action",
            "apply"
          ],
          "detector": null
        }
      ]
    },
    {
      "id": "docs",
      "file": "pages/docs.html",
      "mode": "Read",
      "productMd": "# driftctl\n\n<!-- impeccable:product-schema 1 -->\n\n## Platform\nweb\n\n## Users\nPlatform engineers adopting driftctl to find infrastructure that changed outside their infrastructure code.\n\n## Product Purpose\nGet a new user from download to a first useful scan in ten minutes.\n\n## Operating Context\nRead beside a terminal, often on a wide monitor; code samples are copied into the shell.\n",
      "defects": [
        {
          "id": "docs-unbounded-measure",
          "description": "The content column has max-width: none, so body lines run far past 75 characters on wide screens.",
          "selector": ".content",
          "evidence": ".content { max-width: none;",
          "keywords": [
            "line length",
            "measure",
            "max-width",
            "characters",
            "wide"
          ],
          "detector": null
        },
        {
          "id": "docs-skipped-heading",
          "description": "Section headings jump from h1 to h4.",
          "selector": "h4.section",
          "evidence": "<h4 class=\"section\"",
          "keywords": [
            "heading",
            "h4",
            "hierarchy",
            "h2",
            "skipped"
          ],
          "detector": "skipped-heading"
        },
        {
          "id": "docs-code-contrast",
          "description": "Code text #a0a0a0 on #f5f5f5 is about 2.4:1.",
          "selector": "code, pre",
          "evidence": "color: #a0a0a0",
          "keywords": [
            "code",
            "contrast",
            "#a0a0a0",
            "gray"
          ],
          "detector": "low-contrast"
        },
        {
          "id": "docs-tight-leading",
          "description": "Body paragraphs use a 1.15 line height.",
          "selector": "p",
          "evidence": "line-height: 1.15",
          "keywords": [
            "line-height",
            "leading",
            "1.15",
            "tight"
          ],
          "detector": "tight-leading"
        },
        {
          "id": "docs-todo-placeholder",
          "description": "The Configuration section ships \"TODO: write this section.\"",
          "selector": "#configuration + p",
          "evidence": "TODO: write this section.",
          "keywords": [
            "TODO",
            "placeholder",
            "configuration",
            "unfinished",
            "empty"
          ],
          "detector": null
        }
      ]
    },
    {
      "id": "portfolio",
      "file": "pages/portfolio.html",
      "mode": "Experience",
      "productMd": "# Mara Quell\n\n<!-- impeccable:product-schema 1 -->\n\n## Platform\nweb\n\n## Users\nMagazine editors and gallery curators deciding whether to commission a coastal photographer.\n\n## Product Purpose\nShow a tight selection of Mara Quell's coastal work so an editor can judge her eye in under a minute.\n\n## Brand Commitments\nThe photographs lead; the page stays out of their way.\n\n## Evidence on Hand\nThree photographs whose files live in photos/ (not included in this review copy).\n",
      "defects": [
        {
          "id": "portfolio-missing-alt",
          "description": "The gallery images have no alt text.",
          "selector": ".gallery img",
          "evidence": "<img src=\"photos/breakwater.jpg\">",
          "keywords": [
            "alt",
            "alternative text",
            "screen reader",
            "images"
          ],
          "detector": null
        },
        {
          "id": "portfolio-image-hover-zoom",
          "description": "Images scale to 1.1 on hover though they are not links or controls.",
          "selector": ".gallery img:hover",
          "evidence": ".gallery img:hover { transform: scale(1.1); }",
          "keywords": [
            "hover",
            "scale",
            "zoom",
            "transform"
          ],
          "detector": "image-hover-transform"
        },
        {
          "id": "portfolio-eyebrow",
          "description": "Tracked uppercase eyebrow labels sit above the name and the section heading.",
          "selector": ".eyebrow",
          "evidence": "<span class=\"eyebrow\">Portfolio</span>",
          "keywords": [
            "eyebrow",
            "kicker",
            "uppercase",
            "label above",
            "letter-spacing"
          ],
          "detector": "hero-eyebrow-chip"
        },
        {
          "id": "portfolio-hard-shadow",
          "description": "The cards use a hard offset black shadow (6px 6px 0), unrelated to the coastal work.",
          "selector": ".card",
          "evidence": "box-shadow: 6px 6px 0 #000000",
          "keywords": [
            "shadow",
            "offset",
            "brutalist",
            "box-shadow",
            "6px"
          ],
          "detector": null
        },
        {
          "id": "portfolio-system-display-face",
          "description": "The display name is set in Impact, a system face.",
          "selector": "h1",
          "evidence": "font-family: Impact",
          "keywords": [
            "Impact",
            "font",
            "typeface",
            "display",
            "system font"
          ],
          "detector": null
        },
        {
          "id": "portfolio-caption-contrast",
          "description": "The captions, #aaaaaa on white, are about 2.3:1.",
          "selector": ".caption",
          "evidence": "color: #aaaaaa",
          "keywords": [
            "caption",
            "contrast",
            "#aaaaaa",
            "gray"
          ],
          "detector": "low-contrast"
        }
      ]
    }
  ]
}
```

- [ ] **Step 3: Write the failing fixture and tone tests**

`tests/wording-experiment/fixtures.test.mjs`:

```js
import { it } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { ENGINE_BIN, EXPERIMENT_DIR } from './config.mjs';

const fixtures = path.join(EXPERIMENT_DIR, 'fixtures');
const defects = JSON.parse(fs.readFileSync(path.join(fixtures, 'defects.json'), 'utf8'));

it('five pages, one per surface kind, carry five or six planted defects each', () => {
  assert.deepEqual(defects.pages.map((page) => page.id), ['landing', 'dashboard', 'form', 'docs', 'portfolio']);
  for (const page of defects.pages) assert.ok(page.defects.length >= 5 && page.defects.length <= 6, page.id);
  const ids = defects.pages.flatMap((page) => page.defects.map((defect) => defect.id));
  assert.equal(new Set(ids).size, ids.length, 'defect ids are unique');
});

it('every planted defect is present in its page source', () => {
  for (const page of defects.pages) {
    const html = fs.readFileSync(path.join(fixtures, page.file), 'utf8');
    for (const defect of page.defects) assert.ok(html.includes(defect.evidence), `${defect.id}: evidence not in ${page.file}`);
  }
});

it('each page ships product context long enough to keep init out of the lane', () => {
  for (const page of defects.pages) {
    assert.match(page.productMd, /<!-- impeccable:product-schema 1 -->/);
    assert.match(page.productMd, /## Platform\nweb/);
    assert.ok(page.productMd.length > 200, page.id);
  }
});

it('the detector flags exactly the defects recorded as detector-visible', () => {
  assert.ok(fs.existsSync(ENGINE_BIN), `engine binary missing at ${ENGINE_BIN}`);
  for (const page of defects.pages) {
    const run = spawnSync(ENGINE_BIN, ['detect', '--json', '--platform', 'web', path.join(fixtures, page.file)], { encoding: 'utf8' });
    assert.ok(run.status === 0 || run.status === 2, `detect exit ${run.status}: ${run.stderr}`);
    const rules = new Set(JSON.parse(run.stdout).map((finding) => finding.antipattern));
    for (const defect of page.defects.filter((entry) => entry.detector)) {
      assert.ok(rules.has(defect.detector), `${defect.id}: detector no longer reports ${defect.detector}`);
    }
  }
});
```

`tests/wording-experiment/tone.test.mjs`:

```js
import { it } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { MockLanguageModelV3 } from 'ai/test';
import { REPO_ROOT } from './config.mjs';
import { openLedger } from './cost.mjs';
import { openStore } from './store.mjs';
import { stubModel } from './stub-model.mjs';
import { runToneLane } from './lanes/tone.mjs';

function setup() {
  const out = fs.mkdtempSync(path.join(os.tmpdir(), 'wording-tone-'));
  return { store: openStore(path.join(out, 'store.jsonl')), ledger: openLedger(path.join(out, 'cost.jsonl'), 40) };
}
const lane = (extra) => ({ arms: ['original'], armDirs: { original: REPO_ROOT }, models: ['stub-model'], dryRun: true, ...extra });

it('a critique stores what the user saw and stops at its closing question; an audit runs to completion', async () => {
  const { store, ledger } = setup();
  await runToneLane(lane({ store, ledger, limit: 2, modelFor: () => stubModel('tone') }));
  const critique = store.all({ item: 'landing/critique' })[0];
  const audit = store.all({ item: 'landing/audit' })[0];
  assert.equal(critique.status, 'ok');
  assert.equal(critique.data.outcome, 'checkpoint');
  assert.match(critique.data.userText, /Priority issues/);
  assert.match(critique.data.userText, /Question: Which issue first\?/);
  assert.equal(audit.data.outcome, 'complete');
  assert.doesNotMatch(audit.data.userText, /Question:/);
  assert.ok(ledger.spent() > 0);
  assert.equal(store.all().length, 2);
});

const failing = () => new MockLanguageModelV3({ modelId: 'stub', doGenerate: async () => { throw new Error('socket hang up'); } });

it('a first item failing twice stops the lane and records nothing, so a bad model id can be fixed and rerun', async () => {
  const { store, ledger } = setup();
  await assert.rejects(runToneLane(lane({ store, ledger, limit: 1, modelFor: failing })), /nothing was recorded/);
  assert.equal(store.all().length, 0);
  assert.equal(ledger.spent(), 0);
  assert.equal(ledger.unpriced(), 2);
});

it('a later failure in a group that already succeeded is retried once, then stored as missing and never priced', async () => {
  const { store, ledger } = setup();
  store.append({ lane: 'tone', arm: 'original', model: 'stub-model', item: 'landing/critique', status: 'ok', data: {} });
  await runToneLane(lane({ store, ledger, limit: 1, modelFor: failing }));
  const record = store.all({ item: 'landing/audit' })[0];
  assert.equal(record.status, 'missing');
  assert.match(record.data.error, /socket hang up/);
  assert.equal(ledger.spent(), 0);
  assert.equal(ledger.unpriced(), 2);
});

it('a question asked before the report does not end the critique', async () => {
  const { store, ledger } = setup();
  const question = (id) => ({ type: 'tool-call', toolCallId: id, toolName: 'ask_user_question',
    input: JSON.stringify({ questions: [{ question: 'Who is this page for?', options: [{ label: 'Buyers' }] }] }) });
  const usage = { inputTokens: { total: 1000 }, outputTokens: { total: 200 } };
  const asksFirst = () => new MockLanguageModelV3({
    modelId: 'stub',
    doGenerate: async ({ prompt }) => {
      const answered = prompt.some((message) => message.role === 'tool');
      const content = answered
        ? [{ type: 'text', text: '## Priority issues\n1. Contrast on the hero copy.' }, question('closing')]
        : [question('opening')];
      return { content, finishReason: { unified: 'tool-calls', raw: 'tool-calls' }, usage, warnings: [] };
    },
  });
  await runToneLane(lane({ store, ledger, limit: 1, modelFor: asksFirst }));
  const critique = store.all({ item: 'landing/critique' })[0];
  assert.equal(critique.status, 'ok');
  assert.equal(critique.data.questions, 2);
  assert.equal(critique.data.reportSeen, true);
  assert.equal(critique.data.outcome, 'checkpoint');
});
```

The mock's `usage` and `finishReason` shapes must match the ones `stub-model.mjs` uses (`USAGE`, `finish()`); if the SDK rejects them, copy those two values from `stub-model.mjs` rather than changing the assertion.

- [ ] **Step 4: Run and watch them fail**

Run: `node --test tests/wording-experiment/fixtures.test.mjs tests/wording-experiment/tone.test.mjs`
Expected: `fixtures.test.mjs` passes once Steps 1 and 2 are written (the fixtures are data; this confirms them); `tone.test.mjs` fails at import with `ERR_MODULE_NOT_FOUND` for `./lanes/tone.mjs`.

- [ ] **Step 5: Write `lanes/tone.mjs`**

```js
// Lane 2: /impeccable critique and /impeccable audit on each fixture page, through the arm's own harness.
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { ENGINE_BIN, EXPERIMENT_DIR } from '../config.mjs';
import { chargeTurn } from '../cost.mjs';
import { assertNotSystematic, withOneRetry } from '../store.mjs';

export const TONE_COMMANDS = ['critique', 'audit'];
export const TONE_MAX_STEPS = 30;
export const TONE_TIMEOUT_MS = 600_000;
export const TONE_ENVIRONMENT = 'No subagent, browser, or image-generation tools are available in this session. The workspace holds one static page, index.html, and its PRODUCT.md.';
// Decision 37: the same quiet engine in every arm.
export const TONE_ENV = { DO_NOT_TRACK: '1', IMPECCABLE_NO_TELEMETRY: '1', IMPECCABLE_NO_UPDATE_CHECK: '1', IMPECCABLE_NO_STALENESS_CHECK: '1' };
// Decision 36: headings of the critique and audit report templates; a question counts as closing only after one appears.
export const REPORT_MARKER = /priority issues|design health score|audit health score/i;
const reportWritten = (trace) => REPORT_MARKER.test((trace.assistantTexts ?? []).join('\n'));

export function loadDefects() {
  return JSON.parse(fs.readFileSync(path.join(EXPERIMENT_DIR, 'fixtures', 'defects.json'), 'utf8'));
}

// Decision 10: the harness reads IMPECCABLE_BIN when it loads, and a worktree has no binary of its own.
export async function loadArmHarness(armDir) {
  process.env.IMPECCABLE_BIN ||= ENGINE_BIN;
  return import(pathToFileURL(path.join(armDir, 'tests', 'skill-behavior', 'harness.mjs')).href);
}

// Decision 28: the user sees every step's text and every question asked, not only the last step.
export function userFacingText(trace) {
  const texts = (trace.assistantTexts ?? []).map((text) => text.trim()).filter(Boolean);
  const questions = trace.questionCalls.flatMap((call) => call.questions ?? []).map((question) => {
    const options = question.options?.length ? ` Options: ${question.options.map((option) => option.label).join(' | ')}` : '';
    return `Question: ${question.question}${options}`;
  });
  return [...texts, ...questions].join('\n\n');
}

export function compactTrace(harness, trace) {
  const { totalCalls, byName, bashCommands, readPaths, writePaths } = harness.summarizeTrace(trace);
  return { totalCalls, byName, bashCommands, readPaths, writePaths };
}

async function runToneItem({ harness, page, command, llm }) {
  const workspace = harness.prepareWorkspace({ files: {
    'index.html': fs.readFileSync(path.join(EXPERIMENT_DIR, 'fixtures', page.file), 'utf8'),
    'PRODUCT.md': page.productMd,
  } });
  try {
    const result = await harness.runTurn({
      workspace, model: llm, userPrompt: `/impeccable ${command} index.html`,
      maxSteps: TONE_MAX_STEPS, timeoutMs: TONE_TIMEOUT_MS, env: TONE_ENV, environment: TONE_ENVIRONMENT,
      stopAfter: (trace) => trace.questionCalls.length > 0 && reportWritten(trace),
    });
    return {
      userText: userFacingText(result.trace), finalText: result.text, outcome: result.outcome, steps: result.steps,
      questions: result.trace.questionCalls.length, reportSeen: reportWritten(result.trace),
      trace: compactTrace(harness, result.trace), usages: result.stepUsages ?? [result.usage],
    };
  } finally {
    harness.cleanupWorkspace(workspace);
  }
}

export async function runToneLane({ arms, armDirs, models, store, ledger, dryRun = false, modelFor, limit = Infinity }) {
  const { pages } = loadDefects();
  for (const arm of arms) {
    const harness = await loadArmHarness(armDirs[arm]);
    for (const model of models) {
      let started = 0;
      for (const page of pages) {
        for (const command of TONE_COMMANDS) {
          const key = { lane: 'tone', arm, model, item: `${page.id}/${command}` };
          if (store.has(key) || started >= limit) continue;
          started++;
          const outcome = await withOneRetry(async (attempt) => {
            ledger.assertCanSpend(`tone ${arm} ${model} ${key.item} (attempt ${attempt})`);
            try {
              return await runToneItem({ harness, page, command, llm: modelFor(model, 'tone') });
            } catch (error) {
              ledger.chargeUnpriced({ ...key, reason: error.message });
              throw error;
            }
          }, { isFatal: (error) => error.name === 'CapExceeded' });
          if (!outcome.ok) {
            assertNotSystematic(store, key, outcome.error);
            store.append({ ...key, status: 'missing', data: { error: outcome.error, firstError: outcome.firstError } });
            continue;
          }
          const { usages, ...data } = outcome.value;
          const usd = chargeTurn(ledger, key, dryRun ? 'stub' : model, usages);
          store.append({ ...key, status: 'ok', data: { ...data, usd, retried: outcome.retried } });
        }
      }
    }
  }
}
```

- [ ] **Step 6: Run green**

Run: `node --test tests/wording-experiment/fixtures.test.mjs tests/wording-experiment/tone.test.mjs`
Expected: 8 tests pass (4 fixture, 4 tone). The tone tests use the main checkout's harness (`REPO_ROOT`), which carries Task 1's `stepUsages`.

- [ ] **Step 6b: Probe the boot output for question-inviting directives (free)**

The critique stop rule (Decision 36) assumes the only questions a critique asks come from the skill text, not from the engine. Check the boot output the lane's workspaces will see:

```bash
cd /Volumes/Containers/impeccable
probe=$(mktemp -d)
node -e "const d=JSON.parse(require('fs').readFileSync('tests/wording-experiment/fixtures/defects.json','utf8')).pages[0]; require('fs').writeFileSync(process.argv[1]+'/PRODUCT.md', d.productMd)" "$probe"
cp tests/wording-experiment/fixtures/pages/landing.html "$probe/index.html"
(cd "$probe" && env DO_NOT_TRACK=1 IMPECCABLE_NO_TELEMETRY=1 IMPECCABLE_NO_UPDATE_CHECK=1 IMPECCABLE_NO_STALENESS_CHECK=1 /Volumes/Containers/impeccable/skill/scripts/impeccable context > "$probe/context.txt" 2>&1; echo "exit $?")
rtk proxy grep -n -i -E "ask the user|ask_user|NO_PRODUCT|init|UPDATE_AVAILABLE|CONTEXT_STALE|WARNING" "$probe/context.txt"
rm -rf "$probe"
```

Expected: `exit 0`, and no `NO_PRODUCT`, `UPDATE_AVAILABLE`, `CONTEXT_STALE` or `WARNING` lines. Lines that only name the word `init` inside a command list are fine. If a directive tells the agent to run `init` or to ask the user something before the report, stop and report it: every critique would then open with a question the fixtures did not intend, and `questions` in the report would measure the engine rather than the wording.

- [ ] **Step 7: Register and commit**

Append `'tests/wording-experiment/fixtures.test.mjs',` and `'tests/wording-experiment/tone.test.mjs',` to the suite's `files` list, then:

```bash
cd /Volumes/Containers/impeccable
node --test tests/test-suites.test.mjs
git add scripts/test-suites.mjs tests/wording-experiment/fixtures/pages tests/wording-experiment/fixtures/defects.json tests/wording-experiment/lanes/tone.mjs tests/wording-experiment/fixtures.test.mjs tests/wording-experiment/tone.test.mjs
git commit -m "wording-experiment: lane 2 fixtures and tone lane" -m "Five fixture pages with 29 planted defects recorded in defects.json (evidence string, selector, judge keywords, and the engine rule that flags each, 12 of 29); the tone lane runs critique and audit on each page through the arm's own harness and stores the text the user saw, stopping at critique's closing question." -m "AI assistance: drafted with Claude Code."
```

**Checkpoint:** lane 2 runs end to end against a stub; fixture ground truth is pinned against the detector.

---

## Task 9: Lane 3 briefs and the builds lane

**Files:**
- Create: `tests/wording-experiment/fixtures/briefs.json`
- Create: `tests/wording-experiment/lanes/builds.mjs`
- Test: `tests/wording-experiment/builds.test.mjs`
- Modify: `scripts/test-suites.mjs` (append `'tests/wording-experiment/builds.test.mjs'`)

**Interfaces:**
- Consumes: `loadArmHarness`, `compactTrace` from `lanes/tone.mjs`; `chargeTurn`; `withOneRetry`; `config.mjs` (`CATALOG_DIR`, `ENGINE_BIN`, `EXPERIMENT_DIR`, `modelSlug`); Playwright `chromium`; `engineEnv` from `tests/lib/engine-bin.mjs` (test only).
- Produces: `briefs.json` as an array of `{ id, mode: 'Persuade'|'Operate'|'Read'|'Experience', seed: <8 hex>, surface, productMd }`.
- Produces (`lanes/builds.mjs`): `BUILD_MAX_STEPS = 40`, `BUILD_TIMEOUT_MS = 840000`, `VIEWPORTS`, `MAX_SHOT_HEIGHT = 3000`, `BUILD_ENVIRONMENT`, `loadBriefs()`, `buildPrompt(brief)`, `buildEnv(brief)`, `artifactDir(out, arm, model, briefId)`, `screenshotPage(browser, htmlPath, outDir) -> { desktop, mobile, <name>Warning? }`, `runDetect(htmlPath) -> { count, byRule } | { error }`, `runBuildsLane({ arms, armDirs, models, store, ledger, out, dryRun, modelFor, limit })`.
- Store records: `{ lane: 'builds', arm, model, item: <briefId>, status, data: { outcome, steps, finalText, trace, artifact, shots?, shotError?, detect?, usd, retried } }`; the page and screenshots live in `<out>/builds/<arm>/<modelSlug>/<briefId>/`.

- [ ] **Step 1: Make sure Chromium launches (free)**

Run: `node --input-type=module -e "import { chromium } from 'playwright'; const b = await chromium.launch({ headless: true }); console.log('chromium', b.version()); await b.close();"`
Expected: `chromium <version>`. If it fails with a missing executable (the cache holds revision 1208, `playwright` 1.63 wants 1243), run `npx playwright install chromium` and rerun until it prints the version.

- [ ] **Step 2: Write `tests/wording-experiment/fixtures/briefs.json` exactly as below**

Seeds are fixed per brief and shared by all arms (Decision 22). On 2026-09-26, `concept-seed --scope direction --mode persuade` with seed `5eed0101` and the local catalog produced byte-identical output on two runs (`source: local`).

```json
[
  {
    "id": "tidewell-landing",
    "mode": "Persuade",
    "seed": "5eed0101",
    "surface": "a landing page",
    "productMd": "# Tidewell\n\n<!-- impeccable:product-schema 1 -->\n\n## Platform\nweb\n\n## Users\nWeekend sailors on tidal coasts who plan short passages around harbor bars and shallow channels, usually the evening before, on a phone.\n\n## Product Purpose\nTidewell turns local tide tables into sailing windows: when the channel carries enough water for a given keel, and when to leave so the ebb carries the boat out and the flood brings it home. The landing page should get a sailor to install the app.\n\n## Positioning\nA passage planner built around the tide rather than the weather forecast.\n\n## Brand Commitments\nPlain, seamanlike language. No invented user counts, ratings, or testimonials.\n\n## Evidence on Hand\nThree features: tide windows per keel depth, departure timing for ebb and flood, private harbor notes. Free on iOS and Android.\n"
  },
  {
    "id": "kilnlog-pricing",
    "mode": "Persuade",
    "seed": "5eed0102",
    "surface": "a pricing page",
    "productMd": "# Kilnlog\n\n<!-- impeccable:product-schema 1 -->\n\n## Platform\nweb\n\n## Users\nOwners of small pottery studios (one to six kilns) who log firing schedules on paper today.\n\n## Product Purpose\nKilnlog records every firing (cone, ramp segments, hold times, results) and predicts element wear. The pricing page should let a studio owner pick a plan without a sales call.\n\n## Positioning\nThe firing log a potter would keep if paper could chart itself.\n\n## Evidence on Hand\nPlans: Solo, free, one kiln; Studio, 12 dollars a month, up to four kilns and shared logs; Collective, 29 dollars a month, unlimited kilns, member accounts, and export. Annual billing saves two months.\n\n## Brand Commitments\nWarm and exact; no fake urgency, no \"most popular\" badge unless a plan truly is.\n"
  },
  {
    "id": "fernhouse-campaign",
    "mode": "Persuade",
    "seed": "5eed0103",
    "surface": "a campaign page for the autumn plant swap",
    "productMd": "# Fernhouse\n\n<!-- impeccable:product-schema 1 -->\n\n## Platform\nweb\n\n## Users\nNeighbors in a mid-sized city who grow houseplants and want to trade cuttings instead of buying new plants.\n\n## Product Purpose\nFernhouse is a volunteer-run plant library. The campaign page should get people to register a swap table for the autumn swap on Saturday 18 October, 10:00 to 14:00, at the Old Tram Depot.\n\n## Evidence on Hand\nLast spring 140 people came and about 900 cuttings changed hands. Registration closes 12 October. Tables are free.\n\n## Brand Commitments\nCommunity notice board energy; hand-made, never corporate.\n"
  },
  {
    "id": "berthboard-dashboard",
    "mode": "Operate",
    "seed": "5eed0201",
    "surface": "the night-shift berth allocation dashboard",
    "productMd": "# Berthboard\n\n<!-- impeccable:product-schema 1 -->\n\n## Platform\nweb\n\n## Users\nNight-shift harbor dispatchers who assign berths to arriving vessels from a dim control room with constant radio traffic.\n\n## Product Purpose\nShow tonight's berth assignments on a timeline, surface conflicts before a vessel arrives, and let the dispatcher reassign or hold a vessel in one action.\n\n## Operating Context\nEight-hour shifts on a desktop monitor at arm's length, glanced at between radio calls.\n\n## Evidence on Hand\nSynthetic sample data is fine: six berths, about twelve vessels between 01:00 and 07:00, one conflict on berth 4. Label it as sample data.\n"
  },
  {
    "id": "ledgerline-settings",
    "mode": "Operate",
    "seed": "5eed0202",
    "surface": "the account settings page",
    "productMd": "# Ledgerline\n\n<!-- impeccable:product-schema 1 -->\n\n## Platform\nweb\n\n## Users\nFreelancers who use Ledgerline to send invoices and track what clients owe them.\n\n## Product Purpose\nThe settings page covers profile and business details, invoice defaults (currency, payment terms, numbering), reminder emails, connected bank account, and account deletion.\n\n## Operating Context\nVisited rarely, usually to change one thing; errors here cost money, so every destructive action needs a clear confirmation.\n\n## Brand Commitments\nCalm and trustworthy; plain labels over clever ones.\n"
  },
  {
    "id": "proofroom-rota",
    "mode": "Operate",
    "seed": "5eed0203",
    "surface": "the weekly shift rota editor",
    "productMd": "# Proofroom\n\n<!-- impeccable:product-schema 1 -->\n\n## Platform\nweb\n\n## Users\nThe head baker of a neighborhood bakery who plans shifts for nine staff every Thursday.\n\n## Product Purpose\nBuild next week's rota: shifts from 04:00 to 20:00, seven days, with each person's availability and contracted hours visible while assigning, and a warning when someone goes over their hours or a day is understaffed.\n\n## Operating Context\nA laptop on a flour-dusted counter, often with one hand free.\n\n## Evidence on Hand\nSynthetic staff names and hours are fine; label them as sample data.\n"
  },
  {
    "id": "driftctl-docs",
    "mode": "Read",
    "seed": "5eed0301",
    "surface": "the getting-started documentation page",
    "productMd": "# driftctl\n\n<!-- impeccable:product-schema 1 -->\n\n## Platform\nweb\n\n## Users\nPlatform engineers adopting driftctl to find infrastructure that changed outside their infrastructure code.\n\n## Product Purpose\nGet a new user from download to a first useful scan in ten minutes: install, authenticate a provider, run a first scan, read the report, and ignore expected drift.\n\n## Operating Context\nRead beside a terminal, often on a wide monitor; code samples are copied into the shell.\n\n## Evidence on Hand\nCommands: driftctl version; driftctl scan --from tfstate://terraform.tfstate; driftctl scan --output json://report.json; ignore rules live in .driftignore.\n"
  },
  {
    "id": "upland-waters-article",
    "mode": "Read",
    "seed": "5eed0302",
    "surface": "a long-form article page for the essay \"What the spring melt told us\"",
    "productMd": "# Upland Waters\n\n<!-- impeccable:product-schema 1 -->\n\n## Platform\nweb\n\n## Users\nReaders of a small watershed research group's journal: hydrologists, local water boards, and interested residents.\n\n## Product Purpose\nPublish field essays that explain a season of measurements in plain language, with figures a reader can trust.\n\n## Evidence on Hand\nThe essay text is not written yet. Write a clearly labeled synthetic draft of about 600 words about snowmelt timing at three gauging stations, with one small data table, also labeled synthetic.\n\n## Brand Commitments\nScholarly but warm; figures are never decorative.\n"
  },
  {
    "id": "kilnlog-changelog",
    "mode": "Read",
    "seed": "5eed0303",
    "surface": "the product changelog page",
    "productMd": "# Kilnlog\n\n<!-- impeccable:product-schema 1 -->\n\n## Platform\nweb\n\n## Users\nPottery studio owners who use Kilnlog and want to know what changed and whether it affects their firings.\n\n## Product Purpose\nList releases newest first; each entry says what changed, who it affects, and any action needed.\n\n## Evidence on Hand\nSynthetic releases are fine and must be labeled as sample content: 2.4 (element wear prediction), 2.3 (shared logs for Studio plans), 2.2 (cone chart export), 2.1.1 (fixed a ramp rounding bug).\n\n## Brand Commitments\nWarm and exact.\n"
  },
  {
    "id": "mara-quell-portfolio",
    "mode": "Experience",
    "seed": "5eed0401",
    "surface": "the portfolio home page",
    "productMd": "# Mara Quell\n\n<!-- impeccable:product-schema 1 -->\n\n## Platform\nweb\n\n## Users\nMagazine editors and gallery curators deciding whether to commission a coastal photographer.\n\n## Product Purpose\nShow a tight selection of Mara Quell's coastal work so an editor can judge her eye in under a minute, then reach her.\n\n## Evidence on Hand\nNo image files are available in this workspace. Represent the six works with honest, labeled placeholders (title, place, year) rather than stock imagery: Breakwater at low water, Porthleven, 2019; Slate quay after rain, Port Isaac, 2021; Net lofts in winter light, Mousehole, 2022; Tide pool, Kynance, 2023; Harbor at 5 a.m., Newlyn, 2024; Salt on glass, St Ives, 2025.\n\n## Brand Commitments\nThe work leads; the page stays out of its way.\n"
  },
  {
    "id": "northmark-specimen",
    "mode": "Experience",
    "seed": "5eed0402",
    "surface": "the type specimen showcase page for the typeface Harrow Grotesk",
    "productMd": "# Northmark Type\n\n<!-- impeccable:product-schema 1 -->\n\n## Platform\nweb\n\n## Users\nGraphic designers and art directors choosing a typeface for an identity or an editorial system.\n\n## Product Purpose\nShow Harrow Grotesk in use at display and text sizes so a designer can decide to license it.\n\n## Evidence on Hand\nThe font files are not available here; use a similar freely available grotesk from a web font service as a stand-in and label it as a stand-in. Weights: Light, Regular, Medium, Bold. Features: tabular figures, a single-story g alternate, small caps.\n\n## Brand Commitments\nThe letterforms are the product.\n"
  },
  {
    "id": "estuary-sound-archive",
    "mode": "Experience",
    "seed": "5eed0403",
    "surface": "the gallery page for the field recording collection",
    "productMd": "# Estuary Sound Archive\n\n<!-- impeccable:product-schema 1 -->\n\n## Platform\nweb\n\n## Users\nSound artists, radio producers, and naturalists browsing field recordings of one river estuary.\n\n## Product Purpose\nLet a visitor wander the collection by place and season and listen; each recording has a title, location, date, duration, and a short note.\n\n## Evidence on Hand\nNo audio files are available; controls may be non-functional but must look and behave like real players. Use eight synthetic recordings, labeled as sample entries.\n\n## Brand Commitments\nQuiet, patient, attentive to place.\n"
  }
]
```

- [ ] **Step 3: Write the failing tests**

`tests/wording-experiment/builds.test.mjs`:

```js
import { it } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { chromium } from 'playwright';
import { MockLanguageModelV3 } from 'ai/test';
import { engineEnv } from '../lib/engine-bin.mjs';
import { ENGINE_BIN, REPO_ROOT } from './config.mjs';
import { openLedger } from './cost.mjs';
import { openStore } from './store.mjs';
import { stubModel } from './stub-model.mjs';
import { buildEnv, loadBriefs, runBuildsLane } from './lanes/builds.mjs';

const pngSize = (file) => {
  const bytes = fs.readFileSync(file);
  return { width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) };
};
function setup() {
  const out = fs.mkdtempSync(path.join(os.tmpdir(), 'wording-builds-'));
  return { out, store: openStore(path.join(out, 'store.jsonl')), ledger: openLedger(path.join(out, 'cost.jsonl'), 40) };
}
const lane = (extra) => ({ arms: ['original'], armDirs: { original: REPO_ROOT }, models: ['stub-model'], dryRun: true, limit: 1, ...extra });

it('Chromium launches for the screenshots', async () => {
  let browser;
  try {
    browser = await chromium.launch({ headless: true });
  } catch (error) {
    throw new Error(`Chromium did not launch; run: npx playwright install chromium (${error.message.split('\n')[0]})`);
  }
  await browser.close();
});

it('twelve briefs, three per mode, each with its own seed and product context', () => {
  const briefs = loadBriefs();
  assert.equal(briefs.length, 12);
  for (const mode of ['Persuade', 'Operate', 'Read', 'Experience']) {
    assert.equal(briefs.filter((brief) => brief.mode === mode).length, 3, mode);
  }
  assert.equal(new Set(briefs.map((brief) => brief.seed)).size, 12);
  assert.equal(new Set(briefs.map((brief) => brief.id)).size, 12);
  for (const brief of briefs) {
    assert.match(brief.seed, /^[0-9a-f]{8}$/);
    assert.match(brief.productMd, /<!-- impeccable:product-schema 1 -->/);
    assert.ok(brief.productMd.length > 200, brief.id);
  }
});

it('a brief seed deals the same concept roll every time, from the local catalog', () => {
  const brief = loadBriefs()[0];
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'wording-seed-'));
  fs.writeFileSync(path.join(dir, 'PRODUCT.md'), brief.productMd);
  const roll = () => spawnSync(ENGINE_BIN, ['concept-seed', '--scope', 'direction', '--mode', brief.mode.toLowerCase()],
    { cwd: dir, encoding: 'utf8', env: engineEnv(ENGINE_BIN, buildEnv(brief)) });
  const first = roll();
  const second = roll();
  assert.equal(first.status, 0, first.stderr);
  assert.equal(first.stdout, second.stdout);
  assert.match(first.stdout, new RegExp(`key: ${brief.seed}; mode: ${brief.mode.toLowerCase()}; source: local`));
});

it('a finished build is saved with desktop and mobile screenshots and its detector findings', async () => {
  const { out, store, ledger } = setup();
  await runBuildsLane(lane({ out, store, ledger, modelFor: () => stubModel('build') }));
  const record = store.all({ lane: 'builds' })[0];
  assert.equal(record.item, loadBriefs()[0].id);
  assert.equal(record.data.artifact, true);
  assert.equal(pngSize(record.data.shots.desktop).width, 1440);
  assert.ok(pngSize(record.data.shots.desktop).height <= 3000);
  assert.equal(pngSize(record.data.shots.mobile).width, 390);
  assert.ok(record.data.detect.byRule['gradient-text'] >= 1, JSON.stringify(record.data.detect));
  assert.ok(ledger.spent() > 0);
});

it('a turn that writes no page is stored with artifact false and no screenshots', async () => {
  const { out, store, ledger } = setup();
  const talker = () => new MockLanguageModelV3({ modelId: 'stub', doGenerate: async () => ({
    content: [{ type: 'text', text: 'I would rather describe the page than build it.' }],
    finishReason: { unified: 'stop', raw: 'stop' },
    usage: { inputTokens: { total: 10 }, outputTokens: { total: 5 } }, warnings: [],
  }) });
  await runBuildsLane(lane({ out, store, ledger, modelFor: talker }));
  const record = store.all({ lane: 'builds' })[0];
  assert.equal(record.status, 'ok');
  assert.equal(record.data.artifact, false);
  assert.equal(record.data.shots, undefined);
});
```

- [ ] **Step 4: Run and watch them fail**

Run: `node --test tests/wording-experiment/builds.test.mjs`
Expected: fails at import with `ERR_MODULE_NOT_FOUND` for `./lanes/builds.mjs`.

- [ ] **Step 5: Write `lanes/builds.mjs`**

```js
// Lane 3: one index.html per brief through the arm's own harness, then screenshots and the detector.
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
import { chromium } from 'playwright';
import { CATALOG_DIR, ENGINE_BIN, EXPERIMENT_DIR, modelSlug } from '../config.mjs';
import { chargeTurn } from '../cost.mjs';
import { assertNotSystematic, withOneRetry } from '../store.mjs';
import { compactTrace, loadArmHarness } from './tone.mjs';

export const BUILD_MAX_STEPS = 40;
export const BUILD_TIMEOUT_MS = 840_000;
export const VIEWPORTS = [['desktop', 1440, 900], ['mobile', 390, 844]];
export const MAX_SHOT_HEIGHT = 3000;
export const BUILD_ENVIRONMENT = 'No subagent, browser, or image-generation tools are available in this session. The workspace has no image files.';

export function loadBriefs() {
  return JSON.parse(fs.readFileSync(path.join(EXPERIMENT_DIR, 'fixtures', 'briefs.json'), 'utf8'));
}

export function buildPrompt(brief) {
  return `Build ${brief.surface} for the product in PRODUCT.md. Deliver one self-contained index.html in the workspace root with inline CSS and JavaScript: no build step, no image files (none exist), and no network requests except web fonts. Do not ask me questions; make each decision yourself and list your assumptions in your final message.`;
}

// Decisions 22 and 37: the same seeds and the local catalog in every arm, telemetry and engine chatter off.
export function buildEnv(brief) {
  return {
    DO_NOT_TRACK: '1',
    IMPECCABLE_NO_TELEMETRY: '1',
    IMPECCABLE_NO_UPDATE_CHECK: '1',
    IMPECCABLE_NO_STALENESS_CHECK: '1',
    IMPECCABLE_CATALOG_DIR: CATALOG_DIR,
    IMPECCABLE_CONCEPT_SEED: brief.seed,
    IMPECCABLE_PALETTE_SEED: brief.seed,
  };
}

export const artifactDir = (out, arm, model, briefId) => path.join(out, 'builds', arm, modelSlug(model), briefId);

export async function screenshotPage(browser, htmlPath, outDir) {
  const shots = {};
  for (const [name, width, height] of VIEWPORTS) {
    const page = await browser.newPage({ viewport: { width, height } });
    try {
      try {
        await page.goto(pathToFileURL(htmlPath).href, { waitUntil: 'load', timeout: 30_000 });
      } catch (error) {
        shots[`${name}Warning`] = `load did not finish: ${error.message.split('\n')[0]}`;
      }
      await page.waitForTimeout(800);
      const full = await page.evaluate(() => document.documentElement.scrollHeight);
      const file = path.join(outDir, `${name}.png`);
      await page.screenshot({ path: file, fullPage: true,
        clip: { x: 0, y: 0, width, height: Math.min(Math.max(full, height), MAX_SHOT_HEIGHT) } });
      shots[name] = file;
    } finally {
      await page.close();
    }
  }
  return shots;
}

export function runDetect(htmlPath) {
  const result = spawnSync(ENGINE_BIN, ['detect', '--json', '--platform', 'web', htmlPath], { encoding: 'utf8', timeout: 120_000, maxBuffer: 16 * 1024 * 1024 });
  if (result.status !== 0 && result.status !== 2) return { error: `detect exit ${result.status}: ${(result.stderr || '').slice(0, 500)}` };
  const findings = JSON.parse(result.stdout || '[]');
  const byRule = {};
  for (const finding of findings) byRule[finding.antipattern] = (byRule[finding.antipattern] ?? 0) + 1;
  return { count: findings.length, byRule };
}

async function runBuildItem({ harness, brief, llm, browser, dir }) {
  const workspace = harness.prepareWorkspace({ files: { 'PRODUCT.md': brief.productMd } });
  try {
    const result = await harness.runTurn({
      workspace, model: llm, userPrompt: buildPrompt(brief),
      maxSteps: BUILD_MAX_STEPS, timeoutMs: BUILD_TIMEOUT_MS, env: buildEnv(brief), environment: BUILD_ENVIRONMENT,
    });
    const html = path.join(workspace, 'index.html');
    const data = {
      outcome: result.outcome, steps: result.steps, finalText: result.text,
      trace: compactTrace(harness, result.trace), usages: result.stepUsages ?? [result.usage],
      artifact: fs.existsSync(html),
    };
    if (data.artifact) {
      fs.rmSync(dir, { recursive: true, force: true });
      fs.mkdirSync(dir, { recursive: true });
      const saved = path.join(dir, 'index.html');
      fs.copyFileSync(html, saved);
      // The turn is already paid for: a screenshot or detector failure is recorded, never retried.
      try {
        data.shots = await screenshotPage(browser, saved, dir);
      } catch (error) {
        data.shotError = error.message.split('\n')[0];
      }
      data.detect = runDetect(saved);
    }
    return data;
  } finally {
    harness.cleanupWorkspace(workspace);
  }
}

export async function runBuildsLane({ arms, armDirs, models, store, ledger, out, dryRun = false, modelFor, limit = Infinity }) {
  const briefs = loadBriefs();
  const browser = await chromium.launch({ headless: true });
  try {
    for (const arm of arms) {
      const harness = await loadArmHarness(armDirs[arm]);
      for (const model of models) {
        let started = 0;
        for (const brief of briefs) {
          const key = { lane: 'builds', arm, model, item: brief.id };
          if (store.has(key) || started >= limit) continue;
          started++;
          const outcome = await withOneRetry(async (attempt) => {
            ledger.assertCanSpend(`builds ${arm} ${model} ${brief.id} (attempt ${attempt})`);
            try {
              return await runBuildItem({ harness, brief, llm: modelFor(model, 'build'), browser, dir: artifactDir(out, arm, model, brief.id) });
            } catch (error) {
              ledger.chargeUnpriced({ ...key, reason: error.message });
              throw error;
            }
          }, { isFatal: (error) => error.name === 'CapExceeded' });
          if (!outcome.ok) {
            assertNotSystematic(store, key, outcome.error);
            store.append({ ...key, status: 'missing', data: { error: outcome.error, firstError: outcome.firstError } });
            continue;
          }
          const { usages, ...data } = outcome.value;
          const usd = chargeTurn(ledger, key, dryRun ? 'stub' : model, usages);
          store.append({ ...key, status: 'ok', data: { ...data, usd, retried: outcome.retried } });
        }
      }
    }
  } finally {
    await browser.close();
  }
}
```

- [ ] **Step 6: Run green**

Run: `node --test tests/wording-experiment/builds.test.mjs`
Expected: 5 tests pass. If the seed test fails because two rolls differ, stop: Decision 22's premise is broken and the owner decides whether builds may vary by roll.

- [ ] **Step 7: Register and commit**

Append `'tests/wording-experiment/builds.test.mjs',` to the suite's `files` list, then:

```bash
cd /Volumes/Containers/impeccable
node --test tests/test-suites.test.mjs
git add scripts/test-suites.mjs tests/wording-experiment/fixtures/briefs.json tests/wording-experiment/lanes/builds.mjs tests/wording-experiment/builds.test.mjs
git commit -m "wording-experiment: lane 3 briefs and builds lane" -m "Twelve briefs, three per mode, each with its own PRODUCT.md and a fixed concept and palette seed shared across arms; the builds lane runs each through the arm's own harness, saves index.html, screenshots it at 1440 and 390 wide (clipped at 3000 px), and records the engine's detect findings." -m "AI assistance: drafted with Claude Code."
```

**Checkpoint:** lane 3 runs end to end against a stub, including screenshots and the detector; seeds are proven reproducible.

---

## Task 10: `judge.mjs`: blinding, order swap, rubric prompts, verdicts

**Files:**
- Create: `tests/wording-experiment/judge.mjs`
- Test: `tests/wording-experiment/judge.test.mjs`
- Modify: `scripts/test-suites.mjs` (append `'tests/wording-experiment/judge.test.mjs'`)

**Interfaces:**
- Consumes: store records from lanes 2 and 3; `loadDefects()`, `loadBriefs()`; `COMPARISONS`, `JUDGE_MODEL`; `chargeTurn`, `withOneRetry`; `generateObject` from `ai`; `z` from `zod`; `modelFor(modelId, 'judge')`.
- Produces: `LABEL_LEXICON`, `lexiconHits(text) -> { term, sentence }[]`, `ReportVerdict` and `PairVerdict` (zod schemas), `openBlindMap(file) -> { idFor(target), target(id), all() }`, `reportPrompt({ blindId, page, text, hits })`, `pairPrompt({ brief, firstId, secondId })`, `runJudgeLane({ store, ledger, out, dryRun, modelFor, limit, judgeModel })`.
- Store records: report verdicts `{ lane: 'judge', arm, model, item: 'report:<page>/<command>', data: { blindId, hits, verdict, usd } }`; pair verdicts `{ lane: 'judge', arm: '<a>-vs-<b>', model, item: 'pair:<briefId>#<order>', data: { firstArm, blindIds, verdict, usd } | { firstArm, auto: 'missing-artifact', verdict: { preference } } }`. The blind map (`<out>/judge/blind-map.json`) maps each random id to `{ kind, arm, model, item }`; arm names exist only in the store and the map, never in a prompt.

**Label lexicon (lane 2).** Words that, aimed at the work or its author, are judgmental labels: `slop`, `sloppy`, `lazy`, `generic`, `amateur`, `amateurish`, `cheap`, `cheap-looking`, `costume`, `lifeless`, `bland`, `boring`, `ugly`, `tacky`, `soulless`, `cookie-cutter`, `templated`, `template-y`, `cliché`, `cliche`, `clichéd`, `cliched`, `dated`, `clumsy`, `careless`, `mediocre`, `uninspired`, `forgettable`, `machine-made`, `AI-generated`, `AI slop`. The lexicon finds candidates (whole words, case-insensitive, not inside a longer word); the judge confirms each one.

- [ ] **Step 1: Write the failing tests**

`tests/wording-experiment/judge.test.mjs`:

```js
import { it } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { MockLanguageModelV3 } from 'ai/test';
import { ARMS } from './config.mjs';
import { openLedger } from './cost.mjs';
import { openStore } from './store.mjs';
import { stubVerdict } from './stub-model.mjs';
import { lexiconHits, runJudgeLane } from './judge.mjs';
import { loadBriefs } from './lanes/builds.mjs';

const PNG = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==', 'base64');

function recordingJudge() {
  const prompts = [];
  const model = new MockLanguageModelV3({ modelId: 'stub', doGenerate: async ({ prompt }) => {
    const parts = prompt.flatMap((message) => (typeof message.content === 'string' ? [{ type: 'text', text: message.content }] : message.content));
    const text = parts.filter((part) => part.type === 'text').map((part) => part.text).join('\n');
    prompts.push({ text, images: parts.filter((part) => part.type === 'file' || part.type === 'image').length });
    return {
      content: [{ type: 'text', text: JSON.stringify(stubVerdict(text)) }],
      finishReason: { unified: 'stop', raw: 'stop' },
      usage: { inputTokens: { total: 100 }, outputTokens: { total: 20 } }, warnings: [],
    };
  } });
  return { model, prompts };
}

function seed({ pagelessArm = null } = {}) {
  const out = fs.mkdtempSync(path.join(os.tmpdir(), 'wording-judge-'));
  const store = openStore(path.join(out, 'store.jsonl'));
  const brief = loadBriefs()[0];
  for (const arm of ARMS) {
    store.append({ lane: 'tone', arm, model: 'm1', item: 'landing/critique', status: 'ok',
      data: { userText: 'The hero gradient looks generic. The fine print fails contrast; darken it to #5f6b73.' } });
    const dir = path.join(out, 'builds', arm);
    fs.mkdirSync(dir, { recursive: true });
    const shots = {};
    for (const name of ['desktop', 'mobile']) {
      shots[name] = path.join(dir, `${name}.png`);
      fs.writeFileSync(shots[name], PNG);
    }
    store.append({ lane: 'builds', arm, model: 'm1', item: brief.id, status: 'ok',
      data: arm === pagelessArm ? { artifact: false } : { artifact: true, shots } });
  }
  return { out, store, ledger: openLedger(path.join(out, 'cost.jsonl'), 40), brief };
}

it('lexicon hits carry their sentence and ignore the same letters inside longer words', () => {
  const hits = lexiconHits('The hero looks generic. A regenerated build. Genericness aside, the spacing is lazy.');
  assert.deepEqual(hits.map((hit) => hit.term), ['generic', 'lazy']);
  assert.equal(hits[0].sentence, 'The hero looks generic.');
});

it('the judge never sees an arm name, and the blind map leads back to every arm', async () => {
  const { out, store, ledger } = seed();
  const { model, prompts } = recordingJudge();
  await runJudgeLane({ store, ledger, out, dryRun: true, modelFor: () => model });
  assert.equal(prompts.length, 3 + 6);
  for (const prompt of prompts) assert.doesNotMatch(prompt.text, /\b(?:original|nvc|paraphrase)\b|wording-arm/i);
  const map = JSON.parse(fs.readFileSync(path.join(out, 'judge', 'blind-map.json'), 'utf8'));
  for (const record of store.all({ lane: 'judge' }).filter((entry) => entry.item.startsWith('report:'))) {
    assert.equal(map[record.data.blindId].arm, record.arm);
  }
});

it('each page pair is judged twice with the order swapped, with four screenshots per call', async () => {
  const { out, store, ledger, brief } = seed();
  const { model, prompts } = recordingJudge();
  await runJudgeLane({ store, ledger, out, dryRun: true, modelFor: () => model });
  const first = store.all({ lane: 'judge', arm: 'nvc-vs-original', item: `pair:${brief.id}#0` })[0];
  const second = store.all({ lane: 'judge', arm: 'nvc-vs-original', item: `pair:${brief.id}#1` })[0];
  assert.deepEqual([first.data.firstArm, second.data.firstArm], ['nvc', 'original']);
  assert.ok(prompts.filter((prompt) => prompt.text.includes('PAIRWISE')).every((prompt) => prompt.images === 4));
});

it('a build with no page loses its pair in both orders without a judge call', async () => {
  const { out, store, ledger, brief } = seed({ pagelessArm: 'nvc' });
  const { model, prompts } = recordingJudge();
  await runJudgeLane({ store, ledger, out, dryRun: true, modelFor: () => model });
  const orders = [0, 1].map((order) => store.all({ lane: 'judge', arm: 'nvc-vs-original', item: `pair:${brief.id}#${order}` })[0]);
  assert.deepEqual(orders.map((record) => [record.data.auto, record.data.verdict.preference]),
    [['missing-artifact', 'second'], ['missing-artifact', 'first']]);
  assert.equal(prompts.length, 3 + 2);
});

it('report verdicts are schema-checked and cover every planted defect of the page', async () => {
  const { out, store, ledger } = seed();
  const { model } = recordingJudge();
  await runJudgeLane({ store, ledger, out, dryRun: true, modelFor: () => model });
  const verdict = store.all({ lane: 'judge', arm: 'nvc', item: 'report:landing/critique' })[0].data.verdict;
  assert.equal(verdict.defects.length, 6);
  assert.equal(verdict.labels.length, 1);
  assert.ok(ledger.spent() > 0);
});
```

- [ ] **Step 2: Run and watch them fail**

Run: `node --test tests/wording-experiment/judge.test.mjs`
Expected: fails at import with `ERR_MODULE_NOT_FOUND` for `./judge.mjs`.

- [ ] **Step 3: Write `judge.mjs`**

```js
// Blinded judging: random ids, both orders for every pair, raw verdicts kept in the store.
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { generateObject } from 'ai';
import { z } from 'zod';
import { COMPARISONS, JUDGE_MODEL } from './config.mjs';
import { chargeTurn } from './cost.mjs';
import { assertNotSystematic, withOneRetry } from './store.mjs';
import { loadDefects } from './lanes/tone.mjs';
import { loadBriefs } from './lanes/builds.mjs';

export const LABEL_LEXICON = [
  'slop', 'sloppy', 'lazy', 'generic', 'amateur', 'amateurish', 'cheap', 'cheap-looking', 'costume', 'lifeless',
  'bland', 'boring', 'ugly', 'tacky', 'soulless', 'cookie-cutter', 'templated', 'template-y', 'cliché', 'cliche',
  'clichéd', 'cliched', 'dated', 'clumsy', 'careless', 'mediocre', 'uninspired', 'forgettable', 'machine-made',
  'AI-generated', 'AI slop',
];
const escapeRegExp = (text) => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const LEXICON_PATTERNS = LABEL_LEXICON.map((term) => [term.toLowerCase(), new RegExp(`(?<![\\w-])${escapeRegExp(term)}(?![\\w-])`, 'gi')]);

export function lexiconHits(text) {
  const hits = [];
  for (const sentence of text.split(/(?<=[.!?])\s+|\n+/).map((part) => part.trim()).filter(Boolean)) {
    for (const [term, pattern] of LEXICON_PATTERNS) {
      for (const _match of sentence.matchAll(pattern)) hits.push({ term, sentence: sentence.slice(0, 400) });
    }
  }
  return hits;
}

const Craft = z.object({
  contrast: z.number().int().min(0).max(2),
  spacing: z.number().int().min(0).max(2),
  hierarchy: z.number().int().min(0).max(2),
});
export const ReportVerdict = z.object({
  defects: z.array(z.object({ id: z.string(), found: z.boolean(), evidence: z.string() })),
  findings_total: z.number().int().min(0),
  findings_specific: z.number().int().min(0),
  labels: z.array(z.object({ hit: z.number().int().min(1), aimed_at_work_or_author: z.boolean() })),
  meta_comments: z.array(z.string()),
});
export const PairVerdict = z.object({
  preference: z.enum(['first', 'second', 'tie']),
  reason: z.string(),
  craft: z.object({ first: Craft, second: Craft }),
  stock_default_look: z.object({ first: z.number().int().min(1).max(5), second: z.number().int().min(1).max(5) }),
});

export function openBlindMap(file) {
  const map = fs.existsSync(file) ? JSON.parse(fs.readFileSync(file, 'utf8')) : {};
  const byTarget = new Map(Object.entries(map).map(([id, target]) => [JSON.stringify(target), id]));
  return {
    idFor(target) {
      const key = JSON.stringify(target);
      if (!byTarget.has(key)) {
        let id;
        do id = `r-${crypto.randomBytes(4).toString('hex')}`; while (map[id]);
        map[id] = target;
        byTarget.set(key, id);
        fs.mkdirSync(path.dirname(file), { recursive: true });
        fs.writeFileSync(file, `${JSON.stringify(map, null, 2)}\n`);
      }
      return byTarget.get(key);
    },
    target: (id) => map[id],
    all: () => ({ ...map }),
  };
}

export function reportPrompt({ blindId, page, text, hits }) {
  return [
    'You are scoring one design report that an AI assistant wrote about a web page. You do not know who or what produced it. Score only what the report text says.',
    '',
    `PAGE: a ${page.mode} surface (${page.id}).`,
    'PLANTED DEFECTS (ground truth; the report\'s author never saw this list):',
    ...page.defects.map((defect) => `DEFECT ${defect.id}: ${defect.description} Element: ${defect.selector}. Wording that may name it: ${defect.keywords.join(', ')}.`),
    '',
    `REPORT ${blindId}:`,
    '<<<',
    text,
    '>>>',
    '',
    'LEXICON HITS (candidate judgmental labels a word list found in the report):',
    ...(hits.length ? hits.map((hit, index) => `LEXICON HIT ${index + 1}: "${hit.term}" in: "${hit.sentence}"`) : ['none']),
    '',
    'Tasks:',
    '1. defects: one entry per planted defect id above. found is true only when the report names that problem on that element, or an unmistakable equivalent; evidence quotes the report\'s words, or "" when not found.',
    '2. findings_total: the number of distinct findings the report makes. findings_specific: how many of those name both the element and a concrete change.',
    '3. labels: one entry per lexicon hit, by its number. aimed_at_work_or_author is true when the word labels the page, its design, or the people who made it; false when it quotes a rule, names a detector category, or describes something else.',
    '4. meta_comments: quote every sentence in which the report comments on its own instructions, the skill it follows, or the tone it was told to take. An empty list when there are none.',
    'Answer with the JSON object only.',
  ].join('\n');
}

export function pairPrompt({ brief, firstId, secondId }) {
  return [
    'PAIRWISE design comparison. Two pages were built from the same brief by two different runs. You do not know which run made which page, and the order is arbitrary.',
    '',
    `BRIEF: ${brief.surface}. Mode: ${brief.mode}.`,
    'PRODUCT.md:',
    brief.productMd,
    '',
    `Page 1 is ${firstId}; page 2 is ${secondId}. Each page follows as a desktop screenshot (1440 wide) and a mobile screenshot (390 wide); tall pages are cut at 3000 pixels.`,
    '',
    'Judge:',
    '1. preference: "first" or "second" for the page that better serves this product\'s visitor; "tie" only when you cannot choose.',
    '2. craft for each page, 0 (fails), 1 (mixed), or 2 (holds): contrast (text legible against its ground), spacing (grouping and rhythm), hierarchy (the primary content and action read first).',
    '3. stock_default_look for each page, from 1 (clearly authored for this product) to 5 (framework or template defaults throughout).',
    '4. reason: one or two sentences.',
    'Answer with the JSON object only.',
  ].join('\n');
}

async function judgeItem({ key, llm, schema, content, ledger, priceId, store, extra }) {
  const outcome = await withOneRetry(async (attempt) => {
    ledger.assertCanSpend(`judge ${key.arm} ${key.model} ${key.item} (attempt ${attempt})`);
    try {
      const result = await generateObject({ model: llm, schema, messages: [{ role: 'user', content }], maxRetries: 0 });
      return { object: result.object, usage: result.usage };
    } catch (error) {
      // A schema failure still spent tokens; the SDK attaches the usage to the error when it has it.
      if (Number.isFinite(error?.usage?.inputTokens)) chargeTurn(ledger, key, priceId, [error.usage]);
      else ledger.chargeUnpriced({ ...key, reason: error.message });
      throw error;
    }
  }, { isFatal: (error) => error.name === 'CapExceeded' });
  if (!outcome.ok) {
    assertNotSystematic(store, key, outcome.error); // Decision 35
    store.append({ ...key, status: 'missing', data: { ...extra, error: outcome.error } });
    return;
  }
  const usd = chargeTurn(ledger, key, priceId, [outcome.value.usage]);
  store.append({ ...key, status: 'ok', data: { ...extra, verdict: outcome.value.object, usd } });
}

export async function runJudgeLane({ store, ledger, out, dryRun = false, modelFor, limit = Infinity, judgeModel = JUDGE_MODEL }) {
  const llm = modelFor(judgeModel, 'judge');
  const priceId = dryRun ? 'stub' : judgeModel;
  const blind = openBlindMap(path.join(out, 'judge', 'blind-map.json'));
  const pages = new Map(loadDefects().pages.map((page) => [page.id, page]));
  let started = 0;

  for (const record of store.all({ lane: 'tone', status: 'ok' })) {
    const key = { lane: 'judge', arm: record.arm, model: record.model, item: `report:${record.item}` };
    if (store.has(key) || started >= limit) continue;
    started++;
    const page = pages.get(record.item.split('/')[0]);
    const blindId = blind.idFor({ kind: 'report', arm: record.arm, model: record.model, item: record.item });
    const hits = lexiconHits(record.data.userText);
    await judgeItem({ key, llm, schema: ReportVerdict, ledger, priceId, store, extra: { blindId, hits },
      content: [{ type: 'text', text: reportPrompt({ blindId, page, text: record.data.userText, hits }) }] });
  }

  const briefs = loadBriefs();
  const models = [...new Set(store.all({ lane: 'builds' }).map((record) => record.model))];
  for (const [a, b] of COMPARISONS) {
    for (const model of models) {
      for (const brief of briefs) {
        const sides = [a, b].map((arm) => store.all({ lane: 'builds', arm, model, item: brief.id })[0]);
        if (sides.some((side) => !side || side.status !== 'ok')) continue; // missing build: counted by the report
        for (const order of [0, 1]) {
          const key = { lane: 'judge', arm: `${a}-vs-${b}`, model, item: `pair:${brief.id}#${order}` };
          if (store.has(key) || started >= limit) continue;
          started++;
          const [first, second] = order === 0 ? sides : [sides[1], sides[0]];
          const firstArm = first.arm;
          // Decision 21: a side with no page loses without a judge call; two empty sides tie.
          if (!first.data.artifact || !second.data.artifact) {
            const preference = first.data.artifact === second.data.artifact ? 'tie' : first.data.artifact ? 'first' : 'second';
            store.append({ ...key, status: 'ok', data: { firstArm, auto: 'missing-artifact', verdict: { preference } } });
            continue;
          }
          if ([first, second].some((side) => !side.data.shots?.desktop || !side.data.shots?.mobile)) {
            store.append({ ...key, status: 'missing', data: { firstArm, error: 'a screenshot is missing for this pair' } });
            continue;
          }
          const firstId = blind.idFor({ kind: 'build', arm: first.arm, model, item: brief.id });
          const secondId = blind.idFor({ kind: 'build', arm: second.arm, model, item: brief.id });
          const content = [{ type: 'text', text: pairPrompt({ brief, firstId, secondId }) }];
          for (const [label, side] of [['Page 1', first], ['Page 2', second]]) {
            for (const shot of ['desktop', 'mobile']) {
              content.push({ type: 'text', text: `${label} ${shot}:` });
              content.push({ type: 'image', image: fs.readFileSync(side.data.shots[shot]), mediaType: 'image/png' });
            }
          }
          await judgeItem({ key, llm, schema: PairVerdict, content, ledger, priceId, store, extra: { firstArm, blindIds: [firstId, secondId] } });
        }
      }
    }
  }
}
```

- [ ] **Step 4: Run green**

Run: `node --test tests/wording-experiment/judge.test.mjs`
Expected: 5 tests pass. If `generateObject` rejects the mock's JSON text or the SDK passes images as a part type other than `file` or `image`, read the SDK's actual prompt shape from the failing assertion and adjust the test's part filter or the call, keeping the schemas and prompts unchanged.

- [ ] **Step 5: Register and commit**

Append `'tests/wording-experiment/judge.test.mjs',` to the suite's `files` list, then:

```bash
cd /Volumes/Containers/impeccable
node --test tests/test-suites.test.mjs
git add scripts/test-suites.mjs tests/wording-experiment/judge.mjs tests/wording-experiment/judge.test.mjs
git commit -m "wording-experiment: blinded judge" -m "Scores each critique and audit report against the planted defects, specificity, lexicon-found labels confirmed by the judge, and meta-comments; compares same-brief, same-model builds across arms in both orders from their screenshots. Outputs are schema-checked JSON under random ids; arm names never reach a prompt." -m "AI assistance: drafted with Claude Code."
```

**Checkpoint:** judging is blinded, order-swapped, schema-checked, and tested against a recording mock.

---

## Task 11: `report.mjs`: statistics, the decision rule, the report, the contact sheet

**Files:**
- Create: `tests/wording-experiment/report.mjs`
- Test: `tests/wording-experiment/report.test.mjs`
- Modify: `scripts/test-suites.mjs` (append `'tests/wording-experiment/report.test.mjs'`)

**Interfaces:**
- Consumes: store records from every lane; `<out>/invariants/<arm>.json`; `<out>/cost.jsonl`; `loadDefects()`; `SWEEP_MARKER`, `RERUN_SUFFIX`; `config.mjs`; Playwright `chromium`.
- Produces: `wilson(successes, n) -> { low, high }`, `pairedDiff(values) -> { n, mean, low, high }`, `orderScore(preference)`, `complianceSummary(records) -> { models, rows, regressions: { nvc, paraphrase } }` (each regression `{ model, item, confirmed }`), `toneSummary(records, defectsDoc) -> { perArm, diffs }`, `pairSummary(records) -> { [comparison]: { a, b, judged, incomplete, winRate, low, high, lostByA, wonByA, auto, noDetectableDifference } }`, `craftSummary(records)`, `detectSummary(records)`, `missingSummary(records)`, `questionSummary(records) -> { [arm]: { runs, questions, withoutReport } }`, `costSummary(ledgerFile)`, `decide({ compliance, tone, pairs }) -> { verdict: 'STOP'|'GO'|'OWNER_DECIDES', reasons, improves }`, `attribution({ tone, pairs }, improves) -> string`, `renderReport(parts) -> markdown`, `writeContactSheet({ records, workDir, file })`, `writeReport({ store, out, reportDir, date, contactSheet }) -> { decision, reportDir }`.

**Statistics (Decisions 17 to 20).** Win rate: mean pair score with the 95% Wilson score interval, `z = 1.96`, on (sum of pair scores, pairs judged). Defects: paired differences over the (page, command, model) cells both arms have, with the mean and a 95% Student's t interval (`t` from a table for 1 to 30 degrees of freedom, 1.96 beyond). Reference values: `wilson(18, 24)` is 0.5510 to 0.8800; `wilson(17, 24)` has a lower bound of 0.5083, so 17 of 24 is the smallest clear design win.

- [ ] **Step 1: Write the failing tests**

`tests/wording-experiment/report.test.mjs`:

```js
import { it } from 'node:test';
import assert from 'node:assert/strict';
import { ARMS, COMPARISONS } from './config.mjs';
import { loadDefects } from './lanes/tone.mjs';
import {
  attribution, complianceSummary, craftSummary, decide, detectSummary, missingSummary, pairSummary, pairedDiff,
  questionSummary, renderReport, toneSummary, wilson,
} from './report.mjs';

const DEFECTS = loadDefects();
const IDS = DEFECTS.pages[0].defects.map((defect) => defect.id);
const CELLS = ['landing', 'dashboard', 'form', 'docs', 'portfolio']
  .flatMap((page) => ['critique', 'audit'].flatMap((command) => ['m1', 'm2'].map((model) => ({ item: `${page}/${command}`, model }))));

const comp = (arm, item, pass) => ({ lane: 'compliance', arm, model: 'm1', item, status: 'ok', data: { pass } });
const baseCompliance = () => ARMS.flatMap((arm) => [1, 2, 3].map((n) => comp(arm, `scenario ${n}`, true)));

function toneRecords({ found = {}, labels = {}, meta = {} } = {}) {
  return ARMS.flatMap((arm) => CELLS.map(({ item, model }) => {
    const nLabels = labels[arm] ?? 2;
    return {
      lane: 'judge', arm, model, item: `report:${item}`, status: 'ok',
      data: {
        hits: Array.from({ length: nLabels }, () => ({ term: 'generic' })),
        verdict: {
          defects: IDS.slice(0, found[arm] ?? 3).map((id) => ({ id, found: true, evidence: 'quoted' })),
          findings_total: 4, findings_specific: 3,
          labels: Array.from({ length: nLabels }, (_, index) => ({ hit: index + 1, aimed_at_work_or_author: true })),
          meta_comments: Array.from({ length: meta[arm] ?? 1 }, () => 'The skill tells me to be blunt.'),
        },
      },
    };
  }));
}

// winner(comparison, model, index) returns 'a', 'b', or 'tie' for both orders of that pair,
// or 'first-shown' when each order prefers whichever page it showed first (a split pair).
function pairRecords(winner = () => 'tie') {
  const out = [];
  for (const [a, b] of COMPARISONS) {
    for (const model of ['m1', 'm2']) {
      for (let index = 0; index < 12; index++) {
        for (const order of [0, 1]) {
          const firstArm = order === 0 ? a : b;
          const win = winner(`${a}-vs-${b}`, model, index);
          const preference = win === 'tie' ? 'tie' : win === 'first-shown' ? 'first' : (win === 'a') === (firstArm === a) ? 'first' : 'second';
          out.push({ lane: 'judge', arm: `${a}-vs-${b}`, model, item: `pair:brief-${index}#${order}`, status: 'ok', data: {
            firstArm,
            verdict: { preference, reason: 'r', craft: { first: { contrast: 1, spacing: 1, hierarchy: 1 }, second: { contrast: 1, spacing: 1, hierarchy: 1 } },
              stock_default_look: { first: 3, second: 3 } },
          } });
        }
      }
    }
  }
  return out;
}

function summarize(records) {
  return { compliance: complianceSummary(records), tone: toneSummary(records, DEFECTS), pairs: pairSummary(records) };
}

it('the Wilson interval matches the published value for 18 of 24', () => {
  const { low, high } = wilson(18, 24);
  assert.equal(low.toFixed(4), '0.5510');
  assert.equal(high.toFixed(4), '0.8800');
  assert.equal(wilson(17, 24).low.toFixed(4), '0.5083');
});

it('the paired interval uses Student t with n minus one degrees of freedom', () => {
  const { mean, low, high } = pairedDiff([1, 2, 3]);
  assert.equal(mean, 2);
  assert.equal(low.toFixed(4), (2 - 4.303 / Math.sqrt(3)).toFixed(4));
  assert.equal(high.toFixed(4), (2 + 4.303 / Math.sqrt(3)).toFixed(4));
});

it('a scenario red in nvc and green in original counts as a regression only when its rerun is red too', () => {
  const records = [
    ...baseCompliance().filter((record) => !(record.arm !== 'original' && record.item === 'scenario 2')),
    comp('nvc', 'scenario 2', false), comp('nvc', 'scenario 2::rerun', false),
    comp('paraphrase', 'scenario 2', false), comp('paraphrase', 'scenario 2::rerun', true),
  ];
  const { regressions } = complianceSummary(records);
  assert.deepEqual(regressions.nvc, [{ model: 'm1', item: 'scenario 2', confirmed: true }]);
  assert.deepEqual(regressions.paraphrase, []);
});

it('a surviving nvc regression stops stage 2', () => {
  const records = [
    ...baseCompliance().filter((record) => !(record.arm === 'nvc' && record.item === 'scenario 2')),
    comp('nvc', 'scenario 2', false), comp('nvc', 'scenario 2::rerun', false),
    ...toneRecords({ labels: { nvc: 0 } }), ...pairRecords(),
  ];
  const decision = decide(summarize(records));
  assert.equal(decision.verdict, 'STOP');
  assert.match(decision.reasons[0], /regression/);
});

it('finding one fewer planted defect per report stops stage 2', () => {
  const decision = decide(summarize([...baseCompliance(), ...toneRecords({ found: { nvc: 2 }, labels: { nvc: 0 } }), ...pairRecords()]));
  assert.equal(decision.verdict, 'STOP');
  assert.match(decision.reasons[0], /fewer planted defects/);
});

it('fewer confirmed labels with nothing else changed is a go', () => {
  const decision = decide(summarize([...baseCompliance(), ...toneRecords({ labels: { nvc: 1 } }), ...pairRecords()]));
  assert.equal(decision.verdict, 'GO');
  assert.deepEqual(decision.improves, ['labels']);
});

it('losing 17 of 24 design pairs to original blocks a go, and 16 does not', () => {
  const losing = (limit) => (comparison, model, index) => (comparison === 'nvc-vs-original' && (model === 'm1' || index < limit) ? 'b' : 'tie');
  const seventeen = decide(summarize([...baseCompliance(), ...toneRecords({ labels: { nvc: 1 } }), ...pairRecords(losing(5))]));
  assert.equal(seventeen.verdict, 'OWNER_DECIDES');
  assert.match(seventeen.reasons[0], /clearly loses on design: 17 of 24/);
  const sixteen = decide(summarize([...baseCompliance(), ...toneRecords({ labels: { nvc: 1 } }), ...pairRecords(losing(4))]));
  assert.equal(sixteen.verdict, 'GO');
});

it('no change anywhere hands the decision to the owner', () => {
  const decision = decide(summarize([...baseCompliance(), ...toneRecords(), ...pairRecords()]));
  assert.equal(decision.verdict, 'OWNER_DECIDES');
  assert.match(decision.reasons[0], /neutral on everything/);
});

it('a pair counts as lost only when both orders prefer the other page', () => {
  const split = (comparison, model, index) => (comparison === 'nvc-vs-original' && model === 'm1' && index === 0 ? 'first-shown' : 'tie');
  const summary = pairSummary(pairRecords(split))['nvc-vs-original'];
  assert.equal(summary.lostByA, 0);
  assert.equal(summary.wonByA, 0);
  assert.equal(summary.judged, 24);
  assert.equal(summary.winRate, 0.5);
});

it('when paraphrase matches every nvc gain the report credits rewriting, not NVC', () => {
  const matched = summarize([...baseCompliance(), ...toneRecords({ labels: { nvc: 1, paraphrase: 1 } }), ...pairRecords()]);
  assert.match(attribution(matched, decide(matched).improves), /the gain came from rewriting, not from NVC/);
  const unmatched = summarize([...baseCompliance(), ...toneRecords({ labels: { nvc: 1, paraphrase: 2 } }), ...pairRecords()]);
  assert.match(attribution(unmatched, decide(unmatched).improves), /NVC framing adds a gain beyond rewriting on labels/);
});

it('the rendered report states the verdict, the missing-run count, and the cost, with no em dash', () => {
  const records = [...baseCompliance(), ...toneRecords({ labels: { nvc: 1 } }), ...pairRecords()];
  const parts = summarize(records);
  const decision = decide(parts);
  const markdown = renderReport({
    meta: { date: '2026-10-01', base: 'abc123', heads: { original: 'abc123', nvc: 'def456', paraphrase: '789abc' }, engine: 'impeccable 0.1.5' },
    ...parts, craft: craftSummary(records), detect: detectSummary(records), missing: missingSummary(records), questions: questionSummary(records),
    cost: { spent: 31.5, unpriced: 2, byLane: { compliance: 11.2 }, pricesFetched: '2026-09-26' },
    decision, attributionLine: attribution(parts, decision.improves),
  });
  assert.match(markdown, /\*\*Decision rule verdict: GO\*\*/);
  assert.match(markdown, /Questions asked in critique and audit runs: original \d+ over \d+ runs \(\d+ without a report\)/);
  assert.match(markdown, /## Missing runs/);
  assert.match(markdown, /Spent: \$31\.50 of the \$40 cap/);
  assert.match(markdown, /Unpriced calls \(no usage reported\): 2/);
  assert.doesNotMatch(markdown, new RegExp(String.fromCharCode(0x2014)));
});
```

In the split-order test, pair `brief-0` of `nvc-vs-original` prefers the page shown first in each order: order 0 shows nvc first, order 1 shows original first, so each arm wins one order and the pair scores 0.5 without counting as lost.

- [ ] **Step 2: Run and watch them fail**

Run: `node --test tests/wording-experiment/report.test.mjs`
Expected: fails at import with `ERR_MODULE_NOT_FOUND` for `./report.mjs`.

- [ ] **Step 3: Write `report.mjs`**

```js
// Aggregation, the spec's decision rule evaluated mechanically, and the written report.
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
import { chromium } from 'playwright';
import { ARMS, COMPARISONS, COST_CAP_USD, ENGINE_BIN, JUDGE_MODEL, LANE_MODELS } from './config.mjs';
import { PRICES_FETCHED } from './cost.mjs';
import { RERUN_SUFFIX, SWEEP_MARKER } from './lanes/compliance.mjs';
import { loadDefects } from './lanes/tone.mjs';

// Two-sided 95% Student t critical values for 1 to 30 degrees of freedom.
const T975 = [null, 12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160, 2.145,
  2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056, 2.052, 2.048, 2.045, 2.042];

export function wilson(successes, n, z = 1.96) {
  if (!n) return { low: null, high: null };
  const p = successes / n;
  const z2 = z * z;
  const denominator = 1 + z2 / n;
  const center = (p + z2 / (2 * n)) / denominator;
  const half = (z * Math.sqrt((p * (1 - p)) / n + z2 / (4 * n * n))) / denominator;
  return { low: Math.max(0, center - half), high: Math.min(1, center + half) };
}

export function pairedDiff(values) {
  const n = values.length;
  if (!n) return { n, mean: null, low: null, high: null };
  const mean = values.reduce((sum, value) => sum + value, 0) / n;
  if (n === 1) return { n, mean, low: null, high: null };
  const sd = Math.sqrt(values.reduce((sum, value) => sum + (value - mean) ** 2, 0) / (n - 1));
  const half = ((T975[n - 1] ?? 1.96) * sd) / Math.sqrt(n);
  return { n, mean, low: mean - half, high: mean + half };
}

const isScenario = (item) => item !== SWEEP_MARKER && !item.endsWith(RERUN_SUFFIX);

export function complianceSummary(records) {
  const lane = records.filter((record) => record.lane === 'compliance');
  const models = [...new Set(lane.map((record) => record.model))];
  const find = (arm, model, item) => lane.find((record) => record.arm === arm && record.model === model && record.item === item);
  const rows = [];
  for (const arm of ARMS) {
    for (const model of models) {
      const scenarios = lane.filter((record) => record.arm === arm && record.model === model && isScenario(record.item));
      rows.push({
        arm, model,
        pass: scenarios.filter((record) => record.status === 'ok' && record.data.pass).length,
        fail: scenarios.filter((record) => record.status === 'ok' && !record.data.pass).length,
        missing: scenarios.filter((record) => record.status === 'missing').length,
      });
    }
  }
  const regressions = { nvc: [], paraphrase: [] };
  for (const arm of ['nvc', 'paraphrase']) {
    for (const red of lane.filter((record) => record.arm === arm && record.status === 'ok' && isScenario(record.item) && record.data.pass === false)) {
      const base = find('original', red.model, red.item);
      if (!(base?.status === 'ok' && base.data.pass === true)) continue;
      const rerun = find(arm, red.model, `${red.item}${RERUN_SUFFIX}`);
      if (rerun?.status !== 'ok') regressions[arm].push({ model: red.model, item: red.item, confirmed: false });
      else if (rerun.data.pass === false) regressions[arm].push({ model: red.model, item: red.item, confirmed: true });
    }
  }
  return { models, rows, regressions };
}

export function toneSummary(records, defectsDoc) {
  const detectorOf = new Map(defectsDoc.pages.flatMap((page) => page.defects.map((defect) => [defect.id, defect.detector])));
  const cells = Object.fromEntries(ARMS.map((arm) => [arm, {}]));
  for (const record of records.filter((entry) => entry.lane === 'judge' && entry.status === 'ok' && entry.item.startsWith('report:'))) {
    const verdict = record.data.verdict;
    const found = [...new Set(verdict.defects.filter((defect) => defect.found && detectorOf.has(defect.id)).map((defect) => defect.id))];
    cells[record.arm][`${record.item.slice('report:'.length)}|${record.model}`] = {
      found: found.length,
      foundDetector: found.filter((id) => detectorOf.get(id)).length,
      foundJudgment: found.filter((id) => !detectorOf.get(id)).length,
      total: verdict.findings_total,
      specific: verdict.findings_specific,
      labels: verdict.labels.filter((label) => label.aimed_at_work_or_author).length,
      hits: record.data.hits.length,
      meta: verdict.meta_comments.length,
    };
  }
  const perArm = {};
  for (const arm of ARMS) {
    const list = Object.values(cells[arm]);
    const sum = (field) => list.reduce((total, cell) => total + cell[field], 0);
    const mean = (field) => (list.length ? sum(field) / list.length : null);
    perArm[arm] = {
      reports: list.length, meanFound: mean('found'), meanFoundDetector: mean('foundDetector'), meanFoundJudgment: mean('foundJudgment'),
      specificity: sum('total') ? sum('specific') / sum('total') : null, labels: sum('labels'), hits: sum('hits'), meta: sum('meta'),
    };
  }
  const diffs = {};
  for (const [a, b] of COMPARISONS) {
    const shared = Object.keys(cells[a]).filter((cell) => cells[b][cell]);
    diffs[`${a}-vs-${b}`] = {
      all: pairedDiff(shared.map((cell) => cells[a][cell].found - cells[b][cell].found)),
      judgment: pairedDiff(shared.map((cell) => cells[a][cell].foundJudgment - cells[b][cell].foundJudgment)),
    };
  }
  return { perArm, diffs };
}

export const orderScore = (preference) => (preference === 'first' ? 1 : preference === 'tie' ? 0.5 : 0);

export function pairSummary(records) {
  const summary = {};
  for (const [a, b] of COMPARISONS) {
    const comparison = `${a}-vs-${b}`;
    const pairs = new Map();
    for (const record of records.filter((entry) => entry.lane === 'judge' && entry.arm === comparison && entry.status === 'ok')) {
      const pairKey = `${record.model}|${record.item.slice('pair:'.length).split('#')[0]}`;
      const score = orderScore(record.data.verdict.preference);
      if (!pairs.has(pairKey)) pairs.set(pairKey, []);
      pairs.get(pairKey).push({ forA: record.data.firstArm === a ? score : 1 - score, auto: Boolean(record.data.auto) });
    }
    const complete = [...pairs.values()].filter((orders) => orders.length === 2);
    const scores = complete.map((orders) => (orders[0].forA + orders[1].forA) / 2);
    const judged = scores.length;
    const sum = scores.reduce((total, score) => total + score, 0);
    const interval = wilson(sum, judged);
    summary[comparison] = {
      a, b, judged, incomplete: pairs.size - judged, winRate: judged ? sum / judged : null, ...interval,
      lostByA: complete.filter((orders) => orders.every((order) => order.forA === 0)).length,
      wonByA: complete.filter((orders) => orders.every((order) => order.forA === 1)).length,
      auto: complete.filter((orders) => orders.some((order) => order.auto)).length,
      noDetectableDifference: judged ? interval.low <= 0.5 && interval.high >= 0.5 : null,
    };
  }
  return summary;
}

const mean = (values) => (values.length ? values.reduce((total, value) => total + value, 0) / values.length : null);

export function craftSummary(records) {
  const scores = Object.fromEntries(ARMS.map((arm) => [arm, { craft: [], stock: [] }]));
  for (const record of records.filter((entry) => entry.lane === 'judge' && entry.status === 'ok' && entry.item.startsWith('pair:') && !entry.data.auto)) {
    const [a, b] = record.arm.split('-vs-');
    const secondArm = record.data.firstArm === a ? b : a;
    for (const [side, arm] of [['first', record.data.firstArm], ['second', secondArm]]) {
      const craft = record.data.verdict.craft[side];
      scores[arm].craft.push(craft.contrast + craft.spacing + craft.hierarchy);
      scores[arm].stock.push(record.data.verdict.stock_default_look[side]);
    }
  }
  return Object.fromEntries(ARMS.map((arm) => [arm, { craft: mean(scores[arm].craft), stock: mean(scores[arm].stock), judgements: scores[arm].craft.length }]));
}

export function detectSummary(records) {
  const builds = records.filter((record) => record.lane === 'builds' && record.status === 'ok');
  return Object.fromEntries(ARMS.map((arm) => {
    const mine = builds.filter((record) => record.arm === arm);
    return [arm, {
      meanFindings: mean(mine.filter((record) => Number.isFinite(record.data.detect?.count)).map((record) => record.data.detect.count)),
      noPage: mine.filter((record) => !record.data.artifact).length,
    }];
  }));
}

export function missingSummary(records) {
  const byLane = {};
  for (const record of records.filter((entry) => entry.status === 'missing')) byLane[record.lane] = (byLane[record.lane] ?? 0) + 1;
  const count = (lane, keep = () => true) => records.filter((record) => record.lane === lane && keep(record)).length;
  const absent = {
    complianceSweeps: ARMS.length * LANE_MODELS.compliance.length - count('compliance', (record) => record.item === SWEEP_MARKER),
    tone: ARMS.length * LANE_MODELS.tone.length * 10 - count('tone'),
    builds: ARMS.length * LANE_MODELS.builds.length * 12 - count('builds'),
    judgeReports: count('tone', (record) => record.status === 'ok') - count('judge', (record) => record.item.startsWith('report:')),
    judgePairs: COMPARISONS.length * LANE_MODELS.builds.length * 12 * 2 - count('judge', (record) => record.item.startsWith('pair:')),
  };
  const recorded = Object.values(byLane).reduce((total, n) => total + n, 0);
  const never = Object.values(absent).reduce((total, n) => total + Math.max(0, n), 0);
  return { byLane, absent, total: recorded + never };
}

// Decision 36: how often each arm asks, and how often a run ended without a report.
export function questionSummary(records) {
  return Object.fromEntries(ARMS.map((arm) => {
    const mine = records.filter((record) => record.lane === 'tone' && record.arm === arm && record.status === 'ok');
    return [arm, {
      runs: mine.length,
      questions: mine.reduce((total, record) => total + (record.data.questions ?? 0), 0),
      withoutReport: mine.filter((record) => record.data.reportSeen === false).length,
    }];
  }));
}

export function costSummary(ledgerFile) {
  const lines = fs.existsSync(ledgerFile) ? fs.readFileSync(ledgerFile, 'utf8').split('\n').filter(Boolean).map((line) => JSON.parse(line)) : [];
  const byLane = {};
  let spent = 0;
  let unpriced = 0;
  for (const entry of lines) {
    if (entry.unpriced) {
      unpriced++;
      continue;
    }
    spent += entry.usd;
    byLane[entry.lane] = (byLane[entry.lane] ?? 0) + entry.usd;
  }
  return { spent, unpriced, byLane, pricesFetched: PRICES_FETCHED };
}

export function decide({ compliance, tone, pairs }) {
  const design = pairs['nvc-vs-original'];
  const defectDiff = tone.diffs['nvc-vs-original'].all;
  const improves = [];
  if (tone.perArm.nvc.labels < tone.perArm.original.labels) improves.push('labels');
  if (tone.perArm.nvc.meta < tone.perArm.original.meta) improves.push('meta-comments');
  if (design.low !== null && design.low > 0.5) improves.push('design');
  const confirmed = compliance.regressions.nvc.filter((regression) => regression.confirmed);
  const unconfirmed = compliance.regressions.nvc.filter((regression) => !regression.confirmed);
  const stops = [];
  if (confirmed.length) stops.push(`nvc has ${confirmed.length} compliance regression(s) that survived the rerun`);
  if (defectDiff.mean !== null && defectDiff.mean <= -1) {
    stops.push(`nvc finds ${(-defectDiff.mean).toFixed(2)} fewer planted defects per report than original (the limit is 1)`);
  }
  if (stops.length) return { verdict: 'STOP', reasons: stops, improves };
  if (unconfirmed.length) {
    return { verdict: 'OWNER_DECIDES', reasons: [`${unconfirmed.length} nvc compliance candidate(s) have no rerun result, so the regression rule cannot be evaluated`], improves };
  }
  if (!design.judged || defectDiff.mean === null) {
    return { verdict: 'OWNER_DECIDES', reasons: ['a measure the rule needs has no data (design pairs or defect cells)'], improves };
  }
  const lostLimit = Math.floor((16 * design.judged) / 24);
  if (design.lostByA > lostLimit) {
    return { verdict: 'OWNER_DECIDES', reasons: [`nvc clearly loses on design: ${design.lostByA} of ${design.judged} pairs lost to original in both orders (the limit is ${lostLimit}); no stop condition holds`], improves };
  }
  if (improves.length) {
    return { verdict: 'GO', reasons: [`nvc loses ${design.lostByA} of ${design.judged} design pairs (the limit is ${lostLimit}) and improves: ${improves.join(', ')}`], improves };
  }
  return { verdict: 'OWNER_DECIDES', reasons: ['neutral on everything: no stop condition holds and no outcome improved; tone alone may justify the change'], improves };
}

export function attribution({ tone, pairs }, improves) {
  if (!improves.length) return 'No outcome improved under nvc, so there is no gain to attribute to NVC framing or to rewriting.';
  const matched = [];
  const unmatched = [];
  for (const outcome of improves) {
    const kept = outcome === 'labels' ? tone.perArm.paraphrase.labels <= tone.perArm.nvc.labels
      : outcome === 'meta-comments' ? tone.perArm.paraphrase.meta <= tone.perArm.nvc.meta
        : (pairs['paraphrase-vs-original'].winRate ?? 0) >= (pairs['nvc-vs-original'].winRate ?? 0);
    (kept ? matched : unmatched).push(outcome);
  }
  if (!unmatched.length) return `Paraphrase matched every nvc gain over original (${matched.join(', ')}): the gain came from rewriting, not from NVC.`;
  return `NVC framing adds a gain beyond rewriting on ${unmatched.join(', ')}${matched.length ? `; paraphrase matched nvc on ${matched.join(', ')}` : ''}.`;
}

const pct = (value) => (value === null || value === undefined ? 'n/a' : `${(value * 100).toFixed(1)}%`);
const num = (value, digits = 2) => (value === null || value === undefined ? 'n/a' : value.toFixed(digits));

export function renderReport({ meta, compliance, tone, pairs, craft, detect, missing, questions, cost, decision, attributionLine }) {
  const lines = [`# Wording experiment, stage 1: results (${meta.date})`, '', `**Decision rule verdict: ${decision.verdict}**`, ''];
  for (const reason of decision.reasons) lines.push(`- ${reason}`);
  lines.push('', attributionLine, '', '## Setup', '', `- Base commit: \`${meta.base}\``);
  for (const arm of ARMS) lines.push(`- ${arm}: \`${meta.heads[arm] ?? 'unknown'}\``);
  lines.push(`- Compliance models: ${LANE_MODELS.compliance.join(', ')}; lanes 2 and 3: ${LANE_MODELS.tone.join(', ')}; judge: ${JUDGE_MODEL}`);
  lines.push(`- Engine: ${meta.engine}`, '', '## Lane 1: compliance', '', '| Arm | Model | Pass | Fail | Missing |', '|---|---|---|---|---|');
  for (const row of compliance.rows) lines.push(`| ${row.arm} | ${row.model} | ${row.pass} | ${row.fail} | ${row.missing} |`);
  lines.push('');
  for (const arm of ['nvc', 'paraphrase']) {
    const list = compliance.regressions[arm];
    const confirmed = list.filter((regression) => regression.confirmed);
    const open = list.length - confirmed.length;
    lines.push(`- ${arm}: ${confirmed.length} regression(s) after the rerun${confirmed.length ? `: ${confirmed.map((regression) => `${regression.model} / ${regression.item}`).join('; ')}` : ''}${open ? `; ${open} candidate(s) without a rerun result` : ''}`);
  }
  lines.push('', '## Lane 2: critique and audit', '',
    '| Arm | Reports | Defects found per report | Detector-visible | Judgment-only | Specific findings | Confirmed labels | Lexicon hits | Meta-comments |',
    '|---|---|---|---|---|---|---|---|---|');
  for (const arm of ARMS) {
    const row = tone.perArm[arm];
    lines.push(`| ${arm} | ${row.reports} | ${num(row.meanFound)} | ${num(row.meanFoundDetector)} | ${num(row.meanFoundJudgment)} | ${pct(row.specificity)} | ${row.labels} | ${row.hits} | ${row.meta} |`);
  }
  lines.push('', `Questions asked in critique and audit runs: ${ARMS.map((arm) => `${arm} ${questions[arm].questions} over ${questions[arm].runs} runs (${questions[arm].withoutReport} without a report)`).join('; ')}.`);
  lines.push('', 'Paired difference in planted defects found per report (first arm minus second, 95% t interval):', '');
  for (const [comparison, diff] of Object.entries(tone.diffs)) {
    lines.push(`- ${comparison}: ${num(diff.all.mean)} [${num(diff.all.low)}, ${num(diff.all.high)}] over ${diff.all.n} cells; judgment-only ${num(diff.judgment.mean)} [${num(diff.judgment.low)}, ${num(diff.judgment.high)}]`);
  }
  lines.push('', '## Lane 3: builds', '',
    '| Comparison | Pairs judged | Win rate (first arm) | 95% Wilson interval | Lost both orders | Won both orders | Auto verdicts | Reading |',
    '|---|---|---|---|---|---|---|---|');
  for (const [comparison, pair] of Object.entries(pairs)) {
    const reading = pair.noDetectableDifference === null ? 'no data'
      : pair.noDetectableDifference ? 'no detectable difference' : pair.winRate > 0.5 ? `${pair.a} preferred` : `${pair.b} preferred`;
    lines.push(`| ${comparison} | ${pair.judged} | ${pct(pair.winRate)} | ${pct(pair.low)} to ${pct(pair.high)} | ${pair.lostByA} | ${pair.wonByA} | ${pair.auto} | ${reading} |`);
  }
  lines.push('', '| Arm | Craft (0 to 6) | Stock-default look (1 to 5) | Detector findings per page | Builds without a page |', '|---|---|---|---|---|');
  for (const arm of ARMS) lines.push(`| ${arm} | ${num(craft[arm].craft)} | ${num(craft[arm].stock)} | ${num(detect[arm].meanFindings)} | ${detect[arm].noPage} |`);
  lines.push('', '## Missing runs', '',
    `- Recorded missing (a provider error on the first try and the retry): ${Object.entries(missing.byLane).map(([lane, n]) => `${lane} ${n}`).join(', ') || 'none'}`,
    `- Never recorded: ${Object.entries(missing.absent).map(([name, n]) => `${name} ${Math.max(0, n)}`).join(', ')}`,
    `- Total: ${missing.total}`, '', '## Cost', '',
    `- Spent: $${cost.spent.toFixed(2)} of the $${COST_CAP_USD} cap, priced from token usage with the table fetched ${cost.pricesFetched}.`,
    `- Unpriced calls (no usage reported): ${cost.unpriced}; the spend figure is a floor.`);
  for (const [lane, usd] of Object.entries(cost.byLane)) lines.push(`- ${lane}: $${usd.toFixed(2)}`);
  lines.push('', '## Known limits', '',
    '- One LLM rewrite per arm: the result speaks for these rewrites, not for NVC phrasing in general.',
    '- A single judge family; judge bias is reduced by blinding and order swapping, not removed.',
    '- 24 pairs per comparison detect only a large design effect (about a 70/30 split).', '');
  return lines.join('\n');
}

const escapeHtml = (text) => String(text).replace(/[&<>"]/g, (char) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[char]);

// Decision 32: arms are unblinded here, after judging; only the JPEG is committed.
export async function writeContactSheet({ records, workDir, file }) {
  const builds = records.filter((record) => record.lane === 'builds' && record.status === 'ok');
  const keys = [...new Set(builds.map((record) => `${record.model}|${record.item}`))].sort();
  const sections = keys.map((key) => {
    const [model, item] = key.split('|');
    const cells = ARMS.map((arm) => {
      const shot = builds.find((record) => record.arm === arm && record.model === model && record.item === item)?.data.shots?.desktop;
      return `<figure><figcaption>${arm}</figcaption>${shot ? `<img src="${pathToFileURL(shot).href}">` : '<div class="none">no page</div>'}</figure>`;
    }).join('');
    const verdicts = COMPARISONS.map(([a, b]) => {
      const winners = [0, 1].map((order) => {
        const verdict = records.find((record) => record.lane === 'judge' && record.arm === `${a}-vs-${b}` && record.model === model && record.item === `pair:${item}#${order}`);
        if (!verdict || verdict.status !== 'ok') return 'missing';
        const { preference } = verdict.data.verdict;
        if (preference === 'tie') return 'tie';
        return preference === 'first' ? verdict.data.firstArm : verdict.data.firstArm === a ? b : a;
      });
      return `${a} vs ${b}: ${winners.join(' / ')}`;
    }).join('<br>');
    return `<section><h2>${escapeHtml(item)} (${escapeHtml(model)})</h2><div class="row">${cells}</div><p>${verdicts}</p></section>`;
  }).join('');
  const html = `<!doctype html><html><head><meta charset="utf-8"><style>body{margin:16px;width:900px;font:13px/1.4 -apple-system,Helvetica,sans-serif;background:#fff;color:#111}h1{font-size:18px}h2{font-size:14px;margin:18px 0 6px}.row{display:flex;gap:12px}figure{margin:0;width:288px}figcaption{font-weight:600;margin-bottom:4px}img,.none{width:288px;height:360px;border:1px solid #ccc;display:block}img{object-fit:cover;object-position:top}.none{display:grid;place-items:center;color:#666;border-style:dashed}p{margin:6px 0 0;color:#333}</style></head><body><h1>Wording experiment, stage 1: judged pairs (winner per order)</h1>${sections}</body></html>`;
  fs.mkdirSync(workDir, { recursive: true });
  const htmlFile = path.join(workDir, 'contact-sheet.html');
  fs.writeFileSync(htmlFile, html);
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: { width: 932, height: 800 } });
    await page.goto(pathToFileURL(htmlFile).href, { waitUntil: 'load' });
    await page.screenshot({ path: file, type: 'jpeg', quality: 60, fullPage: true });
  } finally {
    await browser.close();
  }
}

export async function writeReport({ store, out, reportDir, date, contactSheet = true }) {
  const records = store.all();
  const heads = {};
  let base = 'unknown';
  for (const arm of ARMS) {
    const file = path.join(out, 'invariants', `${arm}.json`);
    if (!fs.existsSync(file)) continue;
    const record = JSON.parse(fs.readFileSync(file, 'utf8'));
    heads[arm] = record.head;
    base = record.base;
  }
  const engine = spawnSync(ENGINE_BIN, ['--version'], { encoding: 'utf8' }).stdout?.trim() || 'unknown';
  const parts = { compliance: complianceSummary(records), tone: toneSummary(records, loadDefects()), pairs: pairSummary(records) };
  const decision = decide(parts);
  const markdown = renderReport({
    meta: { date, base, heads, engine }, ...parts,
    craft: craftSummary(records), detect: detectSummary(records), missing: missingSummary(records), questions: questionSummary(records),
    cost: costSummary(path.join(out, 'cost.jsonl')), decision, attributionLine: attribution(parts, decision.improves),
  });
  fs.mkdirSync(reportDir, { recursive: true });
  fs.writeFileSync(path.join(reportDir, 'report.md'), markdown);
  fs.writeFileSync(path.join(reportDir, 'results.jsonl'), records.map((record) => JSON.stringify(record)).join('\n') + '\n');
  if (contactSheet) await writeContactSheet({ records, workDir: path.join(out, 'report-work'), file: path.join(reportDir, 'contact-sheet.jpg') });
  return { decision, reportDir };
}
```

- [ ] **Step 4: Run green**

Run: `node --test tests/wording-experiment/report.test.mjs`
Expected: 11 tests pass.

- [ ] **Step 5: Register and commit**

Append `'tests/wording-experiment/report.test.mjs',` to the suite's `files` list, then:

```bash
cd /Volumes/Containers/impeccable
node --test tests/test-suites.test.mjs
git add scripts/test-suites.mjs tests/wording-experiment/report.mjs tests/wording-experiment/report.test.mjs
git commit -m "wording-experiment: report and decision rule" -m "Aggregates compliance regressions after the rerun rule, paired defect differences with t intervals, confirmed labels and meta-comments, and pairwise design win rates with Wilson intervals; evaluates the spec's stop and go rule mechanically, states whether paraphrase explains nvc's gains, counts missing runs and cost, and renders a contact sheet of the judged pairs." -m "AI assistance: drafted with Claude Code."
```

**Checkpoint:** the decision rule is tested against every branch of the spec's rule with synthetic results.

---

## Task 12: `run.mjs`: the entry point

**Files:**
- Create: `tests/wording-experiment/run.mjs`
- Test: `tests/wording-experiment/run.test.mjs`
- Modify: `scripts/test-suites.mjs` (append `'tests/wording-experiment/run.test.mjs'`)

**Interfaces:**
- Consumes: every module above; `getModel` from the main checkout's `tests/skill-behavior/providers.mjs` (its import loads the main `.env`).
- Produces: CLI `node tests/wording-experiment/run.mjs --lane compliance|tone|builds|judge|report [--arm all|<arm>[,<arm>]] [--model <id>] [--dry-run] [--limit <n>] [--date YYYY-MM-DD]`. Exit 0 on success, 1 on an error, 3 on a cost-cap stop. Every run ends with one line: `<lane>: <n> new record(s); spent $<x> of $40; unpriced calls <k>`. Billed lanes (`compliance`, `tone`, `builds`) refuse to start unless all three arms have passing invariants at their current heads with clean trees. `--dry-run` writes under `wording-results-dry/` and uses stub models; the `report` lane writes to `docs/superpowers/research/<date>-wording-experiment/` (or `wording-results-dry/report/` on a dry run).

- [ ] **Step 1: Write the failing tests**

`tests/wording-experiment/run.test.mjs`:

```js
import { it } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { EXPERIMENT_DIR } from './config.mjs';

const run = (...args) => spawnSync(process.execPath, [path.join(EXPERIMENT_DIR, 'run.mjs'), ...args], {
  encoding: 'utf8',
  env: { ...process.env, WORDING_SCRATCH: fs.mkdtempSync(path.join(os.tmpdir(), 'wording-run-')) },
});

it('an unknown lane is refused before anything runs', () => {
  const result = run('--lane', 'bogus');
  assert.equal(result.status, 1);
  assert.match(result.stderr, /unknown lane "bogus"/);
});

it('a billed lane refuses to start until every arm has passed its invariants', () => {
  const result = run('--lane', 'tone', '--arm', 'nvc', '--dry-run');
  assert.equal(result.status, 1);
  assert.match(result.stderr, /invariants have not run for original/);
});

it('the report lane needs the run date that names the research folder', () => {
  const result = run('--lane', 'report', '--dry-run');
  assert.equal(result.status, 1);
  assert.match(result.stderr, /--date YYYY-MM-DD/);
});
```

- [ ] **Step 2: Run and watch them fail**

Run: `node --test tests/wording-experiment/run.test.mjs`
Expected: 3 failing; each child exits 1 with `Cannot find module .../run.mjs`, so the `stderr` assertions fail.

- [ ] **Step 3: Write `run.mjs`**

```js
// Entry point for every lane. See the plan's Task 12 for the flags.
import fs from 'node:fs';
import path from 'node:path';
import { parseArgs } from 'node:util';
// Decision 9: this import loads the main checkout's .env into process.env for this process and its children.
import { getModel } from '../skill-behavior/providers.mjs';
import { ARMS, ARMS_ROOT, COST_CAP_USD, DRY_RESULTS_ROOT, ENGINE_BIN, LANE_MODELS, REPO_ROOT, RESULTS_ROOT, armDir } from './config.mjs';
import { openLedger } from './cost.mjs';
import { requirePassingInvariants } from './invariants.mjs';
import { openStore } from './store.mjs';
import { stubModel } from './stub-model.mjs';

const LANES = ['compliance', 'tone', 'builds', 'judge', 'report'];
const BILLED_LANES = ['compliance', 'tone', 'builds'];

async function main() {
  const { values } = parseArgs({
    options: {
      lane: { type: 'string' },
      arm: { type: 'string', default: 'all' },
      model: { type: 'string' },
      'dry-run': { type: 'boolean', default: false },
      limit: { type: 'string' },
      date: { type: 'string' },
    },
  });
  const { lane } = values;
  if (!LANES.includes(lane)) throw new Error(`unknown lane "${lane}"; expected one of ${LANES.join(', ')}`);
  const dryRun = values['dry-run'];
  const arms = values.arm === 'all' ? ARMS : values.arm.split(',');
  for (const arm of arms) if (!ARMS.includes(arm)) throw new Error(`unknown arm "${arm}"; expected all or one of ${ARMS.join(', ')}`);
  const limit = values.limit === undefined ? Infinity : Number(values.limit);
  if (!(limit > 0)) throw new Error('--limit takes a positive number');
  if (lane === 'report' && !/^\d{4}-\d{2}-\d{2}$/.test(values.date ?? '')) {
    throw new Error('--lane report needs --date YYYY-MM-DD (the run date that names the research folder)');
  }
  if (!fs.existsSync(ENGINE_BIN)) throw new Error(`engine binary missing at ${ENGINE_BIN}; run cargo build --release -p impeccable in the main checkout`);
  process.env.IMPECCABLE_BIN = ENGINE_BIN; // Decision 10: set before any arm harness is imported.

  const out = dryRun ? DRY_RESULTS_ROOT : RESULTS_ROOT;
  const armDirs = Object.fromEntries(ARMS.map((arm) => [arm, armDir(arm, ARMS_ROOT)]));
  // The spec: an invariant failure blocks every billed lane, so all three arms are checked whichever arm runs.
  if (BILLED_LANES.includes(lane)) requirePassingInvariants({ arms: ARMS, out, armDirs });

  const store = openStore(path.join(out, 'store.jsonl'));
  const ledger = openLedger(path.join(out, 'cost.jsonl'), COST_CAP_USD);
  const modelFor = (modelId, kind) => (dryRun ? stubModel(kind) : getModel(modelId));
  const modelsFor = (name) => LANE_MODELS[name].filter((model) => !values.model || model === values.model);
  const before = store.all().length;

  if (lane === 'compliance') {
    const { runComplianceLane } = await import('./lanes/compliance.mjs');
    await runComplianceLane({ arms, armDirs, models: modelsFor('compliance'), store, ledger, out, dryRun });
  } else if (lane === 'tone') {
    const { runToneLane } = await import('./lanes/tone.mjs');
    await runToneLane({ arms, armDirs, models: modelsFor('tone'), store, ledger, dryRun, modelFor, limit });
  } else if (lane === 'builds') {
    const { runBuildsLane } = await import('./lanes/builds.mjs');
    await runBuildsLane({ arms, armDirs, models: modelsFor('builds'), store, ledger, out, dryRun, modelFor, limit });
  } else if (lane === 'judge') {
    const { runJudgeLane } = await import('./judge.mjs');
    await runJudgeLane({ store, ledger, out, dryRun, modelFor, limit });
  } else {
    const { writeReport } = await import('./report.mjs');
    const reportDir = dryRun
      ? path.join(out, 'report')
      : path.join(REPO_ROOT, 'docs', 'superpowers', 'research', `${values.date}-wording-experiment`);
    const { decision } = await writeReport({ store, out, reportDir, date: values.date });
    console.log(`report: ${path.join(reportDir, 'report.md')} (verdict ${decision.verdict})`);
  }
  console.log(`${lane}: ${store.all().length - before} new record(s); spent $${ledger.spent().toFixed(4)} of $${COST_CAP_USD}; unpriced calls ${ledger.unpriced()}`);
}

main().catch((error) => {
  if (error.name === 'CapExceeded') {
    console.error(`STOPPED: ${error.message}`);
    process.exitCode = 3;
    return;
  }
  console.error(error.stack ?? String(error));
  process.exitCode = 1;
});
```

- [ ] **Step 4: Run green, then the whole unit suite**

Run: `node --test tests/wording-experiment/run.test.mjs && node scripts/run-tests.mjs wording-experiment`
Expected: the three run tests pass; the suite runs every file registered so far (store, cost, invariants, arms, compliance, fixtures, tone, builds, judge, report, run) with `# fail 0`.

- [ ] **Step 5: Register and commit**

Append `'tests/wording-experiment/run.test.mjs',` to the suite's `files` list, then:

```bash
cd /Volumes/Containers/impeccable
node --test tests/test-suites.test.mjs
git add scripts/test-suites.mjs tests/wording-experiment/run.mjs tests/wording-experiment/run.test.mjs
git commit -m "wording-experiment: run entry point" -m "One CLI for every lane with --arm, --model, --limit, --dry-run, and --date; billed lanes refuse to start until all three arms pass their invariants; a cost-cap stop exits 3 and names what it stopped before." -m "AI assistance: drafted with Claude Code."
```

**Checkpoint:** every lane is reachable from one CLI; the unit suite is green.

---

## Task 13: End-to-end dry run (stub models, stub judge, no network, no cost)

**Files:** none created in the repo; outputs go to `.../scratchpad/wording-results-dry/`.

**Interfaces:**
- Consumes: the three committed arms (Tasks 5 and 6), every module.
- Produces: a dry store with the counts below, a dry report with verdict `STOP` (the stub makes scenario 2 regress in nvc), and a contact sheet; the proof that the billed tasks can run.

- [ ] **Step 1: Start clean and run the invariants into the dry folder**

```bash
cd /Volumes/Containers/impeccable
export DRY=/private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/wording-results-dry
rm -rf "$DRY"
node tests/wording-experiment/invariants.mjs --out "$DRY"
```

Expected: `original: PASSED`, `nvc: PASSED`, `paraphrase: PASSED`, each at its committed head.

- [ ] **Step 2: Run every lane with `--dry-run`, one at a time**

```bash
node tests/wording-experiment/run.mjs --lane compliance --dry-run
node tests/wording-experiment/run.mjs --lane tone --dry-run
node tests/wording-experiment/run.mjs --lane builds --dry-run
node tests/wording-experiment/run.mjs --lane judge --dry-run
node tests/wording-experiment/run.mjs --lane report --dry-run --date 2026-09-26
```

Expected closing lines, in order: `compliance: 60 new record(s)` (3 arms x 3 models x (5 scenarios + 1 sweep marker), plus 2 reruns per model), `tone: 60 new record(s)`, `builds: 72 new record(s)`, `judge: 204 new record(s)` (60 reports + 3 comparisons x 2 models x 12 briefs x 2 orders), then `report: .../wording-results-dry/report/report.md (verdict STOP)`. The spend stays under $1 and `unpriced calls 3` (one stub provider failure per compliance model).

- [ ] **Step 3: Check the store counts independently**

```bash
node --input-type=module -e "
import fs from 'node:fs';
const rows = fs.readFileSync(process.env.DRY + '/store.jsonl', 'utf8').trim().split('\n').map((line) => JSON.parse(line));
const by = {};
for (const row of rows) by[row.lane + ':' + row.status] = (by[row.lane + ':' + row.status] ?? 0) + 1;
console.log(JSON.stringify(by));
" 
```

Run it in the same shell as Step 1 (`DRY` is exported there). Expected: `{"compliance:ok":60,"tone:ok":60,"builds:ok":72,"judge:ok":204}` (key order may differ). Any `missing` count or a different total is a defect: find it before any billed task.

- [ ] **Step 4: Prove resume pays for nothing twice**

Run: `node tests/wording-experiment/run.mjs --lane tone --dry-run && node tests/wording-experiment/run.mjs --lane compliance --dry-run`
Expected: `tone: 0 new record(s)` and `compliance: 0 new record(s)`, spend unchanged.

- [ ] **Step 5: Read the dry report and the contact sheet**

Open `$DRY/report/report.md`: it states `**Decision rule verdict: STOP**` with the reason `nvc has 3 compliance regression(s) that survived the rerun`, the three comparison rows, a missing-run section whose counts are all zero, and the cost section. `rtk proxy grep -c "$(printf '\342\200\224')" "$DRY/report/report.md"` prints `0`. `$DRY/report/contact-sheet.jpg` exists and shows 24 rows of three stub pages. Inspect one stored tone record and confirm `userText` is the stub report text, and one builds record's `shots.desktop` exists.

- [ ] **Step 6: Checkpoint (no commit)**

Nothing in the repo changed. Record in the checkpoint: the dry counts, the dry spend, and that the three arm heads passed invariants. If any step deviated, fix the code (with a test for the defect, committed on `wording-experiment`) and rerun Task 13 from Step 1 before any billed task.

---

## Task 14: BILLED: lane 1, compliance, three arms in sequence

**Files:** results under `.../scratchpad/wording-results/`; a checkpoint copy committed under `docs/superpowers/research/<RUN_DATE>-wording-experiment/`.

**Interfaces:**
- Consumes: passing invariants for all arms in `wording-results/invariants/`; `OPENAI_API_KEY` and `OPENROUTER_API_KEY` in the main `.env`.
- Produces: compliance records for 3 arms x 3 models, reruns for every candidate, and the running spend.

- [ ] **Step 1: Preconditions (free)**

```bash
cd /Volumes/Containers/impeccable
rtk proxy grep -o '^[A-Z_]*=' .env
node tests/wording-experiment/invariants.mjs
date +%F > /private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/wording-results/run-date.txt
```

Expected: the names include `OPENAI_API_KEY=` and `OPENROUTER_API_KEY=`; all three arms `PASSED`; `run-date.txt` holds today's date (`RUN_DATE`, used by every later task). Task 13 must have passed on these same arm heads; if an arm moved since, rerun Task 13 first.

- [ ] **Step 2: Pilot: the original arm, one model at a time**

```bash
node tests/wording-experiment/run.mjs --lane compliance --arm original --model gpt-5.6-terra
node tests/wording-experiment/run.mjs --lane compliance --arm original --model anthropic/claude-sonnet-5
node tests/wording-experiment/run.mjs --lane compliance --arm original --model google/gemini-3.7-flash
```

Expected per run: `compliance: <n> new record(s)` where n is the scenario count plus 1, no skip error, and a rising spend. Sanity-check the original arm's pass counts against the suite's recent baseline in `tests/skill-behavior/README.md`; a model far below its baseline points at the environment (key, binary, network), not at wording: stop and diagnose.

- [ ] **Step 3: Projection gate**

Read the spend printed by the last pilot run (`S1`). The projected lane-1 total is `3 x S1 x 1.1` (three arms, 10% for reruns). If it exceeds $15, stop here and report `S1`, the projection, and the remaining budget to the owner; do not continue without an answer.

- [ ] **Step 4: The rewritten arms**

```bash
node tests/wording-experiment/run.mjs --lane compliance --arm nvc
node tests/wording-experiment/run.mjs --lane compliance --arm paraphrase
```

Expected: each ends with its new-record count; the paraphrase run also performs the reruns for every model (its three sweeps complete the set), so its count includes `::rerun` records. Exit 3 means the cap stopped the run: report to the owner and stop. Rerunning a command after a crash resumes where it stopped.

- [ ] **Step 5: Read the lane-1 result**

```bash
node --input-type=module -e "
import { openStore } from './tests/wording-experiment/store.mjs';
import { complianceSummary } from './tests/wording-experiment/report.mjs';
const store = openStore('/private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/wording-results/store.jsonl');
const summary = complianceSummary(store.all());
for (const row of summary.rows) console.log(row.arm, row.model, 'pass', row.pass, 'fail', row.fail, 'missing', row.missing);
console.log('regressions', JSON.stringify(summary.regressions));
"
```

Expected: nine rows and the regression lists. Any candidate with `confirmed: false` means a rerun is missing: rerun `--lane compliance --arm all` once to fill it.

- [ ] **Step 6: Checkpoint commit of the raw data**

```bash
cd /Volumes/Containers/impeccable
RUN_DATE=$(cat /private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/wording-results/run-date.txt)
mkdir -p "docs/superpowers/research/${RUN_DATE}-wording-experiment"
cp /private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/wording-results/store.jsonl "docs/superpowers/research/${RUN_DATE}-wording-experiment/results.jsonl"
cp /private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/wording-results/cost.jsonl "docs/superpowers/research/${RUN_DATE}-wording-experiment/cost.jsonl"
git add "docs/superpowers/research/${RUN_DATE}-wording-experiment/results.jsonl" "docs/superpowers/research/${RUN_DATE}-wording-experiment/cost.jsonl"
git commit -m "wording experiment: lane 1 results" -m "Raw compliance records for the original, nvc, and paraphrase arms on gpt-5.6-terra, claude-sonnet-5, and gemini-3.7-flash, with reruns, and the cost ledger so far. Checkpoint copy; the report follows." -m "AI assistance: drafted with Claude Code."
```

**Checkpoint:** lane 1 done; spend so far recorded; regressions listed. If nvc has a confirmed regression, the decision rule already says STOP for stage 2: finish lanes 2 and 3 anyway only if the owner says so (they still inform the report); otherwise go straight to Task 16. Ask the owner which.

---

## Task 15: BILLED: lane 2 (tone), lane 3 (builds), then the judge

**Files:** results under `.../scratchpad/wording-results/`; checkpoint copies in the research folder.

**Interfaces:**
- Consumes: passing invariants (checked again by `run.mjs`), the lane-1 spend, Chromium installed (Task 9 Step 1), the boot probe (Task 8 Step 6b).
- Produces: 60 tone records, 72 build records with screenshots and detect findings, 204 judge records, and the spend.

- [ ] **Step 1: Pilots: one critique and one build per model on the original arm, then a judge pilot**

```bash
cd /Volumes/Containers/impeccable
node tests/wording-experiment/run.mjs --lane tone --arm original --limit 1
node tests/wording-experiment/run.mjs --lane builds --arm original --limit 1
```

Expected: `tone: 2 new record(s)` and `builds: 2 new record(s)`. Open the pilot build's `desktop.png` (under `wording-results/builds/original/<model>/tidewell-landing/`) and read the pilot critiques in the store: a page must exist, and each critique's `userText` must contain a report heading (`reportSeen: true`, `Priority issues` or a health score heading). A critique that stopped on a question with no report means Decision 36's stop rule or the boot output (Task 8 Step 6b) is wrong; stop and report it rather than paying for 58 more.

```bash
node --input-type=module -e "
import { openStore } from './tests/wording-experiment/store.mjs';
const store = openStore('/private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/wording-results/store.jsonl');
for (const r of store.all({ lane: 'tone' })) console.log(r.model, r.item, r.status, 'reportSeen=' + r.data.reportSeen, 'questions=' + r.data.questions, 'outcome=' + r.data.outcome);
"
```

Expected: two lines, each `ok reportSeen=true`.

Then pilot the judge, which needs one pair: build the same brief once on the nvc arm with one model, and judge three items (the two pilot reports and one pair order):

```bash
node tests/wording-experiment/run.mjs --lane builds --arm nvc --limit 1 --model anthropic/claude-sonnet-5
node tests/wording-experiment/run.mjs --lane judge --limit 3
node --input-type=module -e "
import { openStore } from './tests/wording-experiment/store.mjs';
const store = openStore('/private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/wording-results/store.jsonl');
for (const r of store.all({ lane: 'judge' })) console.log(r.arm, r.item, r.status, r.data.error ?? '');
"
```

Expected: `builds: 1 new record(s)`, `judge: 3 new record(s)`, and three lines with status `ok`, one of them a `pair:tidewell-landing#0` item. A judge that throws "nothing was recorded" (Decision 35) has a wrong slug or a rejected request shape: fix it and rerun this step; nothing was stored, so resume picks the items up again.

- [ ] **Step 2: Projection gate**

```bash
node --input-type=module -e "
import fs from 'node:fs';
const file = '/private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/wording-results/cost.jsonl';
const lines = fs.readFileSync(file, 'utf8').trim().split('\n').map((line) => JSON.parse(line));
const spent = lines.reduce((sum, line) => sum + (line.usd ?? 0), 0);
const mean = (lane, model) => { const xs = lines.filter((line) => line.lane === lane && line.model === model && line.usd != null).map((line) => line.usd); return xs.reduce((a, b) => a + b, 0) / xs.length; };
const models = ['anthropic/claude-sonnet-5', 'gpt-5.6-terra'];
const tone = models.reduce((sum, model) => sum + mean('tone', model) * 29, 0);
const builds = models.reduce((sum, model) => sum + mean('builds', model) * 35, 0);
const judge = 204 * 0.02;
console.log(JSON.stringify({ spent: spent.toFixed(2), tone: tone.toFixed(2), builds: builds.toFixed(2), judge: judge.toFixed(2), projected: (spent + tone + builds + judge).toFixed(2), ceiling: 38.5 }));
"
```

The multipliers are the remaining runs per model (tone 30 minus the pilot, builds 36 minus the pilot; claude-sonnet-5 has one more build pilot, so the builds figure is slightly high, the safe direction); the judge allowance is a conservative $0.02 per call at Gemini flash rates. Expected: one JSON line. If `projected` exceeds 38.5 (the cap less the minimum reserve), stop and send the owner the line with three options: drop one model from lanes 2 and 3, cut the briefs to two per mode, or raise the cap. Do not choose for them.

- [ ] **Step 3: Tone lane, all arms**

Run: `node tests/wording-experiment/run.mjs --lane tone`
Expected: `tone: 58 new record(s)` (60 minus the 2 pilots). Exit 3 is a cap stop: report and stop. Then copy `store.jsonl` and `cost.jsonl` into the research folder and commit exactly as in Task 14 Step 6, with the subject `wording experiment: lane 2 results`.

- [ ] **Step 4: Builds lane, all arms**

Run: `node tests/wording-experiment/run.mjs --lane builds`
Expected: `builds: 69 new record(s)` (72 minus the 3 pilots). Spot-check three builds' screenshots from different arms. Checkpoint-commit as in Step 3 with the subject `wording experiment: lane 3 results`.

- [ ] **Step 5: Judge**

Run: `node tests/wording-experiment/run.mjs --lane judge`
Expected: `judge: <n> new record(s)` with n at most 201 (60 reports plus up to 144 pair orders, less the 3 piloted; pairs with a missing build are skipped and counted by the report). Checkpoint-commit as in Step 3 with the subject `wording experiment: judge verdicts`.

- [ ] **Step 6: Missing-run check**

```bash
node --input-type=module -e "
import { openStore } from './tests/wording-experiment/store.mjs';
import { missingSummary } from './tests/wording-experiment/report.mjs';
const store = openStore('/private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/wording-results/store.jsonl');
console.log(JSON.stringify(missingSummary(store.all())));
"
```

Expected: a JSON line; `absent` values of zero mean every lane ran fully. Missing runs are reported, not rerun (the spec: retried once, then recorded as missing).

**Checkpoint:** all billed work done; spend, missing counts, and three checkpoint commits recorded.

---

## Task 16: Report, commit, and hand the decision to the owner

**Files:**
- Create: `docs/superpowers/research/<RUN_DATE>-wording-experiment/report.md`, `contact-sheet.jpg`; overwrite `results.jsonl` with the final store.

**Interfaces:**
- Consumes: the final store, ledger, and invariants records.
- Produces: the committed report and a hand-off message.

- [ ] **Step 1: Write the report**

```bash
cd /Volumes/Containers/impeccable
RUN_DATE=$(cat /private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/wording-results/run-date.txt)
node tests/wording-experiment/run.mjs --lane report --date "$RUN_DATE"
cp /private/tmp/claude-501/-Volumes-Containers-impeccable/adf81fc6-abd8-4db3-ae30-3a51b3a3424a/scratchpad/wording-results/cost.jsonl "docs/superpowers/research/${RUN_DATE}-wording-experiment/cost.jsonl"
rtk proxy grep -c "$(printf '\342\200\224')" "docs/superpowers/research/${RUN_DATE}-wording-experiment/report.md"
```

Expected: `report: .../report.md (verdict <STOP|GO|OWNER_DECIDES>)`; the em-dash count is `0` (ripgrep exits 1 on zero matches; that is the pass). Read `report.md` top to bottom with the Read tool (it contains tables) and check it against the store: the verdict and its reasons, the attribution sentence, the compliance table, the defects paired differences, the label and meta-comment counts, the three win rates with intervals, the missing-run count, and the spend. Open `contact-sheet.jpg`.

- [ ] **Step 2: Commit the outputs**

```bash
git add "docs/superpowers/research/${RUN_DATE}-wording-experiment/report.md" "docs/superpowers/research/${RUN_DATE}-wording-experiment/results.jsonl" "docs/superpowers/research/${RUN_DATE}-wording-experiment/cost.jsonl" "docs/superpowers/research/${RUN_DATE}-wording-experiment/contact-sheet.jpg"
git commit -m "wording experiment: stage 1 report" -m "Report, final results, cost ledger, and contact sheet for the stage 1 wording experiment. The decision rule verdict is in report.md; the arm branches stay unmerged." -m "AI assistance: drafted with Claude Code."
```

- [ ] **Step 3: Hand off and stop**

Send the owner: the verdict line and its reasons, the attribution sentence, the spend and the missing-run count, the report path, and the arm heads. State plainly that no arm is merged, `wording-experiment` is not merged, and the worktrees remain in the scratchpad (teardown with `node tests/wording-experiment/arms.mjs teardown` only on request; the branches stay either way). Then stop. The executor never merges an arm, never opens a PR, and never starts stage 2; a winning text reaches `main` only through its own PR after the owner reads the report.

**Checkpoint:** the report is committed on `wording-experiment`; the decision is with the owner.
