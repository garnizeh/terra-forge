# Development lifecycle

This document defines how work moves from an idea to merged code in Terra-Forge. It governs *process*, not architecture or scope — those are settled in the RFCs (see [CLAUDE.md](../../CLAUDE.md)). If a step here conflicts with an RFC, the RFC wins on *what* to build; this doc only governs *how* the work is organized and shipped.

## 1. Task planning

Before any code is written for an objective (a feature, a phase of the build order, a bug fix that touches multiple files), it must be broken down into a **task document**:

- One task file per task, grouped by milestone and either build-order phase (`docs/tasks/<milestone>/phase-<N>-<slug>/`) or standalone objective (`docs/tasks/<milestone>/<objective-slug>/`) — see [docs/tasks/README.md](../tasks/README.md) for the full layout, naming convention, and template. Each milestone has a high-level `overview.md` covering all its phases; the current milestone is `docs/tasks/01-mvp/`.
- The objective is organized into **phases** when it's large enough to need them (small objectives can skip straight to a flat task list).
- Each **task is scoped to exactly one pull request**. If a task can't reasonably ship as a single reviewable PR, split it into more tasks rather than letting one task grow.
- Every task has explicit, testable **acceptance criteria** — the bar for "done," not a vague description.
- Anything not covered by the RFCs or the task doc is out of scope by default. Do not add it silently.

The task document is a planning artifact and needs the user's approval before execution starts on any task inside it.

## 2. Executing a task

Each task from an approved task doc is executed independently, in this order:

1. **Create a branch** for the task (one branch per task/PR).
2. **Present the implementation plan for review** before writing code — grounded in the task's acceptance criteria and the relevant RFC sections. Wait for approval before proceeding.
3. **Develop** against the approved plan, writing tests as the work proceeds (see §4) rather than after the fact.
4. **Check acceptance criteria** explicitly, one by one, once implementation is complete.
5. **Request approval before opening the pull request.** Do not push or open a PR unprompted, even if CI would pass and criteria are met.
6. Once approved, open the PR. The PR description references the task doc entry (and any ADR from §3) and includes the acceptance-criteria checklist.

Every task has its own PR — do not bundle multiple tasks into one, and do not open a PR for partial task completion without flagging it as a draft/WIP and asking first.

## 3. Architecture changes → ADR

Any change to an architectural decision — a new one, or a revision of something already decided in RFC-002 or a prior ADR — must be captured as an **Architecture Decision Record** in `docs/adr/` (see [docs/adr/README.md](../adr/README.md)) before or alongside the implementation that depends on it.

- If the change contradicts an existing RFC, the relevant RFC must also be amended (per the existing rule in [CLAUDE.md](../../CLAUDE.md)) — the ADR records the decision and its rationale; the RFC stays the canonical spec.
- An ADR is proposed for review the same way a task plan is: present it, wait for approval, then mark it Accepted.

## 4. Testing

Special care applies here — test everything that can reasonably be tested, per the expectations already defined in [CLAUDE.md](../../CLAUDE.md) (`cargo test`, `proptest` for the attrition matrix and map generation, `wasm-pack test --headless`, integration tests against real ephemeral Postgres, Vitest/RTL, Playwright for the React↔Wasm↔Canvas seam, and the cross-cutting determinism check). A task's acceptance criteria should make the required test coverage explicit, not leave it implied.

## 5. Never invent — when in doubt, present options

Nothing gets built, named, or architected beyond what's explicitly planned in an RFC, an approved task doc, or an approved ADR. If a gap or ambiguity shows up during planning or implementation:

- **Stop.**
- Present the available options with their tradeoffs.
- Wait for an explicit decision before proceeding.

This applies to scope, architecture/library choices, naming, and anything else not already written down and approved.
