---
id: 87
title: Reach the views the project declared
type: feature
status: done
milestone: v0.6
depends_on:
- 82
created: 2026-09-18
updated: 2026-09-18
priority: p2
area: chrome
---

## Problem

A project declares its own views:

```toml
[[view]]
name = "triage"
description = "Items that still need a milestone or a priority"
filter = "milestone=,category!=done"
```

harrow reads them. `--doctor` parses every one and reports any it cannot
evaluate (0056). `--view triage` opens on one. And once harrow is running
there is no way to reach any of them — you quit and start again with a flag,
losing the marks and the item you were reading.

These are the queries a project decided were worth naming. They are the best
filters in the repository and the interface cannot get at them.

## Proposal

`V` swaps the whole view line (0082) for a declared view: its filter, and its
sort where it names one.

`v` and `V` pair honestly — both answer *how am I looking at this*, one by
rearranging and one by replacing. The list comes from the schema, with each
view's `description` beside its name, because a project that bothered to write
one has said what the view is for better than harrow can infer it.

After swapping, the line shows the resulting grammar rather than the view's
name — or rather, it shows both: `▸ triage · milestone=,category!=done`. The
name is what you chose; the grammar is what you got, and editing any clause
drops the name, because it is no longer that view.

## Cost

Small, and mostly already paid — `Schema` holds the views, `--view` already
resolves one, and 0056 already validates them. What is new is the picker and
the rule about when a view stops being itself.

## Acceptance criteria

- [x] A key lists the project's saved views, with their descriptions
- [x] Choosing one applies its filter and its sort
- [x] The view in force is named on the line, and named only while it is unedited
- [x] A view whose filter does not parse says so, the way `--doctor` does, rather than matching nothing

## 2026-09-18

Everything a view needs was already there — `reparse_filter` ANDs its filter, `follow_view` adopts its grouping, `esc` puts it back. Except its **sort**, which nothing had ever applied: until `S` landed there was no way to order a backlog at all, so a view naming one was declaring something harrow could not do. It applies now.

It replaces rather than narrows. `--view now -f priority=p0` from the command line ANDs, and that is right for a flag somebody typed deliberately; picking one from inside on top of a half-typed filter would be looking at something nobody described.

The name stays on the line after you narrow it further, which is a deviation from the criterion as I wrote it. The criterion said the name should be dropped once edited. It should not: the view is still in force and still ANDed, so dropping the name would hide a clause rather than a label. The line shows `now · category=active,priority=p1` — what you chose, and what you got.

The picker's note column starved the label: a project's description of a view is a sentence, and every name came out as an ellipsis. It now gives ground only when the label would fall below eighteen columns, so the status picker's right-aligned categories are unchanged.
