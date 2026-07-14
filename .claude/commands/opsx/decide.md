---
name: "OPSX: Decide"
description: Build-vs-adopt gate — for each critical concern, decide Rent/Adopt/Extend/Fork/Build and record it in design.md
category: Workflow
tags: [workflow, decision, adopt, experimental]
---

Resolve the **build-vs-adopt** question for a change before implementing it.

For each critical concern, walk the hierarchy `Rent > Adopt > Extend > Fork > Build` (see `openspec/guidelines.md`), research current options, recommend one, and record the decision as an ADR-style block in the change's `design.md`. Choosing to **build** something a mature tool already does well is the failure this gate prevents.

**Input**: Optionally a change name (e.g. `/opsx:decide add-auth`). If omitted, infer from context; if only one active change exists, use it; otherwise run `openspec list --json` and ask via **AskUserQuestion**. Announce: "Using change: <name>".

**Steps**

1. **Resolve paths** — `openspec status --change "<name>" --json`. Use `artifactPaths.design.resolvedOutputPath` for the design file and read `proposal.md` + design for context. If `actionContext.mode` is `workspace-planning`, STOP (not supported here).

2. **List the critical concerns** — pull the concerns flagged in the proposal's Capabilities (correctness-, security-, or reliability-sensitive). If none are flagged, scan proposal + specs and propose a short list; confirm with the user.

3. **For each concern, run the gate:**
   - **Infrastructure?** → decision is `Rent`, no further evaluation.
   - Otherwise **research with WebSearch** — find the current, actively-maintained options. Do NOT rely on memory; tooling moves.
   - Score candidates against the maturity rubric in `openspec/guidelines.md`. Apply hard rejects (security risk / incompatible license / abandoned).
   - Present **at most 3 options** as **pure-build vs adopt-a-tool**, each a one-line trade-off, and **always end with a clear recommendation** (which, and why) plus its hierarchy tier (Adopt/Extend/Fork/Build).
   - **Wait for the user's pick** before treating any tool as settled. Status of each decision is `draft` until the user confirms, then `approved`. (No "rejected" state — an unpicked option is simply dropped.)

4. **Record into `design.md`** — append/update a `## Decisions` section, one block per concern:

   ```markdown
   ### Decision: <concern> — <Adopt|Extend|Fork|Build> <tool-or-"hand-written">

   - **Status**: approved
   - **Why**: <one line>
   - **Considered**: <other 1–2 options, one line each>
   - **Isolation**: <the adapter/boundary the choice lives behind>
   ```

   Keep concrete tool names in `design.md` only — never push them into `config.yaml` or `specs/` (those stay abstract).

5. **Summary** — list each concern → decision → status. Note any left in `draft` (awaiting the user).

**Guardrails**

- Research before recommending — current options, not remembered ones.
- ≤3 options per concern, always with a recommendation.
- Default toward Adopt/Extend; a `Build` decision needs an explicit one-line justification.
- Decisions are draft → approved only.
- Tool names live in `design.md`; specs and config stay language-agnostic.
- This gate decides HOW, not WHAT — don't change scope or behavior here; if a decision reveals a scope problem, suggest updating the proposal/specs instead.
