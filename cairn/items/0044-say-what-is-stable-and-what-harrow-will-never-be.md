---
id: 44
title: Say what is stable, and what harrow will never be
type: docs
status: backlog
milestone: v1.0
created: 2026-09-12
updated: 2026-09-12
priority: p1
area: docs
---

## Problem

1.0 is a promise, and harrow has not made one. Somebody deciding whether to
put this in their toolchain needs to know three things nobody has written
down.

**What harrow will never do.** cairn has PROMISE.md, and the reason it is
worth having is that it answers feature requests by principle rather than by
mood. harrow's line is architectural and already in CLAUDE.md, where only
contributors see it: reads go straight to the files, writes go through cairn,
and harrow does not write item files. So harrow will never be a second cairn,
never take over the lock, never grow a format of its own, and never require
cairn to be installed to read a backlog. Those are the guarantees that make it
safe to adopt — and the last one is the reason there is a second reader of the
format at all.

**What is stable.** A 1.0 fixes the config file's keys, the action names a
`[keys]` table binds to, the theme roles a theme file sets, and the
command-line flags. It does not fix the screen: the interface is the product
and is expected to improve, which is what the snapshots are for.

**What is not public API.** harrow is published as a crate with a library
inside it, because the integration tests need one. `harrow::app::App` is not
an interface anybody should build on, and saying so now costs nothing and
saying it later costs a major version.

Also unstated: the minimum Rust version is pinned in `Cargo.toml` at 1.98 with
no policy attached, and 1.0 should say how it moves.

## Proposal

A PROMISE.md, reproduced in the README and held to it by a test the way cairn
holds its own. A section in the README on what is stable and what is not. The
crate's library documented as an implementation detail.

## Acceptance criteria

- [ ] PROMISE.md exists and says what harrow will never do, with the reasons
- [ ] It is reproduced in the README and a test fails if the two disagree
- [ ] The stability of the config keys, action names, theme roles and flags is stated
- [ ] The library is documented as not public API
- [ ] The minimum Rust version has a written policy
