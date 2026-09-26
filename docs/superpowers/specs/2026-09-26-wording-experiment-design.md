# Wording experiment: non-violent phrasing in the skill text

Status: design approved by the fork owner, 2026-09-26. Stage 1 only; stage 2 gets its own plan after the stage 1 report.

## Why

Three times, models from different families remarked that the skill's wording reads as offensive. The skill text is mostly prohibitive (222 "do not/don't", 172 "never", 33 "ban") and some of it judges the work itself ("slop" 12, "lazy" 15, "generic" 26). Two known mechanisms make that worth testing rather than assuming:

- Prohibitions name the thing they forbid, and emphatic absolutes make current models over-cautious. The root CLAUDE.md already records a case where design-time reminders made output "safe, underdesigned".
- Judgmental vocabulary can reach the user: critique and audit reports are written about the user's work.

The risk runs the other way too: softer text can let models skip constraints. The experiment measures both.

## Question

Does rewriting the skill's prohibitive and judgmental phrasing into non-violent-communication form (an observation, the visitor need it fails, a concrete request), with every constraint kept, change:

1. instruction following (does the model still do what the skill requires),
2. the tone and thoroughness of critique and audit reports, and
3. the design quality of built pages?

## Arms

Three branches off one fork `main` commit:

| Arm | Branch | Text |
|---|---|---|
| original | `wording-arm-original` | unchanged |
| nvc | `wording-arm-nvc` | every prohibition or judgment becomes observation, visitor need, request |
| paraphrase | `wording-arm-paraphrase` | the same sentences rewritten to a comparable degree, prohibitive framing kept |

The paraphrase arm separates the effect of tone from the effect of rewriting at all.

Example of one rule in each arm:

- original: "Never ship generic, lazy defaults: they are slop."
- nvc: "When a page keeps the framework's stock defaults, visitors read it as unconsidered and trust it less. Choose each default on purpose."
- paraphrase: "Do not ship stock framework defaults; they read as unconsidered."

### Stage 1 scope

`skill/SKILL.src.md` (the shared design laws section only), `skill/reference/craft-floor.md`, `skill/reference/critique.md`, `skill/reference/audit.md`, `skill/agents/impeccable-finish-reviewer.md`. These carry about 145 of the 426 charged-word matches in `skill/`, and they are the text that shapes builds, critique and audit reports, and the reviewer's voice. `new-work.md` and `live.md` stay untouched in stage 1 so a routing change can be traced to a file.

### Rewrite invariants

Checked by a script on every arm before any billed run:

- Constraint parity: for each rule in an original file, the same core instruction exists in every arm (same thresholds, values, file names, commands, and `<!-- rule:... -->` markers). The rule list is extracted from the original once and checked by fixed-string presence of its key tokens (numbers, identifiers, paths, commands).
- Length parity: each file's word count within 10% of the original.
- Gates: no em dashes, `bun run build` passes, `node --test tests/skill-reference.test.mjs` passes.
- Absolutes stay absolute: where the original states a true hard limit (for example writing outside the workspace, auto-running an update), the nvc arm keeps it firm and adds the reason. NVC changes the framing, never the strength of a constraint.

The rewrites are drafted by Claude and reviewed by a separate reviewer that checks both rewritten arms for equal rewrite effort.

## Measures

### Lane 1: compliance

`bun run test:skill-behavior` (78 scenarios) in each arm's worktree, on `gpt-5.6-terra` (OpenAI), `anthropic/claude-sonnet-5` and `google/gemini-3.7-flash` (OpenRouter). Prerequisite: the OpenRouter provider in `tests/skill-behavior/providers.mjs` is committed to the fork's `main` first, so no arm needs a file overlay.

Metric: passes per arm. A regression is a scenario red in an arm, green in `original` for the same model, and red again on one rerun.

### Lane 2: critique and audit

Five fixture pages (landing, dashboard, form, docs, portfolio), each with five or six planted defects recorded in `defects.json` (low contrast, missing focus state, placeholder copy, stock gradient, cramped spacing, and similar). Each arm runs `/impeccable critique` and `/impeccable audit` on each page with `anthropic/claude-sonnet-5` and `gpt-5.6-terra`: 5 x 2 x 2 x 3 = 60 runs.

A blinded judge scores each user-facing report on:

- defects found: planted defects the report names (objective recall);
- specificity: each finding names the element and a concrete change;
- judgmental labels aimed at the work or its author, counted by a lexicon and confirmed by the judge;
- meta-comments about the skill's own tone.

### Lane 3: builds

Twelve briefs, three per mode (Persuade, Operate, Read, Experience), each with a prepared `PRODUCT.md` and an instruction to build one `index.html` without asking questions (the harness's simulated user answers any question anyway). 12 x 2 models x 3 arms = 72 builds, screenshotted with Playwright at 1440 and 390 pixels wide.

A blinded pairwise judge compares same-brief, same-model outputs across arms, each pair shown twice with the order swapped, and records: overall preference, a craft-floor checklist (contrast, spacing, hierarchy), and a stock-default look score. `impeccable detect --json` runs on every page as a judge-independent measure.

### Judge

`google/gemini-3.7-flash` or a stronger Gemini tier if available through OpenRouter: a family that generates nothing in lanes 2 and 3. Outputs carry random ids; the judge never sees an arm name.

## Decision rule (fixed before any billed run)

Comparisons: nvc vs paraphrase (tone), nvc vs original (the full change), paraphrase vs original (rewrite alone).

Reporting:

- Builds: win rate over 24 pairs per comparison (12 briefs x 2 models, both orders averaged) with a 95% interval. 24 pairs detect only a large effect (about a 70/30 split); a smaller one is reported as "no detectable difference".
- Defects found: paired difference over 20 cells (fixture x command x model).
- Labels and meta-comments: counts per arm.
- Compliance: regressions, after the rerun rule.

Stage 2 (whole skill):

- Stop if nvc has any compliance regression that survives the rerun, or finds one or more fewer planted defects per report on average than original.
- Go if neither holds, nvc does not clearly lose on design (it loses no more than 16 of 24 pairs to original), and at least one outcome improves (fewer labels, fewer meta-comments, or a better design win rate).
- Neutral on everything: the owner decides; tone alone may justify it.
- If paraphrase matches nvc's gains over original, the report says the gain came from rewriting, not from NVC.

## Components

All new code lives in `tests/wording-experiment/`, opt-in, never in the default `bun run test`:

| File | Role |
|---|---|
| `invariants.mjs` | the pre-run arm checks above; a failure blocks every billed lane |
| `fixtures/pages/*.html`, `fixtures/defects.json` | lane 2 inputs and ground truth |
| `fixtures/briefs.json` | lane 3 briefs and their PRODUCT.md text |
| `lanes/tone.mjs`, `lanes/builds.mjs` | drive `runTurn` imported from the arm worktree's own `tests/skill-behavior/harness.mjs`, so each arm loads its own skill text; builds also screenshot and run detect |
| `judge.mjs` | blinding, order swap, judge calls, raw verdicts saved |
| `cost.mjs` | per-call cost from token usage and a price table; hard stop at $40 |
| `run.mjs`, `report.mjs` | entry point (`--lane`, `--arm`, `--dry-run`) and aggregation into the report |

Arms are git worktrees on the session scratchpad volume (not the SD card that wedged during the follow-ups work), with `node_modules` linked.

## Execution

Order, one billed job at a time: invariants, compliance (three arms), tone, builds, judge, report. A `--dry-run` with a stub model exercises the whole pipeline first at no cost.

Every run appends one JSON line keyed by (lane, arm, model, item); a rerun skips keys already present, so a crash or the cost cap never repeats paid work. A provider error is retried once, then recorded as missing, and the report counts missing runs.

## Budget

About $35 for stage 1, hard cap $40:

| Lane | Estimate |
|---|---|
| Compliance, 3 arms x 78 scenarios x 3 models | about $12 |
| Tone, 60 runs plus judging | about $7 |
| Builds, 72 runs plus 144 judged comparisons | about $16 |

Estimates come from the 2026-09-24 and 2026-09-25 sweeps ("a few dollars" per 78-scenario sweep); `cost.mjs` reports actuals.

## Outputs

`docs/superpowers/research/<date>-wording-experiment/report.md` and `results.jsonl`, plus a committed contact sheet of the judged pairs. Screenshots stay in the scratchpad. Arm branches stay unmerged; a winning text reaches `main` only through its own PR after the owner reads the report.

## Out of scope

- Stage 2 (whole skill): its own plan after the stage 1 report.
- Upstream: nothing here goes to `pbakaus/impeccable` unless the results support it, and then only as an issue first.
- `live.md` and `new-work.md` in stage 1.
- Terminal-platform builds in lane 3 (no screenshot path for text captures).

## Known limits

- One LLM rewrite per arm: the result speaks for these rewrites, not for NVC phrasing in general.
- A single judge family; judge bias is reduced by blinding and order swapping, not removed.
- 24 pairs per comparison detect only large design effects.
