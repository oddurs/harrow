---
id: 7dceda4a-8de3-415b-8553-025511cd1941
title: Read immutable UUID identities and keep the Cairn contract
type: feature
status: done
assignee: codex
created_by: codex
created: 2026-09-23
updated: 2026-09-23
closed_at: 2026-09-23
priority: p1
area: read
---

## Problem

Cairn format 4 replaces sequential numbers with immutable UUIDv4 identities. Harrow must remain an independent reader, preserve old-format access, and send full identities when delegating writes.

## Proposal

Read canonical UUID identities and declared ID references directly; display unambiguous short prefixes, resolve the frozen migration alias map, and retain selection, marks, history and undo by full identity. Refuse interrupted migrations and malformed or ambiguous identities. Mirror the versioned corpus and agreement contract. Keep file reads independent of the Cairn executable.

## Acceptance criteria

- [x] Format 4 and historical formats read correctly, including frozen numeric aliases and ambiguous prefixes.
- [x] Every write, history request and undo carries full immutable identity; selection and marks survive reloads.
- [x] Conformance, agreement, snapshots and the full project checks pass.
- [x] The project backlog is migrated in a dedicated history-preserving commit and the installed pair works together.

## 2026-09-23

Direct-reader and state changes are underway: full UUID identities now key items, dependencies, selection, marks, history, undo, and write arguments; display prefixes and numeric counts are separate. Format-aware parsing rejects incomplete format-4 identities, the reader stops on a pending migration journal, and prefix filters surface ambiguity. The core compiles; fixture conversion and UUID-specific regressions are still in progress. No live project migration or installation has occurred.

## 2026-09-23

All standalone checks passed, including the 49-case format corpus and seven UUID identity regressions. Four explicit paired agreement tests passed against Cairn format 4. Existing legacy UI snapshots are unchanged. Before the live migration, integrate newer main work and advance the reciprocal exact CI pins.

## 2026-09-23

Integrated newer main, retaining the new lens entry points and ordinary-listing semantics. The pre-migration integer collision on 0107 was repaired to 0110 before UUID conversion; no references to this new item existed. All five cross-tool agreement cases pass. Every plain-output lens now has a full-UUID regression. The counterpart pin and unmodified 49-case corpus identify the committed Cairn alpha revision, and refresh now refuses dirty upstream trees.

## 2026-09-23

Live migration verified all 110 bodies byte-for-byte and every non-reference metadata value unchanged. Historical lookup, full-UUID references and all five agreement tests pass; doctor agrees with Cairn on all 110 records and six views. Pre-push exposed a lens-alias screenshot flake: only the read-only footer differed because one host Cairn version probe timed out. The equivalence test now fixes writer availability using an explicitly absent executable and asserts both exit codes; the complete screenshot comparison is unchanged.

## 2026-09-23

Installed matching Cairn 1.0.0-alpha.1 and Harrow 0.2.0-alpha.1; doctor verifies every migrated record and saved view in both projects. Unlinked the older Homebrew Cairn to remove PATH shadowing while keeping that stable installation for recovery. Both migration ranges contain zero semantic item changes, and historical numeric lookup still reaches the pre-migration records. Full standalone checks, unchanged legacy snapshots, all five agreement cases and the ordinary pre-push gate pass. This is an unreleased development pair, not a stable 1.0 promise.
