---
id: 7
title: Triage without typing an id
type: feature
status: done
milestone: v0.1
created: 2026-09-08
updated: 2026-09-08
priority: p0
area: chrome
---

## Problem

The daily loop — `next`, `claim`, `close` — is three short commands and does not
need a screen. What commands genuinely serve badly is triage: forty items,
deciding what matters, retyping an id every time.

## Proposal

One keystroke per decision. `c` claims, `s` picks a status, `p` a priority, `M`
a milestone, `l` and `h` walk an item through the columns, `x` closes with a
confirm. The cursor stays on the item while everything rearranges around it.

## Acceptance criteria

- [x] Forty items triaged without typing an id
- [x] A change that is already true is not a change
- [x] Every change says how to undo it
- [x] Closing asks first; nothing else does
