# 03 - Write

You create or update the smallest documentation set that satisfies the confirmed contract.

## Input

- Use the confirmed contract, complete evidence ledger, existing documentation, and file-level update map.
- Use the project's current format, navigation, terminology, and generated-content boundaries.

## Output

- Return complete documentation drafts with accurate links, examples, commands, navigation, and source-backed claims.

## Process

1. **Preserve structure.** You retain useful existing explanations, authored notes, anchors, frontmatter, and public URLs unless the contract requires a specific change.
2. **Write for the audience.** You lead with the reader's goal, state prerequisites before actions, define unfamiliar terms once, and keep each section responsible for one subject.
3. **Use evidence precisely.** You describe only verified behavior, supported defaults, actual errors, and current interfaces. You label optional, platform-specific, experimental, deprecated, or planned behavior explicitly.
4. **Make procedures reproducible.** You order steps, name the working context, use safe placeholders, show expected observable results, and include rollback or cleanup when the documented task changes state.
5. **Make references navigable.** You use stable headings, descriptive link text, valid relative paths, and existing navigation or index conventions. You avoid duplicating canonical content and link to its owner instead.
6. **Protect generated content.** You update only a documentation template, schema description, or other documentation-owned source when the contract explicitly covers it. You never hand-edit a generated block. You report a tooling mismatch instead of changing executable generator code within this skill.
7. **Render complete batches.** You prepare at most 20 files or 100 sections per numbered batch, continue until every update-map entry is rendered, and keep cross-file terminology consistent.
8. **Write safely.** You validate canonical destinations, render sibling temporary files where supported, recheck current originals, and replace only the approved documentation files. You preserve unrelated text and never stage changes.

## Stop conditions

- You stop before writing when a destination changed concurrently, escapes the project, or would overwrite unresolved user content.
- You stop when a material claim, command, example, link target, or navigation decision remains unsupported.
- You do not modify application code, tests, schemas, dependencies, deployment state, or version-control history.

## Test

- Confirm that every contracted audience goal and subject appears in the draft and no placeholder remains.
- Confirm that every material claim still maps to evidence and every cross-file term has one consistent meaning.
- Confirm that only documentation files and explicitly included navigation sources changed.
