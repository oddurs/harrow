---
id: 2
key: v1.0
title: Stable release
type: milestone
status: backlog
depends_on:
- 25
created: 2026-09-08
due: 2027-03-01
---

Documented, tested, and safe to depend on.

## What 1.0 means here

Not *finished*. A version number is a promise about what will not change
under somebody who has already adopted the thing, and harrow has not made
one. Three questions have to have written answers before the number is worth
anything.

**Can I rely on it?** The interface is the product and is expected to keep
improving — that is what the recorded screens are for. What must hold still
is everything somebody builds a habit or a config file on: the keys of
`config.toml`, the action names a `[keys]` table binds to, the theme roles a
theme file sets, the command-line flags. And the architectural guarantee
underneath all of it, which is that harrow reads the files directly and hands
every write to cairn, so a backlog is never harrow's to own and harrow is
never required in order to read one.

**Will it hold up?** Today harrow is tested against one fixture of six items
and driven by a randomised suite that knows forty keys. Neither has ever seen
a backlog the size of a real one, a schema somebody else designed, or a
terminal that cannot draw `◐`. The rebuild is already superlinear — ten times
the items costs thirty times the work — and it runs on every keystroke typed
into the filter box.

**Can I take it back?** harrow computes the exact undo of every write it
makes and prints it in a toast that disappears after three seconds. One
keystroke per decision is only a good trade if a decision is cheap to
reverse.

## What is deliberately not in it

Anything that needs harrow to know about more than one repository, or to
notice something while it is not running. Those are the boundaries cairn drew
for itself and they are the right ones here, for the same reason: a program
that runs when you invoke it, inside one clone, cannot meet them honestly.
0015 is filed under *someday* and that is where it belongs.

Rendering more Markdown than a pane this size earns (0022) is a spike, not a
commitment, and 1.0 does not wait on the answer.
