---
id: 50
title: Suspend it, and come back to it
type: feature
status: backlog
milestone: v1.0
created: 2026-09-12
updated: 2026-09-12
priority: p3
area: runtime
---

## Problem

`ctrl-z` does nothing. Raw mode clears `ISIG`, so the keystroke never becomes
SIGTSTP — it arrives as an ordinary key event, matches nothing, and is
dropped.

For most full-screen programs that is the correct behaviour. For this one it
is not. harrow is explicitly built to sit in a pane beside the work, and the
shell it was started from is the thing you want back for ten seconds to run a
command and return. Every other terminal program of that shape — `less`,
`vim`, `top` — suspends.

Quitting and restarting is not the same: it costs the filter, the grouping,
the marks, the cursor and a reload of the backlog.

## Proposal

Handle the key: leave the alternate screen, restore the terminal, raise
SIGTSTP at ourselves, and on SIGCONT put it all back and redraw. The terminal
lifecycle is already in one place, and `term.rs` says so — "a panic, a fatal
signal, and a dropped guard, and all four go through here" — so this is a
fifth path through machinery that exists rather than new machinery.

The watcher thread keeps running while suspended, which is correct: coming
back to a screen that is already current is the point.

## Cost

A suspend that restores the terminal badly is worse than no suspend, and it is
hard to test without a pty. The guard already handles restoring on the way
out; this reuses it in both directions, and the test is that the existing
lifecycle test grows a suspend and a resume.

## Acceptance criteria

- [ ] `ctrl-z` suspends and returns the shell its terminal, unaltered
- [ ] Resuming redraws, with the filter, grouping, marks and cursor intact
- [ ] The backlog is current on return rather than as it was
- [ ] Suspending while an overlay is open comes back to that overlay
