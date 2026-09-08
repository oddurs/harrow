---
id: 9
title: A board with the project's own columns
type: feature
status: done
milestone: v0.1
created: 2026-09-08
updated: 2026-09-08
priority: p1
area: chrome
---

## Problem

`cairn board` prints. It cannot act, and a printed board is the one view where
that hurts most: a board is a thing you move cards on.

## Proposal

The same items, dealt into the columns the project declared, with `tab` to get
there and `←`/`→` between columns. A status with `board = false` is not a stage
of the work, so nothing walks into it.

## Acceptance criteria

- [x] One column per status with `board = true`, in declared order
- [x] The list and the board show the same set under the same filter
- [x] The selection survives switching between them
