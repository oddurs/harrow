---
id: 23
title: Make the wrong thing structurally impossible
type: chore
status: done
milestone: v0.1
created: 2026-09-08
updated: 2026-09-08
priority: p0
area: packaging
---

## Problem

A repository that two agents and a person share needs a workflow, and a
workflow held up by discipline is a workflow that fails on the first busy day.
The three failures worth designing out are committing to the default branch,
pushing red code, and two units of work sharing a checkout.

## Proposal

Make each one impossible rather than discouraged:

- **`main` advances only through a merged pull request.** A `pre-push` hook
  refuses locally; branch protection refuses on the server. Two independent
  refusals, because the local one is one `git config` away from being gone.
- **A branch is green before it is a pull request.** `pre-push` runs the whole
  suite, and CI runs the same line again.
- **One unit of work, one worktree.** `scripts/agent start` puts every branch
  in `../.worktrees/harrow/<branch>/`, so two branches never share an index or
  a `target/`.

The load-bearing part is `scripts/task`. CI, the hooks and `scripts/agent` know
only `fmt`, `fmt:check`, `lint`, `test`, `build`, `check` — nothing outside that
file knows this project is Rust. That is what stops CI and a local hook from
drifting: they are not two implementations of the same idea, they are one.

Hooks are `sh` in `.githooks/`, tracked in git and wired with `core.hooksPath`.
No husky, no pre-commit framework: a hook that needs a package manager to run
is a hook that does not run on the machine where it mattered.

## Result

`pre-commit` is format and lint — under a second warm, which is what a commit
can absorb. The full suite is in `pre-push`, where it can take as long as it
takes.

## Acceptance criteria

- [x] `scripts/task check` is the only thing CI runs
- [x] A push to the default branch is refused locally and on the server
- [x] Every branch gets its own worktree, created and removed by script
- [x] Hooks need nothing installed beyond git and the toolchain
- [x] No commit, pull request or release note attributes work to an assistant

## Branch protection, 2026-09-08

Classic branch protection alone did not hold, and it is worth writing down
before somebody sets a repository up the same way again.

With `required_pull_request_reviews` present and
`required_approving_review_count: 0` — the setting a solo project needs, so the
only person who can review is not deadlocked — a fast-forward ref update
straight to `main` through the REST API was **accepted**:

    gh api -X PATCH repos/OWNER/REPO/git/refs/heads/main -f sha=<branch head>

`enforce_admins` was on. The local `pre-push` hook refused the same push, so the
first line of defence held; the second did not, which is exactly the one that
matters for anything that never touches this machine.

A repository **ruleset** refuses it:

    Repository rule violations found
    Changes must be made through a pull request.

So `main` is protected by a ruleset — `pull_request`, `required_status_checks`
on the `required` context, `deletion`, `non_fast_forward`, `required_linear_history`,
with no bypass actors — and classic protection is left in place beside it as a
second, visible layer. Verified by re-running the push that got through.

The lesson generalises: a protection setting is not configured until the thing
it forbids has been attempted and refused.
