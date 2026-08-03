# 03 - Write

You render the confirmed plan into a minimal, complete skill bundle without disturbing unrelated work.

## Input

- Use the confirmed frame, action and resource plan, current destination state, and existing bundle when refactoring.

## Output

- Return the final bundle tree, an exact status for every intended file, and a record of preserved or intentionally changed behavior.

## Process

1. **Revalidate the target.** You read [destination-resolution.md](../references/destination-resolution.md), confirm the destination still matches the plan, and reject traversal, symbolic escapes, or changed ownership.
2. **Write the router.** You copy [skill-template.md](../assets/skill-template.md), remove every placeholder, and keep only selection-independent flow, the action table, transversal rules, and direct resource links.
3. **Write actions.** You copy [action-template.md](../assets/action-template.md) for each confirmed job and provide `Input`, `Output`, `Process`, `Stop conditions`, and `Test` sections in direct second-person imperatives.
4. **Write resources.** You create only the planned references, assets, and deterministic scripts. You keep one fact in one home and link to it instead of repeating it.
5. **Write evaluations.** You copy [eval-template.json](../assets/eval-template.json), create bounded valid JSON cases from the confirmed examples, and cover trigger discrimination plus behavioral success.
6. **Preserve existing work.** You modify the smallest owning sections, retain unrelated content, and record every removed or changed capability explicitly.
7. **Apply safe writes.** You verify complete prepared contents before replacement and use the project's safe file-writing mechanism when one exists.
8. **Account for files.** You list every intended file as `created`, `updated`, `unchanged`, `blocked`, `failed`, or `skipped` before validation.

## Stop conditions

- You stop before writing when the target changed, a path escapes the confirmed root, content ownership is unclear, or the prepared bundle contains placeholders.
- You stop and request renewed confirmation for any new file, semantic reduction, destination change, overwrite expansion, or unplanned dependency.
- You do not report an intended file as written when its final bytes were not verified on disk.

## Test

- You verify that the bundle name, folder, frontmatter, links, action table, and evaluation skill identifier agree.
- You verify that every action has all five required sections and every local link resolves inside the bundle.
- You verify that unrelated existing content and all capabilities outside the confirmed change boundary remain intact.
