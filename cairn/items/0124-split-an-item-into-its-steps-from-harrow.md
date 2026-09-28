---
id: 124
uid: 0fba71a1-0a86-4ea1-a151-e938f3a27aec
title: Split an item into its steps from harrow
type: feature
status: backlog
milestone: v0.8
depends_on:
- 122
created: 2026-09-27
updated: 2026-09-27
priority: p2
effort: s
area: write
---

## Problem

`cairn split <ID>` turns an item's numbered Approach into child items that
depend on each other in order (cairn 0157). A person triaging in harrow is
the one who notices an item is three items, and would have to leave harrow to
split it.

## Proposal

A palette command `split`, with no default key because it creates items in
bulk. It runs `cairn split <ID> --dry-run` through the shell and shows the
children cairn names in the confirm box. `y` runs `cairn split <ID>`, and the
children arrive with the next reading like any other new items. Cairn's
refusals ("no numbered steps", "already split") reach the footer verbatim.
Offered only where `cairn split --help` succeeds.

## Acceptance criteria

- [ ] The confirm lists every child the dry run names before anything is written
- [ ] `y` writes through `cairn split`, and the children appear as rows on the next reading
- [ ] Cairn's refusal text reaches the footer unchanged
- [ ] Against a cairn without `split`, the palette does not offer it
