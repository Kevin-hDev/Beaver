# 02 - Write

You build one canonical rule and render it safely into every confirmed project-local target.

## Input

- Use the confirmed capture plan, current destination files, resolved local conventions, and canonical rule meaning.

## Output

- Return the canonical rule, each intended target with `created`, `updated`, `split`, `skipped`, `blocked`, or `failed` status, and an exact account of preserved content and write results.

## Process

1. **Revalidate the plan.** You re-read every intended target and relevant convention file. You stop if the current state no longer matches the confirmed plan.
2. **Draft canonically.** You read [rule-authoring.md](../references/rule-authoring.md), copy [rule-template.md](../assets/rule-template.md), remove placeholders, and express only the confirmed meaning.
3. **Keep the rule focused.** You keep each requirement in a concise bullet, target a 3-7-word actionable core after `You` when clarity permits, and split distinct topics into separately named rules only when the confirmed plan permits the split. You do not bury unrelated policies in headings.
4. **Render each target.** You preserve the canonical behavioral meaning while adapting only the confirmed target's extension, frontmatter, scope syntax, headings, links, or wrapper format.
5. **Preserve existing content.** You modify the smallest owning section, retain unrelated text byte-for-byte where practical, and never replace or substantially restructure content beyond the confirmed boundary.
6. **Check semantics before effects.** You read [consistency-validation.md](../references/consistency-validation.md) and compare every prepared rendering against the canonical rule, including obligations, prohibitions, scope, exceptions, examples, and strength.
7. **Prepare safe writes.** You reject traversal, links that escape the project, non-regular targets, and unresolved symbolic destinations. You prepare sibling temporary files and verify their complete content before replacement.
8. **Apply the complete write set.** You replace validated targets atomically. When all intended targets cannot be made consistent, you stop before unapproved semantic divergence and report the exact partial or blocked state; you never describe an untouched target as written.
9. **Record results.** You account for every intended target and preserve the canonical draft for validation even when one or more targets fail.

## Stop conditions

- You stop before writing when confirmation is incomplete, a destination changed, a path escapes the project, the prepared content conflicts with an active rule, or a required target cannot represent the confirmed scope or meaning.
- You stop and request renewed confirmation when a semantic change, new split, new target, overwrite expansion, or exception becomes necessary.
- You do not weaken a requirement to fit one target, silently omit a required mirror, or claim all-target success after a partial write.

## Test

- You verify that every written rule is focused, enforceable, imperative, scoped, and free of placeholders.
- You verify that every intended target has an exact status and every written rendering preserves canonical meaning.
- You verify that unrelated user content remains intact and every replacement stays inside the project.
