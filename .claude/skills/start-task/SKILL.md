---
name: start-task
description: Execute one task from an approved Terra-Forge task document end to end — branch, plan review, implementation with tests, acceptance-criteria check, then approval before opening the PR. Use when the user wants to start/work on/implement a specific task, e.g. "vamos executar a task A.2", "implementa a task de deploy validation", "bora começar essa task".
---

# Start task

Executes exactly one task from a `docs/tasks/<milestone>/phase-<N>-<slug>/<NN>-<task-slug>.md` file (phase-scoped task) or `docs/tasks/<milestone>/<objective-slug>/<NN>-<task-slug>.md` (standalone objective), per [docs/process/dev-lifecycle.md](../../../docs/process/dev-lifecycle.md). One task = one branch = one pull request — never bundle multiple tasks into a single run of this skill.

## Steps

1. **Identify the task.** Find its file under the relevant milestone/phase folder in `docs/tasks/` (check that milestone's `overview.md` if unsure which phase/number — currently `docs/tasks/01-mvp/overview.md`). If the user didn't name a specific task, or it's ambiguous which one they mean, ask. If the task file itself doesn't exist/isn't approved yet, stop — use the `plan-tasks` skill first.
2. **Check repo state.** Run `git status`; if there are uncommitted changes unrelated to this task, flag it and ask how to proceed before creating a branch.
3. **Create the branch** for this task alone (e.g. `<phase-slug>/<task-id>-<short-desc>`).
4. **Draft the implementation plan** grounded in the task's acceptance criteria and the relevant RFC section(s) — use plan mode (`EnterPlanMode`) so the plan is presented for review before any code is written. Wait for explicit approval.
5. **If an architectural decision emerges during planning or implementation** — something not already settled by an RFC or prior ADR — stop and use the `adr` skill to record it (Proposed), get it approved, then continue. Don't decide architecture inline and silently.
6. **Implement** against the approved plan. Write tests as you go, per the testing expectations in CLAUDE.md (unit tests, proptest where relevant, determinism checks, etc.) — not as an afterthought.
7. **Never invent.** If something the task needs isn't covered by the task doc, the RFCs, or the approved plan, stop and present the options with tradeoffs; wait for a decision rather than guessing.
8. **Verify acceptance criteria** explicitly, one by one, and report which pass.
9. **Request approval before opening a PR.** Do not push the branch or run `gh pr create` until the user explicitly approves — meeting the acceptance criteria is not itself that approval.
10. **On approval**, push and open the PR. Reference the task file (and any ADR from step 5) in the PR description, and include the acceptance-criteria checklist. Then update the task file's `Status`/`PR:` fields and the corresponding row in that milestone's `overview.md`.
