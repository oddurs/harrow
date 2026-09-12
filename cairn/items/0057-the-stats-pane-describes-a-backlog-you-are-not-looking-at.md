---
id: 57
title: The stats pane describes a backlog you are not looking at
type: bug
status: backlog
milestone: v0.4
depends_on:
- 56
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: chrome
---

## What happens

```rust
pub fn stats(&self) -> Stats {
    let work: Vec<&Item> = self.items.iter().filter(|i| !i.container).collect();
```

`!container` and nothing else. The filter never reaches it.

So: type `/priority=p0`, watch the list narrow to three rows, press `tab`
twice, and read *40 items · 12 ready · 3 in flight*. Numbers about a backlog
you are not looking at, on a screen that still shows the filter in its
header, with nothing saying the pane below has ignored it.

This is worse than a stats pane that never had a filter. A reader who has
just narrowed the list has every reason to believe the next screen is about
what they narrowed to.

## What should happen

The filter applies, the way it applies to the list and the board. One
backlog, one filter, three lenses.

If a whole-backlog overview turns out to be worth having — and it might, the
question is open in 0056 — then it is a deliberate mode of this lens with a
key and a line on screen saying which of the two you are reading. Not an
accident of which function was given a `query` and which was not.

## Reproduction

1. `/priority=p0`, enter.
2. `tab`, `tab`.
3. The totals are the unfiltered ones.

## Acceptance criteria

- [ ] Narrowing the list narrows the stats
- [ ] The strip, the list, the board and the stats agree about how many of anything there are
- [ ] A test holds that agreement, rather than three call sites remembering to filter
