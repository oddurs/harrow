---
id: 85
title: Every command has a name before it has a key
type: feature
status: backlog
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

- [ ] Every command in `Command::ALL` is reachable by name
- [ ] Each row shows the key that does the same thing without the palette
- [ ] A command with no key is shown as having none, rather than omitted
- [ ] The palette is built from the command table, so a new command appears without being added here
- [ ] Running one returns an `Action`, like any other key
