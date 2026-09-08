# harrow

[![license: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Work a [cairn](https://oddurs.github.io/cairn) backlog from the terminal.

`cairn` keeps a project's roadmap and issues as Markdown files in the
repository, under a schema the project defines. Reading them one at a time is
fine; deciding what forty of them are worth is not. That is what this is for.

```
 harrow  harrow  7 items · 7 ready                                       updated just now
──────────────────────────────────────────────────────────────────────────────────────────
╭ Backlog · by milestone ──────────────────────────╮╭ 0018 feature ──────────────────────╮
│ ▾ v0.2  Triage that scales pa…▱▱▱▱▱▱▱▱   0%    5 ││ ○ Multi-select, for triage that is │
│  ○ 0018 + Multi-select, for triage that … 0/3 p1 ││ actually bulk                      │
│  ○ 0021 ~ Package it: crates.io and a tap 0/3 p1 ││   backlog · v0.2                   │
│  ○ 0017 + Watch the directory instead of… 0/3 p2 ││ ACCEPTANCE                         │
│  ○ 0019 + Show what changed, from the re… 0/2 p2 ││   0 of 3 ticked                    │
│  ○ 0020 + Answer proposals without leavi… 0/2 p2 ││                                    │
│ ▾ later  Someday              ▱▱▱▱▱▱▱▱   0%    2 ││ FIELDS                             │
│  ○ 0022 ? How much Markdown is worth renderi… p2 ││   priority p1                      │
│  ○ 0015 + Every cairn project on this ma… 0/1 p3 ││   area     chrome                  │
╰──────────────────────────────────────────────────╯╰────────────────────────────────────╯
 ↑↓ move  ↵ read  c claim  s status  x close  / filter  tab board  ? help
```

Run it anywhere inside a repository that has a `cairn.toml`, the way you run
`git`.

## What it is for

Triage. `cairn next`, `cairn claim` and `cairn close` are three short commands
and do not need a screen. What commands serve badly is moving through a backlog
— reading forty items, deciding what matters, changing a status and seeing the
columns rearrange — because every one of those decisions costs you an id typed
out again.

So: one keystroke per decision, the cursor stays where it was, and the whole
thing is read-mostly. Writing prose still belongs in `$EDITOR`, which `e` opens.

## Everything is the project's

harrow has no opinion about how your backlog is organised. The statuses are the
ones in your `cairn.toml`, in the order you declared them. So are the types,
their icons, the fields you can group by, the values a picker offers, and the
columns on the board. Even the colours: if the project says a bug is red, a bug
is red, and the theme fills in the rest.

## Install

```sh
cargo install --path .
```

`cairn` itself is optional for reading — a backlog is a directory of Markdown —
and required for changing anything. harrow says which of the two it is.

## Use

```sh
harrow                        # the interface
harrow -C ../other-project    # somewhere else
harrow --view now             # open in one of the project's saved views
harrow -f 'priority=p0'       # open filtered
harrow --group-by area        # grouped by something other than milestone
harrow -b                     # open on the board
harrow --plain                # one line per item, for scripts
harrow --doctor               # check everything harrow depends on
harrow --screenshot 120x40    # render one frame as text, no terminal needed
harrow --fix-terminal         # undo a terminal left in mouse-reporting mode
```

`HARROW_LOG=/tmp/harrow.log` appends diagnostics to a file.

### Keys

| key | |
|---|---|
| `↑` `↓` / `j` `k` | move between items |
| `←` `→` | previous or next group — a column, on the board |
| `space` | collapse or expand a group |
| `g` / `G` | first / last |
| `tab` | switch between the list and the board |
| `v` | group by something else |
| `enter` / `o` | read the item in full |
| `e` | open it in your editor |
| `c` / `C` | claim / release |
| `s` `p` `M` | set the status, the priority, the milestone |
| `h` / `l` | move it back or forward through the statuses |
| `x` | close, with a confirm |
| `u` | reopen |
| `n` | new item |
| `y` | copy the item's reference |
| `/` | filter, in cairn's own grammar |
| `a` | include finished and dropped items |
| `esc` | back out — clear the filter, close an overlay |
| `r` | re-read the backlog now |
| `ctrl-r` | reload the config and theme |
| `D` | diagnostics — what failed, and why |
| `m` | toggle mouse capture — off restores native text selection |
| `?` | help |
| `q` | quit |

The help overlay is generated from your bindings, not from that table.

### Filtering

`/` takes cairn's grammar, so anything that works after `--filter` works here:

```
priority=p0|p1,category!=done      clauses are AND, alternatives are OR
milestone=                         items with no milestone
blocked=true                       what is waiting on something
body~oauth                         full text
```

A clause with no operator is a plain search, so `/` is useful before any of
that has been learned. The list narrows as you type, and a field the project
does not have is reported rather than silently matching nothing.

## Configuration

Yours lives in `~/.config/harrow/config.toml`, and every key is optional. The
project's schema stays in the repository's `cairn.toml`, which harrow never
writes to.

```sh
harrow config --write        # a commented file with every default
harrow config                # what is actually in effect, and where it came from
```

```toml
theme        = "auto"
group_by     = "milestone"   # or status, type, area, assignee, none
sort         = ""            # cairn's spelling: "priority,-updated"
view         = ""            # open in a saved view from cairn.toml
show_closed  = false
board        = false
refresh_secs = 3
cairn        = "cairn"       # a path, if it is not on PATH
editor       = ""            # falls back to $VISUAL, $EDITOR, vi

[keys]
"x" = "close"
```

`ctrl-r` re-reads both files and repaints, so choosing colours does not mean
restarting. A file that will not parse is reported and the running config is
kept.

## Theming

harrow uses your terminal's colours. There is nothing to configure — if your
terminal is set to Gotham, harrow is Gotham, and it follows when you change it.

```sh
harrow --theme gotham        # for one run
harrow themes                # everything harrow can find
harrow themes gruv           # filtered — Ghostty ships hundreds
```

Built in: `auto` (the default), `mono` (no colour at all), `gotham`, `night`,
`paper`. Your own files go in `~/.config/harrow/themes/`. `NO_COLOR`,
`--no-color` and `TERM=dumb` all select `mono`, where the glyphs carry what the
colours would have. See [THEMES.md](THEMES.md).

## How it works

- **Reading** is direct. `cairn.toml` is parsed with the TOML crate and the
  item frontmatter with a small parser for the subset cairn writes — not by
  shelling out to `cairn export` and parsing the output back. It is the same
  information, it costs a millisecond instead of a process, and it keeps
  working when cairn is not installed. `harrow --doctor` asks cairn for a
  second opinion on the count, so a disagreement is visible rather than
  believed.
- **Writing** is not. Every change is a `cairn` invocation: `cairn claim 12`,
  `cairn set 12 status=doing`. cairn owns the write lock, the id allocation,
  the hooks that keep `ROADMAP.md` current, and the rules about what a valid
  item is. A second writer would own none of them.
- **Everything derived** — what is blocked, what is ready, how much of a
  milestone is done — is computed over the whole set when it is read, because
  none of it is a property of a single file.
- **The core performs no side effects.** `App` returns actions for the shell to
  carry out, including the `cairn` invocations, which is why every write can be
  asserted in a test with no repository underneath it.
- The backlog is re-read every three seconds and only sent on when it has
  changed, so a screen you leave open does not flicker at you.

## Cost

- Reading a 45-item project: **2ms**. A frame costs the same whether the
  backlog has 20 items or 2000; only the rows on screen are built.
- Idle, it redraws once a second, not ten times.

## What it deliberately is not

- **Not a second source of truth.** It shows the files. Close it and nothing is
  lost, because nothing was ever anywhere else.
- **Not an editor.** Prose goes in `$EDITOR`. This changes fields.
- **Not machine-wide.** It opens the project you are standing in, the way `git`
  does. Every cairn project on the machine in one list is [a different
  tool](cairn/items/0015-every-cairn-project-on-this-machine-in-one-list.md),
  and is held rather than planned.

## Durability

- Every subprocess has a deadline and is killed if it overruns.
- A failed read keeps the last good backlog on screen, marked stale, and backs
  off rather than hammering a directory that is not there.
- One unreadable item file is reported in the diagnostics overlay; the rest
  still load.
- The terminal is restored on quit, on panic, on `SIGTERM`/`SIGHUP`/`SIGQUIT`,
  and through an exit hook that covers every other way the process can leave.
  Only clicks and scrolling are requested — never motion tracking, which floods
  a terminal with a report per mouse movement.
- If some *other* program leaves your shell in mouse-reporting mode,
  `harrow --fix-terminal` clears it.

## Development

```sh
git clone https://github.com/oddurs/harrow && cd harrow
scripts/setup            # once: wires the git hooks
scripts/task check       # fmt, lint, test, build — exactly what CI runs
```

One unit of work is one branch, in one worktree, with one pull request:

```sh
scripts/agent doctor                        # is this checkout ready?
scripts/agent start fix/the-thing           # branch + worktree, prints the path
cd ../.worktrees/harrow/fix/the-thing
scripts/agent commit "fix(ui): the thing"
scripts/agent pr                            # checks, pushes, opens the PR
```

`main` only ever advances through a merged pull request — the `pre-push` hook
refuses to push to it, and branch protection refuses again if you get past the
hook. See [CONTRIBUTING.md](CONTRIBUTING.md) for the whole loop, and
[`scripts/task`](scripts/task) for the one seam every piece of automation talks
through: CI, the hooks and the agent script all run the same line, so they
cannot drift.

```sh
cargo test                                   # everything
HARROW_UPDATE_SNAPSHOTS=1 cargo test         # accept a deliberate screen change
HARROW_FUZZ_SEEDS=100000 cargo test --release --test invariants
```

The interface is the product, so it is held to recorded screens rather than to
descriptions of them — including a map of the foreground colours, since colour
is data too. Randomised key sequences are held to the state invariants, because
enumerating the ways modal layers, two views and a live filter interact is
hopeless by hand.

Its own backlog is in [`cairn/items`](cairn/items) — which is also the fixture
it was developed against. `ROADMAP.md` is generated from it; do not edit it by
hand.
