---
id: 125
uid: 586ebbdf-b301-4924-914e-fa9c9fcc6ec1
title: Show prompt checks where cairn check is shown
type: feature
status: backlog
milestone: v0.8
depends_on:
- 122
created: 2026-09-27
updated: 2026-09-27
priority: p2
effort: s
area: chrome
---

## Problem

`cairn check --prompts` reports open items that an agent will misread (cairn
0158): no criteria, no context, a finished dependency with no Result, and
ticked criteria with no note. Harrow's diagnostics overlay runs `cairn check`
on `C`, and it would not show any of these.

## Proposal

Where the cairn on PATH supports it, `C` runs `cairn check --prompts`, and
the overlay shows the prompt findings under their own heading, **Prompts**, as
warnings. Advice must not read as a broken backlog. Harrow does not evaluate
the checks itself: they are cairn's rules, and a second implementation drifts
from the first. That is the lesson of the filter grammar.

Deferred, with the reason: putting prompt findings in the needs queue. On an
item you own, "this will be misread" is arguably a question for you. But
nobody has lived with the checks yet. Revisit once they have run on this
backlog for a milestone.

## Acceptance criteria

- [ ] With support, the diagnostics overlay shows prompt findings under Prompts, as warnings
- [ ] Without support, the overlay is unchanged cell for cell
- [ ] No harrow code evaluates a prompt check; every finding shown is cairn's output
