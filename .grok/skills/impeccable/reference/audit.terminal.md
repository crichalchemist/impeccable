Run systematic **technical** quality checks on a terminal app (`terminal`) and generate a comprehensive report. Don't fix issues; document them for other commands to address.

This is a code-level audit, not a design critique. Audit from source (Ratatui, Bubble Tea and Lip Gloss, Textual and Rich, Ink) and from captures of the running app; no browser tooling applies. From the project root, run `.grok/skills/impeccable/scripts/impeccable detect --json <source directory>`; pass a directory, not single files, so the project signals (TTY guards, adaptive color helpers, icon fallbacks) are collected. With the app running in a tmux pane, add `--tmux <target> --tmux-sizes 80x24,120x40,40x24` to the same command for the `tui-rt-` runtime rules over the rendered grid; those are advisory too, and a saved capture replays with `--tmux-capture <file>`. Every `tui-` finding is advisory: confirm each one against the source before it enters the report. The checks below cover what detect cannot see. Score against [terminal.md](terminal.md); read it before scoring if Setup hasn't already. The report skeleton mirrors [audit.md](audit.md); keep the variants in sync when changing it.

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
- **Suggested command**: Which command to use (prefer: /impeccable adapt, /impeccable animate, /impeccable audit, /impeccable bolder, /impeccable clarify, /impeccable colorize, /impeccable critique, /impeccable delight, /impeccable distill, /impeccable document, /impeccable harden, /impeccable layout, /impeccable onboard, /impeccable optimize, /impeccable overdrive, /impeccable polish, /impeccable quieter, /impeccable shape, /impeccable typeset)

### Patterns & Systemic Issues

Identify recurring problems that indicate systemic gaps rather than one-off mistakes:
- "Hard-coded RGB appears in 12 widgets, should resolve through the theme table"
- "Every panel is double-bordered; one border weight per level would restore hierarchy"

### Positive Findings

Note what's working well: good practices to maintain and replicate.

## Recommended Actions

List recommended commands in priority order (P0 first, then P1, then P2):

1. **[P?] `/command-name`**: Brief description (specific context from audit findings)
2. **[P?] `/command-name`**: Brief description (specific context)

**Rules**: Only recommend commands from: /impeccable adapt, /impeccable animate, /impeccable audit, /impeccable bolder, /impeccable clarify, /impeccable colorize, /impeccable critique, /impeccable delight, /impeccable distill, /impeccable document, /impeccable harden, /impeccable layout, /impeccable onboard, /impeccable optimize, /impeccable overdrive, /impeccable polish, /impeccable quieter, /impeccable shape, /impeccable typeset. Map findings to the most appropriate command. End with `/impeccable polish` as the final step if any fixes were recommended.

After presenting the summary, tell the user:

> You can ask me to run these one at a time, all at once, or in any order you prefer.
>
> Re-run `/impeccable audit` after fixes to see your score improve.

**IMPORTANT**: Be thorough but actionable. Too many P3 issues creates noise. Focus on what actually matters.

**NEVER**:
- Report issues without explaining impact (why does this matter?)
- Provide generic recommendations (be specific and actionable)
- Skip positive findings (celebrate what works)
- Forget to prioritize (everything can't be P0)
- Report false positives without verification
