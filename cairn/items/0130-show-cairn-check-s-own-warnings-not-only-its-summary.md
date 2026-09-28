---
id: 130
uid: 7352b7b9-83f3-448b-a3b6-b743f76204a6
title: Show cairn check's own warnings, not only its summary
type: bug
status: backlog
milestone: v0.8
created_by: cairn-26
depends_on:
- 125
created: 2026-09-28
updated: 2026-09-28
priority: p2
area: chrome
---

## Problem

`C` runs `cairn check` and shows what it printed on stdout, which is the one
summary line (`ok: 9 item(s), 6 warning(s)`). The warnings themselves go to
stderr, and harrow drops them, so the overlay says there are six warnings and
shows none. When the check fails outright, only the first stderr line is
shown. Found while building 0125, which picks the `: prompt: ` lines out of
that stderr and had to leave the rest alone to keep the overlay unchanged
without `--prompts`.

## Proposal

Show cairn's other stderr findings under the cairn check heading, as cairn
wrote them, capped with a count of the rest, as the Prompts section is.

## Acceptance criteria

- [ ] A warning cairn check prints is shown in the overlay, word for word
- [ ] A failed check shows cairn's findings, not only the first line
