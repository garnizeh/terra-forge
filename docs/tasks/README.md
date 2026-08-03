# Task documents

A task document plans one objective (a feature, a build-order phase, a multi-file fix) down to the level of individually shippable pull requests. See [docs/process/dev-lifecycle.md](../process/dev-lifecycle.md) for how these documents are used.

## Layout

Work is grouped into **milestones** first, then **build-order phases**, then **tasks** — three levels, because CLAUDE.md's roadmap already has more phases ahead than the MVP (Phase 5 "Platform expansion", Phase 6 "AI factions/bots"), and a flat `docs/tasks/overview.md` would become ambiguous the moment a second one exists.

```text
docs/tasks/
  README.md                        — this file: convention + template
  01-mvp/                          — milestone folder: build-order Phases 1-4
    overview.md                    — MVP-wide roadmap: every phase in this milestone,
                                      every anticipated task, one-line objective each.
                                      The map, not the territory — no acceptance
                                      criteria here.
    phase-1-core-engine/
      01-<task-slug>.md             — one file per task, full detail (this file's template)
      02-<task-slug>.md
      ...
    phase-2-cli-prototype/
    phase-3-multiplayer-backbone/
    phase-4-frontend-rendering/
  02-<slug>/                       — next milestone (e.g. Phase 5), created when that
                                      work actually starts — not designed yet
```

- **Milestone folder:** `docs/tasks/<NN>-<slug>/`, numbered sequentially. `01-mvp` covers build-order Phases 1–4 (RFC-003's MVP scope). Later milestones get their own numbered folder — and their own `overview.md` — only once planning for that milestone actually begins; don't pre-create or pre-name them.
- **Phase folder:** inside a milestone, one subfolder per CLAUDE.md build-order phase, named `phase-<N>-<slug>` (slug matches the phase's name in that milestone's `overview.md`/CLAUDE.md).
- **Task file:** inside a phase folder, one file per task: `<NN>-<task-slug>.md`, numbered in the rough order they'll be executed. `NN` and the slug should match the task's entry in the milestone's `overview.md` where one exists.
- An objective that isn't tied to a build-order phase (e.g. a standalone multi-file bugfix) uses the same one-file-per-task pattern under `docs/tasks/<milestone>/<objective-slug>/`, instead of a phase folder.
- A milestone's `overview.md` is written first, for that whole milestone, and stays high-level — see its own header for what belongs there. Per-task files (this template) are written per phase, closer to when that phase's work actually starts, so each reflects what's true then rather than a guess made phases earlier. When a phase's per-task files are written, update that phase's rows in the milestone's `overview.md` (status column) rather than letting the two drift apart.
- A phase folder does not need to exist until its first task doc is written.

## Template

One file per task:

```markdown
# <N.M> — <Task title>

## Goal

What this task achieves and why, in 1-2 sentences. Link the RFC section(s) it implements.

## Acceptance criteria

- [ ] Criterion 1 (testable)
- [ ] Criterion 2 (testable)

## Out of scope

Anything adjacent this task deliberately excludes (and, if relevant, which later task or phase covers it).

## Depends on

Other tasks by number, or "none".

## Status

Not started | Planned | In progress | In review | Merged
**PR:** (filled in once opened)
```

## Rules

- Each task file maps to exactly **one pull request**. If a task doesn't fit in one reviewable PR, split it into more task files.
- Acceptance criteria must be concrete and checkable, not aspirational.
- A task file needs approval before it is executed (see `start-task`).
- Update `Status` and `PR` as the task moves through review criteria → PR opened → PR merged; reflect the same in the milestone's `overview.md` status column.
