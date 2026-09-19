---
id: 87
title: Reach the views the project declared
type: feature
status: backlog
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

- [ ] A key lists the project's saved views, with their descriptions
- [ ] Choosing one applies its filter and its sort
- [ ] The view in force is named on the line, and named only while it is unedited
- [ ] A view whose filter does not parse says so, the way `--doctor` does, rather than matching nothing
