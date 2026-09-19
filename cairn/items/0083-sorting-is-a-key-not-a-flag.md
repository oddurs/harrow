---
id: 83
title: Sorting is a key, not a flag
type: feature
status: backlog
milestone: v0.6
depends_on:
- 82
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

- [ ] A key sets the sort order, and the list reorders without restarting
- [ ] The fields offered come from the project's schema
- [ ] `-` reverses, matching what `--sort` already accepts
- [ ] The default order is stated rather than implied
- [ ] Sorting survives a lens change, the way the filter does
