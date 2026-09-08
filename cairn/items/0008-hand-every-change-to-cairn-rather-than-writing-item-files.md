---
id: 8
title: Hand every change to cairn rather than writing item files
type: feature
status: done
milestone: v0.1
created: 2026-09-08
updated: 2026-09-08
priority: p0
area: write
---

## Problem

harrow could write frontmatter itself. It would then own the write lock, the
id allocation, the hooks, the filename rules and the schema validation — badly,
and separately from the tool that already owns them.

## Proposal

Every change becomes a `cairn` invocation, produced by `App` and run by the
shell. The core performs no side effects, which is what lets a test assert on
the exact argv a keystroke produces without a repository underneath it.

## Acceptance criteria

- [x] `App` returns changes; it never runs one
- [x] Every write is asserted as its argv in the test suite
- [x] With cairn missing, harrow reads and says it cannot write
