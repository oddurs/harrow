---
id: 6
title: Group the backlog the way the project is organised
type: feature
status: done
milestone: v0.1
created: 2026-09-08
updated: 2026-09-08
priority: p0
area: chrome
---

## Problem

`cairn list` is a table and `cairn board` is four columns. Neither shows a
backlog as the shape it actually has: work under the milestone it ships in,
under the area it touches, under whoever holds it.

## Proposal

Group the list by any field the project has, cycling with `v`. Milestones are
headings rather than rows, carrying the progress that rolls up to them.

## Acceptance criteria

- [x] The axes offered are the ones this project actually declares
- [x] A milestone is a heading, not a row underneath one
- [x] Group order follows the declared order — statuses, enum values, due dates
- [x] Counts and progress cover what the filter is hiding
