# Destination Contract

Resolve every destination from current project-local files and instructions. Prefer the smallest destination that owns the learning.

| Destination | Use when | Delivery |
| --- | --- | --- |
| Memory | Preserve a durable project fact, convention, or gotcha | Write directly after approval using the existing memory-bank convention |
| ADR | Record a deliberate choice with context, alternatives, and consequences | Write directly after approval using the existing ADR convention |
| Rule | Enforce project behavior repeatedly | Hand the approved packet to an available specialized rule generator |
| Skill | Preserve a reusable multi-step workflow | Hand the approved packet to an available specialized skill generator |

## Resolution rules

- Inspect project instructions, indexes, neighboring files, naming, frontmatter, and templates before proposing a path.
- Ask when a memory bank, ADR location, rule location, skill location, taxonomy, or format is missing or ambiguous. For a missing memory bank, offer an explicitly approved handoff to an available project-memory capability, then reassess after verified setup.
- Never invent a standard directory, write personal or global memory, or silently scaffold a missing bank.
- Update an existing entry when it is the smallest clear owner. Create a new memory or ADR file only when the existing convention defines how.
- Treat `covered` as a no-op and cite the covering entry. Reassess any requested change as `updates` before approval.
- Preserve user edits and unrelated content.

## Supersession

- Require explicit confirmation that the newer decision supersedes the older one.
- Add `Supersedes: <old ADR>` to the new ADR using the project's link style.
- Add `Superseded by: <new ADR>` to the old ADR using the same link style.
- Render and validate both records before replacing either one.

## Handoffs

- Confirm that a project-memory capability is callable before offering setup of a missing memory bank. Pass only the approved setup request, then return the lesson to assessment after verified setup.
- Confirm that a generator specialized for the requested rule or skill destination is actually callable.
- Pass the complete approved learning packet and requested project-local destination convention.
- When no generator is available, return the packet with status `unavailable`; do not write directly or imply application.
