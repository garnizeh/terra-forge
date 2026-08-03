---
name: plan-tasks
description: Break a build-order phase (or a standalone multi-file objective) into per-task Terra-Forge task documents — one file per task, each scoped to exactly one pull request, with testable acceptance criteria. Use when the user wants to plan, scope, or detail tasks for upcoming work, e.g. "planeja o detalhamento da fase 1", "documenta as tasks do combat engine", "quebra isso em tasks".
---

# Plan tasks

Produces one task file per task under `docs/tasks/<milestone>/phase-<N>-<slug>/` (or `docs/tasks/<milestone>/<objective-slug>/` for an objective not tied to a build-order phase), following [docs/tasks/README.md](../../../docs/tasks/README.md) and [docs/process/dev-lifecycle.md](../../../docs/process/dev-lifecycle.md).

## Steps

1. **Clarify the objective.** If the user's request doesn't pin down a clear, single objective (or spans more than one build-order phase), ask before drafting anything.
2. **Find the milestone.** Milestone folders live at `docs/tasks/<NN>-<slug>/` (currently only `01-mvp`, covering build-order Phases 1–4). If the objective's phase doesn't belong to any existing milestone folder yet, stop and ask before creating a new one — don't invent its number/name inline.
3. **Check that milestone's `overview.md` first.** If the objective is a build-order phase, it should already have a provisional task list there — treat that as the starting point to refine (split/merge/reorder), not something to re-derive from scratch or contradict silently. If it isn't there yet, or the objective isn't phase-shaped, that's fine — proceed from the RFCs directly.
4. **Ground the plan.** Read the relevant RFC section(s) (RFC-001/002/003) and CLAUDE.md sections that bound this objective — domain model, MVP scope, build order. Do not plan anything RFC-003 marks as deferred/out-of-scope, and do not plan ahead into a later build-order phase.
5. **Draft one file per task**, using the template in `docs/tasks/README.md`:
   - Each task must fit in one reviewable pull request — if it doesn't, split it further.
   - Every task needs concrete, testable acceptance criteria (tie to the testing expectations in CLAUDE.md — unit tests, proptest, determinism checks, etc. — where relevant).
   - Note task dependencies explicitly, by task number.
   - Number and slug should match the corresponding row in that milestone's `overview.md` where one exists.
6. **Never invent scope.** If something needed to complete the objective isn't specified by the RFCs or the user, stop and present the available options with tradeoffs instead of picking one. This includes architectural choices — flag anything that looks like it needs an ADR (see the `adr` skill) rather than deciding it inline.
7. **Present the drafts before writing any file.** Show the full content of every task file in the conversation for review.
8. **On approval**, write each file to `docs/tasks/<milestone>/phase-<N>-<slug>/<NN>-<task-slug>.md` (or the objective-slug equivalent), and update the corresponding rows in that milestone's `overview.md` (status column) so the two stay in sync. Do not mark any task's Status past "Planned" — creating the doc is not the same as approval to execute it (that's the `start-task` skill).
