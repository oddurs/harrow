---
id: 5
title: Read a cairn project without shelling out
type: feature
status: done
milestone: v0.1
created: 2026-09-08
updated: 2026-09-08
priority: p0
area: read
---

## Problem

The obvious way to get a backlog is `cairn export --to json` and a JSON parser.
It is also a process per read, a dependency on cairn being installed to *look*
at a directory of Markdown, and a second thing to keep in step.

## Proposal

Read `cairn.toml` with the TOML crate and parse the item frontmatter directly,
the way quarry reads the kernel rather than parsing `lsof` back. Derive what the
files do not say — category, blockers, milestone rollups — once, over the set.

Writes still go through `cairn`: it owns the locking, the hooks and the rules
about what a valid item is, and a second writer would own none of them.

## Acceptance criteria

- [x] A project opens with cairn not installed at all
- [x] `harrow --doctor` asks cairn for a second opinion on the item count
- [x] A file that will not parse is reported, and the rest still load
- [x] An unknown frontmatter key is kept rather than dropped
