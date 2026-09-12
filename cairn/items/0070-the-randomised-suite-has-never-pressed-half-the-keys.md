---
id: 70
title: The randomised suite has never pressed half the keys
type: bug
status: done
milestone: v1.0
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: testing
---

## What happens

`tests/invariants.rs` is the suite that exists to find what hand-written
tests miss: forty keys, four hundred seeds, the invariants asserted after
every press. Its key list is a hand-written `const KEYS`, and it has fallen
behind the keymap without anybody noticing.

Bound and never pressed: `t` tick, `N` note, `A` accept, `C` release, `H`
history, `e` edit, `o` read, `a` show-all, `r` refresh, `h` retreat, `<`,
`>`, `q`, and `2` `3` `4` `5` — every lens key but the first. Also `backtab`.

And it has never sent a modifier at all. Every binding behind ctrl is
unreachable to it: `ctrl-k` check, `ctrl-p` propose, `ctrl-d`, `ctrl-u`,
`ctrl-r`, `ctrl-c`.

So roughly half the key surface, including all of v0.4 and v0.5, has never
been randomly exercised. The suite has been reporting that four hundred
seeds pass while never touching the code most recently written — which is
the worst way for a test to fail, because it fails silently and in the
direction of confidence.

## The second half

`check_invariants` does not know about the state the new lenses added. The
queue's cursor, the stats pane's figure, the log's moment, the anchor that
holds a selection the queue cannot show — none of them are asserted to be
in range. So even if the fuzz pressed the keys, it would not check what
they moved.

## Proposal

Generate the key list from the keymap, so it cannot drift again. `Keymap`
already knows every binding and `Command::ALL` already exists for exactly
this kind of exhaustiveness; a test that asks the program what its keys are
cannot fall behind the program's keys.

Keep a handful of keys nothing is bound to, because pressing those is also
a case — and the point of the suite is the sequences nobody would write.

Then extend `check_invariants` over the cursors the lenses added. Each is
an index into a `Vec` that something else rebuilds, which is exactly the
shape of bug the invariants exist to catch.

## Acceptance criteria

- [x] The randomised suite presses every bound key, including modified ones
- [x] Its key list is derived from the keymap and cannot drift from it
- [x] It still presses keys nothing is bound to
- [x] Every cursor the lenses added is asserted in range
- [x] Whatever this finds is fixed, or filed with what it found — it found
      nothing. A hundred thousand seeds over the full key surface, modifiers
      included, with the new cursors asserted, and every one passes. Worth
      recording as an answer rather than a shrug: the clamping in the lenses
      holds, and the finding was the blind spot itself
