---
id: 21
title: 'Package it: crates.io and a tap'
type: chore
status: backlog
milestone: v1.0
created: 2026-09-08
updated: 2026-09-12
priority: p1
area: packaging
---

## Problem

`cargo install --path .` is the only way in, which means the only people who
can use this are the people who have already cloned it.

## What is in the way

The name. `crates.io/crates/harrow` is an HTTP framework by somebody else —
0.10.0, published 2026-04-20, 156 downloads — so `cargo install harrow` will
never install this, whatever this does.

cairn hit the same wall and answered it, and the answer is in its
`Cargo.toml`:

```toml
# `cairn` and `cairn-cli` are both taken on crates.io by unrelated crates, so
# the package is published as `cairn-md`. The binary it installs is still
# `cairn`.
name = "cairn-md"

[[bin]]
name = "cairn"
```

So the mechanism is settled — a package name that is free, a binary name that
is the tool's — and only the word is open. **Deferred deliberately: the tool
may be renamed, and choosing a registry name for a name that is about to
change is the one part of this that cannot be undone.** A published crate can
be yanked but never unpublished, and a name once taken is taken by you.

## What is actually missing, beyond the name

Neither cairn nor harrow is published anywhere today, and the reason is the
same in both repositories: the release workflow creates a GitHub release from
a tag and generates its notes, and attaches nothing to it. No binaries, no
tarball, no checksum.

That is why `oddurs/homebrew-tap` carries `quarry.rb` and `knit.rb` and
nothing for cairn — a formula needs an artefact with a checksum to point at,
and there is none.

So the work here is mostly not the publish. It is:

- binaries for macOS and Linux, built on a tag and attached to the release
- a formula generated from their checksums and pushed to the tap
- `cargo publish` behind a token, as the last step rather than the first

None of which needs the name decided, and all of which is the same missing
piece in cairn.

## Ordering

harrow's README opens by telling people to go and get cairn. Shipping harrow
first would send people to a tool they cannot install, so cairn goes first or
they go together.

## Acceptance criteria

- [ ] A tag builds binaries for macOS and Linux and attaches them to the release
- [ ] A Homebrew formula is generated from their checksums and pushed to the tap
- [ ] `brew install oddurs/tap/harrow`
- [ ] `cargo install <whatever this is called>`, decided after the rename question is settled
- [ ] cairn is installable first, or at the same time
