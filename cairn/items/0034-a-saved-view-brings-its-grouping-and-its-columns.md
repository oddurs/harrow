---
id: 34
title: A saved view brings its grouping and its columns
type: feature
status: done
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

## `columns`: deliberately not

The list is not a table. Every row is one fixed layout that degrades in a
defined order as the pane narrows — the acceptance count goes first, then the
assignee, then the priority — so that the eye can run down a column instead of
hunting along each line. That ordering is a design decision about what answers
least, and it is the reason the list stays readable at sixty columns.

`columns` names an arbitrary set in an arbitrary order. Honouring it means
either a second row layout with no degradation rule, or quietly reordering
within the one that exists and calling that obedience. Both are worse than
saying no.

So: not implemented, on purpose. A project that wants its own columns has
`cairn list`, which is a table and prints one. If this is ever revisited it
needs a degradation rule for arbitrary columns first, and that is a different
piece of work from reading a config key.

## Acceptance criteria

- [x] A view declaring `group_by` opens grouped that way
- [x] Leaving the view restores the grouping that was in force before it
- [x] Regrouping by hand inside a view keeps the view
- [x] `columns` is either honoured or documented as deliberately not, with the reason
