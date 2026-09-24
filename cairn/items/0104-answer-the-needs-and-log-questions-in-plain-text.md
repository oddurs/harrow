---
id: 104
title: Answer the needs and log questions in plain text
type: feature
status: done
milestone: v0.7
created: 2026-09-23
updated: 2026-09-23
closed_at: 2026-09-23
priority: p1
area: cli
effort: m
part_of:
- 102
---

## Problem

`--plain` prints one line per item for the list. The needs queue and the log
are not lists of items — one is a list of questions about items, the other a
list of changes — so `--plain` has nothing to say about either, and an agent
or a shell script cannot ask harrow the two questions that matter most for
supervising work.

This is the half of 0103 that makes it useful without a terminal. A door into
the program is not the same as an answer you can pipe.

## Proposal

With `--lens needs`, `--plain` prints one line per question; with
`--lens log`, one line per change. Tab-separated, like the existing plain
output, so `cut` and `awk` work on it.

    0067	proposal	priority p2 → p1, by an agent
    0094	cold	oddur, 6 days
    0099	finished	4 of 4 ticked, not closed

    2026-09-20	oddur	0094	the map and the thing it maps

The needs line names the item, the kind of question, and its particulars. The
log line is the change harrow already reads out of Git.

Exit status stays 0 for an empty queue: nothing needing attention is an
answer, not a failure. A caller that wants to branch on emptiness counts
lines, which is what makes it composable.

## Acceptance criteria

- [x] `--plain --lens needs` prints one tab-separated line per question
- [x] `--plain --lens log` prints one tab-separated line per change
- [x] Every question kind the needs lens computes has a printed form
- [x] An empty queue prints nothing and exits 0
- [x] The formats are covered by tests that name the behaviour

## 2026-09-23

Shipped. Needs prints id, kind, who, detail; log prints when, who, id, what; both tab-separated. All five Asking variants have a printed form, and the match is exhaustive so a sixth cannot be added without one. The log asks Git in plain mode the same way a screenshot does, since neither has an event loop. Verified against cairn's own repository and a fixture repository built in the test.
