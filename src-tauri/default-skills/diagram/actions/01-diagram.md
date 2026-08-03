# 01 - Diagram

You plan, confirm, generate, validate, deliver, and optionally review one Mermaid diagram from written evidence.

## Input

- Accept a paragraph, list, specification section, architecture description, lifecycle, process, state model, sequence, or data relationship source.
- Accept optional project Mermaid conventions, configured parser version, and requested diagram type or direction.

## Output

- Return a complete confirmed plan before generation.
- Return one exact parser-validated fenced Mermaid block, followed only by a short review question when review remains optional.
- Return the generated block only as an explicitly `unverified` draft when no compatible validator is available, and never describe it as valid.

## Process

1. **Get the source.** In your first response, you list the short workflow `plan → confirm → generate → validate → optional review` so the user knows what is coming. You ask for the written source when none is provided. You identify contradictions or missing relationships that prevent an honest plan.
2. **Read conventions.** You read [mermaid-conventions.md](../references/mermaid-conventions.md) and any established project Mermaid conventions or configured version.
3. **Choose the type.** You select the narrowest Mermaid diagram type that represents the source without distortion and explain the choice in the plan.
4. **Build the plan.** You enumerate every component or participant, logical group, parent and child, direction, hierarchy, relationship, condition, label, and note. You map each item to source evidence and mark ambiguity instead of resolving it silently.
5. **Confirm the complete plan.** You present the complete plan and wait for explicit user confirmation or corrections.
6. **Generate exactly.** You generate one Mermaid block from the confirmed plan. You define elements before relationships, use stable descriptive identifiers, and include no unconfirmed semantic content.
7. **Validate the exact block.** You use a project-configured parser or renderer first, otherwise another available Mermaid-compatible validator. You pass the exact fenced content without wrapper text and capture the parser result.
8. **Repair syntax only.** You correct at most three concrete syntax or compatibility failures per numbered batch and validate again. You continue another bounded batch only while the validator returns a new actionable syntax failure. You return to the plan for any semantic repair.
9. **Deliver.** You return the exact validated fenced Mermaid block without explanatory prose about the diagram. When validation could not run, you instead label the exact block `unverified` and state the missing validation capability. You ask only whether the user wants a review after successful validation.
10. **Review when confirmed.** You compare the exact block with the confirmed plan, check missing, extra, empty, duplicate, isolated, misplaced, or misleading elements, and propose changes. You regenerate only after the user confirms a revised plan.

## Stop conditions

- You stop before planning when no written source exists.
- You stop before generation when the plan is unconfirmed or a semantic ambiguity remains material.
- You report `unverified` and do not call the diagram valid when no compatible parser or renderer is available.
- You stop after repeated non-progressing validation failures and return the exact failure category without inventing a successful block.
- You do not write files, render images, change project code, or switch to another diagram format.

## Test

- Confirm that every node, participant, group, state, note, and relationship maps to the confirmed plan.
- Confirm that no confirmed semantic item is missing and no unconfirmed item appears.
- Confirm that the exact delivered Mermaid source parses or renders successfully with the named compatible validator.
- Confirm that the delivered block follows project and bundled conventions and contains no empty or isolated element unless the confirmed source requires one.
