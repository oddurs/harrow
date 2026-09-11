---
id: 18
title: Multi-select, for triage that is actually bulk
type: feature
status: doing
milestone: v0.2
created: 2026-09-08
updated: 2026-09-11
priority: p1
area: chrome
---

## Problem

Triage is one keystroke per item, which is the point. But "everything in this
milestone is now p2" is one decision and forty keystrokes.

## Proposal

A selection: `space` marks, a marked set takes the next change, `esc` clears it.
cairn already supports the write — `cairn set --filter 'milestone=v0.1'
priority=p2` — so the interesting half is showing clearly what is marked and
what a change is about to touch.

## Acceptance criteria

- [x] Marking is visible without reading a count
- [x] A change to a marked set is confirmed, with the number in the prompt
- [x] Where the marks correspond to a filter, one `cairn set --filter` does it
