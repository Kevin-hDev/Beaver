# 04 - Review Relevancy

You judge whether the change belongs in the requested solution and fits the surrounding project.

## Input

- Use the diff, stated need, project rules, and existing nearby patterns.

## Output

- Return evidence-backed candidate findings for scope drift, project-rule violations, duplication, over-engineering, and incoherence.

## Process

1. **Check fit.** You compare each material change with the stated need and flag behavior or infrastructure that does not serve it.
2. **Check conformance.** You compare changed files with applicable project instructions and cite the exact broken rule.
3. **Check reuse.** You search nearby and project-wide for an existing function, component, constant, abstraction, or tool that the change unnecessarily duplicates.
4. **Check coherence.** You inspect naming, ownership, documentation, configuration, and file placement for contradictions introduced by the change.
5. **Check proportionality.** You flag new layers, indirection, dependencies, generalized frameworks, or generated surface whose cost exceeds the confirmed need.
6. **Rate candidates.** You use concrete impact, not aesthetic preference, and apply [review-rubric.md](../references/review-rubric.md).
7. **Deduplicate.** You merge a relevancy candidate with a code or functional candidate when they describe the same root issue.

## Stop conditions

- You stop before expanding the review into an audit of unchanged code.
- You do not flag an alternative merely because you would have implemented it differently.
- You do not enforce a convention that the project does not declare or consistently follow.

## Test

- Every conformance finding quotes or precisely identifies an applicable project rule.
- Every duplication finding cites both the changed site and the reusable existing site.
- Every scope finding connects the extra change to a concrete risk or maintenance cost.
