<!-- cairn:begin -->
## Roadmap and issues

This project tracks its roadmap and issues with `cairn`. Every item is a Markdown file under `cairn/items`, described by the schema in `cairn.toml`.

**Do not create ad-hoc TODO, PLAN or NOTES files.** Create a cairn item instead, so the work appears on the board and in the generated roadmap.

### The loop

1. `cairn next --view next` — what is ready to start. It excludes anything blocked by unfinished dependencies and puts work already in progress first.
2. `cairn claim <ID>` — take it before you start, so no one duplicates the work. `cairn claim --next --view next` picks and claims the top-ranked unclaimed item in one step, and prints its body so you can begin immediately.
3. Do the work. Record what you learn: `cairn set <ID> <field>=<value>` for fields, `cairn note <ID> "<TEXT>"` for anything that needs a sentence — why you chose something, what you tried, what to watch for.
4. `cairn tick <ID> <N>` as each acceptance criterion becomes true — `cairn show <ID> --criteria` lists them numbered. Tick what is true, not what would let you close.
5. `cairn close <ID>` when it is done, or `cairn release <ID>` to hand it back.
6. `cairn check` before you report finished. It must pass.

### Commands

```sh
cairn next --view next --json                 # ready work, ranked
cairn claim --next --view next                # take the next ready item
cairn search <TEXT> --json        # titles, bodies and labels
cairn list --json                 # all open items
cairn list --filter 'blocked=false,priority=p0'
cairn show <ID> --json            # one item, including its body
cairn new "<TITLE>" --type <TYPE> --milestone <MILESTONE>
cairn set <ID> status=<STATUS>    # also labels+=x, or any field below
cairn note <ID> "<TEXT>"          # append reasoning; never replaces
cairn show <ID> --criteria        # acceptance criteria, numbered
cairn tick <ID> <N>               # tick one; --all for every one
cairn close <ID>
cairn check                       # validate; run before finishing
cairn render                      # regenerate ROADMAP.md
```

Selection uses saved view `next`. Additional filters only narrow it; the view's sort and columns do not change `next` ranking. Over MCP, pass `{"view":"next"}` to `next_items` and to `claim_item` without an id. A direct claim is an explicit assignment outside this selection policy. Regenerate these instructions with `cairn agent --view next --write AGENTS.md`.

Claims coordinate writers in the same item directory, not separate branches, worktrees, or clones. Agree on assignments before splitting work.

Item identities are immutable UUIDv4 strings. Use full `id` values from JSON for durable references; commands also accept unambiguous prefixes of at least 8 hex digits. Store full identities in ID-reference fields, never prefixes. Migrated legacy numbers remain lookup aliases; new items do not receive numbers.

### Schema

- **Types**: `feature`, `bug`, `chore`, `decision`, `spike`, `docs`, `milestone`
- **Statuses**: `backlog` (open), `planned` (open), `doing` (active), `blocked` (active), `done` (done), `dropped` (dropped)
- **`due`**: date, YYYY-MM-DD — when a milestone is meant to land
- **`part_of`**: names any items, by id, several allowed — a larger piece of work this belongs to
- **`priority`**: one of p0, p1, p2, p3 — p0 is a release blocker
- **`effort`**: one of s, m, l, xl — Rough size, not an estimate
- **`area`**: one of read, filter, chrome, write, theme, config, cli, runtime, testing, docs, packaging — Subsystem this touches
- **Milestones**: `v0.1`, `v0.2`, `v0.3`, `v0.4`, `v0.5`, `v0.6`, `v0.7`, `v1.0`, `later`
- **Saved views** (`cairn list --view NAME`): `now`, `next`, `waiting`, `triage`, `decisions`, `history`

### Rules

1. Before starting work, find or create the item and set it to an active status.
2. Use the fields above rather than inventing new ones; add new fields to `cairn.toml` first.
3. Never hand-edit the generated roadmap file — change items and run `cairn render`.
4. `cairn check` must pass before the work is considered done.

<!-- cairn:end -->
