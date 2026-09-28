---
id: 122
uid: c658bc66-fdb3-4741-b6c0-02e77edcc27b
title: Advance the Cairn pin to prompt, split and prompt checks
type: chore
status: backlog
milestone: v0.8
depends_on:
- 120
created: 2026-09-27
updated: 2026-09-27
priority: p1
effort: s
area: read
---

## Why now

Blocked until cairn 0156 (`cairn prompt`), 0157 (`cairn split`) and 0158
(prompt checks) merge. The three harrow items that call them are tested
against this pin. May land in steps if cairn merges them apart; say which
revision covers which command in COMPATIBILITY.md.

## What changes

- CI, COMPATIBILITY.md and the corpus provenance pin the revision that has all
  three, by the procedure in COMPATIBILITY.md.
- COMPATIBILITY.md lists, for each new command, the first Cairn revision
  harrow uses it with, and what harrow does without it.

## Done when

- [ ] CI builds a Cairn with `prompt`, `split` and `check --prompts`
- [ ] COMPATIBILITY.md says what each needs and how harrow degrades without it
