---
id: 63
key: v0.5
title: The work is somebody else's; the deciding is yours
type: milestone
status: backlog
depends_on:
- 55
created: 2026-09-12
updated: 2026-09-12
priority: p2
due: 2027-04-01
---

The work is somebody else's. The deciding is yours.

## What cairn turned out to be

Read cairn's source rather than its README and it is not an issue tracker
with an agent integration bolted on. It is a workspace where programs do the
work and a person is answerable, and nearly every decision in it says so:

- `next --mine` returns what you are assigned, what you *own*, and what
  nobody has claimed — because "owning and working are different questions
  once an agent is doing the work".
- `propose` exists so a program can ask for a change a person decides, and
  `agent = "propose"` marks the fields where it must.
- `note` appends and cannot replace, so that a caller writing down its
  reasoning cannot erase what came before.
- `claim_stale_after` names the case of a worker that took something and did
  not come back.
- `created_by` is *what made the item, when that was not a person*.
- Fourteen tools over the protocol against twenty-two on the command line.

## What harrow is arranged as instead

List, board, stats. That is the arrangement every tracker has had since
Trello, and all three answer one question: **what is the shape of this
project?**

The questions a person supervising work actually asks, in descending
frequency, are:

1. What needs me?
2. What happened while I was away?
3. What is moving, and who has it?
4. What is the shape?
5. Change this one thing.

harrow spends three lenses on (4), is very good at (5), gestures at (3) with
the status strip, and **has no surface at all for (1) or (2)** — the two most
frequent.

## What this milestone does

Makes the primary lens a queue of decisions rather than a list of items.

0064 is *what needs you*: one ranked list where every row is a question with
its answers bound to keys, and whose empty state — *nothing needs you* — is
the best screen harrow can show and is currently unreachable, because no
screen makes the claim.

0065 is *what happened*: reverse-chronological, project-wide, out of git.
cairn already reads git for one item's history and has nothing project-wide,
which is the largest gap between what it stores and what a supervisor needs.

Then the smaller things that follow from the same reading: a detail that
reads as a thread (0066), criteria you can tick (0067), and actors you can
tell apart (0068). And 0069, because five lenses in a cycle is not one press
from every other — which the test written in 0062 will say out loud.

List, board and stats stay. They are demoted from being the product to being
the shape lenses, which is what they are good at.

## What must not happen

PROMISE.md draws cairn's line and a TUI inherits it. No view across
repositories, no notifications, nothing that runs when you do not. *What
happened while I was away* means what this clone's git can tell me when I
ask it — which is bounded, and is enough.
