---
id: 58
title: The detail follows the selection, not the list
type: feature
status: backlog
milestone: v0.4
depends_on:
- 56
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: chrome
---

## Problem

The detail pane belongs to the list's *layout* rather than to the selection:

```rust
} else if app.pane == Pane::Board {
    draw_board(f, app, &t, body);
} else if body.width >= DETAIL_MIN_WIDTH {
    // split, and draw the detail
```

So on the board — the same backlog, the same cursor, the same selected item —
the detail is gone. A hundred and forty columns of terminal, and the item you
have selected is a one-line card, because of which tab you are on.

Nothing about a board makes detail unwanted. The board is the better lens for
*where things are*; it is not a lens for *less about this thing*. The reason
it has no detail is that the code that draws it was written after the split
and never asked.

## Proposal

The detail is a property of *there being a selection*, drawn beside whatever
the lens is, at whatever width the lens can spare. A board of four columns in
a hundred and forty is four columns in eighty plus the pane.

The narrow rule holds unchanged: below the threshold the detail gets out of
the way rather than halving the arrangement. That decision is already made
and already right.

## Cost

The board loses width, and columns that were comfortable become tight. The
mitigation is the one the list already uses — the detail goes first when the
terminal is narrow — and the threshold for a board may want to be higher than
for a list, because a board is already divided.

Worth checking: does a four-column board at eighty columns still read? If it
does not, the answer may be that the detail is a toggle on the board rather
than a default. That is a smaller claim than "the board has no detail", and
it is one somebody chose.

## Acceptance criteria

- [ ] The selected card's detail is reachable from the board
- [ ] The narrow rule still drops the detail before it halves anything
- [ ] A recorded screen shows a board with the detail, at a width where both read
- [ ] Whatever the answer is for a narrow board, it is a decision with a reason beside it
