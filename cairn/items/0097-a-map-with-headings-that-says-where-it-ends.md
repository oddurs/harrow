---
id: 97
title: A map with headings, that says where it ends
type: feature
status: done
milestone: v1.0
created: 2026-09-21
updated: 2026-09-21
priority: p2
area: chrome
---

## Problem

The help overlay was forty-two rows in one run, in an order that made sense to
whoever last added a row. `space` sat between `←/h` and `tab`; `x` for close
sat under `M` for milestone and over `u` for reopen; the ways of looking at the
backlog were interleaved with the ways of changing it. A reader who wanted to
set a status had to read the list until they found it.

That is the exact problem harrow exists to solve for a backlog — a list too
long to take in at once, with no structure over it — reproduced in harrow's own
map.

## Proposal

File every row under what a reader has come to do: **Move**, **Look**,
**Find**, **Read and copy**, **Change**, **Program**, **Pointer**. The headings
are the detail pane's own section rule, so a heading reads the same wherever
harrow draws one. `Command::HELP_SECTIONS` is the single statement of what the
overlay holds and in what order; `help_order()` is now derived from it, so the
contract test from 0094 keeps holding the overlay to `Command::ALL` without a
second list to keep in step.

Sections are kept whole across the two columns and stay in order, broken
wherever the taller column comes out shortest. A heading at the foot of one
column with its rows at the head of the next is a heading over nothing.

## The cost that decided the design

Headings cost lines, and the overlay was already nearly as tall as a common
terminal. Measured rather than estimated: with sections whole and in order,
the best break needs 28 rows a column, and a 32-line terminal leaves 26. Every
arrangement that fit did so with no slack at all — and 0094's contract test
*requires* every new keyed command to join the overlay, so a layout that fits
exactly is a trap set for whoever adds the next one.

So the overlay scrolls when it is taller than the screen, and says so on its
bottom edge with the keys that scroll it, spelled from the bindings in force —
the way the detail pane already does. The keys that move move it; anything
else puts it away, as it always did, and never reaches the backlog underneath.
This also ends a quieter problem: on a short terminal the overlay used to stop
at the edge, and the rows past it did not exist as far as the reader could
tell.

Two smaller choices fell out of it. The blank line under the `Keys` title went,
because the first row is now a heading that parts itself from the title, and
that one line was all that stood between `:toggle-mouse` and the bottom edge
at 32 rows. And `space` reads *mark it, or fold a heading* — its old text was
cut to *mark it for the next change —…*, which hid exactly the half that 0095
made work.

## Rejected

- **Newspaper flow**, letting a section run from one column into the next. It
  balances to the row, and the continuation is a list with no heading over it.
- **A gutter** holding each section's name beside its first row instead of
  above it. It costs width instead of height, and at 100 columns — where the
  overlay first goes to two — there is none to spare; every description would
  truncate harder.
- **Three columns.** At 110 wide each would be about thirty-five characters,
  and the descriptions would be cut to nothing.

## Done when

- Every command is filed under exactly one heading, and a heading with no rows
  is not drawn.
- At a height the whole overlay does not fit, its edge says there is more and
  the keys that move reach all of it; where it fits, it says nothing.
- A key that is not a movement closes the overlay and changes nothing else.
- `help.txt` re-recorded and read; `help-short.txt` records the scrolling case.
