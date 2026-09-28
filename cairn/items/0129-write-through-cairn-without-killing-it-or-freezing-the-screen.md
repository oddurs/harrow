---
id: 129
uid: 2f93b040-6166-44d8-8f14-f0cfa5df9a9b
title: Write through cairn without killing it or freezing the screen
type: bug
status: done
milestone: v0.8
assignee: oddurs
created_by: cairn-26
created: 2026-09-28
updated: 2026-09-28
closed_at: 2026-09-28
priority: p1
area: write
---

## Problem

Every write runs on the interface thread (main.rs dispatch, then run_change,
then exec::run with write_timeout, 8 s by default). For those seconds the
screen is frozen. At the deadline harrow SIGKILLs cairn, and a cairn killed
while it holds the project lock, or the cross-worktree claim lock, leaves the
lock file behind. Later writers then fail for up to five minutes, until cairn's
stale-lock breaker clears it. Cairn now ends every lock wait on its own: 10 s
for any one holder, 120 s in all (cairn 0161). So killing a write buys harrow
nothing, and it risks stranding a lock.

## Proposal

- One background writer thread. The interface keeps drawing and taking keys
  while cairn works, and one change is made at a time: a second is refused
  until the first lands, because it would be worked out from a screen that has
  not caught up.
- At write_timeout, harrow stops waiting visibly: the footer says the write is
  still going. When it ends, the footer reports the outcome, as today.
- A hard limit past cairn's own bounds (150 s at first; five minutes, cairn's
  own age for an abandoned lock, after review). By then only a hung hook can
  still be running, and hooks run after the lock is released. So the write is
  killed then, and the footer says so.
- Quitting waits for queued writes to finish, so none is cut off partway.
- Reads (log, prompt, check, activity) and git keep their short kill: they
  take no cairn lock.

Agreed with the harrow-51 session, which found the problem while reviewing
cairn 0161.

## Acceptance criteria

- [x] A write no longer blocks the interface: the next key is handled while cairn runs
- [x] A change asked for while another is being made is refused with a reason, never worked out from a screen that has not caught up
- [x] A write past write_timeout is reported as still going, and its outcome is reported when it ends
- [x] No write is killed before cairn would treat its lock as abandoned; one killed then says so
- [x] Quitting with a write queued waits for it

## 2026-09-28

Review (/code-review high) of the first version, a FIFO queue, found the queue itself was wrong. Keys are worked out from what is on screen, and a queued write has not reached the screen: > twice under a busy lock submitted status=planned twice and landed one step, not two, and an undo built from the stale value would restore the wrong one. So one change at a time: App::accept_write refuses a second change with 'still writing — <what> — try again when it lands', while navigation, reading and everything that does not write keep working. The criterion that asked for queue order is replaced by that one, here, on this item's own branch before it merged. The writer keeps its queue internally, since it costs nothing; order is still tested at that level.

## 2026-09-28

The same review, the rest, all taken: (1) the 'wrote recently' window was 3 s from submit, so a write waiting on the lock came back announced as somebody else's news; now App::writing marks the whole time a write is in flight, and the 3 s runs from when it lands. (2) The timeout path refreshed without marking the write ours; fixed. (3) Ctrl-C during the quit wait, or a terminal hangup, went to cairn as well, stopping it with the lock in hand; writes now run in their own process group (Unix). (4) Writer::submit swallowed a send error; it returns one, and the footer says the writer has stopped. (5) write_ms still read as a write deadline; its doc and the config template now say it is when harrow says a change is still going, and the deadline for reads. (6) The timeout toast said the hook had been stopped, but only the direct child is killed; it now says a hook may still be running. (7) At quit, already-finished writes were counted as left and their failures were dropped; they are drained and reported first. The overdue toast replacing a result toast in the same frame cannot happen with one write in flight. Not taken: harrow's SIGTERM watchdog still exits about 400 ms after a signal without waiting; the write in flight is in its own process group and carries on to the end, so nothing is cut off.

## 2026-09-28

Second high review, and the design it led to. The first fixes left real holes: the write's output still went through pipes, so harrow quitting or dying took the reading end away and cairn or a hook died on its next line of output — the cut-off this item exists to prevent; stdout was read to its end before stderr, so a hook saying more than a pipe holds deadlocked the write for 150 s; marking the whole in-flight time as 'ours' swallowed other agents' news during exactly the long waits; a dead writer thread left the slot taken forever; only cairn was killed at the limit, not its hook; and every path that ends a write lived in main.rs, untested. So, rebuilt smaller: exec::run_write sends cairn's output to files, not pipes, runs it in its own process group, and at the limit kills the group (through kill -KILL -pgid; std cannot signal a group). writer::Writing is one write on one thread in one slot; a thread that dies still comes back as an ended write. App::finished_writing handles every end and frees the slot, and while a write is in flight only the items it acts on (App::targets at the moment it was accepted) are kept out of the news. Reads keep their pipes and their short kill, and run_advised is no longer grouped, so check and the other reads are unaffected. write_ms is documented as when harrow says a change is still going and the deadline for reads; the template interpolates writer::LIMIT.

## 2026-09-28

Evidence: tests/writer.rs — starting does not wait; the still-going notice is said once and the write still ends; the limit is past cairn's own two minutes; a write past the limit is stopped with the hook it started (a mutation that kills only cairn fails it: 'the hook it started outlived it'); a megabyte on stderr does not stall it; a refusal comes back with what was said. tests/interaction.rs — a second change is refused, and every end (success, timeout, refusal, a writer that stopped) frees the slot for the next; a write stopped at the limit says so and is read again; one that never ran is not; during our write only our own item is kept out of the news, and another item's change is still announced. Not tested, and argued instead: that harrow dying mid-write no longer cuts cairn off — the output goes to files the child holds open, so there is no reading end to lose. Quitting waits on the one write in flight (Writing::wait).

## 2026-09-28

Third high review closed the earlier findings and found new ones, all taken. (1) e was not refused while a write was in flight, so an editor opened on a file cairn was about to change could write the old one back over it: e and every write command are now refused before any box opens (App::run), which also means (2) nothing typed into a note, Result or reason box is thrown away by a refusal. (3) For new and split, 'ours' was the cursor's item: a change that creates items now keeps the news quiet while it is made, since cairn has not chosen their numbers; any other keeps only its targets quiet. (4) accept_write started the 3 s window, swallowing everyone's news at the start; the window now starts when the write lands. (5) Scratch files had guessable names, were opened create+truncate and 0644, and outlived a harrow that did not unwind: now create_new, 0600 on Unix, unlinked as soon as the child has them, read through the handle and capped at 1 MiB. (6) The group was signalled through a kill found on PATH with its result ignored: now libc::kill(-pgid) (libc is already a dependency), and ExecError::Stopped says whether the whole group was stopped, which the toast repeats. (7) Every end of a write now re-reads, since one that failed to report may still have landed, and harrow's own failure is no longer called cairn refusing. (8) A write in flight shows 'writing — <what>' in the top bar for as long as it takes, not one toast. Evidence added: while_writing_no_box_opens_and_no_editor_either, a_write_stopped_at_the_limit_says_whether_its_hooks_stopped_too, a_write_harrow_lost_is_not_reported_as_refused, a_creating_write_keeps_the_news_quiet_while_it_is_made, during_our_write_only_our_own_item_is_kept_out_of_the_news (no longer backdated), a_write_leaves_no_files_behind, and a unit test that a writer which died still ends the write. COMPATIBILITY.md gains the row 0122 left for this.

## Result

A write to cairn runs in the background: the screen keeps drawing and reading, one change is made at a time and a second is refused before any box opens, a write past write_timeout is said to be still going and shown in the top bar until it lands, and nothing is stopped before 150 s, past cairn's own lock bounds, when the whole process group is. Output goes to unlinked files, so a harrow that quits or dies does not cut cairn off; quitting waits for the write in flight.

## 2026-09-28

Fourth high review, all real points taken but one. (1) 150 s was not past cairn's bounds: claim holds the project's lock while it waits up to two minutes more for the cross-worktree claim lock, so a claim could be stopped holding a lock. LIMIT is now five minutes, cairn's own age for an abandoned lock (STALE_AFTER): a lock held longer is broken by the next writer anyway, so stopping cairn then strands nothing not already treated as stranded; the test asserts it. (2) A creating write quieted everyone's news; now only what appears while it is made is ours (App::appeared, computed on each reading), plus split's parent. (3) run_write took a non-zero exit with stdout for success, so a bulk change that stopped part way showed a green toast; a write that does not succeed now fails, whatever it printed. (4) A (accept) and propose opened their box while a write was in flight; they are refused first now, and the test presses A and t too. (5) An error from try_wait freed the slot with cairn still running; it now stops cairn first. (6) The child's handle shared the parent's file offset, so a hook left running in the background could write over what cairn said; the child now gets its own appending handle. (7) The quit path used eprintln!, which panics on a hung-up terminal before the wait; it writes and ignores the error. (8) Tests: the top-bar indicator (a_write_in_flight_is_shown_until_it_lands), the scratch file's 0600 mode, a write that fails after printing, and the no-leftovers test moved to a binary of its own so parallel writes cannot upset it. Declined: a hook that needs a terminal stops when it reads it, because a write runs in its own process group; without the group, Ctrl-C or a hangup would stop cairn with its lock in hand, and a hook reading the terminal under a full-screen harrow was never going to work. Documented in COMPATIBILITY.md.
