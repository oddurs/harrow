---
id: 119
uid: 20946747-dd8e-4480-9b96-ac2df9127b47
title: Show what a finished item concluded
type: feature
status: backlog
milestone: v0.8
depends_on:
- 118
created: 2026-09-27
updated: 2026-09-27
priority: p0
effort: m
area: chrome
---

## Problem

Once items carry Results, harrow shows each one as just another heading inside
Body, below Acceptance, where it is scrolled past. And the detail pane shows
an item's dependencies only while they block it ("Waiting on"). It never shows
what a finished dependency concluded, which is exactly the part `cairn prompt`
hands an agent (cairn 0156, layer 3).

## Proposal

- **Result, hoisted.** In the detail pane and the reader, a closed item with a
  Result shows it as its own section directly under the state lines, above
  Latest. It is removed from Body so it is not printed twice; `hoisted_lines`
  in `ui.rs` already does this for the acceptance criteria.
- **Builds on.** A section listing each direct dependency that is finished:
  reference and title as a link (like Waiting on), then its Result wrapped to
  three lines, or its last dated note where it has none, or nothing.
  Unfinished dependencies stay under Waiting on and are not repeated. This is
  the order `cairn prompt` uses, so the pane and the prompt read the same.

## Acceptance criteria

- [ ] A closed item with a Result shows it first, above Latest, and Body does not repeat it
- [ ] Each finished dependency shows its Result, else its last dated note, else only its title
- [ ] An unfinished dependency stays under Waiting on and does not appear under Builds on
- [ ] A dependency's title in Builds on reaches that item by click and by `↵`, as Waiting on does
- [ ] An item with no dependencies and no Result renders unchanged: the existing snapshots pass untouched
- [ ] One recorded screen holds the new sections
