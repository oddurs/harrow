---
id: 47
title: Say which platforms harrow runs on, and prove it
type: chore
status: backlog
milestone: v1.0
created: 2026-09-12
updated: 2026-09-12
priority: p2
area: packaging
---

## Problem

CI runs on ubuntu and macos, deliberately, because the terminal handling is
POSIX-level. Nothing says whether Windows works, and the honest answer is that
it does not: `term.rs` installs signal handlers through `libc` for SIGTERM,
SIGHUP, SIGQUIT and SIGPIPE, none of which Windows has.

So harrow almost certainly does not compile there, and nothing in the README,
the crate metadata or the help text says so. Somebody on Windows finds out
from a compiler error, which is the worst place to learn it.

The dependency comment in `Cargo.toml` even mentions `notify` using
ReadDirectoryChangesW "on Windows", which reads as a claim of support.

## Proposal

Decide, and say so. The cheap decision is POSIX-only: state it in the README,
put it in the crate metadata, and make the claim testable — a CI job that
proves the build fails cleanly rather than confusingly, or at minimum a
`compile_error!` with a sentence in it rather than sixteen unresolved symbols.

The expensive decision is to support it, which means a Windows path for the
signal handling and a third CI runner. Worth pricing before 1.0 rather than
after, because adding a platform after 1.0 is free and dropping one is not.

## Acceptance criteria

- [ ] The README says which platforms are supported
- [ ] The crate metadata agrees with the README
- [ ] An unsupported platform fails with a sentence, not with link errors
- [ ] Whatever is claimed is proved by CI
