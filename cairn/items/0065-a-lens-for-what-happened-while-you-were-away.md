---
id: 65
title: A lens for what happened while you were away
type: feature
status: done
milestone: v0.5
depends_on:
- 64
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: chrome
---

## Problem

The second question a person asks on sitting down — after *what needs me* —
is *what happened while I was away*. harrow's answer is a dot beside rows
that changed in the last forty-five seconds, which is a hint, not a record,
and is gone by the time you have read the screen.

The information exists. Every item is a file in a git repository, every
change is a commit, and cairn already proves it is willing to read that:
`cairn log <ID>` reports how one item got the way it is, out of the
repository's own history. What it has nothing of is the project-wide view,
which is the one a supervisor wants.

## Proposal

A lens that is reverse-chronological rather than structural: who changed
what, when, and the note they left, most recent first.

Out of git, which is where it is. One `git log` over the items directory,
parsed into (when, who, what changed, which items) — the same shape
`cairn log` produces for one item, over all of them.

Two things it must get right:

**The unit is a change, not a commit.** A commit that moves four items is
four rows, because the reader is asking about items.

**It says who, and who is often not a person.** An agent's writes carry its
name, and telling *I did that* from *something else did that* is most of the
value.

Selecting a row selects its item. `enter` opens that item's own history,
which is the overlay that already exists.

## Cost

Shelling out to git is a process, and this lens would want to do it on
arrival. It is a read, so it takes no lock, and it is bounded — the last
however-many revisions rather than the whole history. Cache it for as long
as the backlog has not changed; the watcher already knows when that is.

A repository with no history — a fresh `cairn init`, or a directory that is
not a repository at all — has to say so rather than look broken. harrow
already has that case for `cairn log`.

## Acceptance criteria

- [x] A lens shows what changed across the project, most recent first
- [x] One row per item changed, not per commit
- [x] It says who, and an agent is distinguishable from a person
- [x] Selecting a row selects that item; opening it shows that item's history
- [x] A project with no repository says so rather than appearing empty
- [x] It is read from git once and not on every frame
