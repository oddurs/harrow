---
id: 52
title: A digit in a picker does not pick outright
type: bug
status: backlog
milestone: v1.0
created: 2026-09-12
updated: 2026-09-12
priority: p3
area: chrome
---

## What happens

The picker's own comment says what it is meant to do:

```rust
// A digit picks outright, which makes `s 2` the whole gesture.
```

It does not. The digit moves the highlight and returns `Action::None`, so
`s 2` selects and waits, and the gesture is `s 2 ↵`.

Found while writing tests for 0036, which had to press Enter after a digit
for reasons the comment says should not exist.

## What should happen

One of the two, and the other deleted. Either the digit picks — which is what
the comment promises and what makes the shortcut worth having — or the
comment stops claiming it.

The first is a one-line change and is probably right: the numbers exist so a
choice is two keystrokes, and three keystrokes is what the arrow keys already
cost.

## Acceptance criteria

- [ ] A digit either submits, or the comment no longer says it does
- [ ] Whichever it is, a test holds it
