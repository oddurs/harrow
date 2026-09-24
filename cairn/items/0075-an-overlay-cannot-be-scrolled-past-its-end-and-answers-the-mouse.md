---
id: 5624bcd6-d15e-4274-97af-befe8d97daa5
title: An overlay cannot be scrolled past its end, and answers the mouse
type: bug
status: done
milestone: v0.2
created: 2026-09-15
updated: 2026-09-15
priority: p1
area: chrome
---

## What happens

Open an item with `↵` and hold `j`. The text stops moving and the overlay keeps
scrolling — the body scrolls up out of the frame until the pane is empty, and
there is no way back but `k` the same number of times. The same is true of the
history overlay.

The mouse is worse than absent. The wheel scrolls the reader, so the pointer
appears to work, but a *click* inside the open overlay is not handled by the
overlay at all: it falls through to the list underneath, because the hit map
still holds the list's rows and nothing has been registered on top of them. So
a click on the body you are reading silently moves the selection behind it, and
a click that happens to land on a tab switches lens with the reader still open
over the new one. The overlay says `any other key closes` on its own edge, and
a click is the one gesture that neither closes it nor is ignored.

## Why it is like this

0024 gave every *pane* its own scroll and clamped each one, with the clamp in
the draw because the draw is the only place that knows how tall the content came
out. The overlays were not panes at the time and did not get the same
treatment: `read_scroll` and `history.scroll` are still raw `u16` counters that
`saturating_add` forever against content whose height nobody measures.

Those two are now the only scrollable surfaces in harrow with no upper bound.

## Proposal

Give the overlays what the panes already have, and no more than that.

- Clamp both overlays in their draw, `over = lines - inner.height`, the same
  expression `draw_detail` uses. The reader's offset needs no identity — it is
  reset on open, and the overlay only ever shows the selected item.
- Register the overlay's own area in the hit map while it is up, so a click
  inside it reaches the overlay instead of the list. Inside is inert: the body
  is text, and there is nothing in it to press.
- A click outside closes it, which is what `any other key closes` already
  promises and what every other overlay on a screen does.
- Say `↑↓ scroll` only while there is somewhere to scroll to, the way a pane
  holding more than it shows puts the keys on its own edge.

## Cost

A click inside the overlay becomes inert where it used to reach the list. That
is the point — the previous behaviour was reaching a surface the reader was
covering — but it does mean a click is no longer a way to dismiss the reader
and select in one gesture. Closing on click-outside covers that in two
gestures that are both visible.

## Acceptance criteria

- [x] The reader cannot be scrolled past its last line
- [x] The history overlay cannot be scrolled past its last line
- [x] Scrolling a body shorter than the frame leaves it where it is
- [x] A click inside an open overlay does not move the selection behind it
- [x] A click outside an open overlay closes it
- [x] An overlay offers the scroll hint only when it holds more than it shows
