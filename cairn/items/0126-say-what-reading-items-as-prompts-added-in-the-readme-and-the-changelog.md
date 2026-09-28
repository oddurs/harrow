---
id: 126
uid: d438a38a-498c-413e-8855-d680982cc58b
title: Say what reading items as prompts added, in the README and the changelog
type: chore
status: backlog
milestone: v0.8
depends_on:
- 119
- 121
- 123
- 124
- 125
created: 2026-09-27
updated: 2026-09-27
priority: p1
effort: s
area: docs
---

## Why now

Each command in this milestone degrades silently against an older cairn. A
reader of the README who does not see `P` or a Result prompt needs to know
it is their cairn, not harrow.

## What changes

- README: a section on Results and prompts: what a Result is, where harrow
  shows one, what `x`, `P` and `split` do, and the cairn each one needs.
- CHANGELOG.md: one entry for the milestone.
- The man page and `--help` list whatever the keymap generates, so nothing is
  hand-written there. Check that holds.

## Done when

- [ ] The README says what each piece needs from cairn and what happens without it
- [ ] CHANGELOG.md has the milestone's entry
- [ ] `harrow man` lists the new commands, generated rather than written by hand
