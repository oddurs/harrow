---
id: a2d88226-f059-4cbf-87bb-338d25bdf39b
title: 'Read Cairn format 5: numbers, tags and type renderings'
type: feature
status: done
assignee: Oddur Sigurdsson
created: 2026-09-27
updated: 2026-09-27
closed_at: 2026-09-27
priority: p0
area: read
effort: m
---

## Why now

Cairn's format-5 PR (oddurs/cairn#105, item 49543fc8) returns ids to numbers
and moves each UUID to a `uid` tag. Cairn's agreement gate runs the pinned
Harrow on every PR, and Harrow refused format 5, so #105 cannot merge green.
Cairn item 0147 (c7ac551e) makes this step 1 of 3, at p0.

## What changes

- `KNOWN_FORMAT` is 5. Items carry `uid`; a malformed tag is refused.
- Types may declare `id_format`. `format_id` renders a number in its type's
  template; `parse_id` reads what Cairn reads, in its order: all digits is a
  number, a type's prefix must match the item's type (`BUG-13` for a feature
  is refused), then the project's rendering, then a full tag or an 8+ hex
  prefix with a letter. An ambiguous prefix is refused.
- Only format 4 demands UUIDs. A UUID left in a format-5 file still reads.
- A number used twice in format 5 names `cairn renumber`; one tag on two
  items is reported as a copy.
- Worktree copies are matched by tag first, so a renumbered copy still pairs,
  and the same number on a differently tagged item is not treated as a copy.
- The corpus is refreshed from Cairn c5435a7 (63 cases, formats 1-5); CI pins
  that revision; the format-4 migration agreement test now migrates to 5 and
  compares numbers, renderings, full tags and tag prefixes.

## Done when

- [x] The format-5 corpus reads as Cairn says, with frozen formats 1-4 unchanged
- [x] Agreement passes against format-5 Cairn, including Cairn's own migrated backlog
- [x] Tag queries agree, and the test fails when tag resolution is removed
- [x] `scripts/task check` passes

## 2026-09-27

Conformance: 63 cases from Cairn c5435a7 read as expected, formats 1-4 frozen corpora unchanged. Agreement 5/5 against a scratch build of c5435a7, CAIRN_PROJECT_DIR the format-5 Cairn checkout (its own backlog is migrated to 5). Removing tag resolution fails migrated_numbers_renderings_and_tags_agree with 'is not the tag of any item'. scripts/task check: 558 passed. Not done here: showing uid on screen (the spec says it is never shown by default), and resolving a UUID left in a format-5 depends_on to its number (cairn renumber's job).
