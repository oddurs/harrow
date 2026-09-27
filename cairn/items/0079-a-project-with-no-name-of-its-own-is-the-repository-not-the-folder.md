---
id: 99586a98-4c1a-4645-a82c-266d66a9ac6f
title: A project with no name of its own is the repository, not the folder
type: bug
status: done
milestone: v0.2
created: 2026-09-16
updated: 2026-09-16
priority: p2
area: config
---

## What happens

A project that does not set `[project].name` in `cairn.toml` is named after the
directory it was opened in. In an ordinary checkout that is usually right, and
it is why the fallback was written that way.

In a worktree it is wrong, and wrong in the most confusing way available. The
worktree layout this project uses puts a checkout at
`../.worktrees/<repo>/<type>/<slug>`, so harrow opens and calls the project
`repo-name-over-folder` — the branch slug, presented in the header as the name
of the project. Two agents on two branches of one repository see two different
project names, neither of which is the repository.

## What should happen

A project with no name of its own takes the repository's, and falls back to the
directory only when there is no repository to ask.

The repository knows its own name twice over, and both answers are files rather
than a process: `origin` in `.git/config` ends in `<name>.git`, and the
directory holding `.git` is named after it. A worktree's `.git` is a file that
says `gitdir: …/<repo>/.git/worktrees/<slug>`, so following one line of it
leads to the same two answers from inside a worktree.

Reading them keeps the rule the rest of harrow keeps: the core performs no side
effects, and finding out what a project is called must not become a process.

## Reproduction

1. `scripts/agent start fix/anything` in a project whose `cairn.toml` has no
   `[project].name`
2. Open harrow in the worktree
3. The header names the project `anything`

## Acceptance criteria

- [x] `[project].name` still wins wherever it is set
- [x] A checkout with no name takes the repository's name
- [x] A worktree takes the same name as its main checkout
- [x] A directory that is not a repository still falls back to its own name
- [x] Nothing runs `git`
