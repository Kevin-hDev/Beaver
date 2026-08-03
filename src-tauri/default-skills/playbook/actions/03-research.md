# 03 - Research a playbook

You find current alternatives, missing coverage, operational pitfalls, and counter-intuitive improvements without writing project state.

## Input

- Accept a topic or an existing playbook selected by the latest list number, slug, title, or unambiguous topic.
- Accept the desired outcome, audience and level, scope, exclusions, grouping, and evidence freshness needs.

## Output

- Return a value-sorted alternatives table `| Alternative | What it is | Pros | Cons | Latest state | Official link |`.
- Return sourced coverage gaps, counter-intuitive wins, deprecations, unknowns, and one explicit recommendation.
- Return `complete`, `partial`, or `blocked` research status without writing files.

## Process

1. **Resolve the target.** You read [locations.md](../references/locations.md), resolve and read an existing playbook when named, or run the list action when a current number or identity is unavailable.
2. **Refine the goal.** You fill [research-goal-checklist.md](../assets/research-goal-checklist.md) with the user when any answer would materially change the search. You do not research a vague target.
3. **Scout every angle.** You read [research-method.md](../references/research-method.md) and investigate alternatives, new methods, coverage gaps, counter-intuitive wins, deprecations, practical gotchas, and migration concerns. You delegate one independent scout per angle when agent delegation is available; otherwise you cover the angles sequentially in resumable batches without reducing their scope.
4. **Curate.** You deduplicate candidates, drop items that neither improve nor extend the target, and sort surviving items by value for the stated goal.
5. **Verify survivors.** You use one independent verification pass per surviving candidate when delegation is available. You confirm each item exists through an official or primary source, capture its canonical link and current version or dated state, and corroborate material claims. You keep community sources only as clearly labeled adoption or operational signals.
6. **Verify examples.** You use a real official example or safely execute a representative command and capture its observable output. You label interactive evidence that requires the user and never fabricate it.
7. **Clear the gate.** You complete [research-completion-checklist.md](../assets/research-completion-checklist.md). You drop unverifiable candidates and mark unresolved required evidence as `partial` or `blocked`.
8. **Present and hand off.** You present all required buckets, give every item its canonical official or primary link, state the recommendation and tradeoffs, and offer to upsert only the insights the user selects.

## Stop conditions

- You stop and ask when the goal, audience, scope, or target identity is materially ambiguous.
- You stop with `blocked` when authoritative sources required for the central recommendation are unavailable or contradictory.
- You do not write or update any playbook, install a candidate, or apply a recommendation during research.

## Test

- Confirm that alternatives include pros, cons, latest state, and official links.
- Confirm that coverage gaps, counter-intuitive wins, deprecations, unknowns, and the recommendation are evidence-backed.
- Confirm that every presented candidate exists and every unverifiable candidate was dropped.
- Confirm that the research completion checklist clears before an upsert handoff and no file changed.
