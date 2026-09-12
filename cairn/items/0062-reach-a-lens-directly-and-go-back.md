---
id: 62
title: Reach a lens directly, and go back
type: feature
status: backlog
milestone: v0.4
depends_on:
- 56
created: 2026-09-12
updated: 2026-09-12
priority: p3
area: chrome
---

## Problem

`tab` cycles forward: list, board, stats, list. There is no way back, no way
to a named lens, and no way to the one you were just in.

At three that is at most two presses and nobody complains. It is still the
wrong shape: the cost of reaching a lens depends on where it sits in a cycle,
which is an implementation detail of an array, and it gets worse with every
lens added.

The tabs are clickable, which is the direct route — for the mouse only.

## Proposal

`shift-tab` goes back, which costs one binding and is what every other tabbed
thing does.

A direct key per lens is the larger half and wants a decision: digits (`1`,
`2`, `3`) are free, positional, and learnable in a second, but they are also
the obvious keys for a future "jump to the nth thing". Initials collide —
`b` is taken by nothing yet but `s` is status and `l` is advance.

Worth settling in 0056 alongside what a lens is, since the answer depends on
how many there will ever be.

## Acceptance criteria

- [ ] `shift-tab` reaches the previous lens
- [ ] A direct route to each lens exists, or the reason not to is recorded
- [ ] Whatever is chosen appears in the help, which is generated from the bindings
