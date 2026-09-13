---
id: 72
title: The board wastes its width and reads as unsorted
type: bug
status: done
milestone: v0.4
created: 2026-09-13
updated: 2026-09-13
priority: p1
area: chrome
---

## What happens

A real board, 147 items, a hundred and fifty columns of terminal:

```
╭ backlog 119 ───────────────╮╭ planned 0 ──────╮╭ in progress 0 ──╮╭ blocked 0 ──────╮╭ done 14 ────────╮
│  ○ 0029 transport.hpp:…  p0││                 ││                 ││                 ││  ✓ 0017 si.hpp…  │
│  ○ 0030 Derive Lambert…  p0││                 ││                 ││                 ││  ✓ 0018 Decide…  │
```

Three of the five columns are empty and each takes a fifth of the screen —
ninety columns of nothing — while the column holding a hundred and nineteen
items gets twenty-eight, which truncates every title to fourteen characters.
The board is mostly blank and the part that is not is unreadable.

Three separate faults, all of them the board rather than the data.

**Equal columns are the wrong rule when a column is empty.** The rule exists
for a good reason — *a board whose columns move as items arrive is a board
you cannot learn the shape of* — but it was written for columns that have
something in them. An empty lane needs enough room for its name, not a
fifth of the terminal.

**The cards are sorted by the list's grouping.** Reading down the backlog
column: p0 ×8, p2 ×5, p0 ×3, p2 ×4. That is not unsorted, it is sorted by
milestone and then priority — and on a board grouped by *status*, the
milestone is invisible, so the priority order appears to restart at random.
0060 sorted the cards "the same way a group does", which was right about the
sort keys and wrong to include the list's group rank.

**`--group-by` does not reach the board.** 0060 gave the board its own axis
so that a list by milestone could sit beside a board by status, and never
wired the flag: `harrow -b --group-by priority` still draws the statuses.
`v` works, the flag does not.

And a fourth, smaller: a column showing twenty of a hundred and nineteen
says nothing about the other ninety-nine. The count is in the heading, but
nothing says you are looking at a sixth of it.

## Proposal

**A column with no cards gets a narrow lane** — its name and its count and
no more — and the columns that have cards share what is left, equally among
themselves. Every declared column is still present, still in declared order,
still the same width as its peers. What changes is that emptiness stops
costing the same as content.

**Cards order by the sort keys alone**, not by the list's grouping. Within a
column, one order that means something.

**The flag sets the axis of the lens it opens.** `-b --group-by priority`
gives a board by priority.

**A column that holds more than it shows says how much more**, on its own
bottom edge, the way the detail pane does.

## Cost

The narrow lane is a second column width, which is the thing the equal-width
rule was protecting against. The protection that matters is against widths
that move *as work arrives* — a card landing in `planned` will widen that
lane once, from empty to its share, and that is a real jump. It is worth it:
the alternative is the screenshot above, which is the common case on any
project whose work is not evenly spread, which is every project.

## Acceptance criteria

- [x] An empty column takes a narrow lane, not an equal share
- [x] Columns with cards share the rest equally among themselves
- [x] Every declared column is still drawn, in declared order
- [x] Cards within a column are ordered by the sort, not by the list's grouping
- [x] `--group-by` sets the board's axis when the board is what opens
- [x] A column with more than it shows says how much more
