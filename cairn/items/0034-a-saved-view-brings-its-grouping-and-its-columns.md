---
id: 34
title: A saved view brings its grouping and its columns
type: feature
status: backlog
milestone: v0.3
created: 2026-09-12
updated: 2026-09-12
priority: p2
area: filter
---

## Problem

A saved view in `cairn.toml` declares up to five things: a `filter`, a `sort`,
a `group_by`, a set of `columns`, and a `description`. harrow's `View` holds
three of them and drops `group_by` and `columns` on the floor.

So a project whose `triage` view is *ungrouped, sorted by priority, showing
priority and area* gets, in harrow, the filter and the sort — and then
whatever grouping the user last cycled to with `v`. The view is the project
saying "this is how to look at this"; honouring half of it produces a screen
the project did not describe and the user did not ask for.

## Proposal

Read both. Entering a view applies its grouping; leaving it puts back what was
there before, the way the filter already behaves. `v` while in a view still
regroups — the project's answer is a starting point, not a cage — and doing so
does not silently drop out of the view.

`columns` is the harder half: harrow's list row is a fixed layout that
degrades in a defined order rather than a table of chosen columns. The
honest minimum is to honour `columns` where it names a field harrow already
has a column for, and to keep ignoring the rest rather than inventing a
second row layout. Decide that in the item before writing the code; if it
comes out as "the list is not a table", say so here and close `columns` as
deliberately unimplemented.

## Acceptance criteria

- [ ] A view declaring `group_by` opens grouped that way
- [ ] Leaving the view restores the grouping that was in force before it
- [ ] Regrouping by hand inside a view keeps the view
- [ ] `columns` is either honoured or documented as deliberately not, with the reason
