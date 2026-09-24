# Terminal Platform PR 1 (Guidance) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `terminal` a recognized platform value so a TUI project gets terminal design guidance at boot, terminal variants of `audit` and `adapt`, and a doctor nudge when its manifests say terminal but PRODUCT.md says web.

**Architecture:** The `context` crate parses the value, inlines `reference/terminal.md` at boot, and gains terminal dependency evidence in the staleness module. The skill text routes `audit` and `adapt` to `.terminal.md` variants and stops treating `terminal` as a native platform for detector purposes. Every behavior change is pinned by an oracle golden recorded from the binary and reviewed by hand.

**Tech Stack:** Rust (stable, `cargo test --workspace`), the oracle harness (`tests/oracle/`, Node), Bun for the skill build and JS tests, Markdown reference files under `skill/reference/`.

**Spec:** `docs/superpowers/specs/2026-09-24-terminal-platform-design.md` (sections 1, 2, and the PR 1 part of section 6).

## Global Constraints

- The platform value is the literal string `terminal`. Never `tui` or `cli`.
- `terminal` is NOT native for detector purposes: `is_native` in `crates/context/src/context_cli.rs` and `is_native_platform` in `crates/hook/src/hook_lib.rs` stay unchanged. Live mode stays skipped for `terminal`.
- No em dashes anywhere in `skill/**/*.md`, `README.md`, or `README.npm.md`. `bun run build` fails on them. Also banned in `skill/`: `load-bearing`, `highest-leverage`, `biggest unlock`, `reflex defaults`, `collapses into monoculture`, `data-driven`, `delve`, `tapestry`, `in today's`, `gone are the days`, `let's dive in`, `in summary`, `in conclusion`.
- No version bumps, no changelog entries, no edits to generated provider dirs (`.claude/skills/`, `.cursor/`, `.agents/`, `plugin/`, and siblings). Edit `skill/`, `crates/`, `tests/`, `docs/` only.
- Commit messages: first line is a sentence in the repo's style (`Add ...`, `Fix: ...`). Body ends with the line `AI assistance: drafted with Claude Code.` No `Co-Authored-By` trailer.
- Oracle goldens are recorded from the binary with `node tests/oracle/record.mjs --bin <prefix>` and reviewed by hand before commit. Any existing golden that changes gets a line in `tests/oracle/DELTAS.md`.
- The binary used for tests is the local build: `cargo build --release -p impeccable` then `export IMPECCABLE_BIN="$PWD/target/release/impeccable"`.
- Work on a branch named `terminal-platform-pr1` off the fork's `main`. Merge to `main` when Task 9 passes.

---

## File map

| File | Responsibility in this PR |
|---|---|
| `skill/reference/terminal.md` | Create. The terminal design reference inlined at boot. |
| `skill/reference/audit.terminal.md` | Create. Audit variant for terminal projects; mirrors `audit.md`'s report skeleton. |
| `skill/reference/adapt.terminal.md` | Create. Adapt variant: width classes and color tiers. |
| `skill/SKILL.src.md` | Modify. Description, Modes sentence, Commands table rows for `audit` and `adapt`, Setup routing bullet, post-init sentence. |
| `skill/reference/routing.md` | Modify. Live stays skipped for `terminal`; detect no longer skipped for `terminal`. |
| `skill/reference/init.md` | Modify. Platform hypothesis, bare-value list, post-init reference load. |
| `skill/reference/audit.md`, `skill/reference/adapt.md` | Modify. The web-only guard names the terminal variant. |
| `crates/context/src/context.rs` | Modify. `extract_platform` accepts `terminal`; unit tests. |
| `crates/context/src/context_cli.rs` | Modify. Boot inlines `terminal.md`; WARNING lists `terminal`. |
| `crates/context/src/staleness.rs` | Modify. Terminal dependency evidence; unit tests. |
| `crates/context/src/roll_selection.rs` | Modify. `COMPOSITION_PLATFORMS` gains `terminal`. |
| `tests/oracle/workspaces/ctx-terminal/` | Create. Fixture project declaring `terminal`. |
| `tests/oracle/cases/context.mjs` | Modify. New cases. |
| `tests/oracle/golden/*.json` | Record. New goldens; re-recorded goldens listed in `DELTAS.md`. |
| `tests/oracle/DELTAS.md` | Modify. One section for this PR. |
| `tests/skill-reference.test.mjs` | Modify. Pin the terminal routing text. |
| `docs/CLI-CONTRACT.md` | Modify. Contract lines that enumerate platforms. |
| `CLAUDE.md` (repo root) | Modify. Platform section gains `terminal`; variant rule gains `.terminal.md`. |

---

### Task 1: Branch and baseline

**Files:** none created.

- [ ] **Step 1: Branch**

```bash
cd /Volumes/Containers/impeccable
git checkout main && git pull --ff-only origin main
git checkout -b terminal-platform-pr1
```

- [ ] **Step 2: Build the binary and confirm the oracle is green before any change**

```bash
cargo build --release -p impeccable
export IMPECCABLE_BIN="$PWD/target/release/impeccable"
node tests/oracle/run.mjs context- | tail -3
```

Expected: the last line reports zero failures (no lines beginning `XX ` or `?? `). If it is not green, stop and report; do not proceed on a red baseline.

---

### Task 2: `reference/terminal.md`

**Files:**
- Create: `skill/reference/terminal.md`
- Test: `bun run build` (prose gate) and `tests/skill-reference.test.mjs`

**Interfaces:**
- Produces: the file path `reference/terminal.md`, read by `load_native_platform_references` (Task 5) and named by `init.md` and `SKILL.src.md` (Task 8). Every bullet carries a `<!-- rule:terminal-<section>-<slug> -->` marker.

- [ ] **Step 1: Write the failing test**

Append to the `describe` block in `tests/skill-reference.test.mjs`, before its closing `});`:

```js
  it('gives terminal projects a reference shaped like the mobile ones', () => {
    const terminal = readFileSync(join(ROOT, 'skill/reference/terminal.md'), 'utf-8').replace(/\r\n?/g, '\n');
    for (const heading of [
      '## The terminal slop test',
      '## Layout & structure',
      '## Keyboard & input',
      '## Typography',
      '## Color',
      '## Components & controls',
      '## Motion',
      '## Verifying the build',
    ]) {
      assert.ok(terminal.includes(`\n${heading}\n`), `missing ${heading}`);
    }
    const bullets = terminal.split('\n').filter((l) => l.startsWith('- **'));
    assert.ok(bullets.length >= 25, `expected at least 25 bullets, found ${bullets.length}`);
    for (const b of bullets) assert.match(b, /<!-- rule:terminal-[a-z0-9-]+ -->$/, `bullet lacks a rule marker: ${b.slice(0, 60)}`);
    assert.match(terminal, /NO_COLOR/);
    assert.match(terminal, /tmux capture-pane/);
    assert.doesNotMatch(terminal, /—/, 'em dash');
  });
```

- [ ] **Step 2: Run it to verify it fails**

Run: `node --test tests/skill-reference.test.mjs`
Expected: FAIL with `ENOENT ... skill/reference/terminal.md`.

- [ ] **Step 3: Create the reference**

Write `skill/reference/terminal.md` with exactly this content:

````markdown
# Terminal platform

For apps whose primary surface renders in a terminal: full-screen TUIs and rich line-mode CLIs built with Ratatui, Bubble Tea and Lip Gloss, Textual and Rich, Ink, or their neighbors.

In a terminal the visitor mode narrows to Operate or Read almost every time. The grid, the user's theme, and the keyboard govern structure; brand expresses through the layer the terminal leaves open: one accent, weight and dim, spacing, copy.

## The terminal slop test

A terminal surface reads as generated when it opens with a figlet banner in a gradient, wraps every region in a double-line box nested inside another box, puts an emoji in front of every line, hard-codes RGB colors that vanish on a light theme, spins a braille spinner that never stops and keeps spinning into CI logs, assumes 80 by 24 and centers everything in it, and gives the user no clue which key quits. The tell is decoration with no information behind it. A designed terminal uses the user's 16 colors first, earns each glyph, keeps the footer honest about the keys that work right now, and looks the same in a pipe as on screen, minus the color.

## Layout & structure

- **Design at 80 columns, lay out by constraint.** Compute every region from the current size (`Frame::area()`, `WindowSizeMsg`, `useWindowSize`, Textual CSS) with percentage, minimum, and fill constraints. A fixed length or a literal rectangle is an absolute size that does not respond to the window. <!-- rule:terminal-layout-constraint -->
- **Pick the screen mode the task deserves.** Alternate screen for a resident app (editor, monitor); inline under the prompt for a tool the user runs and returns from. The alternate screen throws away scrollback and history. <!-- rule:terminal-layout-screen-mode -->
- **Handle resize as a first-class event.** The first size arrives at boot and resizes arrive in batches; rerun layout on each and never cache a width. <!-- rule:terminal-layout-resize -->
- **One archetype per app.** Miller columns for drill-down, a widget grid for monitoring, three panels (navigate, work, inspect) for editing. Do not mix them on one screen. <!-- rule:terminal-layout-one-archetype -->
- **Borders are structure, not decoration.** One border weight per level, single-line by default, no box inside a box unless the inner one is focusable. The ASCII border set is the fallback, not an afterthought. <!-- rule:terminal-layout-borders -->
- **Leave the terminal as you found it.** Raw mode and the alternate screen are released on every exit path, including panic. <!-- rule:terminal-layout-restore -->

## Keyboard & input

- **Keyboard first, mouse optional.** Every action has a key. Mouse tracking is an opt-in that some connections cannot deliver, and there is no hover unless the mouse is bound. <!-- rule:terminal-input-keyboard-first -->
- **The footer shows the 3 to 5 keys that work now.** The full reference sits behind `?`; the footer changes with focus. <!-- rule:terminal-input-footer -->
- **Make escape obvious.** `q`, `Esc`, or `Ctrl-C` always works, and the footer says which. <!-- rule:terminal-input-escape -->
- **Prompt only on a TTY, and never require the prompt.** Off a TTY, fail with the flag to pass instead. <!-- rule:terminal-input-tty-prompt -->

## Typography

- **Monospace is the only face; weight, dim, and reverse are the whole type scale.** Hierarchy comes from bold, dim, spacing, and position, not from size. <!-- rule:terminal-typo-scale -->
- **Measure in cells, not characters.** Width comes from a width library (`unicode-width`, `go-runewidth`, `wcwidth`, `string-width`). CJK is two cells, combining marks zero, emoji sequences two, and terminals disagree at the edges. <!-- rule:terminal-typo-cells -->
- **Truncate on grapheme boundaries with an ellipsis, never on bytes.** A cut inside a joiner sequence or a surrogate pair corrupts the line and everything after it. <!-- rule:terminal-typo-grapheme-truncate -->
- **No figlet banners, no block-letter titles.** Concise help when run bare, the tool's name once in the header, nothing that scrolls the first screen away. <!-- rule:terminal-typo-no-banner -->
- **Glyphs need a fallback tier.** Box drawing and braille assume a capable font; Nerd Font icons are an opt-in set with an ASCII or emoji tier behind them. <!-- rule:terminal-typo-glyph-fallback -->
- **Emoji only as an indicator, never as a bullet.** Emoji widths vary by terminal, color emoji ignore the theme, and a row of them hides the text it decorates. <!-- rule:terminal-typo-emoji -->

## Color

- **The user's 16 ANSI colors are the palette; truecolor is an enhancement.** Named colors follow the terminal theme. RGB literals do not, and degrade to default text where 24-bit color is missing. <!-- rule:terminal-color-ansi-first -->
- **Every RGB choice needs a light-background twin.** Detect the background and pick (`AdaptiveColor`, `LightDark`, `HasDarkBackground`, a light theme), or compute contrast the way Textual does. <!-- rule:terminal-color-light-twin -->
- **Honor the disable contract.** A non-empty `NO_COLOR`, `TERM=dumb`, `--no-color`, and a non-TTY stream each strip color. `FORCE_COLOR` re-enables it, and `NO_COLOR` wins over `FORCE_COLOR`. <!-- rule:terminal-color-no-color -->
- **Color is a resource.** One accent, semantic red, yellow, and green, dim for secondary. If everything is a color, nothing is. <!-- rule:terminal-color-budget -->
- **Name colors by role, not by hex.** `accent`, `muted`, `error`, `focused-border` resolved per theme, so a theme swap is one table. <!-- rule:terminal-color-roles -->
- **Green on black is a costume.** It fails on light themes and at small sizes; use the theme's foreground. <!-- rule:terminal-color-green-on-black -->

## Components & controls

- **Use the framework's primitives, not hand-rolled ones.** List, table, input, text area, file picker, pager, spinner, progress, and help exist in Bubbles, Textual, Rich, Ink, and gum. A custom one usually breaks on width or resize. <!-- rule:terminal-components-primitives -->
- **stdout is data, stderr is chrome.** Results go to stdout in a form `grep` can read; status, spinners, and prompts go to stderr. Offer `--plain` or `--json` when the pretty form would break a pipe. <!-- rule:terminal-components-streams -->
- **Status bar reflects state, help reflects context.** Mode, path, and counts in one line; shortcuts listed only if they work here. <!-- rule:terminal-components-status -->

## Motion

- **Spinners tick on work and stop on completion.** Tick per unit processed so a hang is visible; replace the spinner with a past-tense line when done. <!-- rule:terminal-motion-spinner -->
- **No animation off a TTY, and a static mode on one.** A spinner in a pipe becomes hundreds of log lines; a repainting status row re-homes a screen reader's cursor ten times a second. Offer `--no-progress` or a static indicator. <!-- rule:terminal-motion-off-tty -->
- **Print something within 100 ms; never blink.** Show the first frame before the network call. The blink attribute has no legitimate use. <!-- rule:terminal-motion-first-frame -->
- **Offer a line mode.** The append-only stream is what screen readers handle; a full-screen TUI ships a `--plain` twin or a static-chrome flag. <!-- rule:terminal-motion-line-mode -->

## Verifying the build

- **Capture, do not describe.** Run the app in a tmux pane and capture it with `tmux capture-pane -p -e -J -t <target>` (the `-e` keeps colors), or use the stack's own snapshot: Textual `--screenshot` and `snap_compare`, Ratatui `TestBackend` with insta, Ink `lastFrame()`, a VHS tape with `Set Columns` and `Set Rows`. Write the captures where the review flow expects them. <!-- rule:terminal-verify-capture -->
- **Run the matrix.** 80x24, 120x40, and about 30 columns; a live resize; a dark and a light theme; `NO_COLOR=1`; `TERM=dumb`; stdout piped through `cat`; no Nerd Font; rows with CJK and emoji. Say which configurations produced the evidence. <!-- rule:terminal-verify-matrix -->
````

- [ ] **Step 4: Run the test and the prose gate**

Run: `node --test tests/skill-reference.test.mjs && bun run build 2>&1 | tail -5`
Expected: the test passes; the build ends without a `validateSkillProse` error.

- [ ] **Step 5: Commit**

```bash
git add skill/reference/terminal.md tests/skill-reference.test.mjs
git commit -m "Add the terminal platform reference

reference/terminal.md carries the design laws for TUIs and rich CLIs in
the same shape as ios.md and android.md. Nothing loads it yet.

AI assistance: drafted with Claude Code."
```

---

### Task 3: `audit.terminal.md` and `adapt.terminal.md`

**Files:**
- Create: `skill/reference/audit.terminal.md`
- Create: `skill/reference/adapt.terminal.md`
- Test: `tests/skill-reference.test.mjs`

**Interfaces:**
- Produces: two file paths named in the Commands table (Task 8). `audit.terminal.md` mirrors `audit.md`'s report skeleton: the headings `## Diagnostic Scan`, `## Generate Report`, `### Audit Health Score`, `### Executive Summary`, `### Detailed Findings by Severity`, `### Patterns & Systemic Issues`, `### Positive Findings`, `## Recommended Actions`, five dimensions scored 0-4, total out of 20.

- [ ] **Step 1: Write the failing test**

Append to the `describe` block in `tests/skill-reference.test.mjs`:

```js
  it('keeps the audit report skeleton identical across the web, native, and terminal variants', () => {
    // The verdict heading names what each variant judges (implementation
    // integrity on the web, platform conformance elsewhere); every other
    // heading must match exactly.
    const skeleton = (name) => {
      const text = readFileSync(join(ROOT, `skill/reference/${name}`), 'utf-8').replace(/\r\n?/g, '\n');
      return text
        .split('\n')
        .filter((l) => /^#{2,3} /.test(l) && !/^### \d\./.test(l))
        .map((l) => (/^### .* Verdict$/.test(l) ? '### <verdict>' : l));
    };
    const web = skeleton('audit.md');
    assert.deepEqual(skeleton('audit.native.md'), web);
    assert.deepEqual(skeleton('audit.terminal.md'), web);
    const terminal = readFileSync(join(ROOT, 'skill/reference/audit.terminal.md'), 'utf-8');
    assert.equal((terminal.match(/^### \d\. /gm) || []).length, 5, 'five scored dimensions');
    assert.match(terminal, /\*\*\?\?\/20\*\*/);
    assert.match(terminal, /NO_COLOR/);
    const adapt = readFileSync(join(ROOT, 'skill/reference/adapt.terminal.md'), 'utf-8');
    assert.match(adapt, /under 60 columns/);
    assert.match(adapt, /120 columns and above/);
    assert.match(adapt, /16 colors/);
    assert.match(adapt, /truecolor/);
  });
```

- [ ] **Step 2: Run it to verify it fails**

Run: `node --test tests/skill-reference.test.mjs`
Expected: FAIL with `ENOENT ... audit.terminal.md`.

- [ ] **Step 3: Confirm the web skeleton the test compares against**

Run: `grep -nE '^#{2,3} ' skill/reference/audit.md | grep -vE '^[0-9]+:### [0-9]\.'`
Expected output lists, in order: `## Diagnostic Scan`, `## Generate Report`, `### Audit Health Score`, `### Implementation Integrity Verdict`, `### Executive Summary`, `### Detailed Findings by Severity`, `### Patterns & Systemic Issues`, `### Positive Findings`, `## Recommended Actions`. The native variant swaps the verdict heading for `### Platform Conformance Verdict`; the terminal variant uses the same native wording. If any other heading differs from this list, use the actual list from `audit.md` when writing Step 4 and keep the order.

- [ ] **Step 4: Create `audit.terminal.md`**

Write `skill/reference/audit.terminal.md`:

````markdown
Run systematic **technical** quality checks on a terminal app (`terminal`) and generate a comprehensive report. Don't fix issues; document them for other commands to address.

This is a code-level audit, not a design critique. Audit from source (Ratatui, Bubble Tea and Lip Gloss, Textual and Rich, Ink) and from captures of the running app; no browser tooling applies. `{{scripts_path}}/impeccable detect` runs terminal rules once they ship; until then, the checks below are manual. Score against [terminal.md](terminal.md); read it before scoring if Setup hasn't already. The report skeleton mirrors [audit.md](audit.md); keep the variants in sync when changing it.

## Diagnostic Scan

Run comprehensive checks across 5 dimensions. Score each dimension 0-4 using the criteria below.

### 1. Accessibility (screen readers, keyboard, color independence)

**Check for**:
- **No line mode**: a full-screen app with no `--plain`, `--json`, or static-chrome alternative; screen readers cannot follow a repainting grid
- **Mouse-only actions**: anything reachable by click that has no key
- **Missing key hints**: no footer or `?` help, no visible way to quit
- **Color as the only signal**: status conveyed by red or green with no glyph or word
- **Contrast**: foreground and background pairs below 4.5:1 on either a dark or a light default palette
- **Animation with no rest**: spinners or status rows that repaint while nothing changes

**Score 0-4**: 0=Unusable without sight and a mouse, 1=Major gaps (no line mode, mouse-only paths), 2=Partial (keys exist, hints or contrast break), 3=Good (minor gaps), 4=Excellent (keyboard complete, hints honest, line mode offered, contrast holds on both themes)

### 2. Performance

**Check for**:
- **Slow first frame**: network or disk work before anything is drawn
- **Full redraws**: every tick repaints the whole screen instead of the changed regions
- **Blocking input**: long work on the event loop so keys queue up
- **Unbounded buffers**: logs or lists that grow without a cap or virtualization
- **Startup weight**: heavy imports or plugin scans on launch

**Score 0-4**: 0=Frozen or flickering, 1=Major problems (slow launch, blocked input), 2=Partial, 3=Good (minor improvements possible), 4=Excellent (first frame within 100 ms, input never waits)

### 3. Appearance & Theming

**Check for**:
- **Hard-coded RGB with no light twin**: `Color::Rgb`, `lipgloss.Color("#...")`, `color="#..."`, Rich `[#rrggbb]` with no adaptive or theme path
- **Disable contract ignored**: `NO_COLOR`, `TERM=dumb`, `--no-color`, or a piped stream still gets color
- **Color sprawl**: more than one accent plus the semantic trio
- **Costume palettes**: green on black, rainbow gradients on titles
- **Emoji as chrome**: emoji bullets or icons on most lines

**Score 0-4**: 0=Hard-coded everything, 1=Minimal theme awareness, 2=Partial (roles exist, inconsistently used), 3=Good (minor hard-coded values), 4=Excellent (ANSI roles throughout, both themes first-class, disable contract honored)

### 4. Platform Conformance (CRITICAL)

Score against [terminal.md](terminal.md), including its slop test. **Check for**:
- **Figlet banners or block-letter titles** at launch
- **Nested or double-line borders** used as decoration
- **Raw escape sequences** hand-written in app code
- **Terminal not restored**: raw mode or the alternate screen left on after an error or panic
- **Wrong screen mode**: a one-shot tool that clears the screen or takes the alternate screen
- **Glyphs with no fallback tier**: Nerd Font icons or braille with no ASCII path

**Score 0-4**: 0=Website port in a terminal, 1=Heavy violations (3-4 kinds), 2=Some (1-2 noticeable), 3=Mostly conformant (subtle issues), 4=Fully at home (a fluent terminal user trusts every screen)

### 5. Adaptivity

**Check for**:
- **Hard-coded size**: layouts that assume 80x24 or use literal rectangles
- **Collapse under 60 columns**: broken borders, clipped labels, overflowing rows
- **Wasted width above 120 columns**: one narrow column in a wide window
- **Resize ignored**: layout not recomputed on `SIGWINCH` or the framework's resize event
- **Width drift**: CJK or emoji rows misaligning the grid; truncation that breaks graphemes
- **Color tier ignored**: truecolor assumed on a 256-color or 16-color terminal

**Score 0-4**: 0=One size and one terminal only, 1=Major breakage (narrow or resize broken), 2=Partial, 3=Good (minor edge cases), 4=Excellent (adapts across widths, color tiers, and fonts)

## Generate Report

### Audit Health Score

| # | Dimension | Score | Key Finding |
|---|-----------|-------|-------------|
| 1 | Accessibility | ? | [most critical issue or "--"] |
| 2 | Performance | ? | |
| 3 | Appearance & Theming | ? | |
| 4 | Platform Conformance | ? | |
| 5 | Adaptivity | ? | |
| **Total** | | **??/20** | **[Rating band]** |

**Rating bands**: 18-20 Excellent (minor polish), 14-17 Good (address weak dimensions), 10-13 Acceptable (significant work needed), 6-9 Poor (major overhaul), 0-5 Critical (fundamental issues)

### Platform Conformance Verdict
**Start here.** Pass/fail: does this read as a tool a terminal user trusts, or a website costume? List specific violations. Be brutally honest.

### Executive Summary
- Audit Health Score: **??/20** ([rating band])
- Total issues found (count by severity: P0/P1/P2/P3)
- Top 3-5 critical issues
- Recommended next steps

### Detailed Findings by Severity

Tag every issue with **P0-P3 severity**:
- **P0 Blocking**: Prevents task completion. Fix immediately
- **P1 Major**: Significant difficulty or platform-guideline violation. Fix before release
- **P2 Minor**: Annoyance, workaround exists. Fix in next pass
- **P3 Polish**: Nice-to-fix, no real user impact. Fix if time permits

For each issue, document:
- **[P?] Issue name**
- **Location**: Screen, file, line
- **Category**: Accessibility / Performance / Theming / Conformance / Adaptivity
- **Impact**: How it affects users
- **Guideline**: The terminal.md rule it violates (if applicable)
- **Recommendation**: How to fix it
- **Suggested command**: Which command to use (prefer: {{available_commands}})

### Patterns & Systemic Issues

Identify recurring problems that indicate systemic gaps rather than one-off mistakes:
- "Hard-coded RGB appears in 12 widgets, should resolve through the theme table"
- "Every panel is double-bordered; one border weight per level would restore hierarchy"

### Positive Findings

Note what's working well: good practices to maintain and replicate.

## Recommended Actions

List recommended commands in priority order (P0 first, then P1, then P2):

1. **[P?] `{{command_prefix}}command-name`**: Brief description (specific context from audit findings)
2. **[P?] `{{command_prefix}}command-name`**: Brief description (specific context)

**Rules**: Only recommend commands from: {{available_commands}}. Map findings to the most appropriate command. End with `{{command_prefix}}impeccable polish` as the final step if any fixes were recommended.

After presenting the summary, tell the user:

> You can ask me to run these one at a time, all at once, or in any order you prefer.
>
> Re-run `{{command_prefix}}impeccable audit` after fixes to see your score improve.

**IMPORTANT**: Be thorough but actionable. Too many P3 issues creates noise. Focus on what actually matters.

**NEVER**:
- Report issues without explaining impact (why does this matter?)
- Provide generic recommendations (be specific and actionable)
- Skip positive findings (celebrate what works)
- Forget to prioritize (everything can't be P0)
- Report false positives without verification
````

- [ ] **Step 5: Create `adapt.terminal.md`**

Write `skill/reference/adapt.terminal.md`:

````markdown
> **Additional context needed**: the terminal sizes, color tiers, and fonts the app must run in.

Adapt an existing **terminal** design (`terminal`) to a different context: another width class, color tier, font tier, or an off-TTY stream. The trap is treating adaptation as scaling. The job is deciding what each width class shows and what each capability tier degrades to, inside the conventions of [terminal.md](terminal.md); read it before planning if Setup hasn't already.

## Assess Adaptation Challenge

1. **Source context**: what size did the author design at, and what did they assume? (80x24 only? Truecolor? A Nerd Font? Always a TTY?)
2. **Target contexts**: which width classes, color tiers, and streams does the app actually meet? A CI log, an SSH session on a phone, a 200-column ultrawide, and a `| cat` pipe are all real.
3. **What breaks**: borders that wrap, labels that clip, colors that vanish, glyphs that render as tofu, spinners that flood a log?

## Width classes

Decide the layout at each class and drive it from the live size, never from a literal.

| Class | Width | Decision to make |
|---|---|---|
| Narrow | under 60 columns | Which single region survives, how the footer collapses to one line, which labels abbreviate |
| Standard | 80 to 119 columns | The designed layout; 80x24 is the floor every screen must pass |
| Wide | 120 columns and above | What earns the extra width (a detail pane, a wider table), and what stays capped so lines remain readable |

- **Restructure, do not squeeze.** Below 60 columns, hide or stack a region; do not shrink three panes to ten columns each.
- **Cap line length.** Above 120 columns, text blocks stay near 80 to 100 cells; only tables and grids grow.
- **Test a live resize.** Grow and shrink the window while the app runs; the layout must follow every event, and nothing may persist from the old size.

## Color tiers

| Tier | Signal | What the palette degrades to |
|---|---|---|
| 16 colors | `TERM` without `256color`, or the user's choice | The 8 named colors plus bright variants; roles map to them directly |
| 256 colors | `TERM=*-256color`, no `COLORTERM` | Indexed approximations of the accent; text stays in named colors |
| truecolor | `COLORTERM=truecolor` or `24bit` | The full palette, always with a light-background twin |

- **Resolve roles per tier.** `accent`, `muted`, `error`, `focused-border` each get a value at every tier in one table; nothing else names a color.
- **Honor the disable contract at every tier.** `NO_COLOR`, `TERM=dumb`, `--no-color`, and a non-TTY stream strip color entirely; content and hierarchy must survive in weight and spacing alone.

## Font and glyph tiers

- **ASCII** is the floor: `+-|` borders, `*` bullets, `[x]` states.
- **Unicode box drawing** is the default when the terminal is not `dumb`.
- **Nerd Font icons** are an opt-in tier behind a flag or a detected font; never the only rendering of a state.

## Off-TTY

- **Piped stdout** gets data only: no color, no spinners, no alternate screen, results in a form `grep` and `jq` can read.
- **Prompts** fail with the flag to pass instead of blocking a pipe.

## Implement & Verify

- Drive every decision above from runtime signals (size events, `TERM`, `COLORTERM`, `NO_COLOR`, `isatty`), never from a literal or a build flag.
- Capture at each width class and color tier with `tmux capture-pane -p -e -J` or the stack's snapshot tool, plus one run with `NO_COLOR=1` and one piped through `cat`. Say which configurations produced the evidence.

When the adaptation feels at home in each context, hand off to `{{command_prefix}}impeccable polish` for the final pass.

**NEVER**:
- Ship a layout that assumes 80x24
- Squeeze every region into a narrow window instead of choosing what survives
- Assume truecolor or a Nerd Font
- Emit color or animation into a pipe
- Lock the size or clear the screen to dodge a layout bug
````

- [ ] **Step 6: Run the tests and the prose gate**

Run: `node --test tests/skill-reference.test.mjs && bun run build 2>&1 | tail -5`
Expected: all `skill-reference` tests pass; build ends without a prose error.

- [ ] **Step 7: Commit**

```bash
git add skill/reference/audit.terminal.md skill/reference/adapt.terminal.md tests/skill-reference.test.mjs
git commit -m "Add terminal variants of audit and adapt

audit.terminal.md mirrors the audit report skeleton for TUIs;
adapt.terminal.md defines width classes and color tiers. Not routed yet.

AI assistance: drafted with Claude Code."
```

---

### Task 4: `extract_platform` accepts `terminal`

**Files:**
- Modify: `crates/context/src/context.rs` (the `extract_platform` function, around line 1102, and a new test module at the end of the file)

**Interfaces:**
- Produces: `extract_platform(Some("...## Platform\n\nterminal..."))` returns `Some("terminal".to_string())`. Consumed by Task 5 (boot), Task 6 (doctor), and the hook, unchanged.

- [ ] **Step 1: Write the failing tests**

Append to the end of `crates/context/src/context.rs`:

```rust
#[cfg(test)]
mod platform_tests {
    use super::extract_platform;

    fn product(value: &str) -> String {
        format!("# P\n\n## Platform\n\n{}\n\n## Positioning\nx\n", value)
    }

    #[test]
    fn terminal_is_a_recognized_platform() {
        assert_eq!(extract_platform(Some(&product("terminal"))).as_deref(), Some("terminal"));
        assert_eq!(extract_platform(Some(&product("Terminal"))).as_deref(), Some("terminal"));
    }

    #[test]
    fn terminal_never_combines_into_adaptive() {
        assert_eq!(extract_platform(Some(&product("terminal, web"))), None);
        assert_eq!(extract_platform(Some(&product("ios, terminal"))), None);
    }

    #[test]
    fn mobile_pair_still_resolves_to_adaptive() {
        assert_eq!(extract_platform(Some(&product("ios, android"))).as_deref(), Some("adaptive"));
    }

    #[test]
    fn tui_and_cli_are_not_aliases() {
        assert_eq!(extract_platform(Some(&product("tui"))), None);
        assert_eq!(extract_platform(Some(&product("cli"))), None);
    }
}
```

- [ ] **Step 2: Run to verify the first test fails**

Run: `cargo test -p impeccable-context platform_tests`
Expected: `terminal_is_a_recognized_platform` FAILS (`left: None, right: Some("terminal")`); the other three pass.

- [ ] **Step 3: Accept the value**

In `extract_platform`, change:

```rust
    if matches!(value.as_str(), "web" | "ios" | "android" | "adaptive") {
```

to:

```rust
    if matches!(value.as_str(), "web" | "ios" | "android" | "adaptive" | "terminal") {
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p impeccable-context platform_tests`
Expected: 4 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/context/src/context.rs
git commit -m "Accept terminal as a platform value

extract_platform returns terminal for the bare value; the token-list
fallback stays limited to the ios and android pair.

AI assistance: drafted with Claude Code."
```

---

### Task 5: Boot inlines `terminal.md` and the WARNING names the value

**Files:**
- Modify: `crates/context/src/context_cli.rs` (`load_native_platform_references`, around line 551; the WARNING `format!` around line 681)
- Create: `tests/oracle/workspaces/ctx-terminal/PRODUCT.md`, `package.json`, `Cargo.toml`, `src/main.rs`
- Modify: `tests/oracle/cases/context.mjs`
- Modify: `tests/oracle/DELTAS.md`

**Interfaces:**
- Consumes: `extract_platform` (Task 4) and `reference/terminal.md` (Task 2).
- Produces: boot output containing `# NATIVE PLATFORM REFERENCE: TERMINAL (reference/terminal.md)` and `"platform": "terminal"` in `RESOLVED_CONTEXT`; oracle workspace `ctx-terminal` reused by Tasks 6 and 7.

- [ ] **Step 1: Create the fixture workspace**

```bash
mkdir -p tests/oracle/workspaces/ctx-terminal/src
cat > tests/oracle/workspaces/ctx-terminal/PRODUCT.md <<'EOF'
# Oracle Fixture Product

<!-- impeccable:product-schema 1 -->

## Platform

terminal

## Positioning
A fixture TUI the oracle harness uses to pin terminal-platform behavior.
EOF
cat > tests/oracle/workspaces/ctx-terminal/package.json <<'EOF'
{
  "name": "ctx-terminal",
  "private": true
}
EOF
cat > tests/oracle/workspaces/ctx-terminal/Cargo.toml <<'EOF'
[package]
name = "ctx-terminal"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.29"
crossterm = "0.28"
EOF
cat > tests/oracle/workspaces/ctx-terminal/src/main.rs <<'EOF'
use ratatui::widgets::{Block, Paragraph};

fn main() {
    let _ = Paragraph::new("fixture").block(Block::bordered());
}
EOF
```

- [ ] **Step 2: Add the oracle cases**

In `tests/oracle/cases/context.mjs`, directly after the line

```js
  { id: 'context-adaptive', verb: 'context', workspace: 'ctx-adaptive', env: env(), files: IMPECCABLE_FILES },
```

insert:

```js
  { id: 'context-terminal', verb: 'context', workspace: 'ctx-terminal', env: env(), files: IMPECCABLE_FILES },
```

Directly after the `signals-native-ios` line insert:

```js
  { id: 'signals-terminal', verb: 'context-signals', workspace: 'ctx-terminal', env: env() },
```

- [ ] **Step 3: Record the new goldens and confirm they show the gap**

```bash
cargo build --release -p impeccable
export IMPECCABLE_BIN="$PWD/target/release/impeccable"
node tests/oracle/record.mjs --bin context-terminal
node tests/oracle/record.mjs --bin signals-terminal
grep -c 'NATIVE PLATFORM REFERENCE: TERMINAL' tests/oracle/golden/context-terminal.json
```

Expected: the last command prints `0`. The golden shows `"platform": "terminal"` in `RESOLVED_CONTEXT` (Task 4 made that true) but no reference block yet. This golden is a scratch recording; Step 6 overwrites it.

- [ ] **Step 4: Inline the reference**

In `load_native_platform_references`, change:

```rust
    let names: Vec<&str> = match platform {
        Some("adaptive") => vec!["ios", "android"],
        Some("ios") => vec!["ios"],
        Some("android") => vec!["android"],
        _ => vec![],
    };
```

to:

```rust
    let names: Vec<&str> = match platform {
        Some("adaptive") => vec!["ios", "android"],
        Some("ios") => vec!["ios"],
        Some("android") => vec!["android"],
        Some("terminal") => vec!["terminal"],
        _ => vec![],
    };
```

- [ ] **Step 5: Extend the WARNING**

Replace the `format!` string in the WARNING push (the line containing `is not recognized; treating the project as`) with:

```rust
                parts.push(format!("WARNING: PRODUCT.md's `## Platform` value `{}` is not recognized; treating the project as `web`. Valid values are `web`, `ios`, `android`, `adaptive` (cross-platform, ships both), or `terminal` (a TUI or rich CLI). If this project is native or terminal-based, fix the field (name the design language the app renders, not the toolchain) and surface it to the user.", raw));
```

- [ ] **Step 6: Rebuild, re-record, and review**

```bash
cargo build --release -p impeccable
node tests/oracle/record.mjs --bin context-terminal
node tests/oracle/record.mjs --bin signals-terminal
node tests/oracle/record.mjs --bin context-bad-platform
node tests/oracle/run.mjs context- | grep -E '^(XX|\?\?) ' ; echo "unreviewed diffs above (expect none)"
grep -c 'NATIVE PLATFORM REFERENCE: TERMINAL' tests/oracle/golden/context-terminal.json
grep -o '"platform": "terminal"' tests/oracle/golden/context-terminal.json | head -1
grep -o 'or `terminal` (a TUI or rich CLI)' tests/oracle/golden/context-bad-platform.json
git diff --stat tests/oracle/golden
```

Expected: the first grep prints `1`, the second prints `"platform": "terminal"`, the third prints the new phrase, and `git diff --stat` lists exactly `context-bad-platform.json` as modified (plus the two new files as untracked). If any other golden moved, open it, understand why, and either fix the code or add it to the DELTAS entry in Step 7 with the reason.

Open `tests/oracle/golden/context-terminal.json` and read the `stdout` field end to end. Confirm: the `# NATIVE PLATFORM REFERENCE: TERMINAL` block contains the full `terminal.md`, `MANUAL_DETECTOR_REQUIRED` is present (the hook stays active on terminal; under the `source` provider it reports none, so the directive fires as it does for web), and no `WARNING` line appears.

- [ ] **Step 7: Record the delta**

Append to `tests/oracle/DELTAS.md`:

```markdown
## Recorded 2026-09-24: `terminal` platform value

`terminal` joins the platform values. The boot WARNING for an unrecognized
value now lists it, so the one golden that prints that WARNING moved.

- `context-bad-platform`: the WARNING's valid-values sentence reads ``Valid values are `web`, `ios`, `android`, `adaptive` (cross-platform, ships both), or `terminal` (a TUI or rich CLI)`` and the fix clause says `native or terminal-based`. Nothing else in the output changed.
```

- [ ] **Step 8: Run the crate tests and the oracle**

Run: `cargo test -p impeccable-context && node tests/oracle/run.mjs context- | tail -1 && node tests/oracle/run.mjs signals- | tail -1`
Expected: crate tests pass; both oracle summaries report zero failures.

- [ ] **Step 9: Commit**

```bash
git add crates/context/src/context_cli.rs tests/oracle/workspaces/ctx-terminal tests/oracle/cases/context.mjs tests/oracle/golden/context-terminal.json tests/oracle/golden/signals-terminal.json tests/oracle/golden/context-bad-platform.json tests/oracle/DELTAS.md
git commit -m "Inline the terminal reference at boot

impeccable context loads reference/terminal.md for a terminal project the
way it loads ios.md, and the unrecognized-value WARNING names terminal.
Goldens: context-terminal and signals-terminal recorded; context-bad-platform
re-recorded for the WARNING text (DELTAS.md).

AI assistance: drafted with Claude Code."
```

---

### Task 6: Doctor evidence for terminal stacks

**Files:**
- Modify: `crates/context/src/staleness.rs` (the evidence constants around line 48, `check_native_platform_evidence` around line 143, and a new test module at the end of the file)
- Modify: `tests/oracle/cases/context.mjs`

**Interfaces:**
- Consumes: `crate::util::{exists, read_json}` (already imported) and `crate::util::safe_read` (add to the import).
- Produces: `check_native_platform_evidence(project_root, platform, product, product_path) -> Vec<Finding>` unchanged in signature; a `web` or undeclared project with a terminal manifest dependency gets one `platform-native-evidence` finding whose `fix` names `` `terminal` ``.

- [ ] **Step 1: Write the failing tests**

Append to the end of `crates/context/src/staleness.rs`:

```rust
#[cfg(test)]
mod terminal_evidence_tests {
    use super::check_native_platform_evidence;

    fn scratch(name: &str) -> String {
        let base = std::env::temp_dir().join(format!("impeccable-terminal-evidence-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        base.to_string_lossy().into_owned()
    }

    fn write(root: &str, rel: &str, body: &str) {
        let p = std::path::Path::new(root).join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }

    const WEB_PRODUCT: &str = "# P\n\n## Platform\n\nweb\n\n## Positioning\nx\n";

    #[test]
    fn ratatui_dependency_suggests_terminal_for_a_web_project() {
        let root = scratch("ratatui");
        write(&root, "Cargo.toml", "[package]\nname = \"x\"\n\n[dependencies]\nratatui = \"0.29\"\n");
        let f = check_native_platform_evidence(&root, Some("web"), Some(WEB_PRODUCT), Some("PRODUCT.md"));
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].id, "platform-native-evidence");
        assert!(f[0].summary.contains("a ratatui dependency"), "{}", f[0].summary);
        assert!(f[0].summary.contains("terminal codebase"), "{}", f[0].summary);
        assert!(f[0].fix.contains("`terminal`"), "{}", f[0].fix);
    }

    #[test]
    fn go_mod_and_python_manifests_count_as_evidence() {
        let root = scratch("go");
        write(&root, "go.mod", "module example.com/app\n\ngo 1.22\n\nrequire (\n\tgithub.com/charmbracelet/bubbletea v1.2.0\n)\n");
        let f = check_native_platform_evidence(&root, None, Some("# P\n"), Some("PRODUCT.md"));
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("a bubbletea dependency"), "{}", f[0].summary);

        let root = scratch("py");
        write(&root, "pyproject.toml", "[project]\ndependencies = [\n  \"textual>=0.80\",\n]\n");
        let f = check_native_platform_evidence(&root, None, None, None);
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("a textual dependency"), "{}", f[0].summary);

        let root = scratch("req");
        write(&root, "requirements.txt", "rich==13.7.0\n");
        let f = check_native_platform_evidence(&root, None, None, None);
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("a rich dependency"), "{}", f[0].summary);
    }

    #[test]
    fn ink_in_package_json_counts_as_evidence() {
        let root = scratch("ink");
        write(&root, "package.json", "{\"dependencies\":{\"ink\":\"^5.0.0\"}}");
        let f = check_native_platform_evidence(&root, Some("web"), Some(WEB_PRODUCT), Some("PRODUCT.md"));
        assert_eq!(f.len(), 1);
        assert!(f[0].summary.contains("an ink dependency"), "{}", f[0].summary);
        assert!(f[0].fix.contains("`terminal`"), "{}", f[0].fix);
    }

    #[test]
    fn declared_terminal_project_skips_the_check() {
        let root = scratch("declared");
        write(&root, "Cargo.toml", "[dependencies]\nratatui = \"0.29\"\n");
        let f = check_native_platform_evidence(&root, Some("terminal"), Some("# P\n\n## Platform\n\nterminal\n"), Some("PRODUCT.md"));
        assert!(f.is_empty());
    }

    #[test]
    fn mobile_evidence_outranks_terminal_evidence() {
        let root = scratch("mixed");
        write(&root, "ios/Podfile", "platform :ios, '15.0'\n");
        write(&root, "Cargo.toml", "[dependencies]\nratatui = \"0.29\"\n");
        let f = check_native_platform_evidence(&root, Some("web"), Some(WEB_PRODUCT), Some("PRODUCT.md"));
        assert_eq!(f.len(), 1);
        assert!(f[0].fix.contains("`ios`"), "{}", f[0].fix);
        assert!(f[0].summary.contains("native codebase"), "{}", f[0].summary);
    }

    #[test]
    fn a_prefixed_crate_name_is_not_a_match() {
        let root = scratch("prefix");
        write(&root, "Cargo.toml", "[dependencies]\nrichtext = \"1\"\ncrossterm-winapi = \"0.9\"\n");
        let f = check_native_platform_evidence(&root, Some("web"), Some(WEB_PRODUCT), Some("PRODUCT.md"));
        assert!(f.is_empty(), "{:?}", f.iter().map(|x| &x.summary).collect::<Vec<_>>());
    }
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p impeccable-context terminal_evidence_tests`
Expected: `declared_terminal_project_skips_the_check` and `a_prefixed_crate_name_is_not_a_match` pass; the other four FAIL with `assertion failed` on `f.len()`.

- [ ] **Step 3: Add the import and the manifest table**

Change the import line

```rust
use crate::util::{exists, js_trim, mtime_ms, read_json};
```

to

```rust
use crate::util::{exists, js_trim, mtime_ms, read_json, safe_read};
```

Below `NATIVE_EVIDENCE_DEPENDENCIES`, change that constant to include `ink` and add the manifest table:

```rust
const NATIVE_EVIDENCE_DEPENDENCIES: [(&str, &str, &str); 4] = [
    ("react-native", "adaptive", "a react-native dependency"),
    ("expo", "adaptive", "an expo dependency"),
    ("@react-native/metro-config", "adaptive", "a React Native metro config dependency"),
    ("ink", "terminal", "an ink dependency"),
];
/// Text manifests whose dependency lines name a terminal stack. Each entry is
/// (manifest file, dependency name, reason). A line matches when, after
/// trimming whitespace and a leading quote, it starts with the name and the
/// next character is not part of an identifier, so `richtext` is not `rich`.
const TERMINAL_EVIDENCE_MANIFESTS: [(&str, &str, &str); 8] = [
    ("Cargo.toml", "ratatui", "a ratatui dependency"),
    ("Cargo.toml", "crossterm", "a crossterm dependency"),
    ("go.mod", "github.com/charmbracelet/bubbletea", "a bubbletea dependency"),
    ("go.mod", "github.com/charmbracelet/lipgloss", "a lipgloss dependency"),
    ("pyproject.toml", "textual", "a textual dependency"),
    ("pyproject.toml", "rich", "a rich dependency"),
    ("requirements.txt", "textual", "a textual dependency"),
    ("requirements.txt", "rich", "a rich dependency"),
];

fn manifest_names_dependency(text: &str, name: &str) -> bool {
    text.lines().any(|line| {
        let t = line.trim_start().trim_start_matches(|c| c == '"' || c == '\'');
        if !t.starts_with(name) {
            return false;
        }
        match t[name.len()..].chars().next() {
            None => true,
            Some(c) => !(c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '/'),
        }
    })
}
```

- [ ] **Step 4: Collect terminal evidence and pick the suggestion**

In `check_native_platform_evidence`, directly after the `package.json` block (the `if let Some(pkg) = read_json(...) { ... }` block) and before `if evidence.is_empty() {`, insert:

```rust
    for (manifest, name, reason) in TERMINAL_EVIDENCE_MANIFESTS {
        if let Some(text) = safe_read(&jsp::join(&[project_root, manifest])) {
            if manifest_names_dependency(&text, name) {
                evidence.push(NativeEvidence { platform: "terminal", reason });
            }
        }
    }
```

Replace the `suggested` line

```rust
    let suggested = if platforms.len() > 1 || platforms.contains(&"adaptive") { "adaptive" } else { platforms[0] };
```

with

```rust
    // Mobile evidence outranks terminal evidence: a Flutter app with a Rust
    // helper is still a mobile app. Terminal is suggested only when it is the
    // only kind of evidence present.
    let mobile: Vec<&str> = platforms.iter().copied().filter(|p| *p != "terminal").collect();
    let suggested = if mobile.is_empty() {
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
```

Then change the summary `format!` from

```rust
            "{}, but the project carries {}. Web guidance is being applied to a native codebase, and the iOS and Android references never load.",
            declared,
            evidence.iter().map(|e| e.reason).collect::<Vec<_>>().join(" and ")
```

to

```rust
            "{}, but the project carries {}. {}",
            declared,
            evidence.iter().map(|e| e.reason).collect::<Vec<_>>().join(" and "),
            consequence
```

- [ ] **Step 5: Run the tests**

Run: `cargo test -p impeccable-context terminal_evidence_tests`
Expected: 6 passed.

Run: `cargo test -p impeccable-context`
Expected: all pass (the existing evidence path for mobile is unchanged in output).

- [ ] **Step 6: Add the oracle cases**

In `tests/oracle/cases/context.mjs`, directly after the `context-native-evidence-web` line, insert:

```js
  { id: 'context-terminal-evidence-web', verb: 'context', workspace: 'ctx-product-only', setup: (ws) => write(ws, 'Cargo.toml', '[package]\nname = "fixture"\nversion = "0.1.0"\n\n[dependencies]\nratatui = "0.29"\n'), env: env(), files: IMPECCABLE_FILES },
```

Directly after the `doctor-adaptive-json` line, insert:

```js
  { id: 'doctor-terminal-text', verb: 'doctor', workspace: 'ctx-terminal', env: env() },
  { id: 'doctor-terminal-json', verb: 'doctor', workspace: 'ctx-terminal', args: ['--json'], env: env() },
  { id: 'doctor-terminal-evidence-text', verb: 'doctor', workspace: 'ctx-product-only', setup: (ws) => write(ws, 'go.mod', 'module example.com/app\n\ngo 1.22\n\nrequire github.com/charmbracelet/bubbletea v1.2.0\n'), env: env() },
  { id: 'doctor-terminal-evidence-json', verb: 'doctor', workspace: 'ctx-product-only', setup: (ws) => write(ws, 'go.mod', 'module example.com/app\n\ngo 1.22\n\nrequire github.com/charmbracelet/bubbletea v1.2.0\n'), args: ['--json'], env: env() },
```

- [ ] **Step 7: Rebuild, record, review**

```bash
cargo build --release -p impeccable
node tests/oracle/record.mjs --bin context-terminal-evidence-web
node tests/oracle/record.mjs --bin doctor-terminal
node tests/oracle/run.mjs context- | tail -1
node tests/oracle/run.mjs doctor- | tail -1
git diff --stat tests/oracle/golden
grep -o 'a ratatui dependency[^"]*terminal reference never load' tests/oracle/golden/context-terminal-evidence-web.json | head -1
grep -o 'should be `terminal`' tests/oracle/golden/doctor-terminal-evidence-text.json
grep -c 'platform-native-evidence' tests/oracle/golden/doctor-terminal-json.json
```

Expected: both oracle summaries report zero failures; `git diff --stat` shows no modified goldens (only new untracked ones); the first grep prints the summary fragment; the second prints `should be \`terminal\``; the last prints `0` (a declared terminal project gets no evidence finding). Open `doctor-terminal-json.json` and confirm `"platform": "terminal"` at the top level.

- [ ] **Step 8: Commit**

```bash
git add crates/context/src/staleness.rs tests/oracle/cases/context.mjs tests/oracle/golden/context-terminal-evidence-web.json tests/oracle/golden/doctor-terminal-text.json tests/oracle/golden/doctor-terminal-json.json tests/oracle/golden/doctor-terminal-evidence-text.json tests/oracle/golden/doctor-terminal-evidence-json.json
git commit -m "Doctor: suggest terminal when the manifests name a TUI stack

A web or undeclared project whose Cargo.toml, go.mod, pyproject.toml,
requirements.txt, or package.json names ratatui, crossterm, bubbletea,
lipgloss, textual, rich, or ink gets the platform-native-evidence mention
with terminal as the suggested value. Mobile evidence still outranks it.

AI assistance: drafted with Claude Code."
```

---

### Task 7: concept-seed accepts `--platform terminal`

**Files:**
- Modify: `crates/context/src/roll_selection.rs:9`
- Modify: `tests/oracle/cases/context.mjs`
- Modify: `tests/oracle/DELTAS.md`

**Interfaces:**
- Produces: `COMPOSITION_PLATFORMS` contains `"terminal"`; `concept-seed --platform terminal` proceeds; the validation message lists four values.

- [ ] **Step 1: Add the oracle case and record the failure**

In `tests/oracle/cases/context.mjs`, directly after the `seed-platform-invalid` line, insert:

```js
  { id: 'seed-platform-terminal', verb: 'concept-seed', workspace: 'ctx-product-only', args: ['--scope', 'surface', '--from', 'k1', '--platform', 'terminal'], env: seedEnv() },
```

```bash
node tests/oracle/record.mjs --bin seed-platform-terminal
grep -o '"exit": [0-9]*' tests/oracle/golden/seed-platform-terminal.json
```

Expected: `"exit": 1` (rejected today). Scratch recording; Step 3 overwrites it.

- [ ] **Step 2: Accept the value**

In `crates/context/src/roll_selection.rs`, change

```rust
pub const COMPOSITION_PLATFORMS: [&str; 3] = ["web", "ios", "android"];
```

to

```rust
pub const COMPOSITION_PLATFORMS: [&str; 4] = ["web", "ios", "android", "terminal"];
```

- [ ] **Step 3: Rebuild, re-record, review**

```bash
cargo build --release -p impeccable
node tests/oracle/record.mjs --bin seed-platform-terminal
node tests/oracle/record.mjs --bin seed-platform-invalid
grep -o '"exit": [0-9]*' tests/oracle/golden/seed-platform-terminal.json
grep -o 'must be one of web, ios, android, terminal' tests/oracle/golden/seed-platform-invalid.json
node tests/oracle/run.mjs seed- | tail -1
git diff --stat tests/oracle/golden
```

Expected: `"exit": 0`; the phrase is found; the oracle summary reports zero failures; `git diff --stat` lists exactly `seed-platform-invalid.json`. Open `seed-platform-terminal.json` and confirm it contains a seed (the stdout has `challengers`) and no error text.

- [ ] **Step 4: Record the delta**

Append to the `## Recorded 2026-09-24: \`terminal\` platform value` section in `tests/oracle/DELTAS.md`:

```markdown
- `seed-platform-invalid`: the rejection reads `concept-seed: --platform must be one of web, ios, android, terminal`. Exit code and everything else unchanged.
```

- [ ] **Step 5: Commit**

```bash
git add crates/context/src/roll_selection.rs tests/oracle/cases/context.mjs tests/oracle/golden/seed-platform-terminal.json tests/oracle/golden/seed-platform-invalid.json tests/oracle/DELTAS.md
git commit -m "concept-seed: accept --platform terminal

AI assistance: drafted with Claude Code."
```

---

### Task 8: Skill routing text

**Files:**
- Modify: `skill/SKILL.src.md` (line 3 description; the `## Modes` closing paragraph; the `audit` and `adapt` rows; the Setup routing bullet; the post-init sentence)
- Modify: `skill/reference/routing.md:19` and `:22`
- Modify: `skill/reference/init.md:23`, `:100`, `:104`
- Modify: `skill/reference/audit.md:5`, `skill/reference/adapt.md:5`
- Test: `tests/skill-reference.test.mjs`, `bun run build`

**Interfaces:**
- Consumes: the three reference files from Tasks 2 and 3.

- [ ] **Step 1: Write the failing test**

Append to the `describe` block in `tests/skill-reference.test.mjs`:

```js
  it('routes terminal projects to the terminal references and keeps detect available to them', () => {
    const skill = readFileSync(join(ROOT, 'skill/SKILL.src.md'), 'utf-8').replace(/\r\n?/g, '\n');
    const routing = readFileSync(join(ROOT, 'skill/reference/routing.md'), 'utf-8').replace(/\r\n?/g, '\n');
    const init = readFileSync(join(ROOT, 'skill/reference/init.md'), 'utf-8').replace(/\r\n?/g, '\n');
    const audit = readFileSync(join(ROOT, 'skill/reference/audit.md'), 'utf-8');
    const adapt = readFileSync(join(ROOT, 'skill/reference/adapt.md'), 'utf-8');

    const auditRow = skill.split('\n').find((l) => l.startsWith('| `audit [target]`')) ?? '';
    const adaptRow = skill.split('\n').find((l) => l.startsWith('| `adapt [target]`')) ?? '';
    assert.match(auditRow, /terminal: \[reference\/audit\.terminal\.md\]\(reference\/audit\.terminal\.md\)/);
    assert.match(adaptRow, /terminal: \[reference\/adapt\.terminal\.md\]\(reference\/adapt\.terminal\.md\)/);
    assert.match(skill, /platform variant when the Commands table lists one/);
    assert.match(skill, /`ios`, `android`, `adaptive`, or `terminal`/);
    assert.match(skill, /Terminal surfaces usually resolve to Operate or Read/);
    assert.match(skill.split('\n')[2], /terminal UIs \(TUIs\)/);

    assert.match(routing, /`live` and `generate` are web-only/);
    assert.match(routing, /`terminal` projects keep `impeccable detect`/);
    assert.match(init, /`web`, `ios`, `android`, `adaptive`, or `terminal`/);
    assert.match(init, /\[terminal\.md\]\(terminal\.md\)/);
    assert.match(audit, /Terminal projects \(`terminal`\) route to \[audit\.terminal\.md\]/);
    assert.match(adapt, /Terminal projects \(`terminal`\) route to \[adapt\.terminal\.md\]/);
  });
```

- [ ] **Step 2: Run it to verify it fails**

Run: `node --test tests/skill-reference.test.mjs`
Expected: FAIL on the first `assert.match` (audit row).

- [ ] **Step 3: Edit `SKILL.src.md`**

Line 3 (the description). Replace the text `technically extraordinary. Not for backend-only or non-UI tasks.` with `technically extraordinary. Covers terminal UIs (TUIs) and rich CLIs. Not for backend-only or non-UI tasks.` Then verify the length stays under the 1024-character frontmatter limit:

```bash
awk 'NR==3' skill/SKILL.src.md | wc -c
```

Expected: a number at or below 960 (it was 911 before the edit).

In the `## Modes` section, the closing paragraph begins `Choose the mode from the requested surface, not the product`. Replace its second sentence `A tool's landing page is still Persuade; a fashion house's documentation is still Read; a docs index is Read, not Persuade.` with `A tool's landing page is still Persuade; a fashion house's documentation is still Read; a docs index is Read, not Persuade. Terminal surfaces usually resolve to Operate or Read.`

Replace the `audit` row:

```markdown
| `audit [target]` | Evaluate | Technical quality checks (a11y, perf, responsive) | [reference/audit.md](reference/audit.md) · native: [reference/audit.native.md](reference/audit.native.md) · terminal: [reference/audit.terminal.md](reference/audit.terminal.md) |
```

Replace the `adapt` row:

```markdown
| `adapt [target]` | Fix | Adapt for different devices and screen sizes | [reference/adapt.md](reference/adapt.md) · native: [reference/adapt.native.md](reference/adapt.native.md) · terminal: [reference/adapt.terminal.md](reference/adapt.terminal.md) |
```

Replace the routing bullet `- **Explicit or clearly implied request to run a command:** load its reference (native variant on native platforms) and follow it. Ask once if two commands fit.` with:

```markdown
- **Explicit or clearly implied request to run a command:** load its reference (the platform variant when the Commands table lists one for the resolved platform) and follow it. Ask once if two commands fit.
```

Replace the sentence `After init writes PRODUCT.md, resume without rerunning \`impeccable context\`; init loads the native platform reference itself when the platform it recorded is \`ios\`, \`android\`, or \`adaptive\`.` with:

```markdown
After init writes PRODUCT.md, resume without rerunning `impeccable context`; init loads the platform reference itself when the platform it recorded is `ios`, `android`, `adaptive`, or `terminal`.
```

- [ ] **Step 4: Edit `routing.md`**

Line 19: replace the bold clause and the sentence after it, from `**\`live\`, \`generate\`, and the bundled \`impeccable detect\` are web-only.** If \`setup.platform\` is \`ios\`, \`android\`, or \`adaptive\`, don't lead with any of them; the browser overlay and the HTML rule engine don't apply to native app code.` with:

```markdown
**`live` and `generate` are web-only.** If `setup.platform` is `ios`, `android`, `adaptive`, or `terminal`, don't lead with either; the browser overlay does not apply. The bundled `impeccable detect` is web-only for `ios`, `android`, and `adaptive`; `terminal` projects keep `impeccable detect` (its terminal rules arrive with the engine that ships them).
```

Line 22: replace `**If \`scan.targets\` is non-empty and \`setup.platform\` is not \`ios\`/\`android\`/\`adaptive\`, run` with `**If \`scan.targets\` is non-empty and \`setup.platform\` is not \`ios\`/\`android\`/\`adaptive\` (\`terminal\` and \`web\` both qualify), run`, and replace `it reads HTML/CSS, so skip it for native projects)` with `it reads HTML/CSS today, so skip it for native mobile projects)`.

- [ ] **Step 5: Edit `init.md`**

Line 23: replace `Form a platform hypothesis: \`web\`, \`ios\`, \`android\`, or \`adaptive\` (one product that genuinely adapts its design language per OS). Mobile web remains \`web\`; a native wrapper around a website does not make its design language native.` with:

```markdown
Form a platform hypothesis: `web`, `ios`, `android`, `adaptive` (one product that genuinely adapts its design language per OS), or `terminal` (an app whose primary surface renders in a terminal: a TUI or a rich CLI). Mobile web remains `web`; a native wrapper around a website does not make its design language native; a web app with a CLI installer is `web`, not `terminal`.
```

Line 100: replace `Platform is the bare value \`web\`, \`ios\`, \`android\`, or \`adaptive\`.` with `Platform is the bare value \`web\`, \`ios\`, \`android\`, \`adaptive\`, or \`terminal\`.`

Line 104: replace `When the platform you just recorded is \`ios\`, \`android\`, or \`adaptive\`, load [ios.md](ios.md), [android.md](android.md), or both before any design work.` with `When the platform you just recorded is \`ios\`, \`android\`, \`adaptive\`, or \`terminal\`, load [ios.md](ios.md), [android.md](android.md), both, or [terminal.md](terminal.md) before any design work.`

- [ ] **Step 6: Edit the web-only guards**

`skill/reference/audit.md` line 5, replace with:

```markdown
**Web only.** Native platforms (`ios` / `android` / `adaptive`) route to [audit.native.md](audit.native.md) instead. Terminal projects (`terminal`) route to [audit.terminal.md](audit.terminal.md). If the project is either, switch now.
```

`skill/reference/adapt.md` line 5, replace with:

```markdown
**Web only** (mobile web included). Native platforms (`ios` / `android` / `adaptive`) route to [adapt.native.md](adapt.native.md) instead. Terminal projects (`terminal`) route to [adapt.terminal.md](adapt.terminal.md). If the project is either, switch now.
```

- [ ] **Step 7: Run the tests and the build**

Run: `node --test tests/skill-reference.test.mjs && bun run build 2>&1 | tail -8 && bun run test:core 2>&1 | tail -5`
Expected: all reference tests pass; the build ends without prose or count errors (the command count is unchanged: no new command was added); the core suite passes.

- [ ] **Step 8: Commit**

```bash
git add skill/SKILL.src.md skill/reference/routing.md skill/reference/init.md skill/reference/audit.md skill/reference/adapt.md tests/skill-reference.test.mjs
git commit -m "Route terminal projects to the terminal references

audit and adapt list their terminal variants, init records and loads the
terminal platform, and routing keeps detect available to terminal
projects while live stays web-only.

AI assistance: drafted with Claude Code."
```

---

### Task 9: Documentation, full gates, merge

**Files:**
- Modify: `docs/CLI-CONTRACT.md` (lines 397, 399, 531, 596, 626)
- Modify: `CLAUDE.md` (repo root, the Platform section)

- [ ] **Step 1: Update the CLI contract**

In `docs/CLI-CONTRACT.md`:

Line 397: change `--platform web|ios|android` to `--platform web|ios|android|terminal`.

Line 399: change `` `--platform must be one of web, ios, android` `` to `` `--platform must be one of web, ios, android, terminal` ``.

Line 531: change `exact \`web|ios|android|adaptive\` returned` to `exact \`web|ios|android|adaptive|terminal\` returned`.

Line 596: in the native reference parts clause, change `(\`ios\` -> ios; \`android\` -> android; \`adaptive\` -> ios then android; missing file skipped silently)` to `(\`ios\` -> ios; \`android\` -> android; \`adaptive\` -> ios then android; \`terminal\` -> terminal, header \`# NATIVE PLATFORM REFERENCE: TERMINAL (reference/terminal.md)\`; missing file skipped silently)`. In the WARNING text on the same line, change `Valid values are \`web\`, \`ios\`, \`android\`, or \`adaptive\` (cross-platform, ships both). If this project is native, fix the field` to `Valid values are \`web\`, \`ios\`, \`android\`, \`adaptive\` (cross-platform, ships both), or \`terminal\` (a TUI or rich CLI). If this project is native or terminal-based, fix the field`.

Line 626: after `package.json deps/devDeps \`react-native\`, \`expo\`, \`@react-native/metro-config\` (all adaptive).` insert ` \`ink\` (terminal). Text manifests: \`Cargo.toml\` \`ratatui\`/\`crossterm\`, \`go.mod\` \`github.com/charmbracelet/bubbletea\`/\`lipgloss\`, \`pyproject.toml\` and \`requirements.txt\` \`textual\`/\`rich\` (all terminal; a line matches when it starts with the name after trimming whitespace and a leading quote and the next char is not \`[A-Za-z0-9_./-]\`).` Then change `Suggested = \`adaptive\` if >1 platform or any adaptive, else the one.` to `Suggested: drop \`terminal\` from the set; if nothing remains -> \`terminal\`; else \`adaptive\` if >1 or any adaptive, else the one. Summary's closing sentence is \`Web guidance is being applied to a terminal codebase, and the terminal reference never loads.\` when suggested is terminal, otherwise the native sentence.`

- [ ] **Step 2: Update the root `CLAUDE.md`**

In the `### Platform (web / ios / android / adaptive)` section, rename the heading to `### Platform (web / ios / android / adaptive / terminal)` and add a list item after the `adaptive` item:

```markdown
- **terminal** — a full-screen TUI or rich CLI (Ratatui, Bubble Tea, Textual, Ink). Loads `reference/terminal.md`. Unlike the mobile values it does **not** switch off the detector or the design hook: terminal source rules are the engine's job (PR 2 of the terminal spec), and live mode stays web-only.
```

Change the sentence `PRODUCT.md carries a \`## Platform\` section with a bare value (\`web\` / \`ios\` / \`android\` / \`adaptive\`).` to include `/ \`terminal\``. Change `\`impeccable context\` inlines the native reference(s) directly into its output when the value is \`ios\`, \`android\`, or \`adaptive\` (both)` to `... when the value is \`ios\`, \`android\`, \`adaptive\` (both), or \`terminal\``.

In the native-variant paragraph, after `Variants today: \`audit.native.md\`, \`adapt.native.md\`` add `, and their terminal twins \`audit.terminal.md\`, \`adapt.terminal.md\` (suffix \`.terminal.md\`, routed for \`terminal\`)`. Change `\`audit.native.md\` mirrors \`audit.md\`'s report skeleton; change the skeleton in both together.` to `\`audit.native.md\` and \`audit.terminal.md\` mirror \`audit.md\`'s report skeleton; change the skeleton in all three together (\`tests/skill-reference.test.mjs\` pins it).`

In the `**Live mode, \`impeccable detect\`, and the design hook are web-only.**` paragraph, append: `\`terminal\` is the exception: it is not in any native skip set, so the hook and detect stay on for terminal projects.`

- [ ] **Step 3: Run every gate**

```bash
cargo test --workspace 2>&1 | tail -3
cargo build --release -p impeccable
export IMPECCABLE_BIN="$PWD/target/release/impeccable"
bun run build 2>&1 | tail -5
bun run test 2>&1 | tail -15
```

Expected: every cargo test passes; the build ends cleanly; `bun run test` reports every suite passed with the oracle suite included (not skipped, because `IMPECCABLE_BIN` is set).

- [ ] **Step 4: Run the paid skill-behavior suite**

This suite bills the provider keys in `.env` (about $0.50 to $1.50). The spec requires it because Setup-adjacent text changed.

```bash
bun run test:skill-behavior 2>&1 | tail -20
```

Expected: pass, or failures that are already listed as baseline flakes in `tests/skill-behavior/README.md`. Any new failure that mentions platform routing, `terminal`, `audit`, or `adapt` is a defect in Task 8's text: fix the text, re-run Task 8 Step 7, and re-run this suite. If `.env` is missing, the suite skips with a message; record that it was skipped in the merge commit body rather than claiming it passed.

- [ ] **Step 5: Commit the docs**

```bash
git add docs/CLI-CONTRACT.md CLAUDE.md
git commit -m "Document the terminal platform value

CLI-CONTRACT.md records the parser, boot inlining, WARNING, doctor
evidence, and concept-seed changes; the root guide's Platform section
gains terminal and the .terminal.md variant rule.

AI assistance: drafted with Claude Code."
```

- [ ] **Step 6: Merge to the fork's main**

```bash
git checkout main
git merge --no-ff terminal-platform-pr1 -m "Merge terminal-platform-pr1: the terminal platform value (spec PR 1)

AI assistance: drafted with Claude Code."
git push origin main
```

Then tell the fork owner the merge landed and which gates ran, naming any that were skipped.

---

## Self-review

**Spec coverage (section 1):** parsing (Task 4), boot output (Task 5), hook mode unchanged (Global Constraints, verified in Task 5 Step 6 by the presence of `MANUAL_DETECTOR_REQUIRED`), doctor evidence including workspaces (Task 6; `checkWorkspaces` calls the same function, so no separate change), init (Task 8), concept-seed (Task 7), oracle cases `context-terminal`, `context-bad-platform`, `doctor-terminal-evidence-*`, `signals-terminal`, `seed-platform-terminal` (Tasks 5 to 7). Section 1 also lists `doctor-terminal-*` for the declared project; Task 6 records them.

**Spec coverage (section 2):** `terminal.md` (Task 2), router rows and Setup bullet (Task 8), `audit.terminal.md` and `adapt.terminal.md` (Task 3), guards, routing split, Modes sentence, frontmatter description (Task 8), checks owed (Tasks 8 and 9).

**Spec coverage (section 6, PR 1):** files and gates (Task 9), root guide update (Task 9), fork-first merge (Task 9 Step 6). Upstream issue and PR text are the fork owner's to post and are not in this plan.

**Placeholders:** none. Every file edit shows the exact text; every command shows the expected result.

**Type consistency:** `check_native_platform_evidence(&str, Option<&str>, Option<&str>, Option<&str>) -> Vec<Finding>` is used identically in the tests and the existing caller in `staleness_deep.rs:387`. `manifest_names_dependency(&str, &str) -> bool` is defined and called in Task 6 only. `COMPOSITION_PLATFORMS` array length changes from 3 to 4 and is read only through `.contains` and `.join`, both length-agnostic.
