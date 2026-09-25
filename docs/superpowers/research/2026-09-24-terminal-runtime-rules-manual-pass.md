# Terminal runtime rules: manual tmux pass

Date: 2026-09-25. Branch `terminal-platform-pr3`. Binary: `target/release/impeccable` (commit `d58a38a0`, unchanged through this pass).

This is the calibration record spec section 6 requires before any `tui-rt-*` rule leaves `advisory` (PR 3 gate). It also answers PR 3's two deferred open questions: the one-second recapture window for `tui-rt-spinner-never-rests`, and whether `client_termfeatures` is populated from outside the session. All seven rules stay `advisory` in this PR.

Everything ran on a scratch tmux server, socket `impeccable-rt-pass`, kept separate from the user's own `cockpit` session on the default socket throughout.

## Programs staged

| Window | Program | Version |
|---|---|---|
| `top` | macOS `top` | macOS 13.7.8 (build 22H730); no version flag, built-in |
| `vim` | Vim, opened `README.md` | `VIM - Vi IMproved 9.0 (2022 Jun 28, compiled Jul 26 2024 23:25:35)` |
| `less` | `less`, opened `CLAUDE.md` | `less 581.2 (POSIX regular expressions)` |
| `man` | `man tmux` | tmux `3.7c`; macOS's stock `man`/groff |
| `demo2` | Ratatui's `demo2` example app | ratatui/ratatui `21324fe8f1c5dcf10c709cbc1cd77d67fca46926`, 2026-09-23 |

`htop`, `btop`, `lazygit`, `gitui`, `nvim` are not installed on this machine; the brief's `command -v` loop added nothing for them, as expected. Five programs covers the brief's four-program floor.

## Deviations from the brief

1. **`demo2` is a workspace package now, not an `--example` target.** The brief's `cargo build --release --example demo2` fails on this ratatui checkout (`error: no example target named demo2`) because the repo restructured: `examples/apps/demo2/` is its own crate. Built with `cargo build --release -p demo2` instead (`cargo build (106 crates compiled)`, `Finished release profile [optimized] target(s) in 40.39s`, well under the ten-minute ceiling); binary at the workspace `target/release/demo2` (the ratatui checkout's own top-level `target/`, not nested under `examples/`), staged into the `demo2` window the same way.
2. **`vim --clean` was used directly, without first checking whether plain `vim` prompts.** The brief's context note said to fall back to `--clean` only if plain `vim` showed a colorscheme prompt; `--clean` was used up front since it is also more representative of a bare, unconfigured session and avoids a possible interactive prompt blocking the window. `vim --clean README.md` renders with syntax highlighting on (the stock runtime still colors markdown), so the run is not degraded.
3. **A timing re-run.** The first timing pass used shell `date +%s.%N`, which macOS's `date` does not support (no `%N`), so the numbers were whole-second garbage. Re-ran with `/usr/bin/time -p` exactly as the brief specifies, output discarded (`> /dev/null`) since the judged JSON from the first pass was kept as the record of findings.
4. **Four supplementary probes beyond the five staged programs**, all outside the five-program table above, reported under "Open questions" and "Rules with no hits" since they exist to test specific mechanisms, not to calibrate a real program's rules: five repeated single-size scans of `pass:top` (no `--tmux-sizes`); a `spinner-probe` window cycling a 10-glyph braille spinner every 100ms (1000ms period), scanned five times; a second, later `spin800` window cycling an 8-glyph spinner at the same 100ms tick (800ms period) as a control, scanned five times on a freshly restarted scratch server; and a `less-capture-256.txt`/`less-capture-rgb.txt` header-replay pair built from the saved `less-capture.txt` to directly test the `tui-rt-truecolor-on-256` fallback path.
5. **A correction round.** The first version of this record made three claims without running the check behind them (the `less` capture-replay footer claim, the `demo2` nested-box claim, and the truecolor fallback-path claim), had a section that stated the `-J` reasoning backwards for the word-split branch and cited nonexistent `tmux capture-pane` flags (`-S`/`-E`, the `O` flag of `-F`) as a fix path, and overclaimed the restore check for three of five windows. The scratch server was torn down, then restarted to gather the missing direct evidence (engine-restore re-check on `vim`/`less`/`man`, the 800ms spinner control, the header-replay pair), then torn down again; both kill-servers used `-L impeccable-rt-pass` and both were followed by a `tmux ls` check confirming only `cockpit` remained on the default socket.
6. No rule count changed the oracle or `crates/terminal`; no code fix was made (see "Why no fix was made" below), so `git add` in Step 6 only stages the record file.

## Scan times and exit codes

`impeccable detect --no-config --json --tmux "pass:<w>" --tmux-sizes 80x24,120x40,40x24`, timed with `/usr/bin/time -p`:

| Window | Exit | real | user | sys |
|---|---|---|---|---|
| `top` | 0 | 2.20s | 0.08s | 0.08s |
| `vim` | 0 | 2.22s | 0.08s | 0.08s |
| `less` | 0 | 2.22s | 0.08s | 0.08s |
| `man` | 0 | 2.22s | 0.08s | 0.09s |
| `demo2` | 0 | 2.22s | 0.08s | 0.08s |

All five exit 0; no `.err` file has content. The ~2.2s per scan is three `resize-window` + 300ms settle + capture cycles plus the pane's own capture and the one-second recapture, matching the multi-size pass order in spec section 5.

## Restore check

`tmux display -p '#{window_width}x#{window_height} #{window-size}'` reported `100x30 latest` (the session's `-x 100 -y 30`, never stuck at `manual`) at every point it was checked. Two different things were checked, and they are reported separately rather than as one blanket claim:

- **Directly after an engine scan, with no other action in between**: `top` (immediately after its Step 2 multi-size scan) and, in a follow-up check run for this correction, `vim`, `less`, and `man` (each re-scanned with `--tmux-sizes 80x24,120x40,40x24` and checked immediately after). All four: `100x30 latest`. This isolates the engine's own `resize-window` + `set-option -w -u window-size` restore.
- **After a mix of engine scans and this investigation's own manual 40x24 resizes** (done to compare `capture-pane` with and without `-J`, below): every manual resize in this pass was itself followed by a manual restore (`resize-window -x 100 -y 100` + `set-option -w -u window-size`) before the next check, and every one of those checks also read `100x30 latest`. `demo2` was never manually resized, so its `100x30 latest` reading reflects only the engine's own restore across its several scans.

Net: the engine's restore was directly confirmed, isolated from any other resize, on all five windows.

## Palette and capture/replay checks

`--palette light` on `pass:top` (native size only, no `--tmux-sizes`) returned one finding, `tui-rt-no-key-hints` on the bottom process row. This run used a different frame set (native size only) than the dark-palette run in the main sweep (three sizes), so it is not a same-content dark-versus-light comparison; no low-contrast finding differed between the two runs, because `top`'s dark-palette sweep also had zero `tui-rt-low-contrast` hits at any size. No palette-driven contrast difference was actually observed in this pass; only that `--palette light` runs and reports the same rules the dark default does.

The review-flow path (`terminal.md`'s saved-capture replay): `tmux capture-pane -p -e -J -t pass:less > less-capture.txt` (15 lines) replayed through `impeccable detect --no-config --json --tmux-capture less-capture.txt` returned `[]`. The narrow-collapse half of that is straightforward: a single un-resized capture at the window's native 100 columns has no narrow frame to trip `tui-rt-collapse-narrow`. The no-key-hints half needed checking, not assuming, and the first version of this record asserted it without running the check. Reading the actual last non-blank line of the file (SGR stripped) shows it is not a footer or prompt at all: it is ordinary wrapped `CLAUDE.md` prose, cut off mid-sentence ("...It exports \`IMPECCABLE_SKILL_DIR\` (the:"), joined by `-J` from several on-screen rows into one very long line. `less` was not at end-of-file and was not run with `-M`, so it was not showing any status/prompt line in this capture at all. `tui-rt-no-key-hints` correctly found no key-hint token in that text, but this capture cannot be used as evidence that the rule handles a real `less` footer correctly, because there was no footer present to test it against. This ties into the `-J` coordinate issue below: `-J` also changes *which* physical row becomes "the last non-blank row" the rule reads, and this capture cannot rule out that a real footer could be hidden by that shift in a different program.

## Hits per program (full three-size sweep)

| Window | Total | Findings by rule |
|---|---|---|
| `top` | 3 | `tui-rt-no-key-hints` 1, `tui-rt-collapse-narrow` 2 |
| `vim` | 7 | `tui-rt-low-contrast` 2, `tui-rt-no-key-hints` 1, `tui-rt-collapse-narrow` 4 |
| `less` | 3 | `tui-rt-collapse-narrow` 3 |
| `man` | 7 | `tui-rt-no-key-hints` 1, `tui-rt-collapse-narrow` 6 |
| `demo2` | 3 | `tui-rt-low-contrast` 3 |

23 findings total, three rules with hits, four rules with none.

## The known interaction: width-drift and collapse-narrow double-reporting

Spec context named this as an already-known interaction to measure, not discover: `tui-rt-width-drift` and `tui-rt-collapse-narrow` both fire on a row wider than the pane under 60 columns. **It happened zero times in this sample.** `tui-rt-width-drift`'s overflow branch is gated behind a wide-character check before either of its two branches runs at all:

```
crates/terminal/src/rules.rs:278   if !frame.has_wide(r) { continue; }
```

None of the five programs' 40x24 frames contained a double-width glyph (no CJK, no emoji): `top` and `man` are pure ASCII/digits, `vim` and `less` show plain-text markdown/CLAUDE.md prose, and `demo2`'s block-drawing characters (`▀▄`) are single-width. So `tui-rt-width-drift` never evaluated any of the 15 `tui-rt-collapse-narrow` hits, and the double-report the spec flags is real only for wide-glyph rows, which this pass never exercised. The 40-column noise actually observed (below) comes from a different mechanism entirely.

## A second interaction, not in the spec: `-J` line-joining corrupts the collapse-narrow overflow check

This is the pass's main finding, and it dominates the false-positive count.

`tmux.rs:129` captures with `capture-pane -p -e -J -t <target>`. Per `man tmux` (`capture-pane` section, quoted verbatim): **"-J preserves trailing spaces and joins any wrapped lines; -J implies -T."** When an application's output line exceeds the pane's width and the terminal (not the application) wraps it across several screen rows, `-J` stitches those rows back into one long logical line before the parser ever sees them.

**The overflow branch only ever fires on a joined row, by construction.** On a live pane, `capture-pane` without `-J` never returns a row longer than the pane's own width; there is nowhere for the extra cells to come from. So `row.len() > frame.width` (`rules.rs`'s overflow check) can only be true because `-J` joined two or more on-screen rows into one. That makes `-J` a precondition for the overflow branch to run at all, not an occasional source of noise in it: every "row overflows the pane" finding in this sample is, by the mechanism itself, evidence of a join. The question the branch cannot answer is whether that join represents a genuine layout defect (an element that really is too wide and would be clipped) or ordinary wrapped prose that reads fine across several screen rows. This pass's evidence is that on four of five real programs, it was the latter.

Direct evidence, comparing `capture-pane -p -e -J` against `capture-pane -p -e` (no `-J`) at 40x24 on the same live pane, both captured independently (not derived from each other or from a finding's own numbers):

| Window | With `-J`, full non-blank row lengths | Without `-J`, same rows |
|---|---|---|
| `vim` | 12, 153, 185, 18, 182, 80, 39 | 12, 39, 40, 39, 33, 40, 40, 40, 40, 25, 18, 40, 40, 40, 40, 22, 40, 3, 39 (19 rows, every one ≤40) |
| `less` | 33, 23, 181, 219, 281 | 33, 23, then sixteen rows each 40 or fewer (21 rows total, every one ≤40) |
| `man` | 98, 4, 32, 8, 85, 41, 11, 98, 97, 43, 1 | eleven rows collapse to twenty-one, every one ≤40 |
| `top` | 40, 40, 40, 40, 40, 40, 40, 40, 38, 38, 37, 40, 36, 37, 36, 37, 37, 37, 37, 36, 36, 39 | identical row for row (`top` never produces a row over 40, so `-J` has nothing to join) |

`vim`'s with-`-J` rows of 153, 185, and 182 line up exactly with `tui-rt-collapse-narrow`'s hits at lines 3, 5, and 9 (overflow amounts 113, 145, and 142, i.e. joined length minus 40); the row of 80 lines up with the hit at line 11 (40 over). `less`'s 181, 219, and 281 line up with its three hits; `man`'s 98, 85, and the rest of its with-`-J` lengths line up with its six. `top` never joins, which is why its two `tui-rt-collapse-narrow` hits are of the *other* kind (word-split, judged below) and not overflow.

**The word-split branch, conversely, can only ever see rows `-J` did *not* join, because the overflow branch's `continue` claims a joined row first.** Reading `rt_collapse_narrow` in order: a row wider than the pane hits the overflow branch and `continue`s past every later check in the same iteration, including the word-split comparison against `row.last()` and `frame.rows.get(r + 1)`. A row `-J` joined from a genuine soft-wrap (the case that matters: an app that wrapped a word across the screen boundary) becomes exactly the kind of row the overflow branch claims, so word-split never gets to look at it. Word-split can only fire on a row that is independently ≤ the pane width and was never merged with its neighbor, which in this sample was `top`'s output (each stat line self-contained, never wrapped by the terminal). `-J`, as used here, structurally prevents the word-split check from ever seeing the case it plausibly exists to catch, in any program that relies on ordinary soft-wrap.

**Man and top, side by side, show `-J` joining only when the terminal reflows stale content.** At the pane's native 100-column size, `man`'s content did not join: its bottom-row `tui-rt-no-key-hints` finding at line 30 lines up directly with a non-`-J` capture's line 30 (`:`), because `groff` formats each `man` line to the terminal's width with a real newline, so nothing needs reflowing at 100 columns. After the multi-size sweep resized the pane to 40 columns, `man` did not reformat (`man`/`less` do not reflow already-rendered content on `SIGWINCH`), so its still-100-column-wide lines got auto-wrapped by the terminal itself at the new 40-column boundary, and `-J` faithfully reassembled those auto-wrapped screen rows back into their original ~100-column logical lines, each of which then reads as "overflowing" the new 40-column pane. `top`, by contrast, never had a row wider than 40 at any size in this sample, so nothing shifted for it. These are not contradictory statements: `man`'s content is unjoined at its native size and joined only after the pane narrows past its stale content's width, while `top`'s content is unjoined at every size tested.

`-J` also shifts line/column numbers for any row it joins, which breaks the brief's own verification recipe (`tmux capture-pane -p -t pass:<w> | sed -n '<line>p'`, which does **not** pass `-J`) whenever a finding sits on a joined row. Concretely: `vim`'s `tui-rt-no-key-hints` finding reports `line=21`, but a non-`-J` capture of the same pane has 30 rows, and `sed -n '21p'` on it returns an unrelated prose line ("...ing those facts with surface-level visual direction."), not the status line the finding actually names. The finding's *content* is still correct (row 21 of the 21-row `-J`-joined frame genuinely is the last non-blank row, carrying the file-info message), but a reviewer following the brief's literal verification command on a joined row will look at the wrong screen position. Likewise `vim`'s low-contrast hit at `col=160` on a 100-column pane only makes sense against the joined row, not the physical 100-column screen. `man`'s findings did not show this drift at its native size (confirmed above), and `top`'s never join at any size tested in this sample.

**Why no fix was made.** This is not a parser bug (no SGR misread, no missing glyph, no width miscount): `-J` is the capture flag spec section 5 specifies (`tmux.rs:129`) and `terminal.md`'s saved-capture instruction assumes it too. Removing it outright would make the overflow branch permanently dead on live scans (see above: without `-J`, no row can ever exceed the pane width), at the cost of also making an app's own soft-wrapped paragraphs look like an overflow at every wrap boundary if `-J` were kept only some of the time. That trade is a rule-definition change, not a misread correction, so per the brief it is recorded here as a recommendation instead of made:

- Keep `-J`, but discriminate on what the joined row *contains* rather than only its length. The rule's own unit test for the intended true-positive shape (`narrow_frame_overflow_lost_corner_and_split_word_are_collapse`, `rules.rs`) constructs a border row, `┌` plus 48 dashes plus `┐`, that is 50 cells wide on a 40-wide frame: a real box-drawing element that does not fit, not wrapped prose. A join that contains box-drawing glyphs (`TOP_LEFT`/`VERTICAL`/etc.) plausibly is a genuine layout defect; a join that is plain text is plausibly a wrapped paragraph. This sample's false positives (13 of 15) were all plain text; none contained a border glyph.
- Or capture without `-J` for the overflow check specifically, accepting that this reopens the word-split branch to genuinely-wrapped rows (which `-J` currently forecloses, as shown above) at the cost of every app-level soft wrap now looking like an overflow at its own wrap boundary, which would need its own new discriminator.
- Document, in `docs/CLI-CONTRACT.md` or `terminal.md`, that finding line/column numbers index the `-J`-joined frame, not a plain `capture-pane -p` capture, so a reviewer following the manual-verification recipe should also add `-J` when checking a finding by hand.

## Judged samples

### `tui-rt-low-contrast` (5 hits, 5 sampled)

| Program | Line | Col | Frame | Snippet | Class | Reason |
|---|---|---|---|---|---|---|
| `demo2` | 1 | 40 | 100x30 | `"Recipe  Email  Tracerout" #808080 on #101830: 4.4:1 (need 4.5:1)` | TP | Unselected tab label; gray-on-navy, borderline but real, absolute RGB colors |
| `demo2` | 18 | 6 | 100x30 | `"H/←  " #080808 on #444444: 2.1:1` | TP | Dim key-hint row itself (`H/← Left  L/→ Right  K/↑ Up  J/↓ Down  D/Del Destroy  Q/Esc Quit`); near-black on dark gray, genuinely hard to read; absolute colors |
| `demo2` | 18 | 11 | 100x30 | `"Left " #444444 on #080808: 2.1:1` | TP | Same row, inverse half of the same dim pair; absolute colors |
| `vim` | 1 | 3 | 100x30 | `"Impeccable" #cd00cd on #000000: 4.5:1 (need 4.5:1)` | arguable | The markdown H1, magenta-on-black. `#cd00cd`/`#000000` are xterm's *named* ANSI colors (SGR 35/40), not literal RGB; the unrounded ratio is ~4.478 (rounds to "4.5" in the snippet, which reads confusingly like a tie but is a genuine, if marginal, fail against the assumed reference palette). A user who re-themes their terminal's 16-color table would see a different ratio |
| `vim` | 5 | 160 | 100x30 | `"https://impeccable.style" #cd0000 on #000000: 3.6:1` | arguable | The markdown link URL, red-on-black (SGR 31). Clearly fails under the assumed xterm table, but same palette-relative caveat as above; also a `-J`-joined row (see above), so `col=160` does not correspond to an on-screen column on the 100-wide pane |

demo2's three findings use absolute 256-color/RGB-style values (unconditionally true in any terminal); vim's two use named ANSI colors resolved through the engine's assumed xterm reference table (`palette.rs`), which is correct behavior per spec ("Named colors 0 to 15 use xterm's table in both [palettes]") but means the finding is relative to that assumption, not to what any given user's terminal actually renders. 0 hard false positives, but 2 of 5 samples carry this palette-relative caveat.

### `tui-rt-no-key-hints` (3 hits, 3 sampled)

| Program | Line | Col | Frame | Snippet | Class | Reason |
|---|---|---|---|---|---|---|
| `vim` | 21 | 1 | 100x30 | `bottom row "\"/Volumes/Containers/impeccable/README.md\" 514L, 26991B     " names no key"` | TP | Vim's default file-info status line genuinely names no key; well-known vim usability gap |
| `man` | 30 | 1 | 100x30 | `bottom row ":" names no key` | arguable | The bare pager prompt is technically keyless, but it is a decades-old Unix idiom (`less`/`man` users learn `:` means "waiting for a command"), not a UI the rule's "show the keys" framing was written for |
| `top` | 30 | 1 | 100x30 | `bottom row "2287   MIDIServer   0.2  04:16.52 6     1    80     1848K  0" names no key"` | arguable | The "bottom row" here is just wherever the scrolling process table happens to end at the current pane height; it is content, not a designed footer/status bar. The rule's premise (the last non-blank row is where hints live) does not hold for continuously-scrolling table UIs |

0 hard false positives, but 2 of 3 samples expose a real design gap: the rule assumes the last non-blank row is a static footer, which breaks for scrolling-table programs (`top`, and by extension `htop`/`ps`-style views) and for terse pager prompts.

### `tui-rt-collapse-narrow` (15 hits, all 15 classified, 10 printed)

| Program | Line | Col | Frame | Snippet | Class | Reason |
|---|---|---|---|---|---|---|
| `top` | 1 | 40 | 40x24 | `word split at the pane edge: "ng, 717 sle"` | arguable | Real content is lost ("704 sleeping" truncates to "704 sle" with no continuation on the next row, verified: `top`'s next row is an unrelated "Load Avg:..." stat, not "eeping"), so a genuine narrow-collapse problem exists. But the rule's stated mechanism (a word wrapped across the row boundary) did not happen; `top` does not wrap, it truncates, and "sle" + "Load" only coincidentally satisfies "ends in a letter, next starts with a letter" |
| `top` | 5 | 40 | 40x24 | `word split at the pane edge: "d, 767M com"` | arguable | Same mechanism: `top` truncates "767M compressed" mid-word; the match is coincidental, the data loss is real |
| `vim` | 3 | 41 | 40x24 | `row overflows the 40-column pane by 113 cells` | FP | `-J`-joined wrapped prose, see above; joined length 153, corresponding to several physical rows each ≤40 cells without `-J` (independently confirmed in the row-length table above) |
| `vim` | 5 | 41 | 40x24 | `row overflows the 40-column pane by 145 cells` | FP | Same mechanism, joined length 185 |
| `vim` | 9 | 41 | 40x24 | `row overflows the 40-column pane by 142 cells` | FP | Same mechanism, joined length 182 |
| `vim` | 11 | 41 | 40x24 | `row overflows the 40-column pane by 40 cells` | FP | Same mechanism, joined length 80 |
| `less` | 5 | 41 | 40x24 | `row overflows the 40-column pane by 141 cells` | FP | `-J`-joined wrapped prose, joined length 181 |
| `less` | 7 | 41 | 40x24 | `row overflows the 40-column pane by 179 cells` | FP | Joined length 219 |
| `man` | 1 | 41 | 40x24 | `row overflows the 40-column pane by 58 cells` | FP | `man`'s stale, groff-formatted-at-100-columns buffer auto-wrapped by the terminal after the resize to 40; `-J` reassembles the original 98-cell line. Visually this still renders as readable ≤40-column rows, same as an app-level soft wrap |
| `man` | 7 | 41 | 40x24 | `row overflows the 40-column pane by 45 cells` | FP | Same mechanism, joined length 85 |

The remaining five (`less` line 8 by 241 cells; `man` lines 8, 11, 12, 13 by 1, 58, 57, and 3 cells) all match the identical `-J`-join arithmetic (each overflow amount equals the joined row length minus 40, verified programmatically) and are the same false-positive class.

13 of 15 (87%) are `-J`-join false positives; the other 2 (13%) are arguable rather than clean true positives, since the stated word-split mechanism did not literally occur even though the underlying narrow-width data loss is genuine. **0 of 15 are the "border loses its right corner" branch**; that branch was never exercised by any of the five programs.

## Rules with no hits

- **`tui-rt-width-drift`**: unmeasured. Its overflow branch requires a double-width glyph in the row (`rules.rs:278`); none of the five programs render CJK or emoji. Its border-end-mismatch branch is gated behind the *same* `has_wide` check (the `continue` at line 278 sits before both branches), so it too went unevaluated on every ASCII/box-drawing row in this sample. This matches the rule's stated premise, not a defect: its own unit test for the border-mismatch branch (`emoji_row_wider_than_the_pane_is_width_drift_and_a_shifted_border_too`, `rules.rs`) uses a row containing `日本` (CJK, double-width) specifically because the rule is about wide-character-caused misalignment, not general border checking; `tui-rt-collapse-narrow`'s own "border loses its right corner" branch already covers non-wide-char border loss. No recommendation to change the gate.
- **`tui-rt-nested-borders`**: unmeasured, and the reason needed checking rather than a five-line glance at the capture. `grep -rn "border\|Block::" examples/apps/demo2/src` shows the Recipe tab actually scanned (`tabs/recipe.rs:119-124`) builds its `Block::new()` with no `.borders(...)` call at all: its outer frame is `RgbSwatch`-painted background blocks (`▀`/`▄`), not box-drawing glyphs, so `find_boxes` had nothing to find there regardless of nesting. Two other `demo2` tabs, `email.rs:125-129` and `traceroute.rs:95-98`, do use `BorderType::Thick` (real box-drawing glyphs `find_boxes` would recognize), but this pass never scanned those tabs: `demo2` opens on Recipe by default, and a `Tab` keypress sent during exploration did not visibly change the captured content. `vim`/`less`/`man` draw no boxes at all in this configuration. Unmeasured for lack of a nested-box example in what was actually captured, not confirmed-clean.
- **`tui-rt-truecolor-on-256`**: unmeasured on live pane scans, and this ties directly into open question 2 below. `frame.termfeatures` was empty for every scan (no client ever attached to the detached scratch server), and the rule returns immediately when `termfeatures` is empty (`rules.rs:124-126`), before it ever reaches the `COLORTERM`-vouching check. The saved-capture fallback path was replayed with real evidence, not asserted: `less-capture.txt` (no truecolor SGR at all, confirmed by scanning the raw bytes for `38;2;`/`48;2;`) was copied to `less-capture-256.txt` and `less-capture-rgb.txt` with `#!capture width=100 height=30 termfeatures=256` and `termfeatures=256,RGB` headers respectively; both scanned `[]`, which is the correct, expected result given there is no truecolor SGR in the content either header applies to. The committed fixture `tests/fixtures/terminal-captures/truecolor-on-256.txt` (header `termfeatures=256`, a real `38;2;200;100;50` SGR in its body) was also scanned directly and returns exactly one `tui-rt-truecolor-on-256` finding at line 2, column 3, confirming the header-driven fallback path fires correctly when the capture actually contains truecolor content. Separately worth noting for calibration: even with a client attached, `frame.colorterm` is populated from the *engine process's own* `COLORTERM` (`truecolor`, confirmed present in this shell's environment), not from the actual client terminal's capability (spec section 5: "and from `COLORTERM` in the engine's environment"). An agent running the scan from inside a `COLORTERM=truecolor` shell could silence a real finding about a client that genuinely lacks RGB support. This is the design as specified, not a bug, but worth flagging as a caveat for anyone reading a clean truecolor result as confirmation the *viewer* supports RGB.
- **`tui-rt-spinner-never-rests`**: unmeasured on the five real programs; see "Open questions" below for two targeted probes that measure the rule's actual firing rate.

## Open questions

### Is one second the right recapture window for `tui-rt-spinner-never-rests`?

**On the five real programs: no data.** The rule fired zero times, including on `top`, across the required scan and five additional single-size rescans of `pass:top` alone (`top-run1`...`top-run5`, each an independent `--tmux pass:top` invocation with its own internal one-second recapture): 0/5. `top` redraws its whole screen roughly every second, but it never displays a spinner-shaped glyph (`SPINNER_GLYPHS`, the ASCII set plus U+2800-U+28FF); its content is process-table digits and text, so the rule's premise (the *same cell* holds a spinner-class glyph in both frames that differs between them) never had a candidate cell to evaluate on `top`. `vim`, `less`, and `man` were idle and unsurprisingly showed nothing either. `demo2`'s Recipe tab has no animation without input.

**A targeted probe, outside the five-program set, gives the direct evidence the five programs could not.** A throwaway Node one-liner cycling a 10-glyph braille spinner (`⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`) at a 100ms period was run in its own scratch window and scanned five independent times:

```
run1 spinner-hits=0
run2 spinner-hits=1
run3 spinner-hits=0
run4 spinner-hits=0
run5 spinner-hits=0
```

A spinner that was demonstrably animating the entire time (confirmed visually mid-run) was missed 4 of 5 times. This is the aliasing risk the spec's open question names directly: ten frames at a 100ms period is exactly the one-second recapture window, so frame 0 and the recapture land on the same glyph in most trials; only shell/timer jitter (`setInterval` drift, process scheduling) occasionally shifts the alignment enough to catch a genuine mismatch, which is what happened once in five runs.

**A control run makes this causal rather than coincidental.** The same probe, rebuilt with an 8-glyph cycle at the same 100ms period (an 800ms period, which does not divide 1000ms evenly), was scanned five independent times on a freshly-restarted scratch server:

```
run1 spinner-hits=1
run2 spinner-hits=1
run3 spinner-hits=1
run4 spinner-hits=1
run5 spinner-hits=1
```

5/5, versus 1/5 for the 1000ms-period (10-glyph) spinner and 0/5 for any of the five real programs (none of which shows a spinner glyph at all). The only variable changed between the two probes is whether the cycle period divides the one-second recapture window evenly. **Conclusion: the one-second window is not reliable for spinners whose period divides one second evenly, and is reliable for spinners whose period does not**, exactly as spec section 5 anticipated when it made the rule advisory. This is real, controlled evidence, not a restatement of the design doc's caveat, but it comes from two synthetic probes rather than a program from the required list, since none of the five available real programs exercises the rule at all.

### Is `client_termfeatures` populated from outside the session?

**No.** `tmux -L impeccable-rt-pass display -p -t pass:top '#{client_termfeatures}'` on the fully detached scratch server returned an empty string, and `list-clients -t pass` returned nothing: no client has ever attached. There was no second terminal available in this environment to attach a client and re-check with one present; this is stated rather than guessed. This matches the spec's stated fallback exactly: `tui-rt-truecolor-on-256` cannot fire from a pane scan run against a server with no attached client, only from a `--tmux-capture` file whose header names `termfeatures` explicitly (confirmed separately: `less-capture.txt`, captured without a header, also produced empty `termfeatures` and an empty finding array).

## Scan from inside a tmux session

Per the additional required scan (an agent running from inside a tmux session, where `$TMUX` is already set and no `IMPECCABLE_TMUX` wrapper is used):

```
tmux -L impeccable-rt-pass new-window -t pass -n inside \
  "cd /Volumes/Containers/impeccable && env -u IMPECCABLE_TMUX target/release/impeccable detect --no-config --json --tmux pass:less > inside.json 2> inside.err; echo exit=$? > inside.exit"
```

- `inside.exit`: `exit=0`
- `inside.json`: `[]` (valid JSON, matching the native-capture-replay result for `less` above)
- `inside.err`: empty
- `inside.tmux` (a companion check, `echo $TMUX`): `/private/tmp/tmux-501/impeccable-rt-pass,28946,0`

`$TMUX` inside the window points at the scratch server's own socket path, so plain `tmux` (no `-L`, no wrapper) resolved there rather than to the default socket. This is the expected, correct behavior: had resolution fallen through to the default socket, `pass:less` would not exist there and the scan would have failed at `list-panes` with `Error: tmux list-panes: ...` and exit 1, not the clean `exit=0` / `[]` observed. The engine's `$TMUX`-first resolution (spec section 5: "The engine talks to whatever server the `tmux` client resolves: the socket in `$TMUX` inside a session, else the default socket") works correctly for the review-flow path an agent would actually use when it is itself running inside tmux.

## Promotion recommendations

Every rule stays `advisory` in this PR.

| Rule | Hits | Sampled | False positives | Verdict / recommendation |
|---|---|---|---|---|
| `tui-rt-low-contrast` | 5 | 5 | 0 hard FP; 2 arguable (palette-relative) | Promising but not yet a clean promotion candidate: recompute or caveat findings that resolve through the named-ANSI-color table (SGR 30-37/90-97) separately from literal RGB/256 findings, since only the latter are unconditionally true |
| `tui-rt-no-key-hints` | 3 | 3 | 0 hard FP; 2 arguable | Tighten before promotion: exclude, or lower confidence on, the last row of programs whose content scrolls (no stable footer concept), and consider a short-token allowlist (`:`, `(END)`, `--More--`) for pager idioms |
| `tui-rt-collapse-narrow` | 15 | 15 (10 printed) | 13/15 (87%) FP from `-J` joining; 2/15 arguable | Tighten before promotion: the overflow branch is unreliable for any program whose output relies on ordinary line-wrapping (app-level soft-wrap or terminal auto-wrap of stale content), which in this sample was most of the programs tested. See "A second interaction" above for the recommended fix path; not attempted here because it is a capture/row-model design decision, not a misread |
| `tui-rt-width-drift` | 0 | n/a | n/a | Unmeasured; needs a program with double-width (CJK/emoji) content. The `has_wide` gate matches the rule's stated premise (its own unit test uses `日本`); no change recommended |
| `tui-rt-nested-borders` | 0 | n/a | n/a | Unmeasured; needs a scan of a program (or `demo2` tab) that actually draws box-drawing borders with one literally nested inside another. `demo2`'s Recipe tab, the only one scanned here, draws no box-drawing borders at all |
| `tui-rt-truecolor-on-256` | 0 | n/a | n/a | Unmeasured on live scans by construction (no attached client on a detached server), directly re-confirmed with a header-driven replay of `less`'s own capture (no truecolor content, correctly `[]` under both `termfeatures=256` and `termfeatures=256,RGB`) and of the committed fixture (real truecolor SGR, correctly fires). The live-scan gap is the answer to open question 2, not a rule defect |
| `tui-rt-spinner-never-rests` | 0 on the five real programs | n/a (two 5-run synthetic probes outside the required set: 1000ms period 1/5, 800ms-period control 5/5) | n/a | Unmeasured on real programs available here. The controlled probe pair confirms, causally, the spec's stated aliasing risk rather than just restating it; needs a real spinner-bearing program (`htop`, `lazygit`, `gitui`, or similar) for a proper calibration sample |

## Fixes made

None. The dominant false-positive source found (`-J` line-joining, above) is an architectural capture-flag question that changes what "row" means for every geometry rule, not a narrow misread; per the brief it is recorded as a recommendation rather than patched in this pass. No `crates/terminal` source changed, so no unit test was added and no `detect-tmux-*` golden was re-recorded.

## Teardown

`tmux -L impeccable-rt-pass kill-server`, twice (once after the main pass, once after the correction round's follow-up server), each followed by `tmux ls` on the default socket, which confirmed only `cockpit` remained both times.

## PR 4 re-run

Date: 2026-09-25. Branch `terminal-platform-pr4`, commit `d2d4d12f`. Binary: `target/release/impeccable`, built from that commit (PR 4's item 1 box-drawing gate, item 7 pager-prompt exemption, and item 8 `paletteRelative` are all present in the scanned binary). Scratch tmux server, socket `impeccable-rt-pass-pr4`, run under its own `-L` throughout; none of the five staged programs ran on the default socket. `tmux -V`: `tmux 3.7c`, same as the PR 3 pass.

Same five programs as PR 3, same versions: macOS `top`, `vim --clean README.md` (`VIM - Vi IMproved 9.0`), `less CLAUDE.md` (`less 581.2`), `man tmux`, and ratatui's `demo2` example. **`demo2`'s build outcome**: no build was run. Per controller ruling C3, the release binary already existed at `$R/target/release/demo2` (ratatui checkout `21324fe8f1c5dcf10c709cbc1cd77d67fca46926`, the same commit and the same binary PR 3 built, staged this pass without a rebuild). `demo2` opens on its default Recipe tab, which (re-checked in `tabs/recipe.rs`) still calls `Block::new()` with no `.borders(...)`, so this pass again scans a `demo2` screen with no box-drawing glyphs in it.

### Hits per program (full three-size sweep, `--tmux-sizes 80x24,120x40,40x24`)

| Window | Total | collapse-narrow (PR 3) | collapse-narrow (PR 4) | Other rules (PR 4) |
|---|---|---|---|---|
| `top` | 3 | 2 | 2 | `tui-rt-no-key-hints` 1 |
| `vim` | 3 | 4 | 0 | `tui-rt-low-contrast` 2, `tui-rt-no-key-hints` 1 |
| `less` | 0 | 3 | 0 | none |
| `man` | 0 | 6 | 0 | none |
| `demo2` | 3 | 0 | 0 | `tui-rt-low-contrast` 3 |

**`tui-rt-collapse-narrow` total: 2, against PR 3's 15 (a drop of 13, an 87% reduction).** All exits were 0, every `.json` file parsed, and every finding's `severity` field reads `advisory` (checked programmatically across all five files); rules stay `advisory` in this PR too, as the brief requires. Scan times (`/usr/bin/time -p`, same three-size sweep as PR 3): `top` 2.73s real, `vim` 2.37s, `less` 2.34s, `man` 2.35s, `demo2` 2.31s (PR 3's were ~2.2s each; the difference is consistent with PR 4's two recaptures and settle-wait polling, not a regression worth chasing further here).

### The 2 remaining collapse-narrow findings, judged

Both are on `top`, both are the same word-split branch PR 3 already classified as arguable, and the brief's own item 1 explains why they survive: "Collapse-narrow's corner and split-word checks are unchanged." The box-drawing gate (item 1) only narrows the *overflow* branch (`overflow_verdict`, `rules.rs:324`); the word-split and corner checks sit behind `Overflow::Fits` (`rules.rs:375`), reached only when a row does *not* overflow the pane at all, so the gate never touches them.

| Program | Line | Col | Frame | Snippet | Class | Reason |
|---|---|---|---|---|---|---|
| `top` | 1 | 40 | 40x24 | `word split at the pane edge: "ng, 726 sle"` | arguable | Confirmed directly with a separate 40x24 capture (a few minutes later, so `top`'s own live counters read differently, e.g. "723 sle"): `top`'s row is 40 cells with and without `-J` (identical), i.e. `top` truncates rather than wraps; the follow-on continuation on the next row is an unrelated `MemRegions:` stat line, not "eeping". Real content is lost, but the rule's stated word-split mechanism ("wrapped across the row boundary") did not literally happen; "sle" + a next row starting with a letter is coincidental, same finding PR 3 made |
| `top` | 5 | 40 | 40x24 | `word split at the pane edge: "d, 487M com"` | arguable | Same mechanism as above (PR 3's sample read `"d, 767M com"`); `top`'s live process/memory counters change second to second, so the exact digits differ from PR 3's sample, but the truncation-not-wrap behavior is identical |

**0 of 2 are the "border loses its right corner" branch and 0 are overflow**; the box-drawing gate correctly suppressed every overflow-branch finding this pass (13 of PR 3's 15), leaving only the two word-split findings the spec explicitly says are unchanged.

### The key-hint change on `less`, `man`, and `top`

- **`man`: confirmed cleanly.** `man`'s bottom row is exactly `:` (the pager prompt) at all three scanned sizes (80x24, 120x40, 40x24), verified directly with `tmux capture-pane -p -e -J -t pass:man | tail`. `man` produced 0 `tui-rt-no-key-hints` findings across the full sweep (PR 3 had 1). This is a clean, direct confirmation of item 7's `is_pager_prompt` path (`rules.rs:168-169`, `text.trim_end() == ":"`).
- **`top`: unaffected, as expected.** `top`'s bottom row is still the scrolling process table, not a footer, and `tui-rt-no-key-hints` still fires once (same as PR 3). Confirmed the finding is real by capturing `top`'s current bottom row directly; it names no key.
- **`less`: cannot be cleanly tested here, same caveat PR 3 recorded.** This `less` window is not at end-of-file and was not run with `-M`, so unlike `man`, its bottom row was never a bare `:` prompt at any of the three sizes tested; re-verified directly this pass (bottom rows were ordinary wrapped `CLAUDE.md`/`README.md` prose ending mid-sentence at all three sizes). `less` produced 0 `tui-rt-no-key-hints` findings (PR 3 had 0 too, for the same underlying reason), so the count did not change, but tracing *why* it stayed silent shows two different, unrelated mechanisms rather than the item 7 pager-prompt path: at 40x24 the joined bottom row happens to contain the token `<command>` (from `` `<command>.md` `` in `README.md`), which matches `KEY_HINT_RE`'s angle-bracket clause (`<[^>\s]{1,12}>`) by coincidence, not because it names a real key; at 80x24 and 120x40 a follow-up manual re-check of the same window did not reproduce a key-hint or pager-prompt match against the captured text either, and `less`'s own scroll position is known to shift slightly across repeated resizes of the same window, so a manual re-check run minutes after the original detect scan is not guaranteed to see byte-identical content. Net: `less`'s absence of a finding is real and reproducible as a count, but this pass cannot say it demonstrates item 7's `:`-prompt exemption the way `man` does, because `less`'s content here is never actually that prompt.

### The largest-window-size restore check

Per Step 2's required check: `tmux set-option -w -t pass:less window-size largest`, then a scan (`--tmux-sizes 40x24`), then `tmux display -p -t pass:less '#{window_width}x#{window_height} #{window-size}'` read back `100x30 largest`, exactly the expected value (spec item 5: a non-empty `show-options` read is restored with `set-option -w -t <target> window-size <value>`, not `-u`). `pass:top`, never touched by this check, read `100x30 latest` at the same moment, confirming the check did not leak across windows. This directly exercises the restore-value half of item 5 that the corresponding `tests/tmux-engine.test.mjs` case also covers.

The plain restore check (engine scan only, no `largest` override) was also re-confirmed directly on `top`: `100x30 latest`, both immediately after the Step 2 sweep and again after the manual 40x24 word-split verification capture (resized manually, then restored with `resize-window -x 100 -y 30` + `set-option -w -t pass:top -u window-size`), matching PR 3's isolated-restore finding.

### Summary against PR 3's gate record

`tui-rt-collapse-narrow` dropped from 15 to 2 (87% reduction), and both survivors are the pre-existing word-split arguables the spec says item 1 leaves untouched, not new false positives. `tui-rt-no-key-hints` dropped from 3 to 2, with `man`'s drop directly attributable to and confirming item 7's pager-prompt exemption; `less`'s unchanged 0-count is coincidental rather than a confirmation of the same mechanism. `tui-rt-low-contrast` is unchanged (5 hits total, the same two programs as PR 3, `vim` 2 and `demo2` 3), and now carries the `paletteRelative` extra exactly as item 8 specifies: `false` on `demo2`'s three absolute-RGB findings, `true` on `vim`'s two named-ANSI-color findings, matching PR 3's manual judgment call ("vim's two use named ANSI colors... which is correct behavior... but means the finding is relative to that assumption") without needing a human to re-derive it by hand this time. All rules stay `advisory` in this PR, per the brief.

### Fixes made (PR 4 re-run)

None. This re-run's findings confirm PR 4's `crates/terminal` changes already landed correctly (the box-drawing gate, the pager-prompt exemption, and `paletteRelative`); no new bug was exposed on these five programs, so no `crates/terminal` source changed and no `detect-tmux-*` golden was re-recorded.

### Teardown (PR 4 re-run)

`tmux -L impeccable-rt-pass-pr4 kill-server`; the scratch socket no longer answers.
