---
id: 60
title: The board groups by whatever you ask it to
type: feature
status: done
milestone: v0.4
depends_on:
- 56
created: 2026-09-12
updated: 2026-09-12
priority: p2
area: chrome
---

## Problem

`v` cycles the grouping axis on the list. On the board it says:

> the board is grouped by status

phrased as a fact about boards. It is not. `cairn board --group-by milestone`
exists, and `--group-by` takes any field with declared values. So the tool
harrow is a front end for can do this, and harrow tells you it cannot,
without saying that is harrow's limitation rather than the idea's.

A board by milestone is a release plan. A board by assignee is a workload. A
board by area is where the work lives. These are the questions a board is
*good* at and they are all one key away in the CLI.

## Proposal

`v` cycles the board's columns through the same axes it cycles the list's
groups through — which is already computed, in `grouping_axes`, from what the
project declares.

Two things follow that the status board gets for free and the others do not.
Columns come from the status table, in declared order, with `board = false`
honoured; for any other field the columns are the field's declared values, in
their declared order, plus one for items with no value. And dragging a card
sets *that field* rather than the status, which is the same gesture meaning
the same thing — and is why this is worth having rather than a second read-only
arrangement.

## Cost

Dragging onto a milestone column writes a reference rather than an enum, and
a field the project has not declared values for has no columns to draw. Both
are cases cairn has already answered — it refuses the grouping and says why —
and harrow should give the same answer in the same words rather than invent a
second opinion.

## Acceptance criteria

- [x] `v` changes the board's columns, through the axes the project declares
- [x] Columns keep their declared order, and a status with `board = false` stays off
- [x] Dragging a card sets the field the board is grouped by
- [x] An axis that cannot be a board is stepped over rather than landed on —
      `none` is a list with the grouping off, which is a thing to want and not
      a thing a board can be, so cycling skips it instead of showing a blank
      screen and an explanation
- [x] The board keeps its own axis: grouping is arrangement, and a list by
      milestone beside a board by status is the pair that would be lost by
      sharing one
