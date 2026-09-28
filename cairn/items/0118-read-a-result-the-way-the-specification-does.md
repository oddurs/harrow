---
id: 118
uid: d1c60079-a1d3-4415-8fc3-bc16e140e00f
title: Read a Result the way the specification does
type: feature
status: backlog
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

- [ ] `## Result`, `### result` and `# RESULT` each read as the section beneath them
- [ ] A `# comment` inside a fenced block does not end the section
- [ ] The section ends at the next heading at its level or above, and a deeper heading stays inside it
- [ ] An empty or whitespace-only section is no Result
- [ ] Every case cairn's own tests give `result()` on that branch reads the same here, as a unit test naming the case
