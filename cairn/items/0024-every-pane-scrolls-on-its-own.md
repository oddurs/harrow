---
id: 24
title: Every pane scrolls on its own
type: feature
status: doing
milestone: v0.2
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: chrome
---

## Problem

There are three or four things on screen at once and one scroll position
between them.

The wheel always drives the list, wherever the pointer is. The detail pane has
no scroll at all: it measures what is left after the fields and truncates the
body to fit, offering `↵ to read it all` — so reading three more lines of a
proposal means opening an overlay over the list you were reading it against,
which is the one thing a two-pane layout is supposed to avoid. On the board,
every column but the focused one is pinned to its first card, and the wheel
moves the selection rather than the view. The stats pane does not scroll
either, so on a short terminal its bottom is simply gone, with nothing saying
so.

They are panes. A pane you cannot scroll is a pane that has to be small enough
to never need it, which is what has been quietly constraining the detail pane's
contents.

## Proposal

Every pane keeps its own scroll position, and a scroll gesture goes to the pane
under the pointer rather than to the list.

- The detail pane renders its whole body and scrolls, so `↵` becomes a
  comfort rather than the only way to see the end of an item. Its offset
  belongs to the item, not to the pane: selecting something else starts at the
  top of it.
- Each board column scrolls where it sits. The focused one still keeps its
  cursor in view, the way the list does.
- The stats pane scrolls, and because it has no cursor, its arrow keys move it.
- `J` and `K` scroll the detail pane from the keyboard, bound as
  `detail-down` / `detail-up` like everything else.

The rule the list already follows holds everywhere: scrolling moves the view
and takes the cursor with it only when the cursor would otherwise leave.

Where a pane holds more than it shows, it says so on its own bottom edge rather
than in a legend somewhere else.

## Cost

Each scrolling pane is one more piece of state that can be stale — scrolled
half way down an item that is no longer selected, or a column that shrank under
the offset. Clamping happens where the height is known, which is the draw, next
to the clamp the list already does. The alternative, recomputing every offset
from the cursor each frame, is what pins the board's columns today.

## Acceptance criteria

- [x] The wheel scrolls the pane under the pointer, not whichever pane holds the cursor
- [x] The detail pane shows a long body in full, by scrolling, without opening the reader
- [x] Its scroll returns to the top when a different item is selected
- [x] A board column scrolls without moving the selection out of another column
- [x] The stats pane reaches its own bottom on a short terminal
- [x] A pane with more than it shows says so, and says which keys move it
