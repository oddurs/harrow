---
id: 100
title: Give harrow a durable direction and a reconciled backlog
type: chore
status: done
milestone: v0.7
created: 2026-09-23
updated: 2026-09-23
closed_at: 2026-09-23
priority: p1
area: docs
effort: l
---

## Purpose

Assess harrow after several weeks of daily use beside Cairn, state what it is
for in a way that survives the next five years, reconcile a roadmap that had
drifted a long way from the shipped product, and configure this repository so
the next work is obvious.

## Acceptance criteria

- [x] Assess the interface, the reader, the contracts, the tests and the
      documentation against real workflows, with evidence
- [x] Record a durable product concept and its boundaries
- [x] Reconcile the backlog and the milestones without erasing the reasoning
- [x] Configure views and templates that fire on this project's actual data
- [x] Establish a small explicit approved queue
- [x] `cairn check` passes and the counterpart agreement still holds

## Assessment — 2026-09-23

Harrow should be **the lightweight terminal interface for understanding a Cairn
backlog and steering the work in it**. Cairn is the writer and the agent-facing
command surface; Git carries the history; harrow is how a person looks at the
record and decides what happens next. Its enduring value is that it is fast,
keyboard-first, local, reads the files directly, and writes nothing itself.
That is a sharpening of what it already is, not a new direction.

Weeks of the author's daily use are the strongest evidence available here. The
implementation is substantial and, in the places that matter, good. The gap is
not features: it is that the roadmap described a plan that finished shipping
some time ago, and that two of the four questions harrow can answer cannot be
asked from outside the program.

### What I inspected

Harrow at `e7397a5` and the neighbouring Cairn checkout at `cbe7b2c` (v0.3.0):
the CLI surface, the lens model and the questions each lens computes, the
schema reader, the filter grammar, the saved views, the conformance corpus and
its provenance, the agreement suite, `--doctor`, the snapshot suite, the
randomised key and pointer suites, `CONTRIBUTING`, `COMPATIBILITY`, `README`,
and the whole 99-item backlog. Harrow was run against its own project and
against Cairn's 144-item project, and frames were read with `--screenshot`.
This is a product and architecture assessment with targeted reproductions, not
a claim that every path on every terminal has been audited.

### What is already worth protecting

- **The boundary holds.** Harrow reads item files directly and every write is a
  `cairn` invocation. `App` returns an `Action` and the shell carries it out,
  which is what makes each write assertable without a repository underneath.
  There is no daemon, no database, no shared Rust library, and no second write
  authority. Keep all four absences.
- **The counterpart relationship is real, not aspirational.** `--doctor`
  reports `cairn 0.3.0 — agrees on 144 items` against Cairn's own project, and
  the agreement suite passes all three tests over all eight of Cairn's saved
  views. The vendored 36-case corpus is Cairn's own golden files with a
  `PROVENANCE` that says where they came from and refuses to let an expectation
  be edited to conceal a disagreement.
- **The lens contract.** `tests/lenses.rs` holds every lens to the same
  capabilities — obeys the filter, answers the mouse, reaches the detail, keeps
  the selection and the marks — and its `GAPS` table is gone because every row
  was paid off. Very few interfaces have that written down at all.
- **The needs-you queue asks the right four questions**: a proposal waiting on
  an answer, a claim gone cold, work whose criteria are all ticked and which is
  still open, and an item something else filed that nobody owns. That is a
  supervision model, and it is already correct.
- **It opens where the work is** rather than at the top, folds finished work
  behind one row, and states the view in force in a line that `Y` will copy
  back out as the command that reproduces it.
- **The interface is held to recorded screens**, and the randomised suites now
  press keys and drive the pointer. Both have caught real defects.

### Where the record and the product had diverged

| Evidence | Implication | Action |
| --- | --- | --- |
| v0.1 through v0.6 were 68 items, all done, and all six milestone items were still `backlog` | The roadmap described a plan that had finished shipping | Closed on the reconciliation date with outcome notes; no release dates invented |
| 0094 was `doing` with no assignee and no claim date, though it shipped in PR #89 on 2026-09-20 | The only item the `now` view showed was not being worked on | Closed with the shipping evidence |
| The `next` view filtered `status=planned`; nothing in the project had ever been `planned` | There was no approved queue — nothing said what to do next | Three items approved for v0.7 |
| The `triage` view filtered `milestone=`; every item in this project has a milestone | A saved view that could never return anything | Retargeted at open work nobody has sized, which returns 16 |
| 75 items are done and exactly one carries `closed_at` | Almost the whole history was closed by editing frontmatter rather than through `cairn close` — the boundary this project asserts, and in recent sessions that was me | Recorded, not backfilled: inventing completion dates would corrupt the record both tools promise not to invent |
| `--board` and `--stats` open a lens; `needs` and `log` have no flag | The two questions that carry time and attention cannot be asked from the shell | v0.7, items 0103 and 0104 |
| `--plain` prints items only | An agent supervising work cannot ask harrow what needs attention | v0.7, item 0104 |
| CI and `COMPATIBILITY` pin Cairn v0.2.2 (`335a4d3`); Cairn released v0.3.0 and pins harrow at `e7397a5`, which is current | The pin was asymmetric and a release behind | Verified agreement passes against v0.3.0; advancing the pin is its own deliberate change with its own evidence |
| Cairn declares no status icons; two open statuses render as the same glyph in the strip | `○ 4  ○ 3` is unreadable without colour | Recorded against 0048, which already owns the no-glyphs case |
| This clone had the `cairn` merge attributes tracked and no driver registered | Even this project's own Git setup was incomplete | `cairn init --git` run here; it is per-clone and cannot be committed |
| 0099 shipped with no milestone | Work that landed was not filed anywhere | Filed under v1.0, which is where the compatibility promise lives |

### The concept and its boundaries

There is one record: Cairn's item files and the schema the project declares.
Cairn validates and changes it. Git versions it. Harrow reads it directly and
helps a person understand and steer it. An agent uses Cairn.

That gives a test for every proposed addition to harrow: **does it help someone
see the state of the work, or decide what happens to it, faster than reading
the files?** If the answer is no, it belongs in Cairn or nowhere.

Keep the core small. No daemon, no separate database, no shared runtime library
with Cairn, no second write authority. Reading directly is what makes harrow
start instantly and work on a machine that has never heard of Cairn; writing
only through Cairn is what keeps validation, locking, hooks and identifiers in
one place. Those two choices are the product.

Do not spend this cycle on a web interface, a hosted service, an agent runner,
remote backlogs, embedded scripting, or aggregating every project on the
machine into one list. 0015 keeps the last of those visible as a someday item,
honestly filed as `later`.

### What to do, in order

**v0.7 — every question has a door.** Three items: 0103 opens every lens from
the command line, 0104 answers the needs and log questions in plain text, 0105
writes both down. This is one idea, and it is the one that makes the workflows
this project exists for work from outside the program.

**Then the pin.** Advancing the counterpart pin from Cairn v0.2.2 to v0.3.0 is
bounded, already verified to pass, and belongs in its own change with its own
corpus diff to read.

**v1.0 — safe to keep.** Fourteen open items is a bucket, not a milestone. It
holds real promises — 0044 says what is stable, 0047 says which platforms, 0043
ships a man page and completions — mixed with ordinary defects. Before it
means anything, decide what the 1.0 promise covers: the action names, the
`--plain` formats, the filter grammar, the config file. That decision has not
been made and should not be made as a side effect of this assessment.

**Later stays later.** 0015 and 0022 are honest someday items.

### What a good year would look like

A person opening harrow after a month should know what needs them without
reading the backlog. An agent should be able to ask, from a shell, what it has
left unfinished. A second reader should keep agreeing with Cairn across both
projects' releases. The files should stay readable when harrow is gone.

Measure that, not the number of lenses.

### Setup implemented here

Six saved views that fire on this project's real data — `now`, `next`,
`waiting`, `triage`, `decisions`, `history` — replacing two that returned
nothing. A `decision` type with a template, and templates for `chore` and
`docs`, which had none. The six shipped milestones closed with outcome notes.
The phantom active item closed. An approved queue of three. The v0.7 milestone
and its slice, filed and approved.

Item bodies are untouched. Milestones close on the reconciliation date because
they never had release dates and inventing them would put false dates in the
record. No completion date was backfilled for the same reason.

### Verification and limits

- `cairn check` passes: 105 items, zero warnings.
- `scripts/agent check` passes.
- `harrow --doctor` against Cairn: 144 items, eight views readable, agrees with
  `cairn 0.3.0`.
- `scripts/task agreement` against the Cairn v0.3.0 checkout: three tests pass,
  covering all eight of Cairn's saved views and the query fixtures.
- Frames were read with `--screenshot` for both projects.

This session did not publish a release, change a durability path, advance the
counterpart pin, contact anyone, or modify Cairn. The v1.0 promise is left
undecided on purpose.

## 2026-09-23

Reconciliation complete. Six milestones closed with outcome notes, the phantom active item closed, 0099 filed, six working views configured, a decision type and three templates added, and an approved queue of three established for v0.7.

Verified here: cairn check passes with 105 items and zero warnings; scripts/agent check passes; harrow --doctor agrees with cairn 0.3.0 on all 144 items of the Cairn project across its eight views; scripts/task agreement passes all three tests against the Cairn v0.3.0 checkout even though the pin still names v0.2.2, which is why advancing the pin is bounded and separate.

## 2026-09-23

Configuring the decisions view surfaced a cross-tool disagreement that the assessment had not found by inspection: harrow lifts its hide-closed default for an equality on status or category but not for a negation, so 'category!=dropped' returns 96 items in cairn and 18 in harrow. That is every closed item, and it is the shape this repository's own [render] include uses. Filed as 0106 and added to the v0.7 slice, because a second reader that disagrees with Cairn on a saved view is the one failure this product cannot tolerate. The agreement suite missed it because Cairn's eight views all use equalities.

## 2026-09-23

Every milestone carried an invented due date between 2026-12-01 and 2027-05-01, six of them on work that had already shipped. They are cleared. Removing them exposed what the dates had been standing in for: cairn orders milestone groups by the dependency graph first, then due, then id, and with no dates and no graph the roadmap fell into identifier order. The milestones now declare the sequence they actually happened in, which is true, orders the roadmap correctly, and commits nobody to a date.
