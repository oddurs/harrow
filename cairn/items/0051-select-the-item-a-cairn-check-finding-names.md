---
id: 51
title: Select the item a cairn check finding names
type: feature
status: backlog
milestone: v1.0
depends_on:
- 38
created: 2026-09-12
updated: 2026-09-12
priority: p3
area: chrome
---

## Problem

0038 put `cairn check` in the diagnostics overlay, and the value of a
validator inside the tool is that the failure and the item should be one
keystroke apart. They are not: the findings are lines of text you then go and
find by hand.

## Why it was not done there

cairn prints `check` as prose for a person to read. Recovering an identifier
from it means matching whatever shape today's message happens to have, which
is harrow guessing at a format cairn has never promised — and a guess that
works until the wording improves is worse than no feature, because it fails
silently and looks like a harrow bug.

## Proposal

Wait for a machine-readable answer. `cairn check --json`, or any stable
locator in the output, and this becomes a line of code.

Until then the overlay says what cairn said, which is honest and is most of
the value.

## Acceptance criteria

- [ ] A finding that names an item selects it
- [ ] The identifier comes from something cairn promises, not from parsing prose
