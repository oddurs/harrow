---
id: 9a34d78d-0f42-4ca0-9800-cb7329ee97a8
title: A dropdown you cannot click out of
type: bug
status: done
milestone: v1.0
created: 2026-09-22
updated: 2026-09-22
priority: p1
area: chrome
---

## What happened

Reported: "I couldn't click out of the milestone filter. Dropdowns should
behave like GUI ones."

Opened by a click on `v milestone ▾`, the grouping dropdown ignored every
click that was not on one of its own rows — and not quietly. Measured:

| click, with it open | what happened |
|---|---|
| its own segment again | opened it again |
| a row beside it | stayed open, **and selected the row behind it** |
| a tab | stayed open, **and changed the lens behind it** |
| the detail pane | stayed open |

The only way out was a key. The same was true of every popover: the `s`,
`p` and `M` pickers, the palette, help, diagnostics and the close
confirmation all ignored an outside click. Only the history overlay closed on
one, and its code already stated the rule the rest broke: a click outside
means what any other key means.

The cause was structural. A popover registered hits for its own rows and
nothing for its frame, so a click on its border fell through to what it
covered; and nothing told the click handler that anything was open at all.

## What a GUI dropdown does, and now does here

- **A click outside closes it and goes no further.** Each popover claims the
  ground it clears — `claim()`, called where every one of them already calls
  `render_widget(Clear, popup)` — which makes its whole frame inert and
  records where it is, so a click can tell inside from outside without the
  two ever disagreeing.
- **Its own segment shuts it; another segment opens that one,** in one click:
  a menu bar.
- **A release chooses, not a press.** Pressing highlights; the choice is made
  when the button comes up over an option. So a press can be dragged to
  another option, or off the list to choose nothing, and the native
  press-on-the-button, drag-into-the-list, let-go gesture works.
- **The wheel belongs to what is open.** Over it, it moves the highlight (or
  scrolls help); anywhere else it does nothing, rather than scrolling a lens
  hidden behind the popover.
- **`Home` and `End`** reach the ends of an open list.

A close confirmation dismissed by an outside click is answered **no** — which
is what any key but `y` already answers it, and the answer that changes
nothing.

## What it cannot do

Highlight under a pointer that is merely moving. That needs `?1003`
any-motion tracking, which `term.rs` refuses on purpose: it reports every
cell the pointer crosses, forever, and a failure to turn it off leaves an
unusable shell. Motion is reported while a button is held (`?1002`), which is
what makes the press-and-drag gesture work.

## Found on the way

The randomised suite had only ever pressed keys; the pointer had never been
through it. A pointer variant — random presses, drags, releases and scrolls
among random keys, a frame drawn between each — failed on seed 357, on
`main` as much as here: `drag()` moved the board cursor into the column under
the pointer and left its row behind, so dragging a card pressed third in a
column over a column of one left the cursor on a row that column does not
have. It clamps now, as `step_group` always did.

The pointer suite runs a hundred seeds by default and every seed asked for
with `HARROW_FUZZ_SEEDS`, uncapped: a cap would never have reached 357.

Two older tests clicked by sending the button down and never up — half a
gesture no terminal sends — and started drags with that same half-click. The
helpers send a whole click now, and a drag starts with `press`.

## Done when

- A click outside any popover closes it and reaches nothing behind.
- A dropdown's segment toggles it and a neighbour's switches to it.
- An option is chosen on release, including after a press-drag from the
  segment; a press dragged off the list chooses nothing.
- The pointer has a randomised suite, and it passes at 20,000 seeds.
