# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

Day-to-day work is tracked as [cairn items](cairn/items) and rendered into
[ROADMAP.md](ROADMAP.md); this file records what landed in a release.

## [Unreleased]

## [0.2.0-alpha.1] - 2026-09-28

The first release. A development version, paired with Cairn 1.0.0-alpha.1:
it reads Cairn formats 1–5 without cairn installed, and writes through the
`cairn` on PATH, offering only the commands that cairn has. See
COMPATIBILITY.md.

### Added

- Results and prompts, with cairn's own. A finished item's `## Result` comes
  first in the detail pane and the reader, and **Builds on** says what each
  finished dependency concluded. `x` asks what an item concluded and records it
  with `cairn close --result`. `P` shows an item as the prompt `cairn prompt`
  compiles, and `y` copies it whole. `:split` turns an item's numbered steps into
  items of their own after showing each. `C` shows `cairn check --prompts`
  findings under their own heading. A command the installed cairn lacks is not
  offered. See the README's *Results and prompts*.

- Five lenses on one backlog, each answering one question: **needs you**
  (proposals, cold claims, finished work left open, work nobody owns), **the
  list**, **the board**, **the stats** and **the log** (what changed, read from
  the repository). `tab`, `shift-tab` and `1`–`5` move between them, and
  `--lens` opens any of them. `--plain --lens needs` and `--plain --lens log`
  answer the same questions for a script.

- Every lens obeys the filter, answers the mouse, reaches the detail of what
  is selected, and keeps the selection and the marks across a switch.

- A filter panel on `f`, built from the project's own schema, beside the
  `/` box that takes cairn's own grammar; the project's saved views on `V`
  and `--view`, each bringing its own grouping.

- A toolbar that says what is on screen and why: the filter, the order and
  the arrangement, each a dropdown, beside counts that filter when clicked.

- A reader panel on `enter`, beside the backlog rather than over it, and a
  detail pane that renders the body's Markdown and reads as a thread.

- Changing work without leaving: claim and hand back with a reason (`c`,
  `C`), tick a criterion that came true (`t`), add a note (`N`), propose a
  change rather than make it (`ctrl-p`) and accept one (`A`), move through
  the statuses (`<`, `>`). A claim the project would call stale is marked.

- `:` finds every command by name; every command has a name before it has a
  key, and the help is generated from your bindings.

- Work under way in other worktrees of the repository, beside the record: an
  item claimed on another branch turns, the detail names the branch, and the
  claim is announced as it happens.

- Items filed in another worktree appear as they are written, as rows marked
  with their branch. They are read-only here, and `--plain` leaves them out.
  When you have not touched a key for a few seconds, the cursor goes to what
  just changed and the list scrolls to it.

- Nerd Font icons where the terminal has them: GitHub's issue states, a
  filling pie for work under way, a joined progress bar, and icons on the
  lenses, the toolbar and the section headings. `glyphs = "auto" | "nerd" |
  "unicode"` and `--glyphs`; `auto` chooses Nerd only on Ghostty, WezTerm or
  kitty with a UTF-8 locale, and `--doctor` prints a sample.

- `[` and `]` pick a link in the detail pane, and `enter` follows it: every link
  can be reached without the mouse.

- `harrow completions fish|bash|zsh` and `harrow man`. Both are generated from
  the same table as `--help` — which the usage text and the unknown-option check
  now read too, so a flag cannot work while nothing says it exists. `--theme`
  completes from the themes the machine actually has. The man page lists every key and
  every command, generated from the bindings in force.

- Release binaries for macOS and Linux on arm64 and x86_64, with checksums and a
  build attestation, and a Homebrew formula generated from them rather than
  transcribed. `brew install oddurs/tap/harrow`.

- A change is watched out rather than deleted. An item that has just moved off
  the screen — closed, most often — is held where it landed for a few seconds,
  drawn as what it has become, before it goes. A close reads as a movement
  between two states instead of the row vanishing under the cursor. Narrowing
  a filter still drops rows at once: the item has to be what changed, not the
  view.

- Every pane scrolls on its own, and the wheel moves the one under the pointer
  rather than always the list. The detail pane shows a whole item by scrolling
  — `K` and `J`, bound as `detail-up` and `detail-down` — instead of
  truncating the body to what was left after the fields, and its position
  belongs to the item, so selecting something else starts at the top of it.
  Each board column scrolls where it sits, so `done` can be read without
  moving the cursor out of `doing`. The stats pane scrolls too, which is how
  its bottom is reached on a short terminal. A pane holding more than it shows
  says so on its own bottom edge.

- Proposals. `cairn propose` writes the request into the item's body, so harrow
  reads it the way it reads everything else: an item with one carries a `?`, the
  header counts them, and the detail pane shows the field, the change, who asked
  and why. `A` accepts it through `cairn proposals --accept`.

- `H` shows how an item got the way it is, read from the repository through
  `cairn log` rather than by asking git a second time. A project that is not in
  git says so, which is a different answer from an empty history.

- A filesystem watcher, so a change made in another window shows up at once
  instead of on the next poll. The poll stays as the backstop for the places a
  watcher does not work, bursts are settled before reading, and there is a floor
  between reads so a directory being rewritten continuously cannot become a busy
  loop. `watch = false` turns it off; `harrow --doctor` says which is in use.

- Marking. `space` marks the item under the cursor, ctrl-click marks one and
  shift-click a range, and the next change applies to all of them — in a single
  `cairn` invocation, after a confirmation that says how many. Where the marked
  set is exactly what the filter is showing, it becomes one `cairn set --filter`
  rather than a list of ids.

- A mouse interface. Tabs, statuses, rows, group headings, board columns,
  picker options, the confirmation and the footer hints are all clickable;
  double-click reads an item; a card dragged between columns sets its status.
  The map of what is clickable is built as the screen is drawn, so what you can
  see is what you can hit.

- A statistics pane, on `tab` or `--stats`: how much is closed, what is ready,
  blocked or claimed, what closed in the last week, what has waited longest,
  what is in the way of the most other things, how each milestone stands, and
  where the work sits by type and by priority.

- A status strip under the header, so what is happening is legible without
  reading a row, and a mark on anything that moved in the last forty-five
  seconds.

- The terminal's own palette, read with OSC queries at startup. Surfaces,
  borders and the selection are derived from it, and every hue is checked for
  contrast against the real background before it is used.

- `contains`, `descendants`, `depth`, `leaf`, `container`, `owner` and
  `created_by` resolve in the filter box, matching cairn's derived keys.

- The schema reads `agent` on a field or a status, so a project that restricts
  what tooling may change can say so. harrow shows the restriction where the
  choice is made and lets cairn enforce it.

- `.githooks/post-merge` settles ids and re-derives the roadmap after a merge,
  and `scripts/setup` registers cairn's merge driver.

### Changed

- **Cairn formats 4 and 5 are read.** Format 4 identifies items by immutable
  UUID; screens show the shortest unambiguous prefix, while writes, undo, the
  clipboard and `--plain` keep the full identity. Format 5 numbers items again,
  in each type's own rendering, and tags each with a `uid` that still finds it.
  Upgrade harrow before running `cairn migrate` on a live backlog.

- The detail pane's line of facts breaks between facts instead of clipping
  the last one, which was usually the name.

- cairn format 3 is read: a type declares `groups = "one"` or `"many"` rather
  than being called a container because some field happened to name it as a
  target. The declaration creates the field work uses to name one, so nothing
  downstream changes. A format 2 project still opens and still knows what a
  milestone is.

- The layout drops the detail pane below ninety-six columns rather than halving
  the list, and the rows keep fixed columns that degrade in a defined order.

- The detail pane wraps its own text, so a wrapped paragraph keeps its left
  edge, and reflows paragraphs rather than re-wrapping somebody else's line
  breaks.

- Containers — any type a reference field names, which for most projects means
  `milestone` — are no longer rows in the list or cards on the board. They are
  the headings work belongs to. `a` brings them back, as does asking for the
  type by name in the filter. This follows cairn, which removed them from
  `next`, the board and an ordinary `list` for the same reason.

- `a` and `--all` mean *all*: finished, dropped, and containers. The config key
  is `show_all`, replacing `show_closed`.

### Fixed

- A write to cairn is no longer killed after eight seconds, which could leave
  cairn's lock behind and fail every writer after it for minutes. Writes run in
  the background, one at a time, are said to be still going while they wait
  their turn, and are stopped only past cairn's own age for an abandoned lock.

- The diagnostics overlay shows what `cairn check` found, not only its summary.

- The reader no longer clips the last word of a long line.

- Completion dates now read, display, sort and filter as `closed_at`; a later
  edit no longer moves a recorded completion in statistics.

- Query agreement with Cairn includes active readiness, vacuously met empty
  criteria, hierarchy fields, negative alternatives, presence and range
  comparisons. An undeclared `container` filter is now reported as unknown;
  select container types through the project's declared type names.

- The board drew a `done` column and refused to put anything in it: `done 0`
  sat beside `✓ 1 done` in the strip, in the same frame, about the same
  backlog, and a card dragged onto it disappeared on arrival. The board is now
  dealt from its own set — the filter, and the rule that a container is not a
  card. Whether finished work is worth a row is a question about the list, and
  the board answered it when the project wrote `board = true`.

- Required CI compares against Cairn 1.0.0-alpha.1 at a pinned revision,
  including all of Cairn's project views, its conformance corpus and the
  Result of every item. See COMPATIBILITY.md.

[unreleased]: https://github.com/oddurs/harrow/compare/v0.2.0-alpha.1...HEAD
[0.2.0-alpha.1]: https://github.com/oddurs/harrow/releases/tag/v0.2.0-alpha.1
