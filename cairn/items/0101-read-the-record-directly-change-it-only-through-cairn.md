---
id: e14f0716-b490-46c8-9830-1292808a3a5e
title: Read the record directly; change it only through cairn
type: decision
status: done
milestone: v0.7
created: 2026-09-23
updated: 2026-09-23
closed_at: 2026-09-23
priority: p1
area: docs
effort: s
---

## Context

Harrow and Cairn are two programs over one record. After weeks of daily use the
question is what harrow is allowed to become, so that the answer is the same in
five years as it is today. 0100 holds the evidence.

## Options and tradeoffs

Harrow could grow its own writer, and stop depending on Cairn being installed.
That buys one binary and costs the single source of validation, locking, hooks
and identifiers — two writers with different ideas of a valid item is the
failure that cannot be undone from a backup.

Harrow and Cairn could share a Rust library for parsing and queries. That buys
agreement by construction and costs the thing the format promises: a second
implementation that disagrees is how the specification gets tested. A shared
crate makes the two readers one reader wearing two names, and adds a
compatibility surface without removing the need for an independently readable
format.

Harrow could hold a database or an index for speed. That buys startup time on
a very large backlog and costs the property that the files are the truth and
anything else is a cache that can be wrong.

Harrow could run a daemon to watch and serve. That buys push updates and costs
a process to supervise, a socket to secure, and a state to get out of sync.

## Decision

Harrow reads Cairn's files directly and changes the record only by invoking
Cairn. It keeps no database, no index, no daemon, and no shared runtime library
with Cairn. It stays a single local binary that starts instantly, works on a
machine that has never installed Cairn, and writes nothing that Cairn did not
validate.

Agreement with Cairn is established by testing, not by sharing code: the
vendored golden corpus, the pinned counterpart revision, and the agreement
suite that CI requires on both sides.

Harrow's job is understanding and steering: see the state of the work, and
decide what happens to it. A feature that does neither belongs in Cairn or
nowhere. The next investment is reach — making the questions harrow already
answers askable from a shell — not new screens.

## Revisit when

Reading the files directly stops being fast enough on a real backlog, with a
measurement on named hardware rather than an impression. Or the two readers
disagree in a way the shared format and its tests cannot express, which would
be evidence about the format rather than an argument for a shared library.
Record the evidence before changing the boundary.
