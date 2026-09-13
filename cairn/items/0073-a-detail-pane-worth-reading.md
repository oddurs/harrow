---
id: 73
title: A detail pane worth reading
type: feature
status: doing
milestone: v0.5
created: 2026-09-13
updated: 2026-09-13
priority: p1
area: chrome
---

## Problem

The detail pane is where an item is actually read, and it renders a cairn
body about as well as `cat` does.

**Inline markup is deleted rather than rendered.** The whole of it:

```rust
fn plain_markdown(line: &str) -> String {
    line.replace("**", "").replace('`', "")
}
```

So `**must**` loses its emphasis and becomes ordinary prose, `` `cairn note` ``
stops looking like a command, `*maybe*` keeps its asterisks because nobody
handled single ones, and `[the spec](https://…)` is shown with its brackets
and its URL in the middle of a sentence.

**Block markup is thin.** All three heading levels render identically, so a
document's structure is flat. Fenced code blocks are not recognised at all —
only a four-space indent is — so a ```` ``` ```` block shows its own fences
as text. No block quotes, no horizontal rules, no numbered lists, no nesting.

**Nothing is clickable.** A URL in a body is a URL you retype. harrow is
mouse-first everywhere else and the pane where the prose lives is inert.

**The progress bars are decoration.** `▰▰▰▰▰▰▰▰▰▰  3 of 3 ticked` prints a
ten-cell bar next to the exact fraction it is approximating. The bar carries
less information than the number beside it and takes a line to do it.

## Proposal

**Render the markup instead of stripping it.** An inline pass producing
spans rather than a string: `**bold**`, `*emphasis*`, `` `code` `` in a
colour of its own, and `[text](url)` as the text alone, marked as a link.
Block level: heading levels that differ, fenced code as well as indented,
block quotes with a gutter, horizontal rules, ordered lists, and nesting
that indents.

**Make the links work.** The body already becomes `Vec<Line>` and the pane
already knows where it draws them, which is the same shape as the stats
pane's doors: record where each link landed, register a hit, and open it.
Blocker references in *Waiting on* become clickable too — they name an item
harrow already has.

**Replace the bars with what the bar was standing in for.**

Acceptance criteria become the criteria. `3 of 3 ticked` says how many;
`✓ the corpus parses / ✓ the formats parse / ☐ a refresh is one command`
says which, in the space the bar was using, and `t` already ticks the one
you are looking at.

A milestone's rollup becomes a tally rather than a bar: `✓ 3 · ◐ 1 · ○ 8`
says where the remaining work *is*, which is the question somebody looking
at a rollup is asking. A ten-cell bar beside "3 of 12" is a less precise
statement of the number printed next to it.

## Cost

A markdown renderer is a thing that grows. The line to hold: this renders
what cairn writes and what people write in cairn bodies — it is not a
CommonMark implementation, and anything it does not recognise must come out
as its own source text rather than be swallowed. The current code fails that
test in one direction already, by deleting backticks it cannot style.

## Acceptance criteria

- [x] Bold, emphasis and inline code render as emphasis rather than being deleted
- [x] A link shows its text, not its URL, and opens when clicked
- [x] Heading levels are distinguishable from each other and from the pane's own sections
- [x] Fenced code blocks are recognised and keep their shape
- [x] Block quotes, horizontal rules, ordered lists and nesting render
- [x] Anything unrecognised survives as the text that was written
- [x] The acceptance bar is replaced by the criteria themselves
- [x] A rollup says where the remaining work is rather than drawing a bar
- [x] A blocker in *Waiting on* is clickable

## 2026-09-13

Markdown is read by `inline()` into pieces that carry what the markup claimed rather than a style, and `ink()` turns a claim into a colour. The theme decides; the parser does not. That is what lets headings be told apart by weight as well as colour, which is the only way three levels survive a monochrome terminal.

The rule for everything the renderer does not understand is that the source text survives. A half-written markdown renderer that swallows what it cannot parse is worse than one that renders nothing, because the reader cannot tell what is missing. `anything_unrecognised_survives_as_what_was_typed` holds it.

Wrapping is done on words rather than pieces. `in`+`line`+`code` written with backticks in the middle is three pieces and one word, and breaking between them would put half a word on the next line — so the pieces are cut into words first and a break only ever falls where a space was written. The same walk is what stopped markup leaving a phantom space behind it.

Links travel with the lines they are on, in a `Prose` — the same shape the stats pane's `Sheet` uses, for the same reason: a target drawn in one place and registered in another moves when the layout does. A blocker is a target too, and the whole line is clickable rather than the two-character id.

The bars are gone. Acceptance shows the criteria themselves, marked with the one `t` would offer next, and the body then elides exactly the lines the pane hoisted — including a heading left standing over a hole. A milestone gets `✓ 3 · ◐ 1 · ○ 8` instead of a percentage, because where the remaining work is was always the question a bar could not answer.
