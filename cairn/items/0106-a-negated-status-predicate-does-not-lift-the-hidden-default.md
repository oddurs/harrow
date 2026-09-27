---
id: edcd6fc4-bcd3-4364-b801-75afaf66f321
title: A negated status predicate does not lift the hidden default
type: bug
status: done
milestone: v0.7
created: 2026-09-23
updated: 2026-09-23
priority: p1
area: filter
effort: m
---

## What happens

Harrow hides finished and dropped work unless asked, which is the right
default for a screen. It lifts that default when the filter itself mentions
status or category — but only when the predicate is an equality. A negation
does not lift it, so harrow and Cairn return different item sets for the same
saved view.

    filter                            cairn   harrow
    category=done                        78       78   agree
    status=done                          78       78   agree
    category!=dropped                    96       18   DISAGREE
    status!=dropped                      96       18   DISAGREE
    type=decision,category!=dropped       1        0   DISAGREE

Seventy-eight items — every closed one — differ on the third row. This is not
a corner: `category!=dropped` is the shape a project reaches for to mean
*everything that still counts*, and this repository's own `[render] include`
uses exactly it.

Found while configuring a `decisions` view in 0100. A decision is worth most
after it is settled, so the view has to include closed items; written the
obvious way, Cairn showed the decision and harrow showed nothing.

## What should happen

A filter that mentions `status` or `category` at all has spoken about
visibility, whatever the operator. Harrow should add no hidden default in that
case, which is what Cairn does and what the predicate plainly says.

The agreement suite did not catch this because Cairn's eight saved views
happen to use equalities. A fixture with a negated predicate belongs in the
suite, so the next disagreement of this shape fails a build instead of a view.

## Reproduction

1. In this repository: `cairn list --filter 'category!=dropped'` → 96 items.
2. `harrow --plain --filter 'category!=dropped'` → 18 items.
3. Replace `!=dropped` with `=done` in both: they agree.

## Acceptance criteria

- [x] A negated status or category predicate lifts the default, as an equality does
- [x] Harrow and Cairn return the same set for all five filters in the table
- [x] The agreement suite gains a negated-predicate fixture
- [x] `-a` and an explicit predicate still compose as they do now
