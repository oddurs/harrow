# Security Policy

## Supported versions

harrow is pre-1.0. Only the latest release receives fixes.

| Version | Supported |
| ------- | --------- |
| 0.1.x   | ✅        |
| < 0.1   | ❌        |

## Reporting a vulnerability

**Please do not open a public issue.**

Report privately through GitHub Security Advisories:

<https://github.com/oddurs/harrow/security/advisories/new>

Include what you can: the version (`harrow --version`), your platform, what
harrow was pointed at, and the smallest reproduction you have. `harrow
--screenshot 120x40` renders a frame as text with no terminal involved, which
often makes a report reproducible on its own.

## What to expect

- **Acknowledgement within 3 days.** If you have not heard back by then, assume
  it went astray and open a public issue saying only that you are waiting on a
  private report — no details.
- **An assessment within 7 days**, with a severity and a plan.
- **A fix or a documented mitigation within 30 days** for anything exploitable.
- Credit in the release notes, unless you would rather not be named.

## Scope

harrow reads Markdown files from the repository you point it at and shells out
to `cairn` to change them. The interesting boundaries are:

- **Item files are untrusted input.** They may come from a repository you
  cloned. A malformed or hostile item file must never do more than produce a
  warning in the diagnostics overlay.
- **Every subprocess is a `cairn` invocation** built from a fixed argument
  vector, never a shell string. A report that shows an item's contents reaching
  a shell is a real finding.
- **The terminal must always be handed back**, including on panic and on a
  signal. A path that leaves a terminal in raw mode is a bug worth reporting,
  though not usually a security one.

Out of scope: anything requiring an attacker who can already write to your
`~/.config/harrow/`, run arbitrary commands as you, or edit the repository you
are working in.
