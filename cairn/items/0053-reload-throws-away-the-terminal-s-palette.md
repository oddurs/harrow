---
id: 53
title: Reload throws away the terminal's palette
type: bug
status: done
milestone: v0.3
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: theme
---

## What happens

`ctrl-r` makes the interface worse and it does not come back until you
restart.

At startup, `run_tui` asks the terminal for its actual colours — OSC 11 for
the background, OSC 10 for the foreground, OSC 4 for all sixteen slots — and
builds the theme from the answers. Surfaces and borders are *mixed* from the
real background and foreground, hues are taken from the real palette where
they are legible against the real page and derived where they are not. That
is the good version, and it is the one the README is describing.

Reload does not do any of that:

```rust
Action::Reload => {
    let fresh = resolve(args);
    app.theme = fresh.theme;
```

`resolve` produces the *placeholder* `auto` theme — the one that names ANSI
slots and sets `background`, `surface`, `overlay` and `selection` all to
`Color::Reset`. The query that upgrades it lives in `run_tui`, and runs once,
before the loop.

So pressing `ctrl-r` replaces a palette derived from eighteen measured
colours with a flat sixteen-slot approximation. Panes stop sitting above the
page, the selection loses its tint, borders go to `DarkGray`. Nothing says
so, and the only way back is to quit and start again.

The README says *harrow is Gotham, and it follows when you change it*. It
does not follow. Changing the terminal's theme and pressing the key that
exists to pick up changes is exactly the gesture that loses the inheritance.

## Proposal

Re-ask on reload. We are in raw mode and the query costs 150 ms once, which
is what it costs at startup — and asking again is the only way a *changed*
terminal theme is ever noticed, which is the thing the README promises and
the reason somebody presses the key.

The upgrade belongs in one function both paths call, rather than inline in
`run_tui` where reload cannot reach it.

## Cost

A terminal that does not answer pays 150 ms on every reload. It already pays
it once at startup, reload is a deliberate keystroke rather than something
that happens on a timer, and the alternative is the current behaviour, where
the answer is silently thrown away.

## Acceptance criteria

- [x] Reloading keeps a palette-derived theme rather than falling back to ANSI slots
- [x] Changing the terminal's own theme and reloading follows it
- [x] A terminal that answers nothing is no worse off than it is now
- [x] The upgrade is one function, so no third caller can forget it
