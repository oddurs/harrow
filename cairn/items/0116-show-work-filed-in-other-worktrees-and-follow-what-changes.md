---
id: 116
uid: 4e4c7073-da13-4ec5-a59a-1ed2d0694d88
title: Show work filed in other worktrees, and follow what changes
type: feature
status: done
created: 2026-09-27
updated: 2026-09-27
priority: p1
area: read
---

## Problem

An agent working in its own worktree filed an item, claimed it, ticked its
criteria and closed it, and harrow, open on the primary checkout the whole
time, showed nothing. 0111 made changes to *existing* items visible across
worktrees, and left items *filed* on a branch out on purpose: "somebody's
plan, not somebody picking up work". In practice an agent files the item it
is about to do and starts on it a second later. So the most visible thing an
agent does was the one thing the board could not see.

The other half: when something does change, a row gets a dot and the footer
gets a sentence, and the list stays wherever it was scrolled. With a backlog
longer than the pane, the change is usually off-screen. You are told that
something moved and then have to go and find it.

## Proposal

**Items filed elsewhere are shown, as provisional rows.** The loader already
asks git what each other worktree changed since it diverged. It now also asks
what each one added, committed or not (`git diff --diff-filter=A` against
the merge-base, and `git ls-files --others`). Each added item it reads
becomes a row, marked with the branch it lives on, in every lens. Such a row:

- is read-only. Every write refuses it by name ("0117 is filed on feat/x —
  change it there"), because cairn in this checkout cannot see it. It cannot
  be marked, for the same reason.
- is kept out of `--plain`, which is the record for scripts and has to keep
  agreeing with `cairn list` here.
- keeps the number it has there, unless that number already names something
  here. Then it goes by its tag, and if it has no tag it is not shown: an
  item nothing can name cannot be told apart from the one it collides with.
- leaves once its branch merges, because the file is then in the record.

**The view follows what changes, while you are watching rather than
driving.** When a change arrives from anywhere but harrow's own write, and
no key or click has come in for a few seconds, the cursor moves to it. The
list or the board scrolls to show it, and the detail pane shows what
happened. While you are moving, it only marks and announces, as now. A
cursor taken away from under your hand is worse than a change you have to
look for.

### What it costs

- One more git process per other worktree per reading (`ls-files`), in the
  same parallel pass.
- Filters apply to provisional rows as to any row. So in the interface a
  filter can show an item `cairn list` here does not. That is the point of
  showing them, and the row says where it lives. `--plain` and the agreement
  suite stay the record's alone.
- Following needs to know when you last touched anything. That is the frame
  clock the core already carries, recorded on each key and click, so it
  stays testable.

## Acceptance criteria

- [x] An item filed in another worktree, committed or not, appears as a row in every lens, marked with its branch, within one reading of being written
- [x] Every write and mark refuses a provisional row and says where it lives; `--plain` does not print it
- [x] A provisional item whose number is taken here goes by its tag, and one with no tag is left out
- [x] Once merged, it is an ordinary row, and appears once
- [x] A change from elsewhere moves the cursor to it and scrolls it into view when the reader is idle, and does neither while they are active
- [x] The detail pane says where a provisional item lives

## 2026-09-27

Built as proposed. Verified against this repository: with 0116 filed only in feat/follow-live-writes, harrow on the primary checkout lists it with its branch in the holder's place; --plain leaves it out; --doctor says '1 other — 0 items changed there, 0 under way, 1 filed'. A format-4 worktree (feat/nerd-font-glyphs) is skipped as before. New tests: 5 against real git worktrees, 6 engine keying cases, 10 on following and refusal, one recorded screen. Mutating ls-files, the --plain filter, the idle check or the follow call each fails a test. One existing test (identities: selection survives a prefix change) now marks its reader as active, because an idle reader is correctly taken to the item that arrived. scripts/task check green.
