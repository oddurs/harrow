---
id: 118
uid: d1c60079-a1d3-4415-8fc3-bc16e140e00f
title: Read a Result the way the specification does
type: feature
status: done
milestone: v0.8
created: 2026-09-27
updated: 2026-09-27
priority: p0
effort: s
area: read
---

## Problem

Cairn 0155 makes a `## Result` section what a finished item concluded. It is
specification §10.2, written on oddurs/cairn branch `feat/0155-result`. Harrow
parses item bodies itself. If it finds a Result differently from cairn, the
same item quotes two different conclusions in the two tools. That is the silent
disagreement the filter-grammar rule in CLAUDE.md exists to prevent.

## Proposal

`Item.result: Option<String>`, derived at load in `engine::derive` beside the
criteria counts, from a reader that follows §10.2 exactly:

- the section under the first heading whose text is `Result`, at any level,
  compared without regard to case, with trailing `#`s ignored;
- it runs to the next heading at the same level or above; a deeper heading
  stays inside it;
- a heading inside a ``` or ~~~ fence is code, not a heading;
- the text is trimmed, and an empty section is no Result.

Cairn's implementation is `Item::section_span`, `Item::result` and `headings`
in `src/item.rs` on that branch. Port its behaviour, not its code shape.

## Acceptance criteria

- [x] `## Result`, `### result` and `# RESULT` each read as the section beneath them
- [x] A `# comment` inside a fenced block does not end the section
- [x] The section ends at the next heading at its level or above, and a deeper heading stays inside it
- [x] An empty or whitespace-only section is no Result
- [x] Every case cairn's own tests give `result()` on that branch reads the same here, as a unit test naming the case

## 2026-09-27

Ported from cairn feat/0155-result as it stood on 2026-09-27 (uncommitted there): Item::section_span, Item::result and headings in src/item.rs. Same details, not only the spec's words: an indented heading counts, #s must be followed by a space, closing #s are trimmed, a fence closes only on the marker that opened it, the first Result heading wins. Cairn's one reading case (a_result_ends_at_the_next_heading_and_ignores_code) is ported byte for byte. Removing the fence rule or the same-level-or-above rule each fails it. Not yet checked against cairn itself; that is 0120's agreement gate, once 0155 merges. Nothing on screen yet; that is 0119.

## 2026-09-27

Cairn revised §10.2 after review (cairn 0155, a959263 plus uncommitted changes on feat/0155-result): a Result also ends at a note's heading whatever its level (a date, 'Released by', 'Proposed '); a fence never closed is not a fence and headings after it count; and §10.1 now says a Result contributes no acceptance criteria. All three ported, each with a unit test. tests/conformance.rs now compares 'result' (absent where there is none, like uid), so the vendored corpus holds the rule from the next refresh. Cairn's three new golden cases (result, result-empty, result-above-a-note) were copied in locally and pass, then removed: vendoring them is 0120's job, from the pinned merge commit. Dropping the note rule fails result-above-a-note.
