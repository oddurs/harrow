---
id: 80
title: Two parts of the screen disagree about what counts as work
type: bug
status: done
milestone: v0.5
created: 2026-09-18
updated: 2026-09-18
priority: p0
area: read
---

## What happens

In poptop the strip says `○ 7 backlog` and the list says *"Nothing open
here."* Both are drawn from the same backlog, in the same frame. All seven
are milestones, every work item is closed.

Two true-looking numbers that cannot both be right is worse than either
being wrong, because there is nothing on screen to say which to believe.

## Why

`Schema::is_container` asked the wrong question first:

```rust
if let Some(kind) = self.item_type(kind) {
    return kind.groups.is_some();      // format 3
}
self.format < 3 && self.fields.iter().any(...)   // format 2
```

Every type an item can have is declared, so the first branch always
returned, and `groups` does not exist before format 3. The format 2 rule was
unreachable — in *every* format 2 project, which is most of them, nothing was
a container at all. Milestones were counted as work in the strip, dealt onto
the board as cards, tallied by type in the statistics and named as the oldest
open item; only the list left them out, and it left them out for an unrelated
reason.

Pulling that thread found two more faces of the same rule:

- **Naming a status did not reveal closed work.** `harrow -f status=done`
  listed nothing in a project with seventy finished items. cairn returns all
  seventy: *"naming a type is how you ask for them, which is the same rule
  closed items follow for status."* An empty answer a reader cannot
  distinguish from an empty backlog is the defect this whole item is about,
  and `tests/agreement.rs` is supposed to hold the two tools to the same ids.
- **The grouping decided whether `a` worked.** An item of the type being
  grouped by is drawn as the heading rather than as a row. That exclusion was
  unconditional, so under the default grouping `a` — *"show everything:
  finished, dropped, milestones"* — showed no milestones, and
  `type=milestone` returned none, while every other grouping returned all
  seven.

## What should happen

One rule, in one place, for both of the things an ordinary listing leaves
out: **absent unless you ask for them by name.** That is cairn's rule, and
`is_container` is now cairn's disjunction rather than a branch on the format
— *either* the type declares `groups` *or* a field names it as a specific
`target`. A project part-way through `cairn migrate` satisfies one and not
the other, and either answer alone is wrong for somebody.

The empty state then says what is being withheld rather than that there is
nothing:

    Every piece of work here is finished.
    7 milestones still open, and 70 finished. a shows them.

cairn arrived at the same conclusion about its own listing: *"the filter is
not what was wrong, so saying the filter found nothing sent people to
rewrite it."*

## The milestone that has shipped

cairn has `finished_but_open` — a container, still open, with every item
under it done — and deliberately refuses to act on it:

> Closing one automatically would be cairn deciding that finished work means
> a shipped milestone, and those are different claims: a project's `later` or
> `Someday` milestone can have every item under it done and be meant to stay
> open for good.

So harrow reports the same thing and takes the same position. It belongs in
the needs-you queue, which is already where harrow collects what is waiting
on a person, and the closing stays a key somebody presses. A container with
nothing filed under it is not reported: nothing filed is vacuously complete.

(cairn's version is in its working tree, uncommitted, and reaches only
`cairn roadmap`. The wording here follows it and may need to follow it
again.)

## Acceptance criteria

- [x] The strip and the list agree about what counts as work, in a format 2 project and a format 3 one
- [x] Every number in the strip is what clicking it lists
- [x] A milestone is not a card, and is not a type in the statistics
- [x] `status=done` and `category=done` list finished work, agreeing with cairn
- [x] `a` and `type=milestone` show milestones under every grouping, agreeing with cairn
- [x] The empty state names what it is leaving out and how to see it
- [x] A finished-but-open container is reported where a person will see it, and never closed automatically

## 2026-09-18

Found by the report, but the reported symptom was the smallest of the three. The strip counting milestones was one broken rule; naming a status not revealing closed work, and the grouping deciding whether `a` worked, were the same rule broken in two more places. All three are now one predicate each: `belongs` for what an ordinary listing withholds, `is_row` for what is a heading rather than a row.

`tests/counting.rs` holds the promise the strip makes in its own doc comment — *each count is exactly what clicking it would give you* — by clicking every cell through the hit map and counting the rows. That is the test that would have caught this, and it failed on the sample project too, for the closed-status half.

Checked against cairn rather than reasoned about: `is_container` is now its disjunction verbatim, `status=done` and `type=milestone` return the same ids in both tools on poptop, and the finished-but-open signal takes cairn's phrase and cairn's refusal to act on it. Worth knowing that cairn's own ROADMAP.md has the same defect from the other end: v0.2 renders 100% while 0081 through 0084 sit at backlog.
