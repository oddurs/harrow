---
id: 41
title: Undo the last thing you did
type: feature
status: backlog
milestone: v1.0
created: 2026-09-12
updated: 2026-09-18
priority: p1
area: write
---

## Problem

harrow already knows how to undo every write it makes. `Change` carries an
`undo` field, every status change, claim, release and close fills it in, and
`run_change` puts it in the toast:

```rust
let message = match &change.undo {
    Some(undo) => format!("{} · undo: {undo}", change.describe),
    None => change.describe.clone(),
};
```

So after closing the wrong item you are shown `undo: cairn reopen 12`, for
3.2 seconds, and then it is gone and you are expected to have memorised it.
The program computed the exact remedy, displayed it, and made you retype it
into another terminal.

Triage is one keystroke per decision. That is the whole pitch, and it is only
safe if the keystroke is cheap to take back. Right now every decision is
cheap to make and expensive to reverse, which is the wrong way round — it is
what makes people slow down and read twice before pressing `x`, and reading
twice before every keystroke is the cost the screen was supposed to remove.

## Proposal

A key that runs the undo of the last write, through cairn, the way the write
went through cairn.

One level, not a stack. A stack invites walking backwards through a session,
which against a backlog other people and agents are also writing is a fiction:
the third undo back may be undoing somebody else's work. One level covers what
this is for — *that was the wrong item* — and says so honestly.

It expires. An undo offered ten minutes later is an undo of a state that no
longer exists; hold it for as long as the change is still marked as recent and
then let it go, so the key is never a way to quietly revert something you have
since thought about.

A bulk change is one undo, because it was one decision.

## Cost

Two ways to get to the same state, and the second one is a command string
built by formatting rather than by the same code path that made the change.
That string is already being printed, so the risk exists today; making it
executable is what makes it worth testing. Every `undo` a change carries
should be asserted in the write tests beside the change itself.

## Acceptance criteria

- [ ] A key reverses the last write, through cairn
- [ ] What it will do is visible before it is pressed, not only in a toast that has gone
- [ ] It expires rather than sitting there indefinitely
- [ ] A bulk change undoes as one
- [ ] Every change that carries an undo has that undo asserted in a test

## 2026-09-18

Two design points from the interaction-model work in v0.6 (0081), recorded here because this is where undo lives.

**Undo is a new change, not an erasure.** It goes through cairn like the write it reverses, so cairn's history records both moves. A backlog under version control should never quietly lose a step, and the toast should say which way round it is: `0042 back to doing — undoing 'done'`. That also settles how deep it goes. One step is honest and cheap; a stack invites the expectation that harrow knows the whole history, which cairn owns and harrow only reads.

**What has no inverse says so.** A note and a new item are not undoable. The key should refuse them out loud rather than silently doing nothing, which is the same rule 0080 landed on for empty listings.
