# Terminal UI design research (for `reference/terminal.md`)

Researched 2026-09-24. Every kept claim carries the URL it was read from. Where a claim comes from a secondary summary rather than a page I read, it is marked *(secondary)*. Pages I could not fetch (GNU Coding Standards returned 403 from every path tried) are not cited.

Scope note: the target file mirrors `ios.md` / `android.md`: a slop-test paragraph, then Layout & structure, Touch targets (here: Keyboard & input), Typography, Color & materials, Components & controls, Motion, Verifying the build. The "Distillation candidates" section at the end proposes that file's bullets and the anti-pattern table.

---

## 1. Terminal capability model

**The surface is a cell grid, not pixels.**
- Ratatui's coordinate system runs left to right, top to bottom, origin `(0, 0)` top left, `u16` columns and rows; a widget is placed into a `Rect`, either absolute or from `Layout` constraints. https://ratatui.rs/concepts/layout/
- Ratatui renders in immediate mode: the whole frame is rebuilt every draw from app state, unlike retained-mode GUI toolkits. https://ratatui.rs/concepts/rendering/
- A cell holds one character plus foreground, background and attributes; a wide glyph occupies two cells; the "boxes" in a TUI are Unicode characters painted into cells, not windows. https://gist.github.com/MangaD/cd8b8ab9b4f119ac5214fa4f3424ccd7 *(gist self-describes as ChatGPT-generated; used only for uncontroversial framing)*
- Font size is the user's, not the app's: "ratatui itself doesn't control the terminal's font size ... You typically will not know what the terminal font size is ahead of time." https://ratatui.rs/faq/

**Color depth: 16 / 256 / truecolor, and how it is detected.**
- Ratatui's `Color` enum is the canonical shape of the space: 16 named ANSI colors (`Black` ... `LightRed` ..., escape codes 30-37 / 90-97), `Indexed(u8)` for 256-color, `Rgb(u8,u8,u8)` for 24-bit, and `Reset`. "If the terminal does not support true color, code using the TermwizBackend will fallback to the default text color." https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html
- `supports-color` (the detector behind chalk): `FORCE_COLOR=1|2|3` sets level 1 (16), 2 (256), 3 (16m) and "overrides the detected color support and sets the level directly"; `FORCE_COLOR=0` disables. https://github.com/chalk/supports-color
- `COLORTERM=truecolor` / `24bit` is the de-facto truecolor signal; the Terminal Renaissance essay states the detection rule directly ("check `$COLORTERM` for `truecolor` or `24bit`. Respect `$NO_COLOR` unconditionally") and reminds that users may be on SSH with a 16-color connection. https://hyperbliss.tech/blog/2026.04.04_terminal-renaissance/
- Rich: `NO_COLOR` disables all color and "takes precedence over `FORCE_COLOR`"; `FORCE_COLOR` non-empty enables styles regardless of `TERM`; `TERM` of `dumb` or `unknown` "will disable color/style and some features that require moving the cursor, such as progress bars"; `TTY_COMPATIBLE=0/1` overrides TTY autodetection; `COLUMNS`/`LINES` set width and height. https://rich.readthedocs.io/en/stable/console.html
- Lip Gloss v1 detected the terminal color profile and coerced out-of-gamut colors to the nearest available value; `CompleteColor{TrueColor, ANSI256, ANSI}` lets you specify each tier explicitly. https://github.com/charmbracelet/lipgloss and https://github.com/charmbracelet/lipgloss/blob/main/UPGRADE_GUIDE_V2.md
- clig.dev lists `TERM`, `TERMINFO`, `TERMCAP` as the variables to consult "if you're going to use terminal-specific escape sequences", and `LINES`/`COLUMNS` "for output that's dependent on screen size (e.g. tables)". https://clig.dev/

**`NO_COLOR` and its relatives.**
- The standard: "Command-line software which adds ANSI color to its output by default should check for a `NO_COLOR` environment variable that, when present and not an empty string (regardless of its value), prevents the addition of ANSI color." Config files and CLI flags override it; it does not disable bold/underline/italic; software that is uncolored by default need not care. https://no-color.org/
- Clang's implementation discussion is a good worked example: value does not matter, empty means unset, `-fcolor-diagnostics` on the command line wins. https://reviews.llvm.org/D152285
- CMake documents the same convention and states `NO_COLOR` takes precedence over `CLICOLOR` / `CLICOLOR_FORCE`. https://cmake.org/cmake/help/latest/envvar/NO_COLOR.html
- util-linux honors `NO_COLOR` since 2.41 and also has a system-wide `terminal-colors.d/disable` file. https://man7.org/linux/man-pages/man5/terminal-colors.d.5.html
- terminfo.dev's summary: `NO_COLOR` is user-set, not terminal-set, and is consulted alongside `COLORTERM` and `TERM`. https://terminfo.dev/device/env-no-color
- clig.dev's full disable list: stdout/stderr not a TTY (check each separately), `NO_COLOR` set and non-empty, `TERM=dumb`, `--no-color`, optionally `MYAPP_NO_COLOR`. https://clig.dev/

**Unicode width: East Asian Width, grapheme clusters, emoji.**
- UAX #11 assigns each character one of six East_Asian_Width values (Ambiguous, Fullwidth, Halfwidth, Narrow, Wide, Neutral) which resolve to narrow or wide by context; Ambiguous characters "occur in East Asian legacy character sets as wide characters, but as narrow ... in non-East Asian usage" (Greek, Cyrillic, some math symbols). https://www.unicode.org/reports/tr11/
- Mitchell Hashimoto: paste "🧑‍🌾" and the cursor may advance 2, 4, 5 or 6 cells depending on the emulator; most terminals do not do grapheme clustering because shells and editors assume `wcwidth`, which is "consistently wrong" for multi-codepoint graphemes; Ghostty had to disable clustering because fish redrew its prompt in the wrong place; mode 2027 is the opt-in proposal for grapheme-aware terminals. https://mitchellh.com/writing/grapheme-clusters-in-terminals
- Jeff Quast (wcwidth / ucs-detect author): measured 23 different implementations of which codepoints are Wide, 19 of ZWJ sequences, 7 and 6 of VS15/VS16 widths; "the 'Terminal Unicode Width' problem cannot be solved by clear specification, implementation, and compliance reporting alone", so wcwidth now ships per-terminal correction tables. https://www.jeffquast.com/post/perfecting-terminal-character-width-using-correction-tables/
- `unicode-width` (Rust): string width is not the sum of char widths; well-formed emoji ZWJ sequences, modifier sequences and presentation sequences are width 2; text-presentation sequences width 1 outside East Asian context; `"\r\n"` is width 1. https://docs.rs/unicode-width/latest/unicode_width/
- Practitioner pain, HN thread on emoji in terminals: emoji width is "a crapshoot"; VS15/VS16 do not change East Asian Width but fonts render 1.5 or 2 cells; color emoji override the user's theme colors; LLM-style emoji spraying obscures the text it decorates; emoji is acceptable as a traffic-light indicator, and "I'd stick to nerd fonts if I want icons". https://news.ycombinator.com/item?id=44362272
- Blessed exposes the cost directly: `fullUnicode` is an opt-in that enables East Asian double-width, UTF-16 surrogate pairs and combining characters; `forceUnicode` overrides terminfo/env detection. https://github.com/chjj/blessed

**Box drawing and Nerd Font fallbacks.**
- Ratatui FAQ: box-drawing, braille and icon glyphs render as a white square or replacement character if the font lacks them; the recommended fix is a Nerd Font, and glyphs "may render a bit differently from font to font"; some terminals (Alacritty `builtin_box_drawing`, iTerm2) draw box characters with a built-in font. https://ratatui.rs/faq/
- Starship ships a "No Nerd Fonts" preset that "restricts the use of symbols to those from emoji and powerline sets", so users without a Nerd Font still see every module symbol. https://starship.rs/presets/no-nerd-font
- Lip Gloss offers `NormalBorder`, `RoundedBorder`, `ThickBorder`, `DoubleBorder`, `ASCIIBorder`, `MarkdownBorder` and per-side toggles, i.e. an ASCII fallback is one call away. https://github.com/charmbracelet/lipgloss

**Alt screen vs inline rendering.**
- Bubble Tea: `tea.WithAltScreen()` "starts the program with the alternate screen buffer enabled (i.e. the program starts in full window mode)" and exits it automatically on quit; `EnterAltScreen` can be issued later. https://pkg.go.dev/github.com/charmbracelet/bubbletea
- Textual: `run()` enters "application mode" (no echo, Textual owns keyboard and mouse, prior content restored on exit); `run(inline=True)` "will cause the app to appear beneath the prompt (and won't go into application mode)". The docs recommend the design-system colors for full-screen apps but suggest preserving ANSI colors for inline apps. https://textual.textualize.io/guide/app/
- How inline works: the app is drawn under the prompt and, instead of a final newline, an escape moves the cursor back to its prior position; the cursor can be hidden but the terminal still tracks it. https://textual.textualize.io/blog/2024/04/20/behind-the-curtain-of-inline-terminal-applications/
- prompt_toolkit: `Application(full_screen=True)` runs in the alternate screen; without it "the application would not run in the alternate screen buffer, and only consume the least amount of space required for the layout". https://python-prompt-toolkit.readthedocs.io/en/master/pages/full_screen_apps.html
- Ink 7 added an alternate-screen render mode (same buffer vim and less use) and an `interactive` option to override auto-detection for CI or pipes. https://www.heise.de/en/news/React-in-the-Terminal-Ink-7-0-fundamentally-revises-input-handling-11249949.html Ink's `<Static>` renders permanent output above the live region; re-entering the alt screen after a suspend gives "an empty buffer with no scrollback behind it" (issue #1011), which is the trade-off of the alt screen in one sentence. https://github.com/vadimdemedes/ink and https://github.com/vadimdemedes/ink/issues/1011
- tmux `capture-pane -a` captures the alternate screen "and the history is not accessible". https://man7.org/linux/man-pages/man1/tmux.1.html

**Resize (SIGWINCH).**
- ncurses installs its own `SIGWINCH` handler at `initscr`/`newterm` unless the application already set one, and calls `resizeterm` to update `LINES` and `COLS`; an app that owns the handler must call `resizeterm` itself or "proper layout will rely upon the application". https://invisible-island.net/ncurses/man/resizeterm.3x.html
- crossterm delivers `Event::Resize(columns, rows)` and warns "resize events can occur in batches"; it also has `FocusGained`/`FocusLost`, `Mouse`, `Paste`. https://docs.rs/crossterm/latest/crossterm/event/enum.Event.html
- Bubble Tea sends `WindowSizeMsg` once at start and on every resize, and notes "Windows does not have support for reporting when resizes occur as it does not support the SIGWINCH signal". https://pkg.go.dev/github.com/charmbracelet/bubbletea
- Blessed has a `resizeTimeout` (default 300 ms) before redrawing after a resize. https://github.com/chjj/blessed
- Ink: when the terminal is "resized narrower, ghost lines may briefly appear depending on the terminal emulator's reflow behavior"; in non-interactive mode `useWindowSize` and friends return no-op values and resize events are not listened to. https://github.com/vadimdemedes/ink

**Mouse is optional; there is no hover.**
- Bubble Tea: `WithMouseCellMotion` enables click, release, wheel and drag; it tries SGR extended mode and falls back to X10; mouse is disabled automatically on exit. Mouse is a program option, not a default. https://pkg.go.dev/github.com/charmbracelet/bubbletea
- Blessed: `hover` is "only set if mouse events are bound", i.e. hover exists only when the app has opted into mouse tracking. https://github.com/chjj/blessed
- Textual `run(mouse=True)` is a parameter; `size=(W, H)` can force a terminal size. https://textual.textualize.io/api/app/
- Terminal Renaissance: "Keyboard-first, mouse-optional means every feature is reachable without a mouse". https://hyperbliss.tech/blog/2026.04.04_terminal-renaissance/
- Ink: `isRawModeSupported` exists "to nicely fall back in environments where raw mode is not supported". https://github.com/vadimdemedes/ink

**Restoring the terminal.**
- Ratatui: a panic hook must disable raw mode and leave the alt screen, otherwise "the terminal [gets] stuck in a modified state". https://ratatui.rs/recipes/apps/panic-hooks/

---

## 2. Framework guidance and idiomatic components

**Ratatui (Rust).**
- Layout: `Layout::split` takes `Frame::area()` and divides by `Constraint::Length` (absolute, "not responsive to the overall terminal window size"), `Percentage`, `Ratio`, `Min`, `Max`, plus `Flex` alignment strategies. https://ratatui.rs/concepts/layout/
- Idiomatic widgets (named in the docs sidebar and gist): `Block` (borders/titles), `Paragraph`, `List`, `Table`, `Tabs`, `Gauge`, `Chart`, `Scrollbar`, `Canvas`, `Sparkline`, `Calendar`. https://ratatui.rs/concepts/layout/ (sidebar), https://gist.github.com/MangaD/cd8b8ab9b4f119ac5214fa4f3424ccd7
- Colors: prefer the 16 named ANSI variants when you want the user's theme; `Rgb` is truecolor-only. https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html
- Panic hooks and stdout/stderr choice are FAQ entries. https://ratatui.rs/recipes/apps/panic-hooks/ https://ratatui.rs/faq/

**Bubble Tea + Lip Gloss + Bubbles (Go, Charm).**
- Lip Gloss is "the layout engine": `Width`, `MaxHeight`, `Place(w, h, hPos, vPos, str)` for alignment, `JoinVertical`/`JoinHorizontal`, six border sets with per-side toggles, `Color("#…")` truecolor plus `AdaptiveColor{Light, Dark}` chosen at runtime by detected background, and `CompleteColor` for explicit degradation tiers. https://github.com/charmbracelet/lipgloss
- v2 makes background detection explicit and app-controlled: `lipgloss.HasDarkBackground(os.Stdin, os.Stdout)` for CLIs, or query the background in `Init` and listen for `BackgroundColorMsg` in Bubble Tea; then `lipgloss.LightDark(isDark)(light, dark)` picks a color. The global renderer and implicit detection are gone. https://github.com/charmbracelet/lipgloss/blob/main/UPGRADE_GUIDE_V2.md
- v2 announcement (Feb 2026): Bubble Tea is "the interaction layer", Lip Gloss "the layout engine", Bubbles "user interface primitives"; the v2 branches ran Crush in production first. https://charm.land/blog/v2/
- Bubbles components: text input, text area, paginator, file picker, stopwatch, list ("pagination, fuzzy filtering, auto-generated help, an activity spinner"), and a `help` bubble: "a customizable horizontal mini help view that automatically generates itself" from `key.Binding` definitions with `key.WithHelp("↑/k", "move up")`. https://github.com/charmbracelet/bubbles
- Program options: `WithAltScreen`, `WithMouseCellMotion`, `WindowSizeMsg`. https://pkg.go.dev/github.com/charmbracelet/bubbletea
- gum (shell glue for these primitives): `choose`, `input`, `write`, `filter`, `confirm`, `spin`, `file`, `table`, `pager`, `style` (`--border double`, `--padding "1 5"`). https://github.com/charmbracelet/gum
- glow: has both a TUI pager and a plain CLI; `-w` sets a wrap width; with no `-s` flag it "tries to detect your terminal's current background color and automatically picks either the dark or the light style". https://github.com/charmbracelet/glow

**Textual (Python).**
- Design system: themes are 11 base colors (`$primary` required, others generated); three lighten/darken shades each; `$text` "is set to a slightly transparent black or white, depending on which has better contrast against the background"; `$text-primary` etc. are "tinted to ensure legibility"; muted colors blend the base with `$background` at 70%; light and dark themes; `textual colors` previews the system. https://textual.textualize.io/guide/design/
- Why no ANSI themes by default: "Not everyone has a carefully chosen ANSI color theme. Color combinations which may look fine on your system, may be unreadable on another machine"; ANSI colors cannot be blended; the design system "guarantees apps will be readable on all platforms and terminals". `App.ansi_color=True` opts back in; recommended for inline apps. https://textual.textualize.io/FAQ/ and https://textual.textualize.io/guide/app/
- `Footer` "displays available keybindings for the currently focused widget" from `BINDINGS`, with `compact` and a command-palette hint. https://textual.textualize.io/widgets/footer/
- Widgets: Header, Footer, Input, Button, DataTable, Tree, TabbedContent, TextArea, ProgressBar, LoadingIndicator, Sparkline, Toast, Markdown viewer, RichLog. https://textual.textualize.io/widgets/footer/ (sidebar)
- Devtools: `textual run --dev` for live CSS reload and a debug console (because `print` would overwrite the screen). https://textual.textualize.io/guide/devtools/

**Ink (React for CLIs).**
- Yoga flexbox layout with CSS-like props; `<Static>` for permanent output; `useStdout`/`useStderr`/`useWindowSize`/`useFocus`; `isRawModeSupported`; non-interactive detection with no-op hooks; `exitOnCtrlC`; alt-screen mode and kitty keyboard auto-detection in 7.x. https://github.com/vadimdemedes/ink https://www.heise.de/en/news/React-in-the-Terminal-Ink-7-0-fundamentally-revises-input-handling-11249949.html

**Blessed (Node).**
- Reimplements ncurses by compiling terminfo/termcap; `smartCSR`/`fastCSR` scroll-region optimizations; `dockBorders` merges adjacent borders; `fullUnicode`; hover only with mouse bound; `resizeTimeout`. https://github.com/chjj/blessed

**OpenTUI (TypeScript, Zig core).**
- "You arrange boxes and text with flexbox", React and Solid renderers, selects/inputs/scroll boxes "with keyboard and mouse controls", plus `@opentui/keymap` for key-binding routing; used in production by OpenCode. https://github.com/sst/opentui

**Rich (Python).**
- `Console` handles detection (`is_terminal`, `color_system`, `NO_COLOR`, `FORCE_COLOR`, `TERM=dumb`, `COLUMNS`/`LINES`), and exports SVG via `console.export_svg(title=...)` when created with `record=True`. https://rich.readthedocs.io/en/stable/console.html https://github.com/Textualize/textual/blob/main/src/textual/_doc.py
- Markup syntax is `[bold red]...[/]` style tags, usable "virtually everywhere Rich would accept a string". https://rich.readthedocs.io/en/stable/markup.html https://github.com/textualize/rich

**prompt_toolkit.**
- An app is a layout (containers `HSplit`, `VSplit`, `FloatContainer`, `Window` + `UIControl`), a set of key bindings, and a style; the loop dispatches input to bindings. https://python-prompt-toolkit.readthedocs.io/en/master/pages/full_screen_apps.html

**ncurses idioms.**
- `initscr`/`newterm`, `SIGWINCH` handled by the library, `resizeterm`, `LINES`/`COLS`. https://invisible-island.net/ncurses/man/resizeterm.3x.html
- Shell-level dialog tools (`dialog`, `whiptail`, `fzf`, `gum`) cover pickers without a framework. https://gist.github.com/MangaD/cd8b8ab9b4f119ac5214fa4f3424ccd7

---

## 3. CLI/TUI style guides and accessibility

**clig.dev (Command Line Interface Guidelines).** https://clig.dev/
- Output: "Humans come first, machines second"; the heuristic for a human reader is whether the stream is a TTY; offer `--plain` when human formatting breaks `grep`/`awk`.
- Send output to stdout, messaging to stderr; zero exit on success.
- Color: use it intentionally ("if everything is a different color, then the color means nothing"); disable per the list above; "If `stdout` is not an interactive terminal, don't display any animations. This will stop progress bars turning into Christmas trees in CI log output."
- Symbols and emoji "where it makes things clearer" (yubikey-agent example); the Empathy section explicitly warns against "using lots of emoji".
- Robustness: "Responsive is more important than fast. Print something to the user in <100ms"; show progress; an animated component reassures that the program is alive.
- Interactivity: prompts only if stdin is a TTY; honor `--no-input`; "Let the user escape. Make it clear how to get out. (Don't do what vim does.)"
- Help: concise help when run bare, full on `-h`/`--help`; suggest next commands.
- Errors: put the most important information at the end; use red "intentionally and sparingly".

**12 Factor CLI Apps (Jeff Dickey, Heroku).** Original: https://medium.com/@jdxcode/12-factor-cli-apps-dd3c227a0e46 (403 to fetchers; read via mirror https://github.com/flameddd/blog/blob/master/2019-06-15%EF%BC%9AAt%20Heroku%2012%20Factor%20CLI%20Apps.md)
- stdout for output, stderr for messages; when piped, "make sure they don't contain ansi color codes".
- Prompt if stdin is a TTY, but "Never require a prompt" so scripts can override.
- Use colors/dimming to highlight, spinners and progress bars for long tasks ("Even just a spinner will give the impression the CLI is much faster than it is"); truncate table rows "that are going to spill over the current screen width".

**POSIX Utility Syntax Guidelines (XBD 12.2).** https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap12.html
- Guidelines 3-14 govern option syntax (single-char options, `-` delimiter, `--` ends options, options precede operands). Presentation is out of scope for POSIX; only the stdout/stderr split above is convention. (GNU Coding Standards on `--help`/`--version` and diagnostics could not be fetched: HTTP 403.)

**Charm's design writing.**
- The v2 post frames the stack as interaction layer / layout engine / primitives and the design goal as "very predictable output". https://charm.land/blog/v2/
- Lip Gloss README and v2 guide are where the color guidance lives: adaptive light/dark, explicit degradation tiers, borders as a system. https://github.com/charmbracelet/lipgloss https://github.com/charmbracelet/lipgloss/blob/main/UPGRADE_GUIDE_V2.md

**"Terminal design" essays.**
- Terminal Renaissance (Hyperbliss, Apr 2026): "Designing for 80 Columns: ... every cell matters in a way that pixels don't"; layout archetypes (Miller columns as in k9s, widget dashboard grid, IDE three-panel: navigate left, work center, inspect right); keyboard-first, mouse-optional; three-tier help (footer with 3-5 keys always visible, `?` overlay, docs); "Color in a terminal is a resource, not a paintbrush"; semantic tokens over raw hex ("focused panel borders use accent"); palette → tokens → styles → gradients; contract tests for themes; status bar reflects state; help shows only shortcuts available in context. https://hyperbliss.tech/blog/2026.04.04_terminal-renaissance/
- Evil Martians on progress: spinner, "X of Y", progress bar; tick the spinner per unit of work so a stuck loop is visible; clear spinners on completion; redirect stdout to a file to inspect what you leave behind; switch "-ing" messages to "-ed" when done. https://evilmartians.com/chronicles/cli-ux-best-practices-3-patterns-for-improving-progress-displays
- Textual's design guide and FAQ are the most explicit framework-level statement that terminal palettes are untrustworthy and contrast must be computed. https://textual.textualize.io/guide/design/ https://textual.textualize.io/FAQ/

**Screen readers and full-screen TUIs.**
- "The text mode lie" (xogium, Jan 2026): the CLI stream (append below, cursor moves down) is "ideal" for kernel-level readers like Speakup; Ink, Bubble Tea and tcell apps "are actively destroying the experience for blind users" because every update redraws, the cursor "is teleporting all over the screen to update status indicators, spinners, and history", Speakup reads whatever is under the cursor at that millisecond, and frameworks "rarely offer a 'no-cursor' or 'headless' mode". Irssi is held up as the accessible gold standard because it uses VT100 scrolling regions and protects the input line. https://xogium.me/the-text-mode-lie-why-modern-tuis-are-a-nightmare-for-accessibility (syndicated with discussion at https://www.osnews.com/story/144892/the-text-mode-lie-why-modern-tuis-are-a-nightmare-for-accessibility/)
- Hermes agent issue (Sep 2026): "the root cause is that its status chrome repaints continuously. Every repaint of a terminal line is an update event for the screen reader, which re-homes the review cursor at the newest output"; the quietest indicator still repaints ten times per second plus a 1 s clock; request is a static plain-text mode with no tick. https://github.com/NousResearch/hermes-agent/issues/120584
- OpenCode issue: "actively hostile to any screen readers"; asks for an accessibility mode where the spinner becomes text that is "not constantly updating", because each update makes "the screen reader's review focus jump to the bottom of the window". https://github.com/anomalyco/opencode/issues/8565
- Textual's stated intent: color blending "will also be used to power future accessibility features". https://textual.textualize.io/FAQ/
- The recommendation that falls out of these: ship a `--plain` / line mode (clig.dev), suppress animation off-TTY (clig.dev), and offer a static-status flag for interactive use (Hermes, OpenCode issues).

---

## 4. Recognizable TUI anti-patterns

Evidence comes from three kinds of source: guidelines that prohibit the pattern, complaints from users who hit it, and (most telling) AI "TUI design skills" and generated specs that *prescribe* it. The last group shows what generated TUIs converge on.

- **Rainbow / gradient titles.** The Terminal Renaissance essay names ChromaCat ("turns any terminal output into animated gradient art") as the emblem of the aesthetic. https://hyperbliss.tech/blog/2026.04.04_terminal-renaissance/ Web-side AI-pattern lists ban "Rainbow gradient text: screams 'AI made this'". https://github.com/iliaal/whetstone/blob/master/plugins/whetstone/skills/ia-frontend-design/references/banned-ai-patterns.md The AI-rainbow mechanism, "reaching for a new hue every time something needs emphasis", is described at https://dev.to/kiwibreaksme/why-ai-generated-uis-look-off-and-the-one-principle-that-fixes-it-4j20. A circulating "terminal-ui-design" skill tells the model to use "Gradient Fills" and "Block ASCII (figlet-style)" headers. https://playbooks.com/skills/eddiebe147/claude-settings/terminal-ui-design
  Detection: static, high. Imports of `gradient-string`, `ink-gradient`, `<Gradient`, `rich_gradient`/`Gradient(`, `lolcat`, `chromacat`, `tui-gradient`, `lipgloss.Blend`/`Blend1D` applied to a title string.
- **Emoji as icons everywhere.** clig.dev warns against "lots of emoji"; HN thread: LLM emoji spraying "obscure[s] the thing being nominally highlighted", emoji widths are unreliable, and color emoji override theme colors. A generated CLI spec mandates "Emoji as type indicators" on every list item plus a 14-row emoji table. https://clig.dev/ https://news.ycombinator.com/item?id=44362272 https://github.com/alimtvnetwork/gitmap-v19/blob/main/spec/04-generic-cli/20-terminal-output-design.md
  Detection: static, medium-high by density. Count codepoints in U+1F300-1FAFF and U+2600-27BF (with or without U+FE0F) inside string literals; flag files above a threshold or the canonical set 🚀 ✅ ❌ ⚠️ ✨ 🎉 📦 🔧 🔥 💡.
- **Heavy / double-line boxes, nested boxes.** The generated spec above prescribes a fixed 40-column double-line (`╔═╗`) cyan banner; the AI skill pages list Double as "Bold, formal, retro" and recommend "Mixed styles for hierarchy (heavy for primary, light for secondary)" and "Decorative corners: ◆ ◈ ✦ ⬡". Textual and Lip Gloss offer Double as one option among several; nothing in any framework guide recommends nesting bordered containers. https://github.com/alimtvnetwork/gitmap-v19/blob/main/spec/04-generic-cli/20-terminal-output-design.md https://playbooks.com/skills/ingpoc/skills/terminal-ui-design https://github.com/charmbracelet/lipgloss
  Detection: `DoubleBorder()`, `BorderType::Double`, `borderStyle="double"`, `box.DOUBLE`, `border: double`, `--border double` is static and exact but only medium confidence alone; nesting (a bordered `Block`/`Box`/`Style` rendered inside another) needs light structural analysis, medium.
- **Spinners and animation that never stop / run off-TTY.** clig.dev: no animations when stdout is not a TTY. Lefthook issue: a spinner ticking every 100 ms emitted "300+ lines into the log stream" for a 30 s suite because the isatty check "only guards width, not the spinner". Hermes/OpenCode: a spinner with no static setting defeats screen readers. Evil Martians: clear the spinner when done, tick per completed unit. https://clig.dev/ https://github.com/evilmartians/lefthook/issues/1539 https://github.com/NousResearch/hermes-agent/issues/120584 https://evilmartians.com/chronicles/cli-ux-best-practices-3-patterns-for-improving-progress-displays
  Detection: static, medium. `setInterval(` with no `clearInterval` in the same module; `ink-spinner`/`ora`/`gum spin`/`rich.spinner`/`bubbles/spinner` with no `isatty`/`isTTY`/`is_terminal`/`IsTerminal`/`--no-progress` in the repo; a `SLOW_BLINK`/`RAPID_BLINK`/`blink` modifier anywhere (very high signal, almost never legitimate).
- **Hard-coded 80x24 or absolute coordinates.** Ratatui: `Constraint::Length` is "absolute size and is not responsive to the overall terminal window size", `Frame::area()` is the input to layout; Bubble Tea and Textual deliver size at start and on resize; test harnesses default to 80x24 precisely because real terminals vary. https://ratatui.rs/concepts/layout/ https://pkg.go.dev/github.com/charmbracelet/bubbletea https://textual.textualize.io/guide/testing/
  Detection: static, medium. `Rect::new(` with literal args in a render fn; a `Layout` whose constraints are all `Length`; literals `80`/`24`/`120` assigned to `width`/`columns`/`cols`/`rows`/`height`; `.Width(80)`; `<Box width={80}`; `size=(80, 24)` passed to `run()` outside tests; curses `mvaddstr(y, x` / `move(y, x` with literals; `stdout.cursorTo(` literals.
- **Hard-coded RGB that ignores theme and `NO_COLOR`.** Textual: fixed combinations "may be unreadable on another machine"; Lip Gloss and glow exist to detect background and adapt; Rich and clig.dev define the `NO_COLOR`/`TERM=dumb` contract. https://textual.textualize.io/FAQ/ https://github.com/charmbracelet/lipgloss/blob/main/UPGRADE_GUIDE_V2.md https://github.com/charmbracelet/glow https://rich.readthedocs.io/en/stable/console.html https://no-color.org/
  Detection: static, high when combined. `Color::Rgb(`, `Color::from_u32(0x`, `lipgloss.Color("#`, `chalk.hex(`, `color="#`, `[#rrggbb]` Rich markup, raw `\x1b[38;2;` escapes; strong signal when the repo contains none of `AdaptiveColor`, `LightDark`, `HasDarkBackground`, `ansi_color`, `NO_COLOR`, `isatty`. Textual CSS hex is a weaker signal because its design system computes contrast.
- **Unreadable on light terminals / green-on-black hacker theme.** 21st.dev: "The traditional green-on-black has poor contrast at small sizes, and a terminal dropped into a light theme with its colours unchanged is unreadable." Sailop flags `text-green-400 on bg-gray-900` as the AI terminal stereotype (web mockups, but the same instinct drives TUI palettes). https://21st.dev/blog/terminal-ui-components https://sailop.com/blog/terminal-mockup-problem-why-3-dots-ruin-your-design
  Detection: static, medium. `#00ff00`/`#0f0`/`Color::Green` paired with `Color::Black`/`#000` background; only `Dark` set in `AdaptiveColor` (or `Light` equal to `Dark`); no light theme in a Textual theme list.
- **No keyboard hints / help footer.** Textual `Footer` and Bubbles `help` exist so bindings are always on screen; Terminal Renaissance's three-tier rule; clig.dev "Let the user escape". https://textual.textualize.io/widgets/footer/ https://github.com/charmbracelet/bubbles https://hyperbliss.tech/blog/2026.04.04_terminal-renaissance/ https://clig.dev/
  Detection: absence-based, medium. Key handling present (`KeyCode::`, `tea.KeyMsg`, `useInput(`, `on_key`, `@kb.add`) but no `Footer`, `BINDINGS`, `help.Model`, `key.WithHelp`, `<Text>` containing `q` / `quit` / `?` / `esc`.
- **Mouse-only interactions.** Mouse is opt-in in every framework and unavailable over some connections; keyboard-first is the stated norm. https://pkg.go.dev/github.com/charmbracelet/bubbletea https://hyperbliss.tech/blog/2026.04.04_terminal-renaissance/
  Detection: needs semantics; low-medium. `EnableMouseCapture`/`WithMouseCellMotion`/`mouse_support=True`/`enableMouse()` plus click handlers with no key handler for the same action.
- **Truncation that breaks graphemes or columns.** Widths are not codepoint counts; slicing bytes or chars mid-cluster is exactly the failure Hashimoto and Quast describe; AI CLI digests in 2026 report "surrogate-pair truncation breaks emoji rendering" as a recurring bug class *(secondary; digest summary)*. https://mitchellh.com/writing/grapheme-clusters-in-terminals https://www.jeffquast.com/post/perfecting-terminal-character-width-using-correction-tables/ https://docs.rs/unicode-width/latest/unicode_width/
  Detection: static, medium-high in Go/Rust/JS, low in Rich/Textual/Ink (they measure for you). `&s[..n]`, `.chars().take(n)`, Go `s[:n]` on display strings, JS `.slice(0, n)`/`.substring(0, n)` followed by `"…"`, Python `s[:n]` + `"..."`; strong when the repo lacks `unicode-width`/`unicode-segmentation`/`go-runewidth`/`uniseg`/`ansi.Truncate`/`string-width`/`wcwidth`.
- **ASCII-art banners on launch.** clig.dev's help guidance (concise help when run bare) and 12-factor's stdout discipline leave no room for a figlet block; the AI skills prescribe "ASCII art headers: Dramatic intro, `figlet` style banners". https://clig.dev/ https://playbooks.com/skills/ingpoc/skills/terminal-ui-design
  Detection: static, high. `figlet`, `pyfiglet`, `figlet_rs`/`figlet-rs`, `go-figure`, `cfonts`, `ink-big-text`, `text2art`/`from art import`, `toilet`; or 4+ consecutive string literals composed mostly of `█ ▀ ▄ ╔ ═ ╗ ║ ╚ ╝`.
- **Centered everything.** Layout archetypes in practice are left-anchored (Miller columns, three-panel); the AI skills push "Asymmetry: off-center titles" one line and "centered" banners the next. https://hyperbliss.tech/blog/2026.04.04_terminal-renaissance/ https://playbooks.com/skills/ingpoc/skills/terminal-ui-design
  Detection: static, low-medium. Density of `Alignment::Center`, `.Align(lipgloss.Center)`, `Place(..., Center, Center`, `justifyContent="center"` + `alignItems="center"`, `align: center middle`, `text-align: center`.
- **Clearing the screen or taking the alt screen for a one-shot command.** Alt screen throws away scrollback (Ink #1011, tmux `-a`); inline is the Textual/prompt_toolkit default for tools that "integrate closely with the typical workflow of a terminal". https://github.com/vadimdemedes/ink/issues/1011 https://textual.textualize.io/guide/app/
  Detection: static, medium. `console.clear()`, `os.system("clear")`, `Clear(ClearType::All)`, `\x1b[2J`, `WithAltScreen()` / `alternateScreen: true` / `full_screen=True` in a program that also prints results to stdout.
- **Raw escape sequences in application code.** Every framework above exists to avoid this; raw `\x1b[` literals bypass NO_COLOR/TTY handling unless the author re-implements it. https://no-color.org/ https://rich.readthedocs.io/en/stable/console.html
  Detection: static, high detectability, medium slop confidence. `\x1b[`, `\033[`, `\u001b[`, `\e[` inside string literals in app code (not in a color library).
- **Nerd Font glyphs with no fallback.** Ratatui FAQ and Starship both treat Nerd Font as an opt-in tier. https://ratatui.rs/faq/ https://starship.rs/presets/no-nerd-font
  Detection: static, high detectability. Literals in U+E000-F8FF or U+F0000-FFFFD (private use) with no `--ascii`/`nerd`/`icons` toggle in the repo.
- **Triple-encoding state (emoji + color + word).** The generated spec's "✓ green + (status) dim + counter dim" scheme is the canonical example; clig.dev's "if everything is a different color, the color means nothing". https://github.com/alimtvnetwork/gitmap-v19/blob/main/spec/04-generic-cli/20-terminal-output-design.md https://clig.dev/
  Detection: low; needs judgment.
- **Adjacent (web, not TUI): fake terminal mockups with three macOS dots.** Sailop reports it on ~40% of AI dev-tool landing pages and ships `component-terminal-mockup`; 21st.dev: typed-out commands are not selectable. Out of scope for `terminal.md`, in scope for the web audit. https://sailop.com/blog/terminal-mockup-problem-why-3-dots-ruin-your-design https://21st.dev/blog/terminal-ui-components

---

## 5. Verifying a TUI build

**Capture methods that give review evidence.**
- tmux: `capture-pane -p` prints the pane to stdout; `-e` keeps escape sequences (colors); `-a` captures the alternate screen (history not accessible); `-S`/`-E` select line ranges; `-J` joins wrapped lines. https://man7.org/linux/man-pages/man1/tmux.1.html
- `script(1)`: "makes a typescript of everything on your terminal session", raw output plus an optional timing log for `scriptreplay`. https://man7.org/linux/man-pages/man1/script.1.html
- asciinema: `asciinema rec demo.cast` records to the asciicast format; replay in a terminal or embed the player; `agg` converts to GIF (docs sidebar). https://docs.asciinema.org/manual/cli/
- VHS (Charm): `.tape` files with `Set Width`/`Set Height` (pixels) or `Set Columns`/`Set Rows` (cells), `Set FontSize`, `Set FontFamily`, `Set Shell`, `Type "..."`, `Sleep`, `Require` (must be at the top), `Output demo.gif` or `Output frames/` "a directory of frames as a PNG sequence". Settings go at the top of the tape. https://github.com/charmbracelet/vhs
- Textual: `App.export_screenshot()` returns SVG, `save_screenshot` writes it, `action_screenshot` is bindable; `run(headless=True, size=(cols, rows), auto_pilot=...)` drives it; `textual run --screenshot DELAY` (with `--screenshot-path`/`--screenshot-filename`) is in textual-dev's CLI. https://textual.textualize.io/api/app/ https://github.com/Textualize/textual-dev/blob/main/src/textual_dev/cli.py
- Textual snapshot testing: `run_test()` runs headless with a `Pilot` for key/mouse; default simulated size is (80, 24), overridable with `size=`; `pytest-textual-snapshot` `snap_compare(app, terminal_size=(w, h), press=[...], run_before=...)` generates an SVG and fails on the first run until `--snapshot-update` accepts it. https://textual.textualize.io/guide/testing/ Textual's own suite exercises `(120, 40)`, `(100, 50)`, `(20, 5)`, `(30, 5)`, `pilot.resize_terminal(35, 20)` then `(20, 10)`, and parametrizes `textual-dark`/`textual-light` for ANSI-mapping snapshots. https://github.com/Textualize/textual/blob/main/tests/snapshot_tests/test_snapshots.py The docs screenshotter defaults to 80 columns x 24 lines. https://github.com/Textualize/textual/blob/main/src/textual/_doc.py
- Rich: `Console(record=True, force_terminal=True, color_system="truecolor", width=80, height=24)` then `console.export_svg(title=...)`. https://github.com/Textualize/textual/blob/main/src/textual/_doc.py https://rich.readthedocs.io/en/stable/console.html
- Ratatui: `TestBackend::new(width, height)` "is intended for integration tests that test the entire terminal UI"; `assert_buffer`, `assert_buffer_lines`, `assert_cursor_position`, `resize`, scrollback assertions; the docs prefer unit-testing widgets against a `Buffer` directly. https://docs.rs/ratatui/latest/ratatui/backend/struct.TestBackend.html With insta: `assert_snapshot!(terminal.backend())` after `terminal.draw`, snapshots land in `snapshots/`, review with `cargo insta`; use a consistent size (the recipe uses 80x20). https://ratatui.rs/recipes/testing/snapshots/
- Ink: `ink-testing-library`'s `render()` gives `lastFrame()`, `frames` (all frames, last is current), `rerender`, `stdin.write`, and a stderr twin. https://github.com/vadimdemedes/ink-testing-library
- Off-TTY check (cheap and revealing): redirect stdout to a file and read it; Evil Martians recommends exactly this, and the lefthook issue shows what a spinner leaves behind when nobody did. https://evilmartians.com/chronicles/cli-ux-best-practices-3-patterns-for-improving-progress-displays https://github.com/evilmartians/lefthook/issues/1539

**Size and environment matrix worth running.**
- 80x24 as the floor (Textual `run_test`, Textual `_doc.py` and VHS all default near it; Terminal Renaissance argues for designing at 80 columns). https://textual.textualize.io/guide/testing/ https://hyperbliss.tech/blog/2026.04.04_terminal-renaissance/
- 120x40 and 100x50 as the "wide" cases, 20-35 columns as the "narrow" cases, plus a live resize step (Textual suite). https://github.com/Textualize/textual/blob/main/tests/snapshot_tests/test_snapshots.py
- Light and dark theme (Textual parametrizes both; Lip Gloss/glow detect background, so both branches need a look). https://github.com/Textualize/textual/blob/main/tests/snapshot_tests/test_snapshots.py https://github.com/charmbracelet/glow
- `NO_COLOR=1`, `TERM=dumb`, `FORCE_COLOR=0`, and piped stdout (`| cat`). https://no-color.org/ https://rich.readthedocs.io/en/stable/console.html https://clig.dev/
- 16-color only (`TERM=xterm`, no `COLORTERM`) to see the degradation tier. https://github.com/chalk/supports-color https://hyperbliss.tech/blog/2026.04.04_terminal-renaissance/
- A terminal without a Nerd Font, and CJK / emoji content in list items. https://ratatui.rs/faq/ https://mitchellh.com/writing/grapheme-clusters-in-terminals

---

## Distillation candidates

### (a) Proposed `reference/terminal.md` (sections and ~25 bullets, iOS-file style)

**Slop test.** A terminal surface reads as generated when it opens with a figlet banner in a gradient, wraps every region in a double-line box nested inside another box, puts an emoji in front of every line, hard-codes `#RRGGBB` colors that vanish on a light theme, spins a braille spinner that never stops (and keeps spinning into CI logs), assumes 80x24 and centers everything in it, and gives the user no clue which key quits. The tell is decoration with no information behind it. A terminal that has been designed uses the user's 16 colors first, earns each glyph, keeps the footer honest about the keys that work right now, and looks the same in a pipe as it does on screen, minus the color.

**Layout & structure**
- **Design at 80 columns, lay out by constraint.** Compute every region from the current size (`Frame::area()`, `WindowSizeMsg`, `useWindowSize`) with percentage/min/fill constraints; a fixed `Length` or a literal `Rect` is an absolute size that does not respond to the window. https://ratatui.rs/concepts/layout/
- **Pick the screen mode the task deserves.** Alternate screen for a resident app (editor, monitor), inline under the prompt for a tool the user runs and returns from; the alt screen throws away scrollback and history. https://textual.textualize.io/guide/app/ https://github.com/vadimdemedes/ink/issues/1011
- **Handle resize as a first-class event.** Resizes arrive in batches and the first size arrives at boot; re-run layout on each and never cache a width. https://docs.rs/crossterm/latest/crossterm/event/enum.Event.html https://pkg.go.dev/github.com/charmbracelet/bubbletea
- **One archetype per app.** Miller columns for drill-down, a widget grid for monitoring, three-panel (navigate / work / inspect) for editing; do not mix them on one screen. https://hyperbliss.tech/blog/2026.04.04_terminal-renaissance/
- **Borders are structure, not decoration.** One border weight per level, single-line by default, no box inside a box unless the inner one is focusable; the ASCII border set is the fallback, not an afterthought. https://github.com/charmbracelet/lipgloss https://ratatui.rs/faq/
- **Leave the terminal as you found it.** Raw mode and the alt screen must be released on every exit path, including panic. https://ratatui.rs/recipes/apps/panic-hooks/

**Keyboard & input** (the touch-target equivalent)
- **Keyboard first, mouse optional.** Every action has a key; mouse tracking is an opt-in that some connections cannot deliver, and there is no hover unless the mouse is bound. https://pkg.go.dev/github.com/charmbracelet/bubbletea https://github.com/chjj/blessed
- **The footer shows the 3-5 keys that work now.** Full reference behind `?`; the footer changes with focus. https://textual.textualize.io/widgets/footer/ https://github.com/charmbracelet/bubbles https://hyperbliss.tech/blog/2026.04.04_terminal-renaissance/
- **Make escape obvious.** `q`, `Esc` or `Ctrl-C` must always work, and the footer says which. https://clig.dev/
- **Prompt only on a TTY, and never require the prompt.** Off-TTY, fail with the flag to pass instead. https://clig.dev/ https://medium.com/@jdxcode/12-factor-cli-apps-dd3c227a0e46

**Typography**
- **Monospace is the only face; weight, dim and reverse are the whole type scale.** Hierarchy comes from bold, dim, spacing and position, not from size. https://textual.textualize.io/guide/design/
- **Measure in cells, not characters.** Width comes from a width library (`unicode-width`, `go-runewidth`, `wcwidth`, `string-width`); CJK is two cells, combining marks zero, emoji sequences two, and terminals disagree at the edges. https://docs.rs/unicode-width/latest/unicode_width/ https://www.jeffquast.com/post/perfecting-terminal-character-width-using-correction-tables/
- **Truncate on grapheme boundaries with an ellipsis, never on bytes.** A cut inside a ZWJ sequence or a surrogate pair corrupts the line and everything after it. https://mitchellh.com/writing/grapheme-clusters-in-terminals
- **No figlet banners, no block-letter titles.** Concise help when run bare, the tool's name once in the header, nothing that scrolls the first screen away. https://clig.dev/
- **Glyphs need a fallback tier.** Box drawing and braille assume a capable font; Nerd Font icons are an opt-in set with an ASCII/emoji tier behind them. https://ratatui.rs/faq/ https://starship.rs/presets/no-nerd-font
- **Emoji only as an indicator, never as a bullet.** Emoji widths vary by terminal, color emoji ignore the theme, and a row of them hides the text it decorates. https://news.ycombinator.com/item?id=44362272 https://clig.dev/

**Color & materials**
- **The user's 16 ANSI colors are the palette; truecolor is an enhancement.** Named colors follow the terminal theme; RGB literals do not, and degrade to default text on terminals without 24-bit support. https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html
- **Every RGB choice needs a light-background twin.** Detect the background and pick (`AdaptiveColor`, `LightDark`, `HasDarkBackground`, a light theme), or compute contrast the way Textual does. https://github.com/charmbracelet/lipgloss/blob/main/UPGRADE_GUIDE_V2.md https://textual.textualize.io/guide/design/ https://textual.textualize.io/FAQ/
- **Honor the disable contract.** `NO_COLOR` non-empty, `TERM=dumb`, `--no-color`, and a non-TTY stream each strip color; `FORCE_COLOR` re-enables it, `NO_COLOR` wins over `FORCE_COLOR`. https://no-color.org/ https://rich.readthedocs.io/en/stable/console.html https://clig.dev/
- **Color is a resource: one accent, semantic red/yellow/green, dim for secondary.** If everything is a color, nothing is. https://clig.dev/ https://hyperbliss.tech/blog/2026.04.04_terminal-renaissance/
- **Name colors by role, not by hex.** `accent`, `muted`, `error`, `focused-border` resolved per theme, so a theme swap is one table. https://hyperbliss.tech/blog/2026.04.04_terminal-renaissance/ https://textual.textualize.io/guide/design/
- **Green on black is a costume.** It fails on light themes and at small sizes; use the theme's foreground. https://21st.dev/blog/terminal-ui-components

**Components & controls**
- **Use the framework's primitives, not hand-rolled ones.** List/table/input/text area/file picker/pager/spinner/progress/help exist in Bubbles, Textual, Rich, Ink and gum; a custom one is a maintenance debt that usually breaks on width or resize. https://github.com/charmbracelet/bubbles https://textual.textualize.io/widgets/footer/ https://github.com/charmbracelet/gum
- **stdout is data, stderr is chrome.** Results go to stdout in a form `grep` can read; status, spinners and prompts go to stderr; offer `--plain` or `--json` when the pretty form would break a pipe. https://clig.dev/ https://medium.com/@jdxcode/12-factor-cli-apps-dd3c227a0e46
- **Status bar reflects state, help reflects context.** Mode, path, counts in one line; shortcuts listed only if they work here. https://hyperbliss.tech/blog/2026.04.04_terminal-renaissance/

**Motion**
- **Spinners tick on work and stop on completion.** Tick per unit processed so a hang is visible; replace the spinner with a past-tense line when done. https://evilmartians.com/chronicles/cli-ux-best-practices-3-patterns-for-improving-progress-displays
- **No animation off-TTY, and a static mode on-TTY.** A spinner in a pipe becomes hundreds of log lines; a repainting status rule re-homes a screen reader's cursor ten times a second. Offer `--no-progress` / a static indicator. https://clig.dev/ https://github.com/evilmartians/lefthook/issues/1539 https://github.com/NousResearch/hermes-agent/issues/120584
- **Print something within 100 ms; never blink.** Show the first frame before the network call; the blink attribute has no legitimate use. https://clig.dev/
- **Offer a line mode.** The append-only stream is what screen readers handle; a full-screen TUI should have a `--plain` twin or a static-chrome flag. https://xogium.me/the-text-mode-lie-why-modern-tuis-are-a-nightmare-for-accessibility https://github.com/anomalyco/opencode/issues/8565

**Verifying the build**
- **Capture, do not describe.** `tmux capture-pane -pe`, `script`, `asciinema rec`, a VHS tape with `Set Columns`/`Set Rows`, Textual `--screenshot`/`snap_compare`, Ratatui `TestBackend` + insta, Ink `lastFrame()`. https://man7.org/linux/man-pages/man1/tmux.1.html https://github.com/charmbracelet/vhs https://textual.textualize.io/guide/testing/ https://ratatui.rs/recipes/testing/snapshots/ https://github.com/vadimdemedes/ink-testing-library
- **Run the matrix.** 80x24, 120x40, ~30 columns, a live resize, dark and light theme, `NO_COLOR=1`, `TERM=dumb`, `| cat`, no Nerd Font, CJK and emoji rows. https://github.com/Textualize/textual/blob/main/tests/snapshot_tests/test_snapshots.py https://no-color.org/ https://ratatui.rs/faq/

### (b) Anti-pattern candidates for mechanical detection

Confidence = probability that a hit is generated slop rather than a legitimate choice, judged from the sources above. "Static" means a regex-class rule over source text; "runtime" means it needs a capture.

| id | what it is | detection signal (regex-ish) | languages / frameworks | confidence |
|---|---|---|---|---|
| `tui-figlet-banner` | Block-letter ASCII banner at launch | imports/uses of `figlet`, `pyfiglet`, `figlet_rs`, `figlet-rs`, `go-figure`, `cfonts`, `ink-big-text`, `text2art`, `from art import`, `toilet`; or 4+ consecutive literals mostly `[█▀▄╔═╗║╚╝]` | all | high |
| `tui-gradient-title` | Rainbow/gradient applied to headings or banners | `gradient-string`, `ink-gradient`, `<Gradient`, `rich_gradient`, `Gradient(`, `lolcat`, `chromacat`, `tui-gradient`, `lipgloss.Blend1D(`/`Blend(` fed into a title style | JS/Ink, Python/Rich, Go/Lip Gloss, Rust | high |
| `tui-blink-attribute` | Blinking text | `Modifier::SLOW_BLINK`, `RAPID_BLINK`, `.Blink(`, `blink`, `\x1b[5m`, `[blink]` | all | high |
| `tui-emoji-density` | Emoji used as icons/bullets on most lines | count of `[\u{1F300}-\u{1FAFF}\u{2600}-\u{27BF}]\u{FE0F}?` in string literals over a per-file threshold, or the canonical set 🚀✅❌⚠️✨🎉📦🔧🔥💡 | all | medium-high (threshold) |
| `tui-hardcoded-rgb-no-adapt` | Truecolor literals with no light/dark or NO_COLOR handling | `Color::Rgb(`, `Color::from_u32(0x`, `lipgloss\.Color\("#`, `chalk\.hex\(`, `color="#`, `\[#[0-9a-f]{6}\]`, `\x1b\[38;2;`; strong only when the repo lacks `AdaptiveColor|LightDark|HasDarkBackground|ansi_color|NO_COLOR|isatty|is_terminal|IsTerminal` | Rust, Go, JS, Python (Rich); weak for Textual CSS | medium alone, high combined |
| `tui-raw-escapes` | Hand-written ANSI escapes in app code | `\\x1b\[`, `\\033\[`, `\\u001b\[`, `\\e\[` in string literals outside a color library | all | medium |
| `tui-double-border` | Double-line box borders | `DoubleBorder\(\)`, `BorderType::Double`, `borderStyle="double"`, `box\.DOUBLE`, `border:\s*double`, `--border double` | Go, Rust, JS/Ink, Python/Rich, Textual, gum | medium alone, high with `tui-figlet-banner` or `tui-gradient-title` |
| `tui-nested-borders` | Bordered container inside bordered container | two `borders(Borders::ALL)`/`Block::bordered()` in one render fn with a layout split between them; `.Border(` styles joined inside a `.Border(` style; `<Box borderStyle>` nested; Textual CSS `border:` on parent and child selectors | Rust, Go, JS, Textual | medium (structural) |
| `tui-spinner-no-tty-guard` | Spinner/progress emitted regardless of TTY | `ink-spinner|ora\(|gum spin|rich\.spinner|bubbles/spinner|briandowns/spinner` with no `isatty|isTTY|is_terminal|IsTerminal|--no-progress|CI` in the repo | all | high |
| `tui-interval-never-cleared` | Animation timer with no stop | `setInterval\(` count > `clearInterval\(` count in a module; `tea.Tick` re-armed unconditionally; `set_interval(` with no `.stop()`/`.pause()` | JS/Ink, Go/Bubble Tea, Textual | medium |
| `tui-hardcoded-size` | Assumes 80x24 or a fixed size | `Rect::new\(\s*\d+,\s*\d+,\s*\d+,\s*\d+\)` in render code; a `Layout` whose constraints are all `Constraint::Length`; `(width|cols|columns)\s*[:=]\s*80\b`, `(height|rows|lines)\s*[:=]\s*24\b`; `\.Width\(80\)`; `width=\{80\}`; `size=\(80,\s*24\)` outside tests; curses `mv(w)?addstr\(\s*\d+,\s*\d+`; `cursorTo\(\s*\d+` | Rust, Go, JS, Python, C | medium |
| `tui-grapheme-unsafe-truncate` | Byte/char slicing of display strings | `&\w+\[\.\.\d+\]`, `\.chars\(\)\.take\(`, Go `\w+\[:\w+\]` followed by `"…"`/`"..."`, JS `\.slice\(0,\s*\w+\)`/`\.substring\(0,` + ellipsis, Python `\[:\w+\]\s*\+\s*["']\.\.\.`; strong when the repo lacks `unicode-width|unicode-segmentation|runewidth|uniseg|ansi\.Truncate|string-width|wcwidth|grapheme` | Rust, Go, JS, Python (raw); low for Rich/Textual/Ink | medium-high |
| `tui-nerd-glyph-no-fallback` | Private-use icon glyphs with no ASCII tier | literals in `[\u{E000}-\u{F8FF}\u{F0000}-\u{FFFFD}]` and no `ascii|nerd|icons|--no-icons` option in the repo | all | medium |
| `tui-no-key-hints` | Key handling with no visible help | key handling present (`KeyCode::|tea\.KeyMsg|useInput\(|on_key|@kb\.add|key\.Binding`) and none of `Footer|BINDINGS|help\.Model|key\.WithHelp|ShowHelp|"\?"|quit|Esc` rendered | Rust, Go, Python/Textual, JS/Ink, prompt_toolkit | medium (absence) |
| `tui-mouse-only` | Actions reachable only by click | `EnableMouseCapture|WithMouseCellMotion|mouse_support=True|enableMouse\(|onClick` present and no key handler in the same component | all | low-medium (semantic) |
| `tui-center-everything` | Every block centered | density of `Alignment::Center`, `\.Align\(lipgloss\.Center\)`, `Place\([^)]*Center[^)]*Center`, `justifyContent="center"` + `alignItems="center"`, `align:\s*center middle`, `text-align:\s*center` | all | low-medium |
| `tui-green-on-black` | Hacker-terminal palette | `#00ff00|#0f0\b|Color::Green|"green"` paired with `Color::Black|#000\b|"black"` background; `AdaptiveColor{[^}]*Dark` with no `Light` or `Light` equal to `Dark` | all | medium |
| `tui-clear-or-altscreen-oneshot` | One-shot command clears the screen or takes the alt screen | `console\.clear\(\)|os\.system\(["']clear|Clear\(ClearType::All\)|\\x1b\[2J|WithAltScreen\(\)|alternateScreen:\s*true|full_screen=True` in a program that also writes results to stdout | all | medium |
| `tui-no-plain-mode` | Full-screen TUI with no line/plain/json alternative | `WithAltScreen|full_screen=True|alternateScreen|EnterAlternateScreen` present and no `--plain|--json|--no-tui|--static|NO_TUI` flag | all | medium (absence) |
| `tui-rich-color-sprawl` | Many distinct colors in Rich/Lip Gloss markup | > 6 distinct color names/hexes in `\[[a-z_ #0-9]+\]` Rich markup or `lipgloss\.Color\(` per file | Python/Rich, Go | medium |
| `tui-triple-encoded-status` | Emoji + color + word for the same state | `✅|❌|⚠️` adjacent to `green|red|yellow` style and `SUCCESS|FAIL|WARN` text on one line | all | low |
| `tui-decorative-corners` | Ornamental corner/divider glyphs | `◆|◈|✦|⬡|≋|•{4,}` used in border or divider literals | all | medium |
| `tui-print-in-loop` | `print`/`console.log` inside a TUI render loop (corrupts the screen) | `print\(` in a Textual `App` method other than a worker; `console\.log` in an Ink component without `patchConsole` | Textual, Ink | high detectability; bug, not slop |
| `tui-typed-command-not-selectable` (web) | Fake terminal typing animation | `bg-gray-900` + three `rounded-full` dots + `pre|code` | HTML/React | high (web audit only) |

**Runtime-only signals** (need a capture, not a regex): unreadable on a light theme, spinner outliving its task, layout collapse below 80 columns, glyph tofu without a Nerd Font, width drift on CJK/emoji rows, cursor jitter under a screen reader. The verification matrix in section 5 is how those get evidence.

---

## Sources (cited above)

1. https://no-color.org/
2. https://reviews.llvm.org/D152285
3. https://cmake.org/cmake/help/latest/envvar/NO_COLOR.html
4. https://man7.org/linux/man-pages/man5/terminal-colors.d.5.html
5. https://terminfo.dev/device/env-no-color
6. https://github.com/chalk/supports-color
7. https://rich.readthedocs.io/en/stable/console.html
8. https://rich.readthedocs.io/en/stable/markup.html
9. https://github.com/textualize/rich
10. https://clig.dev/
11. https://medium.com/@jdxcode/12-factor-cli-apps-dd3c227a0e46
12. https://github.com/flameddd/blog/blob/master/2019-06-15%EF%BC%9AAt%20Heroku%2012%20Factor%20CLI%20Apps.md
13. https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap12.html
14. https://www.unicode.org/reports/tr11/
15. https://mitchellh.com/writing/grapheme-clusters-in-terminals
16. https://www.jeffquast.com/post/perfecting-terminal-character-width-using-correction-tables/
17. https://docs.rs/unicode-width/latest/unicode_width/
18. https://news.ycombinator.com/item?id=44362272
19. https://ratatui.rs/faq/
20. https://ratatui.rs/concepts/layout/
21. https://ratatui.rs/concepts/rendering/
22. https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html
23. https://ratatui.rs/recipes/apps/panic-hooks/
24. https://docs.rs/ratatui/latest/ratatui/backend/struct.TestBackend.html
25. https://ratatui.rs/recipes/testing/snapshots/
26. https://docs.rs/crossterm/latest/crossterm/event/enum.Event.html
27. https://invisible-island.net/ncurses/man/resizeterm.3x.html
28. https://pkg.go.dev/github.com/charmbracelet/bubbletea
29. https://github.com/charmbracelet/lipgloss
30. https://github.com/charmbracelet/lipgloss/blob/main/UPGRADE_GUIDE_V2.md
31. https://charm.land/blog/v2/
32. https://github.com/charmbracelet/bubbles
33. https://github.com/charmbracelet/gum
34. https://github.com/charmbracelet/glow
35. https://github.com/charmbracelet/vhs
36. https://textual.textualize.io/guide/design/
37. https://textual.textualize.io/FAQ/
38. https://textual.textualize.io/guide/app/
39. https://textual.textualize.io/widgets/footer/
40. https://textual.textualize.io/guide/devtools/
41. https://textual.textualize.io/guide/testing/
42. https://textual.textualize.io/api/app/
43. https://textual.textualize.io/blog/2024/04/20/behind-the-curtain-of-inline-terminal-applications/
44. https://github.com/Textualize/textual/blob/main/src/textual/_doc.py
45. https://github.com/Textualize/textual/blob/main/tests/snapshot_tests/test_snapshots.py
46. https://github.com/Textualize/textual-dev/blob/main/src/textual_dev/cli.py
47. https://github.com/vadimdemedes/ink
48. https://github.com/vadimdemedes/ink/issues/1011
49. https://github.com/vadimdemedes/ink-testing-library
50. https://www.heise.de/en/news/React-in-the-Terminal-Ink-7-0-fundamentally-revises-input-handling-11249949.html
51. https://github.com/chjj/blessed
52. https://github.com/sst/opentui
53. https://python-prompt-toolkit.readthedocs.io/en/master/pages/full_screen_apps.html
54. https://gist.github.com/MangaD/cd8b8ab9b4f119ac5214fa4f3424ccd7
55. https://starship.rs/presets/no-nerd-font
56. https://hyperbliss.tech/blog/2026.04.04_terminal-renaissance/
57. https://evilmartians.com/chronicles/cli-ux-best-practices-3-patterns-for-improving-progress-displays
58. https://github.com/evilmartians/lefthook/issues/1539
59. https://xogium.me/the-text-mode-lie-why-modern-tuis-are-a-nightmare-for-accessibility
60. https://www.osnews.com/story/144892/the-text-mode-lie-why-modern-tuis-are-a-nightmare-for-accessibility/
61. https://github.com/NousResearch/hermes-agent/issues/120584
62. https://github.com/anomalyco/opencode/issues/8565
63. https://21st.dev/blog/terminal-ui-components
64. https://sailop.com/blog/terminal-mockup-problem-why-3-dots-ruin-your-design
65. https://github.com/iliaal/whetstone/blob/master/plugins/whetstone/skills/ia-frontend-design/references/banned-ai-patterns.md
66. https://dev.to/kiwibreaksme/why-ai-generated-uis-look-off-and-the-one-principle-that-fixes-it-4j20
67. https://playbooks.com/skills/eddiebe147/claude-settings/terminal-ui-design
68. https://playbooks.com/skills/ingpoc/skills/terminal-ui-design
69. https://github.com/alimtvnetwork/gitmap-v19/blob/main/spec/04-generic-cli/20-terminal-output-design.md
70. https://man7.org/linux/man-pages/man1/tmux.1.html
71. https://man7.org/linux/man-pages/man1/script.1.html
72. https://docs.asciinema.org/manual/cli/

Not fetched (HTTP 403 on every path tried): GNU Coding Standards, Command-Line Interfaces / Errors chapters. Not cited.
