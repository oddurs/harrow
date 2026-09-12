---
id: 39
title: Hold the reader to cairn's own conformance corpus
type: chore
status: done
milestone: v0.2
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: testing
---

## Problem

harrow is a second implementation of somebody else's file format. It has tests
for what it parses, all of them written by the same person who wrote the
parser, against the same reading of the prose.

cairn ships a conformance corpus for exactly this — `tests/golden`, one `.md`
beside the `.json` it must parse to, deliberately containing "files a writer
would not produce — bare strings where a sequence belongs, a missing id, CRLF
endings, keys from a version that does not exist". Specification §9 offers it
as "a reasonable conformance suite for another implementation". Beside it is
one directory per format that has ever existed, holding items as that format
wrote them.

Running harrow's parser over it today finds a silent misreading nothing in
harrow's own suite catches (0026).

## Proposal

Vendor the corpus into `tests/fixtures/conformance/` with a note saying where
it came from and at what cairn version, and a test that reads each `.md`,
parses it, and compares against the `.json` field by field for the keys harrow
models.

Vendored rather than read from a sibling checkout: the suite has to pass on a
machine that has never heard of cairn, which is the same reason harrow reads
the files rather than shelling out. A small script to refresh it, and the
version it was taken at recorded beside it, so the drift is visible rather
than silent.

Skip nothing quietly. Where harrow deliberately reads less than cairn writes —
it models no `source`, and the body's meaning is a project convention — the
test says so in the assertion rather than omitting the field.

## Cost

A vendored corpus goes stale, and a stale conformance suite is worse than none
because it reports conformance with a format that has moved. The remedy is the
recorded version and the refresh script, plus the format-N directories, which
are by definition frozen.

## Acceptance criteria

- [x] Every file in the current corpus parses to the values recorded beside it
- [x] The per-format corpora parse too, so a project written by an older cairn still reads
- [x] The corpus records which cairn version it was taken from
- [x] A refresh is one command, and its diff is reviewable
