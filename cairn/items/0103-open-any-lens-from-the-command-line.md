---
id: 103
title: Open any lens from the command line
type: feature
status: planned
milestone: v0.7
created: 2026-09-23
updated: 2026-09-23
priority: p1
area: cli
effort: m
part_of:
- 102
---

## Problem

`harrow --help` offers `--board` and `--stats`. There is no flag for the
needs-you queue or the log, so the two lenses that answer *what needs me* and
*what changed* can only be reached by starting the program and pressing `1` or
`5`. The lens contract in `tests/lenses.rs` holds all five lenses to the same
capabilities inside the program; outside it, two of them do not exist.

`--view` selects a saved view, which is a different axis: a view is a query,
a lens is an arrangement. Neither substitutes for the other, and asking for
`--view next` on the needs lens is a reasonable thing to want.

## Proposal

One flag that names a lens, with the existing two kept as the aliases they
already are:

    harrow --lens needs|list|board|stats|log

`--board` and `--stats` continue to work and are documented as shorthand, so
no existing invocation, script, or line of documentation breaks.

`Pane::ALL` and `Pane::name()` already exist and already spell these names;
the flag reads from them rather than repeating the list, so a sixth lens gets
its door by existing.

## Acceptance criteria

- [ ] `--lens <name>` opens every lens `Pane::ALL` holds
- [ ] `--board` and `--stats` keep working and are documented as aliases
- [ ] An unknown lens name fails with a usage error naming the five
- [ ] The flag takes its names from `Pane`, so a new lens needs no second list
- [ ] `--lens` and `--view` compose
