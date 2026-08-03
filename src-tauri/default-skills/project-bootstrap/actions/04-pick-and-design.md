# 04 - Pick and Design

Obtain an explicit viable selection, resolve warnings, and design a conceptual project structure.

## Input

- Use the audited candidate table, approved needs checklist, and complete review rationales.
- Accept the user's exact candidate selection and any proposed warning mitigations.

## Output

- Return the completed six derived choices, accepted mitigations, a complete conceptual folder tree, a validated Mermaid module diagram, and explicit design confirmation.

## Process

1. **Present choices.** Show the audited table and rationales without preselecting or hiding a candidate. Ask the user to choose one exact candidate name.
2. **Refuse broken choices.** When the user chooses a broken candidate, explain the blocking evidence and refuse to design it. Ask for another audited candidate or return a changed proposal to candidate generation and audit.
3. **Mitigate warnings.** For a warning choice, restate every concern and request a concrete mitigation with owner, action, trigger, and verification. Check each mitigation against the audit evidence. Loop until every warning is credibly mitigated, the user chooses another candidate, or the option becomes broken.
4. **Reaudit hybrids.** Treat any merged or altered candidate as new. Return it through evidence collection and independent audit before selection.
5. **Fill derived choices.** Record architecture pattern, client or interface, back end or core runtime, data or storage, identity or access, and final deployment or distribution. Show the full 24-item checklist and wait for explicit confirmation.
6. **Design the tree.** Produce a conceptual folder tree that follows the selected ecosystem and represents every derived component, boundary, test area, shared package, deployment or distribution surface, and relevant documentation. Emit at most 80 entries per batch, keep a stable cursor, and continue until the complete requested tree is represented.
7. **Design the diagram.** Create a Mermaid diagram from the same modules and relations. Show entry points, trust or process boundaries, data stores, external integrations, and deployment or distribution edges that actually apply.
8. **Validate Mermaid.** Parse or render the exact Mermaid block with an available validator. Repair at most three syntax failures per batch and continue in another bounded repair batch while a concrete repair remains. Never call an unparsed diagram valid.
9. **Confirm design.** Show the complete tree, diagram, derived choices, mitigations, and validation result. Wait for explicit user confirmation before delivery.

## Stop conditions

- Stop before design when the user has not selected explicitly, the choice is broken, or a warning lacks credible mitigation.
- Stop before finalization when no Mermaid parser or renderer is available or the diagram remains invalid after evidence-guided repair batches.
- Do not create the represented folders, files, packages, services, accounts, or infrastructure.

## Test

- Confirm that all six derived choices are concrete and trace to one audited selection.
- Confirm that every warning has an accepted, testable mitigation and no broken choice advanced.
- Confirm that the folder tree covers every planned component and that the exact Mermaid block parses or renders successfully.
- Confirm that the user approved the complete design in writing.
