# Contributing

Thank you for looking. This is a small project with a strict workflow — the
strictness is what lets an agent and a person work in it without stepping on
each other.

## Once, after cloning

```sh
scripts/setup
```

That wires `core.hooksPath` to `.githooks/` and checks your toolchain. The hooks
are the workflow; without them you will find out about a problem in CI instead
of in your terminal.

## The loop

One unit of work is one branch, in one worktree, with one pull request.

```sh
scripts/agent doctor                       # is this checkout ready?
scripts/agent start fix/detail-pane-scroll # branch + worktree, prints the path
cd ../.worktrees/harrow/fix/detail-pane-scroll

# ... work ...

scripts/agent check                        # fmt, lint, test, build
scripts/agent commit "fix(ui): keep the detail pane inside its pane"
scripts/agent pr                           # checks, pushes, opens the PR
scripts/agent sync                         # rebase onto main when it moves
# ... after the PR is merged ...
cd -                                       # back to the primary checkout
scripts/agent done fix/detail-pane-scroll  # remove the worktree and the branch
```

`scripts/agent list` shows every worktree, its branch, and its pull request.

Worktrees live in `../.worktrees/harrow/<branch>/`, outside the repository, so
two branches never share an index or a `target/` directory. That is what makes
parallel work safe rather than merely discouraged.

## `main` only advances through a merged pull request

Not by convention — the local `pre-push` hook refuses a push to `main`, and
branch protection on GitHub refuses it again if you get past the hook. This
holds for the maintainer too.

Required approvals are set to **0**, deliberately: this is a solo project and a
review requirement would deadlock the only person who can review. Everything
else — a pull request, a green `required` check, an up-to-date branch, resolved
conversations — is required. If the project gains a second maintainer, that
number becomes 1.

## Checks

Everything goes through one seam, so CI and your machine cannot disagree:

```sh
scripts/task fmt        # format in place
scripts/task fmt:check  # verify formatting
scripts/task lint       # clippy, warnings denied
scripts/task test       # the full suite
scripts/task build      # compile everything
scripts/task check      # all of the above — what CI runs
```

The hooks run these for you: `pre-commit` formats and lints, `pre-push` runs the
lot. Never reach for `--no-verify`. If a hook is wrong, fix the hook in its own
pull request.

Two suites are worth knowing about:

```sh
HARROW_UPDATE_SNAPSHOTS=1 cargo test          # accept a deliberate screen change
HARROW_FUZZ_SEEDS=100000 cargo test --release --test invariants
```

The interface is held to recorded screens in `tests/snapshots/`. If your change
moves the layout on purpose, re-record them and **read the diff** — that diff is
the review of your change to the interface.

## Commits

[Conventional Commits](https://www.conventionalcommits.org), imperative mood,
subject under 72 characters, no trailing period:

```
fix(ui): keep the detail pane inside its pane

A body long enough to wrap was drawn past the bottom border, which put text
over the status strip on a short terminal.

Refs: 0005
```

Types: `feat` `fix` `chore` `docs` `perf` `refactor` `test` `build` `ci`
`style` `revert`.

The body explains **why**; the diff already says what. Reference the cairn item
in a `Refs:` trailer.

No commit, pull request, comment, or line of documentation attributes work to an
assistant, a model, or a tool. The `commit-msg` hook enforces it. Work published
here is published under its author's name.

## The backlog

This project's roadmap and issues are [cairn](https://oddurs.github.io/cairn)
items in [`cairn/items/`](cairn/items) — plain Markdown, versioned with the
code, reviewable in a pull request. `ROADMAP.md` is generated from them; do not
edit it by hand.

If you are proposing something larger than a fix, write the item first:

```sh
cairn new "Watch the directory instead of re-reading it" -t feature --set area=runtime
```

An item carries the reasoning that produced it — the problem, the proposal, the
costs weighed, how you will know it is done. That is worth more after it closes
than before, because it is then the answer to *why is it like this*.

GitHub issues are fine for reporting a bug or asking a question. Anything that
becomes work becomes an item.

## Code

- Comments explain **why**, not what. If a line needs a comment to say what it
  does, the line is the problem.
- Nothing in the core performs a side effect. `App` returns actions and the
  shell carries them out; that is what makes every write assertable in a test.
- A behaviour worth having is worth a test named after the behaviour, not after
  the function.
