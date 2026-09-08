# Working in this repository

Instructions for an agent. Follow them exactly; they override any default
workflow you would otherwise apply.

## Attribution

Never attribute work in this repository to an assistant, a model, or a tool.
Not in commit messages, trailers, pull request bodies, issue comments, code
comments, documentation, changelogs, or release notes. No `Co-Authored-By:`
naming a model, no "generated with" footer, no robot emoji.

The work is published under the owner's name. The `commit-msg` hook rejects a
message that breaks this, and `scripts/agent pr` strips it from a pull request
body — but do not rely on either. Write it correctly the first time.

## Setup

```sh
scripts/setup          # once per clone: wires core.hooksPath to .githooks
scripts/agent doctor   # verify before starting anything
```

## The loop

One unit of work is one branch, in one worktree, with one pull request. Two
agents must never share a checkout.

```sh
scripts/agent start feat/short-slug     # prints the worktree path
cd ../.worktrees/harrow/feat/short-slug # move there yourself; the script never cds for you

# ... work ...

scripts/agent check                     # before every commit
scripts/agent commit "feat(ui): one imperative line"
scripts/agent pr                        # checks, pushes, opens the PR
```

After the pull request is merged, from the primary checkout:

```sh
scripts/agent done feat/short-slug
```

Rules that are not negotiable:

- **Never commit to `main`.** It advances only through a merged pull request.
  The `pre-push` hook and branch protection both refuse; do not look for a way
  around either.
- **Never use `--no-verify`**, `continue-on-error`, or `|| true` to make a check
  pass. If a check is wrong, fix the check in its own pull request.
- **Never edit `ROADMAP.md`.** It is generated from `cairn/items/` by a cairn
  hook.

## The seam

All automation goes through `scripts/task`. Do not put a `cargo` invocation in
CI, a hook, or a script — put it here, once:

```sh
scripts/task fmt        # format in place
scripts/task fmt:check  # verify formatting
scripts/task lint       # clippy, warnings denied
scripts/task test       # the full suite
scripts/task build      # compile everything
scripts/task check      # all of the above; what CI runs
```

## Commits

Conventional Commits, imperative, subject under 72 characters, no trailing
period:

```
fix(ui): keep the detail pane inside its pane

The body explains why. The diff already says what.

Refs: 0005
```

Types: `feat` `fix` `chore` `docs` `perf` `refactor` `test` `build` `ci`
`style` `revert`. Reference the cairn item in a `Refs:` trailer.

## The backlog

The roadmap and the issues are [cairn](https://oddurs.github.io/cairn) items in
`cairn/items/` — Markdown with YAML frontmatter, versioned with the code.

```sh
cairn next                     # what is ready to start
cairn show 18                  # one item in full
cairn new "Title" -t feature --set area=chrome
cairn set 18 status=doing
```

Before work larger than a fix, write the item first, and write the *reasoning*:
the problem, the proposal, the cost you weighed, how you will know it is done.
An item is worth more after it closes than before, because it is then the answer
to *why is it like this*. A title with no body is not an item.

## Architecture, and what must stay true

- **Reads go straight to the files. Writes go through `cairn`.** `cairn.toml`
  and item frontmatter are parsed directly, so harrow opens a backlog with cairn
  not installed. Every change is a `cairn` invocation — cairn owns the lock, the
  ids, the hooks. Do not write item files from harrow.
- **The core performs no side effects.** `App` returns an `Action` and the shell
  in `main.rs` carries it out, including the `cairn` invocations. That is what
  makes every write assertable in a test with no repository underneath it. Do
  not reach for the filesystem, a process, or the clock from `app.rs` or
  `ui.rs`.
- **Rendering is a pure function of `App`.** No clock, no environment, no
  filesystem while drawing, or a snapshot stops being reproducible.
- **Nothing about a workflow is hardcoded.** Statuses, their order, types,
  icons, fields, board columns and colours all come from the project's
  `cairn.toml`. If you find yourself writing `"doing"` in a match arm, stop.

## Tests

```sh
cargo test                                   # everything
HARROW_UPDATE_SNAPSHOTS=1 cargo test         # accept a deliberate screen change
HARROW_FUZZ_SEEDS=100000 cargo test --release --test invariants
```

The interface is held to recorded screens in `tests/snapshots/`. If a change
moves the layout on purpose, re-record and **read the diff** — it is the review
of your change to the interface. Never re-record to make a failure go away
without reading what moved.

Name a test after the behaviour it protects, not after the function it calls.

## Comments

Explain **why**, not what. If a line needs a comment to say what it does, the
line is the problem. Match the density and voice of the surrounding code.
