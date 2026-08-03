# 02 - Write

You render the confirmed one-shot operation as a compact slash-invokable skill bundle.

## Input

- Use the confirmed command contract, destination state, neighboring skill metadata, and existing bundle when refactoring.

## Output

- Return the written bundle tree, a status for every intended file, and the preserved or changed behavior.

## Process

1. **Revalidate the target.** You confirm the destination and current contents still match the approved boundary and reject traversal, symbolic escapes, or changed ownership.
2. **Draft the bundle.** You read [command-authoring.md](../references/command-authoring.md), copy [command-skill-template.md](../assets/command-skill-template.md), remove placeholders, and write one compact `SKILL.md` with the confirmed objective and steps.
3. **Use inline input.** You instruct the generated command to read the remaining user request directly, validate it, and ask only for missing input that changes execution.
4. **Add only necessary files.** You include bounded evaluations and optional UI metadata when the target convention consumes them. You add no action, reference, asset, or script without a concrete execution need.
5. **Preserve existing work.** You update the smallest owning sections, retain unrelated content, and do not move or broaden an existing bundle without confirmation.
6. **Apply safe writes.** You verify complete prepared contents before replacement and use the project's safe writing mechanism when available.
7. **Account for results.** You list every intended file as `created`, `updated`, `unchanged`, `blocked`, `failed`, or `skipped` and hand the bundle to validation.

## Stop conditions

- You stop before writing when the target changed, a prepared path escapes the confirmed root, a placeholder remains, or the operation no longer fits a single objective.
- You stop for renewed confirmation before any new side effect, semantic expansion, move, overwrite expansion, or unsupported metadata.
- You do not create a standalone command file, built-in app command, shell alias, or unverified tool integration.

## Test

- You verify that the folder name, frontmatter name, slash slug, and evaluation skill identifier agree.
- You verify that the body has one objective, at most eight direct steps, no `$ARGUMENTS`, and explicit missing-input behavior.
- You verify that every intended file exists as reported and unrelated existing content remains intact.
