# Probing Guide

## Match the user's level

Classify the idea as functional, technical, or mixed. Ask at that level. Do not descend into code, file layout, libraries, or commands when the user is defining behavior. When the idea itself is a technical choice, examine that choice without turning the conversation into an implementation plan.

## Follow the live thread

Use the latest answer to find the next question. Prefer a fork where two plausible answers would produce materially different intended outcomes. Name the fork clearly before you ask the user to choose or refine it.

## Use one fitting tactic

- **Five whys:** Use it when the stated goal may be a proposed solution hiding a deeper need.
- **Job to be done:** Reframe the need as “When ..., I want ..., so I can ...” when the actor and outcome are unclear.
- **Concrete examples:** Ask for one matching and one non-matching example when a term has multiple meanings.
- **Premortem:** Ask what caused the idea to fail after launch when important failure modes remain hidden.
- **Boundary walk:** Examine empty, zero, exact-limit, one-over-limit, duplicate, repeated, out-of-order, concurrent, delayed, partial, unavailable, malformed, hostile, offline, or missing-input cases only when they can change the intended behavior.

## Know when to stop

Conclude when a competent reader would understand the same outcome, actors, boundaries, constraints, and success conditions. Leave consciously deferred implementation choices for specification or planning. Stop immediately when the user is satisfied or asks to end the exploration.
