---
id: 126
uid: d438a38a-498c-413e-8855-d680982cc58b
title: Say what reading items as prompts added, in the README and the changelog
type: chore
status: done
milestone: v0.8
assignee: oddurs
depends_on:
- 119
- 121
- 123
- 124
- 125
created: 2026-09-27
updated: 2026-09-28
closed_at: 2026-09-28
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

- [x] The README says what each piece needs from cairn and what happens without it
- [x] CHANGELOG.md has the milestone's entry
- [x] `harrow man` lists the new commands, generated rather than written by hand

## 2026-09-28

Written: a README section, Results and prompts — what a Result is, where harrow shows one (first in the pane and the reader; Builds on), and a table of x, P, :split and C with the cairn each needs, how each degrades, and a pointer to COMPATIBILITY.md; the README's keys table rebuilt from the default keymap (it had drifted: ctrl-k and D are by name only now, h/l move between groups, statuses step on < and >), and its durability line about subprocess deadlines corrected for writes. CHANGELOG [Unreleased] gains the milestone under Added (Results and prompts, the link keys, the man page) and Fixed (writes no longer killed, check findings shown, the reader's clipped words). Checking criterion 3 found the man page listed no keys at all, new or old; harrow man now has a KEYS section generated from Keymap::default().help_sections(), plus every command with no key under 'By name, from the palette', and tests/cli.rs holds every help row and every keyless command to it.

## Result

The README says what a Result is, where harrow shows one, what x, P, :split and C do and which cairn each needs; CHANGELOG has the milestone's entry; and harrow man lists every key and command, generated from the bindings rather than written by hand.
