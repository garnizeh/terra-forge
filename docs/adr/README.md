# Architecture Decision Records

An ADR captures one architectural decision — a new one, or a change to something already decided in RFC-002 or a prior ADR — along with the context and tradeoffs that led to it. See [docs/process/dev-lifecycle.md](../process/dev-lifecycle.md) for when an ADR is required.

ADRs record *why* a decision was made. The RFCs remain the canonical spec of *what* the system is — if a decision here changes something an RFC states, amend that RFC too; don't let it drift silently.

## File naming

`docs/adr/<NNNN>-<slug>.md`, sequential four-digit number, e.g. `0001-event-sourcing-for-match-state.md`. Numbers are never reused, even for rejected/superseded ADRs.

## Template (Michael Nygard / lightweight format)

```markdown
# ADR-NNNN: <Title>

## Status

Proposed | Accepted | Superseded by ADR-NNNN | Deprecated

## Context

What forces are at play — technical, project, constraints — that make this decision necessary. State the problem, not the solution.

## Decision

What was decided, stated plainly ("We will ..."). Note options that were considered and rejected, briefly, if it helps future readers avoid re-litigating them.

## Consequences

What becomes easier or harder as a result. Include negative/accepted tradeoffs, not just benefits.
```

## Rules

- An ADR starts as `Proposed` and is presented for approval before flipping to `Accepted` — same review step as a task plan.
- Don't invent alternatives or rationale that weren't actually discussed; if the tradeoffs aren't clear, ask before writing the Decision section.
- Superseding a decision means a *new* ADR with its own number, with the old one's Status updated to point at it — never edit an Accepted ADR's Decision after the fact.
