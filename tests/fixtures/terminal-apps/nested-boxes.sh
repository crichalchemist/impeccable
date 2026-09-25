#!/bin/sh
# A deliberately bad TUI for tests/tmux-engine.test.mjs: a box drawn directly
# inside a box, gray-on-gray truecolor text, no key hints, and a spinner that
# never stops. Draws once at 60 columns, then animates one cell forever; the
# 0.15 s step keeps the ten-glyph cycle off the engine's one-second recapture
# so the two captures never alias.
printf '\033[2J\033[H'
inner() { printf '││%-56s││\n' "$1"; }
printf '┌──────────────────────────────────────────────────────────┐\n'
printf '│┌────────────────────────────────────────────────────────┐│\n'
inner ' Impeccable fixture'
inner ''
printf '││ \033[38;2;120;120;120m\033[48;2;100;100;100m%-54s\033[0m ││\n' 'faint status text nobody can read'
inner ''
inner ' working  '
printf '│└────────────────────────────────────────────────────────┘│\n'
printf '└──────────────────────────────────────────────────────────┘\n'
printf 'Loading complete.'
while :; do
  for g in '⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏'; do
    # Save the cursor, write the glyph at row 7 column 13, restore.
    printf '\0337\033[7;13H%s\0338' "$g"
    sleep 0.15
  done
done
