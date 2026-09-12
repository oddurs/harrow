---
id: 27
title: Coerce the values the format says a reader must accept
type: bug
status: backlog
milestone: v0.2
depends_on:
- 39
created: 2026-09-12
updated: 2026-09-12
priority: p0
area: read
---

## What happens

The specification names four places where a reader **must** accept a value
written in a shape cairn itself would not write, because people, editors and
other tools write them. harrow accepts none of the four.

| Written | §  | harrow reads |
| --- | --- | --- |
| a `...` line closing the frontmatter | §3 | never closes: the whole body is swallowed as frontmatter |
| `labels: auth, backend, ops` | §4 | one label, `"auth, backend, ops"` |
| `depends_on: [#3, 4]` | §4 | `[4]` — the `#`-prefixed element is dropped |
| `depends_on: 3, 4` | §4 | nothing |
| `labels: [0x1F, 1.20]` | §6 | `["0x1F", "1.20"]`, not `["31", "1.2"]` |

The last is YAML 1.2 core scalar resolution, which §6 names explicitly and at
length, because "several widely used YAML libraries still implement 1.1 by
default". harrow's parser implements neither: it takes the text as it stands.

## Proposal

Fix the first four, which lose information a file plainly carries.

Take §6 as far as the core schema's integer and float forms — `0x1F`, `0o17`,
`1.20`, `1e3` — and no further. harrow renders values into a pane; it does not
compute with them, so resolving `12:30` correctly (as the string it already
is) costs nothing and the numeric forms are the only ones where harrow
currently shows something the file does not say.

## Cost

Each of these makes the parser accept more, and a parser that accepts more can
accept something wrong. The mitigation is 0039, the
conformance corpus: it fixes the expected reading of every one of these, so the coercions
are held to a table somebody else wrote rather than to my reading of the
prose.

## Acceptance criteria

- [ ] `...` closes the frontmatter, and the body after it is the body
- [ ] A single-string `labels` is split on commas, with surrounding whitespace discarded
- [ ] `depends_on` accepts a comma-separated string and a leading `#` on each element
- [ ] Integers and floats resolve under the YAML 1.2 core schema
- [ ] `no`, `yes`, `on`, `off` and `12:30` stay the strings they are
