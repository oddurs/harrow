---
id: fcf9b803-3b17-4e44-b3d0-1fac1848814d
title: Show work under way in other worktrees as it happens
type: feature
status: done
created: 2026-09-24
updated: 2026-09-24
closed_at: 2026-09-24
priority: p1
area: read
effort: m
---

## Problem

Agents work one item per branch, each in a worktree of its own, and they run
`cairn claim`, `cairn set` and `cairn note` there. Each write lands in that
worktree's copy of the item. That is right for cairn, whose claims coordinate
writers in one directory, but no other checkout can see it. harrow, open on
the primary checkout, shows nothing until the branch merges and is pulled.

Seen in rim: four items in progress across seven worktrees, and on `main`
nothing in progress at all. With four agents working, the board looked idle.
It felt synced before only because branches merged every few minutes.

## Proposal

Immutable ids make this answerable: in format 4 an id names the same item in
every checkout. So on every reading, the loader asks git which item files each
other worktree has changed since it diverged from this one:

    git worktree list --porcelain
    git -C <other> diff --name-only --merge-base <our HEAD> -- <items dir>

It reads those files and attaches the difference to the matching item as a
derived fact, `elsewhere`, holding the branch, its status, its assignee and
its newest note.

The record is untouched. Status, filters, board columns, `--plain` and every
write stay this checkout's and keep agreeing with cairn. That is why this is
an annotation rather than an overlay. An overlay would put a claimed item in
the `doing` column here, while cairn in this checkout still says `planned`,
and a write from harrow would then land on a copy the branch is about to
replace.

On screen:

- an item under way on another branch turns, as work under way here does,
  in the colour of that branch's status. It stays in the column the record
  gives it, so a turning card under `planned` reads correctly: claimed, not
  yet merged
- the detail pane gets an *Elsewhere* section: the branch, the status it
  moved to there, who has it, and that copy's newest note
- the change is announced like any other: `96d2dac9 → doing ·
  perf/96d2dac9-chunks`

For liveness, each other worktree's items directory is watched like our own,
and git's worktree registry is watched without recursing into it. So a claim
arrives when it is written, and a worktree created a second later is watched
from its first write.

### What it costs, and what it leaves out

- One `git` process per other worktree per reading, about 20 ms each, run in
  parallel. With no git, no repository, or no other worktrees, the survey is
  empty and nothing changes.
- A worktree on a different format is skipped. Its ids do not name the same
  items as ours.
- Merge-base, not a plain comparison against our copy. A branch cut before
  `main` moved holds stale copies of items it never touched, and those must
  not read as work. A merged branch whose worktree was never removed drops
  out by itself, because its tip is now the merge-base.
- New items filed on a branch have no copy here to annotate, and are left
  out. They are planning, not someone picking work up. Showing them would
  mean items that cairn in this checkout cannot see or change.
- Filters cannot see `elsewhere`. The grammar is cairn's, and harrow does not
  grow private fields in it.

## Acceptance criteria

- [x] An item changed in another worktree since it diverged carries that
      worktree's branch, status, assignee and newest note. One that worktree
      never touched carries nothing, even when the copies differ.
- [x] Rows and cards turn for work under way elsewhere, and the record's
      status, the filters and `cairn list` still agree.
- [x] The detail pane names each branch, and what that branch changed.
- [x] A claim in another worktree reaches an open harrow through the watcher,
      and is announced.
- [x] `--doctor` says how many other worktrees it read, and how many items
      are under way in them.
- [x] No git, no repository, and a worktree on another format each leave the
      reading exactly as it was.

## 2026-09-24

Built as proposed. One git process lists every checkout, this one's HEAD included, and one diff per other worktree runs in parallel. It measured about 110 ms on rim with ten worktrees under a load average of 7, all on the loader thread. The diff is --diff-filter=MR against the merge-base, so an item a branch filed, or one it never touched, is never read as its work. That holds even in counted formats. Where a repository has never had a second worktree, git's directory is watched until the registry appears. Twelve behaviour tests use real git worktrees, and mutating the merge-base, the diff filter, the turning rule or the change detection each fails one of them. Left out on purpose: items new on a branch, and any filter over elsewhere.
