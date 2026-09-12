---
id: 28
title: Find items the way the specification says to find them
type: bug
status: backlog
milestone: v0.2
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: read
---

## What happens

The loader is one `read_dir` over the items directory. Specification §5:

> Subdirectories **must** be searched. Entries whose names begin with `.` or
> `_`, and `README.md`, are **not** items.

So a project that files its items in `cairn/items/archive/` shows harrow an
empty backlog, with no warning that there is anything to miss. And a project
with a `README.md` or a `_template.md` beside its items gets a diagnostic
warning per file per reload, for files the specification says are not items.

## Proposal

Walk the tree. Skip entries whose names begin with `.` or `_`, and
`README.md`, without a warning; every other `.md` file is an item and is
warned about if it does not parse.

The directory watcher in `runtime.rs` watches the items directory; check that
what it watches is recursive, or a change in a subdirectory will not be
noticed and the whole point of watching goes with it.

## Cost

A deep tree is more stat calls per reload. The backlogs this is for are
hundreds of files, not hundreds of thousands, and the reload is already
debounced behind the watcher.

## Acceptance criteria

- [ ] An item in a subdirectory of the items directory is loaded
- [ ] `README.md`, `.hidden.md` and `_template.md` are neither items nor warnings
- [ ] A change in a subdirectory is noticed by the watcher, not only by the next poll
