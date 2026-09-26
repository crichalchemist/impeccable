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
| Compact | 60 to 79 columns | Which secondary region folds away or stacks, how many footer keys still fit, and whether nested borders flatten to one level |
| Standard | 80 to 119 columns | The designed layout; 80x24 is the floor every screen must pass |
| Wide | 120 columns and above | What earns the extra width (a detail pane, a wider table), and what stays capped so lines remain readable |

`.trae/skills/impeccable/scripts/impeccable detect --json --tmux <target> --tmux-sizes 80x24,120x40,40x24` captures the narrow, standard, and wide classes in one run and reports what collapses below 60 columns (`tui-rt-collapse-narrow`) and what drifts on emoji or CJK (`tui-rt-width-drift`). Add a size such as `70x24` to `--tmux-sizes` when the compact class needs its own evidence.

- **Restructure, do not squeeze.** Below 60 columns, hide or stack a region; do not shrink three panes to ten columns each.
- **Cap line length.** At 120 columns and above, text blocks stay near 80 to 100 cells; only tables and grids grow.
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

When the adaptation feels at home in each context, hand off to `/impeccable polish` for the final pass.

**NEVER**:
- Ship a layout that assumes 80x24
- Squeeze every region into a narrow window instead of choosing what survives
- Assume truecolor or a Nerd Font
- Emit color or animation into a pipe
- Lock the size or clear the screen to dodge a layout bug
