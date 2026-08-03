# Architecture Boundaries

## Evidence order

You determine intended boundaries from the strongest available project evidence in this order:

1. You follow explicit project instructions and enforced dependency rules.
2. You follow accepted architecture decisions, current diagrams, module manifests, and package ownership.
3. You follow executable architecture tests, lint rules, build targets, and import restrictions.
4. You infer cautiously from stable dependency direction and established neighboring modules only when explicit evidence is absent.

You report ambiguity instead of imposing a preferred architecture.

## Atomic-change test

A structural change is safe to apply now only when:

- You can finish it without leaving a broken intermediate public contract.
- You can verify it independently with existing tests, type checks, and dependency evidence.
- You can preserve observable behavior and keep the diff within one natural boundary seam.
- You do not require a coordinated data migration, broad public API change, deployment sequence, or unresolved ownership decision.

You place any move that fails one of these checks in the deferred `needs a plan` list.

## Supported moves

- You separate presentation, application or domain logic, and infrastructure according to documented project conventions.
- You introduce an interface, adapter, event, or dependency-inversion point only when it corrects proven dependency direction.
- You split a god module by cohesive responsibilities and stable call seams.
- You move code across a boundary with its exports, imports, registration, tests, and configuration references updated together.

## Graph verification

- You compare the same relevant dependency view before and after each step.
- You confirm the claimed violating edge is gone and no new cycle or wrong-direction edge appears.
- You use configured architecture checks when available and a bounded import inspection otherwise.
- You do not claim a clean graph from passing functional tests alone.
