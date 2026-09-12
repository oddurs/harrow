---
id: 66
title: The detail reads as a thread, not a card
type: feature
status: backlog
milestone: v0.5
depends_on:
- 64
created: 2026-09-12
updated: 2026-09-12
priority: p2
area: chrome
---

## Problem

A cairn item body accumulates. `cairn note` appends reasoning under today's
date; `cairn propose` writes a proposal into it; `cairn release --reason`
leaves a note explaining the handover. Over a piece of work done by several
parties, the body becomes a record of what happened and why.

harrow's detail pane renders it as a card: the title, a state line, a
progress bar, a grid of fields, and then — under everything else, when there
is room left — the body from the top.

For a supervisor the interesting end is the other one. The last note is what
somebody just learned. The open proposal is what is being asked. Both are at
the bottom of the body, below the fields, below the fold.

## Proposal

The detail keeps its identifying head — what this is, where it stands — and
then reads as a thread: the item's own prose, then what has been added to it,
in time order, with the most recent visible without scrolling.

The fields move below. They are reference material and they are static; they
have the top of the pane because they are easy to lay out, not because they
answer anything.

## Cost

An item with no notes is most items, and for those this changes nothing
except that the fields move down. Worth checking that the common case does
not get worse to make the interesting case better.

`cairn note` files under a date heading by default, so the thread has a
structure to read — but a body written by hand has whatever headings its
author chose, and the pane must not pretend those are entries.

## Acceptance criteria

- [ ] The most recent addition to an item is visible without scrolling
- [ ] Notes, proposals and prose are distinguishable from each other
- [ ] An item with no notes reads at least as well as it does now
- [ ] A hand-written body is not parsed into entries it does not have
