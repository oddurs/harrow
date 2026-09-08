---
id: 10
title: cairn's filter grammar in the filter box
type: feature
status: done
milestone: v0.1
created: 2026-09-08
updated: 2026-09-08
priority: p1
area: filter
---

## Problem

A filter box with its own syntax would be a second grammar to learn, and the
one you already know would be the one that did not work.

## Proposal

Implement cairn's grammar — clauses, alternatives, the derived keys, empty for
absence — so anything that works after `--filter` works after `/`. Implemented
rather than delegated: the box runs on every keystroke, and a process per
keystroke is a different program.

One addition: a clause with no operator is a free-text search, so `/` is useful
before any of this has been learned.

## Acceptance criteria

- [x] `priority=p0|p1,category!=done` means what cairn means by it
- [x] `blocked`, `ready` and `body` resolve
- [x] The list narrows as it is typed
- [x] A field the project has not got is reported, not silently empty
