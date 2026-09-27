---
id: 9d1e8be3-a12d-4cc6-9ffe-47a002051b99
title: Sorting is a key, not a flag
type: feature
status: done
milestone: v0.6
depends_on:
- 449588bb-ac84-4170-abea-759389f5cfb1
created: 2026-09-18
updated: 2026-09-18
priority: p1
area: chrome
---

## Problem

You cannot sort. `--sort` exists as a flag, `App::sort` exists as state, and
`sort_keys()` reads it on every rebuild:

```rust
fn sort_keys(&self) -> Vec<crate::filter::SortKey> {
    if !self.sort.trim().is_empty() {
        return crate::filter::parse_sort(&self.sort);
    }
    let mut spec = String::from("status");
    ...
}
```

Nothing but argv ever sets it. Forty-eight commands and not one of them is
`Sort`. To look at the same backlog oldest-first you quit and start again with
a flag — which also throws away the filter you typed, the marks you set and
the item you were reading.

Of the three questions a backlog answers — which items, in what order, grouped
how — this is the one with no door at all.

## Proposal

`S` puts the cursor in the sort segment of the view line (0082). `s` stays
status; the shift pair is the same idiom `c`/`C` already uses for claim and
release.

The fields come from the schema, not from a list here — a project that tracks
`severity` sorts by severity — and the completion offers them the way the
grouping cycle already offers its own. A leading `-` reverses, which is what
`parse_sort` already accepts, so the segment is the flag's own syntax and
nothing new has to be learned or documented twice.

Two smaller things fall out of having somewhere to put them:

- **The default becomes visible.** `status,priority,id` is what every reader
  has been getting and none has been told. Stating it is most of the value.
- **`S` on a board column heading sorts by that column**, which is the
  gesture the mouse would expect anyway.

## Cost

Sorting is cheap — `compare()` already exists and is already the hot path that
0042 is about. A sort typed into a segment rebuilds on every keystroke the way
the filter box does, so if 0042 has not landed this makes the same rebuild
cost more visible. That is an argument for landing them in either order, not
for holding this one back.

## Acceptance criteria

- [x] A key sets the sort order, and the list reorders without restarting
- [x] The fields offered come from the project's schema
- [x] `-` reverses, matching what `--sort` already accepts
- [x] The default order is stated rather than implied
- [x] Sorting survives a lens change, the way the filter does

## 2026-09-18

A typed segment rather than a picker, because `--sort`'s syntax is already the answer and a picker cannot express `-priority,updated`. The completion offers the project's fields as you type, so the syntax is documented by the thing that uses it.

It previews as you type, the way the filter box does, and `esc` puts the order back — the preview changed what is on screen, so a `esc` that kept it would be indistinguishable from `↵`. `S` opens on the order in force rather than empty: you are editing an order, not starting one.

A sort key naming a field the project has not got now says so, the same way an unknown filter field does since 0056. It was the last place left where harrow silently did something other than what it was asked.

Sorting keeps the cursor on the item it was on, which is the difference between reordering a list and losing your place in it.
