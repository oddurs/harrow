---
id: 110
title: Read immutable UUID identities and keep the Cairn contract
type: feature
status: doing
assignee: codex
claimed: 2026-09-23
created_by: codex
created: 2026-09-23
updated: 2026-09-23
priority: p1
area: read
---

## Problem

Cairn format 4 replaces sequential numbers with immutable UUIDv4 identities. Harrow must remain an independent reader, preserve old-format access, and send full identities when delegating writes.

## Proposal

Read canonical UUID identities and declared ID references directly; display unambiguous short prefixes, resolve the frozen migration alias map, and retain selection, marks, history and undo by full identity. Refuse interrupted migrations and malformed or ambiguous identities. Mirror the versioned corpus and agreement contract. Keep file reads independent of the Cairn executable.

## Acceptance criteria

- [ ] Format 4 and historical formats read correctly, including frozen numeric aliases and ambiguous prefixes.
- [ ] Every write, history request and undo carries full immutable identity; selection and marks survive reloads.
- [ ] Conformance, agreement, snapshots and the full project checks pass.
- [ ] The project backlog is migrated in a dedicated history-preserving commit and the installed pair works together.

## 2026-09-23

Direct-reader and state changes are underway: full UUID identities now key items, dependencies, selection, marks, history, undo, and write arguments; display prefixes and numeric counts are separate. Format-aware parsing rejects incomplete format-4 identities, the reader stops on a pending migration journal, and prefix filters surface ambiguity. The core compiles; fixture conversion and UUID-specific regressions are still in progress. No live project migration or installation has occurred.

## 2026-09-23

All standalone checks passed, including the 49-case format corpus and seven UUID identity regressions. Four explicit paired agreement tests passed against Cairn format 4. Existing legacy UI snapshots are unchanged. Before the live migration, integrate newer main work and advance the reciprocal exact CI pins.
