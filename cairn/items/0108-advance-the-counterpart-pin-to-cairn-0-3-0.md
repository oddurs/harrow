---
id: 108
title: Advance the counterpart pin to Cairn 0.3.0
type: chore
status: done
milestone: v1.0
created: 2026-09-23
updated: 2026-09-23
closed_at: 2026-09-23
priority: p2
area: testing
effort: s
---

## Why now

Cairn released 0.3.0 and pins harrow at `e7397a5`. Harrow pinned Cairn 0.2.2.
The pair was asymmetric and a release behind, which 0100 recorded with the
evidence that harrow at that revision already agreed with 0.3.0.

## What changes

The CI revision, `COMPATIBILITY.md`, and the corpus `PROVENANCE`. Nothing
else: refreshing the vendored corpus from the 0.3.0 tag changed **only**
`PROVENANCE`. Every fixture is byte-identical between 0.2.2 and 0.3.0, which
is format 3 being unchanged between the two releases, as Cairn's own
compatibility note says.

## What the refresh nearly vendored

The first refresh read the sibling checkout rather than the release. That
working copy held uncommitted work toward `1.0.0-alpha.1` — 73 files, a
`uuid` dependency, and a deleted `0010-id-from-filename` case, which is
0067's identity question being implemented. Refreshing from it would have
vendored an unreleased corpus under a released version's name and deleted a
golden case for a format nobody has shipped.

That also invalidated a verification claim made earlier in this session:
`../cairn/target/debug/cairn` reports `1.0.0-alpha.1`, so agreement runs
pointed at it tested harrow against unreleased Cairn while reporting 0.3.0.
CI gated those merges against the pinned revision and passed, and the
`--doctor` runs used the installed 0.3.0 and were accurate; the local
agreement claim was not. Re-run here against a clean clone of the tag, whose
binary reports 0.3.0: four tests pass.

`COMPATIBILITY.md` now says to refresh from a clean checkout of the release
and to check the version of the binary being tested, because a sibling
checkout is somebody's desk.

## Done when

- [x] The corpus comes from the 0.3.0 tag, and the diff is `PROVENANCE` alone
- [x] CI pins the 0.3.0 revision
- [x] Agreement passes against a binary that reports 0.3.0
- [x] The document says how to avoid vendoring a working copy

## What this does not do

Cairn still pins harrow at `e7397a5`, which is now several commits behind:
0106 changed a shared filter semantic and v0.7 added `--lens`. Advancing that
pin is Cairn's change to make in Cairn's repository, with its own review of
both projects' contract changes. It is not done here.
