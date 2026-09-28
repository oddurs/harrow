# Cairn and Harrow

Cairn owns the format and every write. Harrow independently reads that format
and supplies the human interface. Neither needs a shared runtime library or a
Cairn process on each read.

Harrow **0.2.0-alpha.1** is paired with Cairn **1.0.0-alpha.1**, revision
`68ac1549383870aa57548da65ed25119895bb201`, tagged v1.0.0-alpha.1, which writes
format 5. These are
development versions, not a stable 1.0 promise. The 66-case corpus covers
formats 1–5, including frozen older corpora, and a `## Result` section as
specification §10.2 reads one. Upgrade both tools before
migrating: Cairn's latest release, 0.3.0, does not understand format 4 or 5.

Format 5 stores a number under `id`, as formats 1–3 did, and a UUIDv4 `uid`
tag that never changes. A type may declare its own rendering, and a type's
prefix must name an item of that type: `BUG-13` for a feature is refused, as
Cairn refuses it. A full tag, or a prefix of at least eight hex digits with a
letter in it, finds its item; all-digit text is always a number. Worktree
copies are matched by tag where both carry one, so an item renumbered on
another branch is still recognised, and the same number on a differently
tagged item is a collision rather than a copy.

Format 4 stored immutable UUIDv4 identities. Screens use unambiguous short
prefixes; writes, undo, history requests, clipboard copies and every `--plain`
lens carry full identities. `_legacy-ids.toml` preserves the frozen aliases from
a migration. Harrow reads this file directly and never allocates or rewrites
identities. Migrate each project once, commit the result, and merge that commit
to other branches instead of migrating each branch independently.

## What each command needs

Harrow asks the `cairn` on `PATH` once, at startup, what it can do, by reading
that command's `--help`. It never finds out by trying. A command the installed
cairn lacks is not offered: it has no key, the help and the palette leave it
out, and nothing on screen promises it. Everything harrow *reads* comes from
the files and needs no particular cairn.

| harrow | Cairn command | First in Cairn | Without it |
|---|---|---|---|
| `x` asks what the item concluded | `close --result` | `cd547d3` (cairn#112, 0155) | `x` asks only for a yes, as before |
| `P` shows the item as a prompt | `prompt` | `5569798` (cairn#113, 0156) | `P` is unbound and not offered |
| `split` in the palette | `split`, `split --dry-run` | `f91642a` (cairn#115, 0157) | the palette does not offer it |
| `C` shows prompt checks | `check --prompts` | `2fb5f31` (cairn#116, 0158) | `C` runs plain `cairn check`; the overlay is unchanged |
| a write is waited on, not killed | lock waits that end on their own | `9c29249` (cairn#121, 0161) | an older cairn can wait on a stuck lock for as long as harrow's limit, five minutes |

A write runs in a process group of its own, so a Ctrl-C or a hangup meant for
harrow does not stop cairn part way. A cairn hook that needs a terminal — a
passphrase prompt, an interactive signing agent — does not work under harrow,
which owns the terminal: it waits until the limit and is stopped with the rest.
Run such writes from a shell.

The pinned revision above has all of them, and CI builds it. A finished item's
`## Result` is read from the file itself (specification §10.2). Showing one, in
the detail pane, the reader and what an item builds on, needs no cairn at all.

## The gate

`scripts/task check` always runs the vendored format corpus. CI also requires
`scripts/task agreement`, against the pinned Cairn checkout in `ci.yml`:

```sh
PATH=/path/to/cairn/target/debug:$PATH \
CAIRN_PROJECT_DIR=/path/to/cairn scripts/task agreement
```

Missing executables, fixtures, or mismatches fail. The comparison is marked
ignored only in the standalone Rust suite, so a checkout remains testable
without Cairn. Required CI explicitly runs it; it is not an optional release
check. It covers Cairn's project views via `cairn list --json` and `harrow --plain`,
plus query fixtures for dates, categories, dependencies, criteria, hierarchy,
UUID prefixes, ambiguity and migrated numeric aliases. Every item's Result, in
the pinned Cairn and in bodies built around fences, heading levels and notes,
must read the same in both tools, null for null: a dependent quotes it.

To advance the pin, inspect Cairn's format/CLI changes, refresh the corpus with
`CAIRN_REPO=/path/to/cairn scripts/task conformance:refresh`, review every diff,
update CI's exact revision, and run both suites. `PROVENANCE` identifies the
corpus source. Do not edit an expectation to conceal a disagreement.

Refresh from a **clean checkout of the exact revision being pinned**, not from a
working copy. A sibling checkout is somebody's desk: ours held uncommitted
work toward a later version, and refreshing from it would have vendored an
unreleased corpus under a released version's name. Clone the revision, and check
that the tree is clean and `cairn --version` is the version you mean —
`target/debug/cairn` in a dirty checkout is not the release it is beside.

## Deliberate differences and shared semantics

- Harrow accepts free text in its interactive filter; Cairn uses `search` for
  that. Shared saved views should use field predicates.
- Harrow rejects an unknown filter field immediately. Cairn warns at query
  time and checks schema problems with `check`. Neither should quietly claim a
  misspelled field is a meaningful empty backlog.
- Grouping can draw containers as headings. Compare ungrouped item sets, not
  screen rows or their order. `next` additionally excludes containers, ranks
  active work first, and applies dependency readiness.
- `ready` means unfinished and dependency-ready, including active items. It
  does not mean unassigned or approved. Use your project's saved view.
- `criteria_met` is true when no criteria are stated; `criteria>0` distinguishes
  an explicitly completed checklist from no checklist.
- An ordinary listing leaves out closed work and containers. Naming the field
  is how you ask for them back, and naming it means saying anything about it —
  `status=done`, `status!=dropped` and `category!=dropped` all lift it, with
  the predicate itself doing any excluding. A closed container is behind both
  defaults and needs the type and the status. Both tools read it this way;
  harrow once required an equality, which made `category!=dropped` differ by
  every closed item (0106).
- Range comparisons use the display value; missing values compare as empty
  strings. Use `closed_at!=,closed_at<2026-09-10` to exclude undated history.
- `closed_at` is the recorded completion date, unaffected by later edits.
  Completion statistics prefer it. For legacy items lacking it, statistics
  retain their old `updated` estimate; date filters never invent a value.
- Harrow refuses unknown future formats and unfinished identity migrations.
  It must not reinterpret an identity using a format it has never tested.

For a mismatch, include both tool versions/revisions, `cairn config --json`,
`harrow --doctor`, the filter or view, and a small non-sensitive reproduction.
