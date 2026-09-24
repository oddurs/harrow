---
id: 107
title: A log screenshot test fails under full-suite load
type: bug
status: dropped
created: 2026-09-23
updated: 2026-09-23
priority: p2
area: testing
effort: s
---

## What happens

`a_screenshot_of_the_log_shows_the_history_rather_than_the_asking` failed once
during a `pre-push` run of the whole suite, on the assertion that the frame no
longer says *Asking the repository*. It has passed on every run since —
alone, in its own file, and in two full-suite runs.

So the log lens rendered the question it asks *before* the repository answers,
which means the `git` call had not finished when the frame was drawn.

## What I could not establish

Not slowness in the path itself. Measured directly on the same fixture shape:

    harrow -C <tmp> --config <cfg> --screenshot 100x20     0.075s total
    harrow -C <tmp> --lens log --plain                     0.057s total

The fixture is not a git repository, so `git` fails immediately and harrow
says *not a git repository, so there is no history to read*. The timeout it
would have to exceed is `write_ms`, which defaults to 8000.

The 52 seconds an earlier single-test run reported was compilation of the
binary under test, not the test.

What is left is a race under the load of thirty-one test binaries running at
once — plausible, and not demonstrated. It was first seen on a run that also
carried the new `plain_answers_the_log_from_the_repository`, which spawns
`git init`, `git add` and `git commit`; that is more load and more subprocess
contention, and it is the honest suspicion rather than a proven cause.

## What should happen

A screenshot of the log either shows history or says there is none, however
loaded the machine is. Establish whether this is the timeout, subprocess
spawn contention, or something in the path that only shows under concurrency
— and only then decide between raising the bound for the test, making the
wait explicit, or fixing what is actually slow.

Do not fix it by asserting less. The assertion is the behaviour 0093 added.

## Reproduction

Not reliably. Seen once in `scripts/agent check` under `pre-push`. A loop of
full-suite runs on a loaded machine is the way to reach it.

## Acceptance criteria

- [ ] The cause is established rather than guessed
- [ ] The test passes under sustained full-suite load
- [ ] The assertion about *Asking the repository* is unchanged

## 2026-09-23

Duplicate. The cause is 0077, already filed with the same analysis and the same proposed fix, and found the same way — a test that passed under check and failed under pre-push.

Established rather than guessed: a git hook exports GIT_DIR, every git subprocess a test spawned inherited it, and so the fixture's git init and git commit operated on harrow's own repository. One run left a stray commit titled 'file the backlog' on the branch, carrying the fixture's items. The screenshot test then read harrow's history instead of the fixture's, which is long enough to exceed the 8000ms bound and draw the frame that says it is still asking.

Both are fixed: the product in 0077, and the suite by clearing the same variables for every subprocess tests/cli.rs spawns. Dropped rather than closed, because nothing was wrong with the test.
