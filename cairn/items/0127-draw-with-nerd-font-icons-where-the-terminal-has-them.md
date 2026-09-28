---
id: 127
uid: 1fa7181f-09e3-4421-ba68-a0e66adc0f34
title: Draw with Nerd Font icons where the terminal has them
type: feature
status: done
created: 2026-09-27
updated: 2026-09-27
priority: p2
area: theme
---

## Problem

harrow draws with the Unicode every font has: `○ ◐ ✓ ⊘ ×` for state, `▰▱`
for progress, `▾` for a fold. It reads, but it is the lowest common
denominator, and most of the people who run a terminal program like this one
have a Nerd Font, or a terminal that ships the symbols itself. Ghostty,
WezTerm and kitty (0.36 on) all bundle them. On those terminals harrow could
look much better and does not.

The Nerd Font glyphs sit in the Private Use Area. No terminal will say whether
its font has them, and one that does not draws a box. So this cannot be turned
on for everybody. Where the answer is unknown, the screen has to be the one it
already is.

## Proposal

A glyph set chosen once, the way the theme is: a `glyphs` config key and a
`--glyphs` flag, `auto`, `nerd` or `unicode`. Every glyph harrow draws goes
through a table of roles in `src/glyphs.rs`, with a Unicode table and a Nerd
Font table, so no drawing code names a character.

- **State** uses GitHub's issue octicons: opened, closed, not planned,
  blocked. Work under way is a pie that fills as it turns.
- **Progress** uses Fira Code's joined progress bar, and the loading spinner
  uses its arcs.
- **Chrome** gets icons where they help find something: the lens tabs and the
  pane titles that match them, the toolbar controls, the section headings,
  the kind of question in the needs queue, milestones, people, and the toast.

`auto` picks `nerd` only on evidence: a terminal that bundles the symbols
(`TERM_PROGRAM`, `TERM`, or the variables each of them exports, which survive
tmux) and a UTF-8 locale. Anything else, including an unknown terminal, keeps
Unicode. A font you installed yourself is not something harrow can see, so
that case is `glyphs = "nerd"`.

In the Unicode table every icon is an empty string, so the Unicode screen
does not change by one cell. The recorded screens prove it.

## Cost

Two ways every screen can look. A Nerd Font icon is often drawn wider than
its cell, so every icon needs a blank cell after it or it runs into the next
character. That rule is held by a test, not left to whoever adds the next
icon. The Unicode screens stay recorded, and each lens gets one Nerd screen.

0048, the ASCII set, is the third table in the same place and is not done
here.

## Acceptance criteria

- [x] Every glyph with a Nerd form is drawn through a role in one table, and the Unicode table draws exactly what harrow drew before
- [x] `glyphs = "auto" | "nerd" | "unicode"` in the config and `--glyphs` on the command line; an unknown value warns and falls back
- [x] `auto` chooses Nerd only on a terminal known to bundle the symbols and a UTF-8 locale, and says why in `harrow config` and `--doctor`
- [x] Every Nerd icon has a blank cell after it, held by a test over every screen state
- [x] Every glyph a Nerd screen draws is declared, and every Nerd codepoint is in the Nerd Fonts ranges
- [x] One recorded Nerd screen per lens

## 2026-09-27

Unicode: all 28 recorded screens unchanged cell for cell; tests/glyphs.rs now also holds that a Unicode screen draws nothing from the Private Use Area. Nerd: list, board, stats, needs and log recorded as *-nerd.txt. The spill test fails when one icon is glued to its text (checked by mutation: U+F45E at 43,8 has no room after it). Codepoints checked against Nerd Fonts 3.5.1 glyphnames.json and drawn in FiraCode Nerd Font Mono to choose them. One behaviour change for both sets: the detail pane's state line breaks between facts instead of clipping the name. scripts/task check green. Not done: 0048's ASCII set, which is a third table in the same place.
