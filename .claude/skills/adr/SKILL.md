---
name: adr
description: Create a Terra-Forge Architecture Decision Record for a new or changed architecture decision, using the lightweight Nygard template. Use whenever an architectural choice is made or revised outside what RFC-002 or a prior ADR already settled — e.g. "cria um adr pra essa decisão", "documenta essa mudança de arquitetura", or when the start-task/plan-tasks skills flag that a decision needs one.
---

# ADR

Writes one Architecture Decision Record to `docs/adr/<NNNN>-<slug>.md`, per [docs/adr/README.md](../../../docs/adr/README.md) and [docs/process/dev-lifecycle.md](../../../docs/process/dev-lifecycle.md).

## Steps

1. **Check it's actually needed.** An ADR is for a decision — new, or revising one already made in RFC-002 or a prior ADR. If this is just an implementation detail with no architectural weight, say so and skip it rather than creating noise.
2. **Determine the number.** List `docs/adr/` and take the highest existing `NNNN` + 1 (skip `README.md`). Numbers are never reused, even for superseded/rejected ADRs.
3. **Gather context and options honestly.** Base the Context, the options considered, and the Decision only on what was actually discussed with the user or is explicit in the RFCs — do not invent alternatives, rationale, or consequences that weren't part of the actual discussion. If the tradeoffs aren't clear yet, present the real options with their tradeoffs and ask before writing the Decision section.
4. **Check for RFC drift.** If this decision changes something an existing RFC states, flag that the RFC needs a corresponding amendment and ask whether to do that now or as a follow-up — don't let the ADR and the RFC disagree silently.
5. **Draft the ADR** using the template in `docs/adr/README.md` (Status / Context / Decision / Consequences — including the negative/accepted tradeoffs, not just the upside) with `Status: Proposed`.
6. **Write `docs/adr/<NNNN>-<slug>.md`** with `Status: Proposed` so it's reviewable in the actual file, then **present it for review** — the same approval gate as a task plan.
7. **On approval**, update the ADR's `Status` line to `Accepted` (a second, tiny commit, or an amendment to the ADR creation commit before merge).
8. **If this supersedes an earlier ADR**, update that ADR's `Status` line to point at the new one (`Superseded by ADR-NNNN`) — never rewrite an Accepted ADR's Decision after the fact.
