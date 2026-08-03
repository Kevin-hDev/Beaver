# 01 - Capture

You settle the rule's evidence, topic, placement, scope, and complete destination set before writing.

## Input

- Accept an explicit rule topic, an existing rule to refactor, or a request to discover rule candidates from project evidence.

## Output

- Return a confirmed plan containing evidence, topic, category or local grouping, slug or local filename, file scope, one-line purpose, canonical meaning, intended targets, and requested operation for each target.

## Process

1. **Validate the project boundary.** You resolve the project root from the working task, reject paths that escape it, and keep repository-fed collections bounded through continuable batches.
2. **Resolve local conventions.** You read [destination-resolution.md](../references/destination-resolution.md), inspect existing project rule and instruction surfaces, and record the evidence for naming, grouping, frontmatter, scope, precedence, canonical ownership, and mirroring.
3. **Choose capture mode.** You use the stated topic when it is specific. When discovery is requested, you scan representative configuration, documentation, rules, tests, and repeated code patterns; propose evidence-backed candidates; and wait for the user to choose. When the request is empty or vague, you ask for the topic.
4. **Prevent duplicates and conflicts.** You search every discovered active surface for equivalent, overlapping, weaker, or contradictory requirements. You propose updating the narrowest existing owner when appropriate instead of creating a duplicate.
5. **Place the rule.** You follow the project's existing category and filename convention. When none exists, you propose a plain descriptive slug plus the optional fallback taxonomy from [rule-authoring.md](../references/rule-authoring.md), and ask the user to choose rather than imposing it.
6. **Define scope.** You state exact file globs or declare all-project scope, explain why the reach is necessary, and map that scope to each destination's existing metadata convention.
7. **Resolve targets.** You list every active destination supported by project evidence, distinguish a canonical owner from required mirrors, and identify unsupported or ambiguous surfaces without silently dropping them.
8. **Confirm the write set.** You show the topic, evidence, grouping, filename, scope, one-line purpose, proposed canonical meaning, every create/update/split/skip operation, and any overwrite or restructuring boundary. You wait for explicit written confirmation.

## Stop conditions

- You stop before drafting or writing when the topic, project boundary, scope, destination convention, canonical owner, required mirrors, or overwrite boundary is ambiguous.
- You stop when evidence reveals contradictory active rules that require a policy choice.
- You do not treat silence, a vague request, or the existence of one familiar directory as target confirmation.

## Test

- You verify that the plan names its evidence, topic, placement, exact scope, canonical meaning, and complete intended target set.
- You verify that every create, update, split, skip, overwrite, and mirror operation is explicitly confirmed.
- You verify that absent project conventions produce a user choice rather than an invented path or format.
