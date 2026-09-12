---
id: 59
title: Every number in the stats pane is a door
type: feature
status: done
milestone: v0.4
depends_on:
- 56
- 61
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: chrome
---

## Problem

Every number on the stats pane is the answer to a query, and not one of them
is a way of getting there.

*2 ready.* Which two? *1 blocked.* On what? *0004 Draw the board — 9 days
open.* Fine: now find it. *0003 — 1 item is waiting on it.* Which item?

The reader's next thought after every line on that pane is *show me* — and
the pane's answer is: go to the list, remember the grammar, type the filter.
For numbers the pane already computed from exactly that filter.

Two rows above, the status strip does the right thing already: click a
status, the list narrows to it, click again to clear. The affordance exists
in the header and is missing from the pane whose entire content is queries.

## Proposal

Every figure is a door. Enter on it, or a click, goes to the list with the
filter that produced it — `ready=true`, `blocked=true`, `category=done`,
`assignee=` — and `esc` comes back out, which it already does.

That turns the stats from a report into the index of the backlog: the one
screen that answers *what is the shape of this* and then takes you to any
part of it.

It also gives the pane a cursor, which is the other half of 0056's question:
a lens with no cursor is a lens you cannot act in, and this is the pane that
proves it does not have to be that way.

## Cost

Not every number has an honest filter behind it. *1 of 2 acceptance criteria
ticked on open work* is an aggregate over a set, not a set. Those should not
pretend to be doors — a figure that looks clickable and is not is worse than
one that plainly is not. So the pane will have two kinds of line, and the
difference has to be visible without trying.

## Acceptance criteria

- [x] A figure that stands for a set is selectable, and opening it filters the list to that set
- [x] The same by click, because the mouse is not a second-class way to drive this
- [x] A figure that is an aggregate rather than a set is visibly not a door
- [x] `esc` comes back to the unfiltered view, as it already does
