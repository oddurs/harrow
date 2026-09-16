---
id: 77
title: The log reads the repository it was pointed at
type: bug
status: backlog
milestone: v1.0
created: 2026-09-15
updated: 2026-09-15
priority: p2
area: read
---

## What happens

`harrow` launched from a git hook shows another repository's history on the log
lens. Git hooks export `GIT_DIR`, `GIT_WORK_TREE` and friends into the
environment, a child `git` reads them, and `git -C <project> log` then resolves
the *hook's* repository rather than the one `-C` names — `-C` changes the
directory, not the discovery.

Found by a test that asserted the temp fixture had no history: under
`scripts/task check` it was right, and under the `pre-push` hook running the
same command it was wrong, because the second one had `GIT_DIR` set.

## What should happen

The log reads the repository the project is in, whatever launched harrow.

## Proposal

Clear the git environment for the child: `GIT_DIR`, `GIT_WORK_TREE`,
`GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY`, `GIT_COMMON_DIR`, `GIT_NAMESPACE`.
`exec::run` takes an argv and a deadline and nothing else, so this is the first
caller that needs to say anything about the environment — which is the part
worth thinking about rather than the list of variables.

## Why it is p2

It takes launching harrow from inside a hook, which nobody does on purpose.
The cost of being wrong is a lens quietly showing somebody else's history, with
nothing to suggest it is not yours.

## Acceptance criteria

- [ ] The log reads the project's repository with `GIT_DIR` set to another one
- [ ] A test sets it, rather than the suite happening to be run that way
