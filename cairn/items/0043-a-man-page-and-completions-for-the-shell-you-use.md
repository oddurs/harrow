---
id: 43
title: A man page, and completions for the shell you use
type: docs
status: backlog
milestone: v1.0
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: docs
---

## Problem

`harrow --help` is the whole of the documentation you get without a browser.
There is no man page and no completions.

cairn ships both — `cairn man` and `cairn completions` — and harrow is the
program that sits beside it. A tool that installs from a tap and then cannot
answer `man harrow` is not finished; the terminal is where this thing lives,
and the terminal's own documentation system is the one it should be in.

Completions matter less for a program with eleven flags, but `--config`,
`--theme` and `--screenshot` take values that can be completed, and a theme
name is exactly the sort of thing nobody remembers.

## Proposal

A man page generated from the same list of flags `--help` prints, so the two
cannot disagree — the help text is already a single string built in `main.rs`,
and a second copy maintained by hand would be wrong within a release.

Completions for bash, zsh and fish, generated the same way. Zsh and fish are
what this author and this audience use; bash because it is what a distribution
expects.

Both emitted by a subcommand rather than committed as generated files, the way
cairn does it, so packaging can produce them at install time and they can
never be stale in the repository.

## Cost

Generated documentation is worse prose than written documentation. A man page
that is only a flag list is thin — but a thin man page that is correct beats
an absent one, and the README stays where the prose lives.

## Acceptance criteria

- [ ] `harrow man` writes a man page, and the packaging installs it
- [ ] `harrow completions <shell>` writes completions for bash, zsh and fish
- [ ] Both are generated from the same source as `--help`, and a test fails if they drift
- [ ] `man harrow` works after a `brew install`
