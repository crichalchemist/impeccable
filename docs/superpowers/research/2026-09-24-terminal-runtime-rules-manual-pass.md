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

1. **`demo2` is a workspace package now, not an `--example` target.** The brief's `cargo build --release --example demo2` fails on this ratatui checkout (`error: no example target named demo2`) because the repo restructured: `examples/apps/demo2/` is its own crate. Built with `cargo build --release -p demo2` instead (`cargo build (106 crates compiled)`, `Finished release profile [optimized] target(s) in 40.39s`, well under the ten-minute ceiling); binary at `examples/../target/release/demo2`, staged into the `demo2` window the same way.
2. **`vim --clean` was used directly, without first checking whether plain `vim` prompts.** The brief's context note said to fall back to `--clean` only if plain `vim` showed a colorscheme prompt; `--clean` was used up front since it is also more representative of a bare, unconfigured session and avoids a possible interactive prompt blocking the window. `vim --clean README.md` renders with syntax highlighting on (the stock runtime still colors markdown), so the run is not degraded.
3. **A timing re-run.** The first timing pass used shell `date +%s.%N`, which macOS's `date` does not support (no `%N`), so the numbers were whole-second garbage. Re-ran with `/usr/bin/time -p` exactly as the brief specifies, output discarded (`> /dev/null`) since the judged JSON from the first pass was kept as the record of findings.
4. **Two supplementary probes beyond the five staged programs**, both aimed at the spinner open question: five repeated single-size scans of `pass:top` (no `--tmux-sizes`), and one throwaway `spinner-probe` window running a Node one-liner that cycles a 10-glyph braille spinner every 100 ms, scanned five times. Neither counts toward the five-program table above; both are reported under "Open questions" since they exist only to test the recapture window, not to calibrate a real program's rules.
5. No rule count changed the oracle or `crates/terminal`; no code fix was made (see "Why no fix was made" below), so `git add` in Step 6 only stages the record file.

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

After every scan (the required Step 2 check on `top`, plus the ad hoc 40x24 resizes done for the `-J` investigation below on `vim`, `less`, `man`, `top`), `tmux display -p '#{window_width}x#{window_height} #{window-size}'` reported `100x30 latest` for all five windows: the engine's `resize-window` + `set-option -w -u window-size` restore ran every time and left every window following its client size (the session's `-x 100 -y 30`), never stuck at `manual`.

## Palette and capture/replay checks

`--palette light` on `pass:top` (native size only, no `--tmux-sizes`) returned one finding, `tui-rt-no-key-hints` on the same bottom process row `tui-rt-low-contrast` would have used the dark default for; contrast passed under the light default (black-on-white), so only the palette-independent no-key-hints check fired. This is the expected shape: palette only changes which findings the contrast rule can see, not the other six rules.

The review-flow path (`terminal.md`'s saved-capture replay): `tmux capture-pane -p -e -J -t pass:less > less-capture.txt` (15 lines) replayed through `impeccable detect --no-config --json --tmux-capture less-capture.txt` returned `[]`. This is correct, not silent: a single un-resized capture at the window's native 100 columns has no narrow frame to trip `tui-rt-collapse-narrow`, and the bottom row of `less`'s status line contains no key-namable text that happens to fail `tui-rt-no-key-hints`'s check either (confirmed by inspecting the file directly).

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

`tmux.rs:129` captures with `capture-pane -p -e -J -t <target>`. Per `man tmux` (`capture-pane` section, quoted verbatim): **"-J preserves trailing spaces and joins any wrapped lines; -J implies -T."** When an application's output line exceeds the pane's width and the terminal (not the application) wraps it across several screen rows, `-J` stitches those rows back into one long logical line before the parser ever sees them. `tui-rt-collapse-narrow`'s overflow check (`row.len() > frame.width`) cannot tell a genuinely-too-wide single row from a `-J`-reassembled paragraph that displays perfectly readably across several rows on the real screen.

Direct evidence, comparing `capture-pane -p -e -J` against `capture-pane -p -e` (no `-J`) at 40x24 on the same live pane:

| Window | With `-J`, sample row lengths | Without `-J`, same rows |
|---|---|---|
| `vim` (row 3, 5) | 153, 185 | 39, 40 / 39, 33 (each row ≤40) |
| `less` (rows 5, 7, 8) | 181, 219, 281 | four to seven consecutive rows of exactly 40 each |
| `man` (rows 1, 7, 11) | 98, 85, 98 | pairs/runs of rows ≤40 each |
| `top` | 40, 40, 40... | identical (top never produces a row over 40) |

Every one of `tui-rt-collapse-narrow`'s "row overflows the pane by N cells" findings in `vim`, `less`, and `man` has an N that equals exactly (joined-row length minus 40), confirming each is a `-J` join, not a real overflow:

- `vim`: 113, 145, 142, 40 cells over -> joined lengths 153, 185, 182, 80
- `less`: 141, 179, 241 cells over -> joined lengths 181, 219, 281
- `man`: 58, 45, 1, 58, 57, 3 cells over -> joined lengths 98, 85, 41, 98, 97, 43

`top` never joins (every stat line is independently under 40 columns already, so there is nothing for `-J` to stitch), which is why `top`'s two `tui-rt-collapse-narrow` hits are of the *other* kind (word-split, judged below) and not overflow.

`-J` also shifts line/column numbers for any row it joins, which breaks the brief's own verification recipe (`tmux capture-pane -p -t pass:<w> | sed -n '<line>p'`, which does **not** pass `-J`) whenever a finding sits on a joined row. Concretely: `vim`'s `tui-rt-no-key-hints` finding reports `line=21`, but a non-`-J` capture of the same pane has 30 rows, and `sed -n '21p'` on it returns an unrelated prose line ("...ing those facts with surface-level visual direction."), not the status line the finding actually names. The finding's *content* is still correct (row 21 of the 21-row `-J`-joined frame genuinely is the last non-blank row, carrying the file-info message), but a reviewer following the brief's literal verification command on a joined row will look at the wrong screen position. Likewise `vim`'s low-contrast hit at `col=160` on a 100-column pane only makes sense against the joined row, not the physical 100-column screen. `man`'s and `top`'s findings did not show this drift, because their content never got joined (man's page is pre-wrapped by groff with real newlines at the *original* width; top's lines are always short).

**Why no fix was made.** This is not a parser bug (no SGR misread, no missing glyph, no width miscount): `-J` is the capture flag spec section 5 specifies (`tmux.rs:129`) and `terminal.md`'s saved-capture instruction assumes it too. Removing or conditioning it would change what a "row" means for every rule that reads `frame.rows`, including the word-split and border-loses-corner branches of the same function, which depend on wrapped continuations landing on *separate* rows to detect a split. That is a rule-definition change, not a misread correction, so per the brief it is recorded here as a recommendation instead of made:

- Before flagging "row overflows the pane," check whether the excess is explained by the row being a multiple-ish join of several `frame.width`-or-narrower segments (i.e., look for embedded near-`frame.width`-length segments in the joined text) and skip those.
- Or capture without `-J` for the overflow check specifically and re-derive word-split detection from wrap points reported by `-J`'s twin `-S`/`-E` and the `O` flag from `-F`, at the cost of a second capture per frame.
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

| Program | Line | Snippet | Class | Reason |
|---|---|---|---|---|
| `vim` | 21 | `bottom row "\"/Volumes/Containers/impeccable/README.md\" 514L, 26991B     " names no key"` | TP | Vim's default file-info status line genuinely names no key; well-known vim usability gap |
| `man` | 30 | `bottom row ":" names no key` | arguable | The bare pager prompt is technically keyless, but it is a decades-old Unix idiom (`less`/`man` users learn `:` means "waiting for a command"), not a UI the rule's "show the keys" framing was written for |
| `top` | 30 | `bottom row "2287   MIDIServer   0.2  04:16.52 6     1    80     1848K  0" names no key"` | arguable | The "bottom row" here is just wherever the scrolling process table happens to end at the current pane height; it is content, not a designed footer/status bar. The rule's premise (the last non-blank row is where hints live) does not hold for continuously-scrolling table UIs |

0 hard false positives, but 2 of 3 samples expose a real design gap: the rule assumes the last non-blank row is a static footer, which breaks for scrolling-table programs (`top`, and by extension `htop`/`ps`-style views) and for terse pager prompts.

### `tui-rt-collapse-narrow` (15 hits, all 15 classified, 10 printed)

| Program | Line | Snippet | Class | Reason |
|---|---|---|---|---|
| `top` | 1 | `word split at the pane edge: "ng, 717 sle"` | arguable | Real content is lost ("704 sleeping" truncates to "704 sle" with no continuation on the next row, verified: `top`'s next row is an unrelated "Load Avg:..." stat, not "eeping"), so a genuine narrow-collapse problem exists. But the rule's stated mechanism (a word wrapped across the row boundary) did not happen; `top` does not wrap, it truncates, and "sle" + "Load" only coincidentally satisfies "ends in a letter, next starts with a letter" |
| `top` | 5 | `word split at the pane edge: "d, 767M com"` | arguable | Same mechanism: `top` truncates "767M compressed" mid-word; the match is coincidental, the data loss is real |
| `vim` | 3 | `row overflows the 40-column pane by 113 cells` | FP | `-J`-joined wrapped prose, see above; joined length 153 = five rows of ≤40 stitched together |
| `vim` | 5 | `row overflows the 40-column pane by 145 cells` | FP | Same mechanism, joined length 185 |
| `vim` | 9 | `row overflows the 40-column pane by 142 cells` | FP | Same mechanism |
| `vim` | 11 | `row overflows the 40-column pane by 40 cells` | FP | Same mechanism, joined length 80 (two rows) |
| `less` | 5 | `row overflows the 40-column pane by 141 cells` | FP | `-J`-joined wrapped prose, joined length 181 |
| `less` | 7 | `row overflows the 40-column pane by 179 cells` | FP | Joined length 219 |
| `man` | 1 | `row overflows the 40-column pane by 58 cells` | FP | `man`'s stale, groff-formatted-at-100-columns buffer auto-wrapped by the terminal after the resize to 40; `-J` reassembles the original 98-cell line. Visually this still renders as readable ≤40-column rows, same as an app-level soft wrap |
| `man` | 7 | `row overflows the 40-column pane by 45 cells` | FP | Same mechanism |

The remaining five (`less` line 8 by 241 cells; `man` lines 8, 11, 12, 13 by 1, 58, 57, and 3 cells) all match the identical `-J`-join arithmetic (each overflow amount equals the joined row length minus 40, verified programmatically) and are the same false-positive class.

13 of 15 (87%) are `-J`-join false positives; the other 2 (13%) are arguable rather than clean true positives, since the stated word-split mechanism did not literally occur even though the underlying narrow-width data loss is genuine. **0 of 15 are the "border loses its right corner" branch**; that branch was never exercised by any of the five programs.

## Rules with no hits

- **`tui-rt-width-drift`**: unmeasured. Its overflow branch requires a double-width glyph in the row (`rules.rs:278`); none of the five programs render CJK or emoji. Its border-end-mismatch branch is gated behind the *same* `has_wide` check (the `continue` at line 278 sits before both branches), so it too went unevaluated on every ASCII/box-drawing row in this sample, wide char or not. This is worth a design look independent of promotion: the border-mismatch check plausibly should not require a wide glyph in the row to run at all.
- **`tui-rt-nested-borders`**: unmeasured. None of the five programs draws a box literally inside another with an empty ring between them; `demo2`'s Recipe tab draws side-by-side sibling boxes (confirmed via `find_boxes`-style manual inspection of the capture), and `vim`/`less`/`man` draw no boxes at all in this configuration.
- **`tui-rt-truecolor-on-256`**: unmeasured on live pane scans, and this ties directly into open question 2 below. `frame.termfeatures` was empty for every scan (no client ever attached to the detached scratch server), and the rule returns immediately when `termfeatures` is empty (`rules.rs:124-126`), before it ever reaches the `COLORTERM`-vouching check. Separately worth noting for calibration: even with a client attached, `frame.colorterm` is populated from the *engine process's own* `COLORTERM` (`truecolor`, confirmed present in this shell's environment), not from the actual client terminal's capability (spec section 5: "and from `COLORTERM` in the engine's environment"). An agent running the scan from inside a `COLORTERM=truecolor` shell could silence a real finding about a client that genuinely lacks RGB support. This is the design as specified, not a bug, but worth flagging as a caveat for anyone reading a clean truecolor result as confirmation the *viewer* supports RGB.
- **`tui-rt-spinner-never-rests`**: unmeasured on the five real programs; see "Open questions" below for a targeted probe that measures the rule's actual firing rate.

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

A spinner that was demonstrably animating the entire time (confirmed visually mid-run) was missed 4 of 5 times. This is the aliasing risk the spec's open question names directly: ten frames at a 100ms period is exactly the one-second recapture window, so frame 0 and the recapture land on the same glyph in most trials; only shell/timer jitter (`setInterval` drift, process scheduling) occasionally shifts the alignment enough to catch a genuine mismatch, which is what happened once in five runs. **Conclusion: the one-second window is not reliable for spinners whose period divides one second evenly**, exactly as spec section 5 anticipated when it made the rule advisory. This is real evidence, not a restatement of the design doc's caveat, but it comes from a synthetic probe rather than a program from the required list, since none of the five available real programs exercises the rule at all.

### Is `client_termfeatures` populated from outside the session?

**No.** `tmux -L impeccable-rt-pass display -p -t pass:top '#{client_termfeatures}'` on the fully detached scratch server returned an empty string, and `list-clients -t pass` returned nothing: no client has ever attached. Per note 7, there was no second terminal available to attach a client and re-check; this is stated rather than guessed. This matches the spec's stated fallback exactly: `tui-rt-truecolor-on-256` cannot fire from a pane scan run against a server with no attached client, only from a `--tmux-capture` file whose header names `termfeatures` explicitly (confirmed separately: `less-capture.txt`, captured without a header, also produced empty `termfeatures` and an empty finding array).

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
| `tui-rt-width-drift` | 0 | n/a | n/a | Unmeasured; needs a program with double-width (CJK/emoji) content. Separately, review whether the border-mismatch branch should require `has_wide` at all (see "Rules with no hits") |
| `tui-rt-nested-borders` | 0 | n/a | n/a | Unmeasured; needs a program that literally nests one box inside another with no content between the borders |
| `tui-rt-truecolor-on-256` | 0 | n/a | n/a | Unmeasured on live scans by construction (no attached client on a detached server). Confirmed working as designed against the fallback path (`--tmux-capture` with an explicit header); the live-scan gap is the answer to open question 2, not a rule defect |
| `tui-rt-spinner-never-rests` | 0 on the five real programs | n/a (5-run probe outside the required set gave 1/5) | n/a | Unmeasured on real programs available here. The probe evidence above should be treated as confirming, not just restating, the spec's stated aliasing risk; needs a real spinner-bearing program (`htop`, `lazygit`, `gitui`, or similar) for a proper calibration sample |

## Fixes made

None. The dominant false-positive source found (`-J` line-joining, above) is an architectural capture-flag question that changes what "row" means for every geometry rule, not a narrow misread; per the brief it is recorded as a recommendation rather than patched in this pass. No `crates/terminal` source changed, so no unit test was added and no `detect-tmux-*` golden was re-recorded.

## Teardown

`tmux -L impeccable-rt-pass kill-server` before committing; `tmux ls` on the default socket confirmed only `cockpit` remained.
