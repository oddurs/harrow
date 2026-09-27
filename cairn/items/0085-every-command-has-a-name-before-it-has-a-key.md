---
id: d8989454-a86d-4e9a-b70d-11b9fc572f67
title: Every command has a name before it has a key
type: feature
status: done
milestone: v0.6
created: 2026-09-18
updated: 2026-09-18
priority: p1
area: chrome
---

## Problem

Forty-eight commands, fifty-eight bindings, forty-one of the available letters
spent. Every new capability has to take a letter from something, and the
evidence that it already has is in the defaults:

```rust
(K::Char('m'), n, C::ToggleMouse),   // a debug switch, on a prime letter
(K::Char('k'), ctrl, C::Check),      // no letter left, so a control key
(K::Char('D'), n, C::Diagnostics),
```

`m` toggles mouse reporting. It is a thing you press once in a session where
the terminal is misbehaving, and it holds a letter that `mark` or `milestone`
would use. `check` is on `ctrl-k` because nothing else was free. Meanwhile
sorting (0083) and saved views (0087) have nowhere to go at all.

The keymap is a zero-sum game, and 0044 is about to freeze it.

## Proposal

A palette on `:`, listing every command, filtered as you type.

harrow already generates its help overlay from three things: `Command::ALL`,
each command's description, and `keys_for()`. The palette is the same three
things in a different arrangement, so it is close to free and — more
importantly — **nothing can exist in harrow and be missing from it**. The
table is the source for both.

```
╭ : cl▏ ──────────────────────────────────────────────────────────╮
│  ▸ claim                 take it before you start          c   │
│    close                 finish it                         x   │
│    release · hand back   give it back to whoever is next   C   │
│    check the project     ask cairn whether it is valid     —   │
╰ ↵ run · the key on the right does it without this ─────────────╯
```

The key sits on the right of every row, so the palette teaches the keymap
rather than replacing it. A command with no key shows a dash, which is also
the honest way to say *this one lives here now*.

It returns an `Action` like any other command, so everything reachable through
it stays assertable in a test with no repository underneath.

What this is for is the economics. After it exists, a new command lands in the
palette and earns a key later, and `m`, `D` and `ctrl-k` can be demoted
(0086) without taking anything away from anyone.

## Why `:` and not `ctrl-p`

`ctrl-p` is the other convention and it is taken by `propose`. `:` is free, it
is the reflex of everybody who has used vim, and it does not collide with a
terminal's own control keys the way `ctrl-`-anything eventually does.

## Cost

Another overlay, in a program that already has four. It earns its place by
being the one that makes the others reachable — and by being generated rather
than written, so it cannot go stale.

## Acceptance criteria

- [x] Every command in `Command::ALL` is reachable by name
- [x] Each row shows the key that does the same thing without the palette
- [x] A command with no key is shown as having none, rather than omitted
- [x] The palette is built from the command table, so a new command appears without being added here
- [x] Running one returns an `Action`, like any other key

## 2026-09-18

Ranked in three bands rather than scored: a command whose stable name starts with what you typed, then one whose name contains it as a subsequence, then one whose sentence contains it. Within a band, declaration order — which is the order the help overlay already uses, so the two agree without being told to.

Both the name and the sentence are searchable. The name is what a `[keys]` table binds and so has to be exact; the sentence is how anybody actually remembers a command. `confirm` finds `close`, which is described as "close it, with a confirm" and whose name says nothing about confirming.

`↵` and a click are the same gesture, so both close the palette before running. Typing goes to the palette and nowhere else — `x` while typing must not close an item, and there is a test for exactly that.
