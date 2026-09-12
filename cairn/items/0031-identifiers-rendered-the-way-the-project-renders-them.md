---
id: 31
title: Identifiers rendered the way the project renders them
type: feature
status: done
milestone: v0.3
created: 2026-09-12
updated: 2026-09-12
priority: p2
area: read
---

## Problem

harrow renders every identifier as a zero-padded number, from
`project.id_width`. cairn 0.2.0 declares `project.id_format`, a template:
`MP-{n}` gives `MP-1002`, `A{n}` gives `A24`, `{n:04}` gives the padding
harrow already does. `id_width` is the older spelling of the same thing.

Specification §4.2 is careful about what this is: a rendering, a property of
the project, never of the item. The value under `id` is the integer whatever
the template says. So the whole of adopting it is display — and input:

> A reader that accepts identifiers as input **should** accept both the
> rendered form and the bare integer.

harrow accepts identifiers in the filter box and in free-text search, and
`Item::matches` compares against `self.id.to_string()` and the rendered
reference. With a prefix in play, both forms have to work.

## Proposal

Teach `Schema` the template, fall back to `id_width` when it is absent, and
render through it everywhere `format_id` is called today. Accept both the
rendered and the bare form wherever harrow takes an identifier from a person.

## Cost

`format_id` is on the hot path of every row of every frame, so the template
compiles once at load rather than being parsed per call — which is what cairn
does with it too.

## Acceptance criteria

- [x] A project declaring `id_format = "MP-{n}"` shows `MP-1002` in the list, the board, the detail pane and the reader
- [x] `id_width` still works, and means `{n:0W}`
- [x] Both `MP-1002` and `1002` find the item in the filter
- [x] A template that does not compile is a warning and the padding is used, rather than a refusal to open
