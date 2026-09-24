---
id: 77
title: The log reads the repository it was pointed at
type: bug
status: done
milestone: v0.7
created: 2026-09-15
updated: 2026-09-23
closed_at: 2026-09-23
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

- [x] The log reads the project's repository with `GIT_DIR` set to another one
- [x] A test sets it, rather than the suite happening to be run that way

## 2026-09-23

Fixed. exec::git clears GIT_DIR, GIT_WORK_TREE, GIT_INDEX_FILE, GIT_OBJECT_DIRECTORY, GIT_ALTERNATE_OBJECT_DIRECTORIES, GIT_COMMON_DIR, GIT_NAMESPACE, GIT_PREFIX and GIT_CEILING_DIRECTORIES for the child.

It is a separate entry point rather than an argument to exec::run, which is the part this item said was worth thinking about. run takes an argv and a deadline; making every caller say something about the environment to get ordinary behaviour is a worse trade than git having its own way in, and nothing else harrow spawns is sensitive to where it was launched from.

the_log_reads_the_project_even_when_a_hook_names_another_repository builds two repositories, points GIT_DIR and GIT_WORK_TREE at the wrong one, and asserts the answer comes from the project. Against the unfixed code it fails, and takes 71 seconds doing it — which is the same slowness that made the screenshot test time out under pre-push.

Raised in priority by v0.7: with --plain --lens log there is now every reason to run harrow from a hook or a CI step, which is where this bites.
