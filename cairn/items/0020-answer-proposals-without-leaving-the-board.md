---
id: 20
title: Answer proposals without leaving the board
type: feature
status: done
milestone: v0.2
created: 2026-09-08
updated: 2026-09-11
priority: p2
area: write
---

## Problem

`cairn propose` exists so an agent can ask for a change a person decides.
Proposals are exactly the sort of thing that piles up unread, and the place to
read them is next to the item they are about.

## Proposal

Show that an item has a proposal against it, and accept or decline it from the
detail pane.

## Acceptance criteria

- [x] An item with a proposal is visible as such in the list
- [x] Accepting runs cairn's own command rather than writing the change directly
