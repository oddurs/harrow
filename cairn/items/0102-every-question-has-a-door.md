---
id: 102
key: v0.7
title: Every question has a door
type: milestone
status: backlog
depends_on:
- 81
created: 2026-09-23
updated: 2026-09-23
priority: p1
---

Every question harrow can answer is answerable from a shell.

Every lens it draws can be opened from the command line, and the two questions
that are not list-shaped — *what needs me* and *what changed* — can be answered
in plain text without opening the program.

## Why this is the next outcome

Harrow answers four questions: what needs me, what changed, how does it stand,
and which items. Three of them have a door from outside: `--view`, `--filter`
and `--plain` answer *which items*; `--board` and `--stats` open *how it
stands*. The two that carry time and attention — the needs-you queue and the
log — can only be reached by starting the program and pressing a key.

Those two are exactly what the workflows this project exists for need. Someone
returning after a fortnight wants *what changed*. Someone supervising an agent
wants *what needs me*: a claim gone cold, a proposal waiting on an answer, work
finished and not closed, an item filed by something else that nobody owns.
Harrow already computes all four; it will not say them except to a person
sitting in front of it.

This is one idea, not three features: **a question harrow can answer should be
answerable from the shell.** It also makes harrow useful to an agent without a
daemon, a second write authority, or a shared library — the reader stays a
reader, and only the way in changes.

## Release gate

Every lens is reachable by name from the command line; `--plain` answers the
needs and log lenses in a stable line format; the manual, the help and the
completions say what each lens answers; the counterpart agreement suite still
passes against the pinned Cairn.

## Sequence

0103 opens the doors, 0104 makes the two new answers printable, 0105 writes
them down. No fourth item is selected: if an implementation slot opens, the
evidence for what to do next comes from using these.
