# Rule Authoring

You use this contract for the canonical rule and every rendering.

## Core contract

- You keep one rule file focused on one topic.
- You write every requirement as a concise second-person imperative statement.
- You use bullets rather than narrative paragraphs for enforceable requirements.
- You keep the actionable core of each bullet to 3-7 words after `You` when clarity permits, and extend it only to encode necessary scope or conditions.
- You make each statement observable or reviewable and avoid vague goals such as "use best practices" or "keep it clean."
- You preserve the confirmed strength: `must`, `must not`, `always`, `never`, or a clearly bounded conditional.
- You state exact scope through the project's supported metadata or an explicit all-project declaration.
- You avoid repeating, weakening, or contradicting another active rule.
- You group closely related statements only when grouping improves scanning.
- You add the smallest good or bad example only when prose alone leaves a plausible ambiguity.
- You write generated rule content in English regardless of the conversation language.

## Naming and grouping

- You follow the project's existing filename and category convention first.
- You use a lowercase descriptive hyphenated slug when the project has no naming convention and the user confirms that fallback.
- You may propose this fallback taxonomy when grouping materially helps and the user confirms it:

| Index | Category | Covers |
| --- | --- | --- |
| 00 | architecture | System boundaries, API design, and structural patterns |
| 01 | standards | Naming, formatting, imports, and language-agnostic style |
| 02 | programming-languages | Language syntax, types, and runtime behavior |
| 03 | frameworks-and-libraries | Framework and dependency usage |
| 04 | tooling | Build, automation, infrastructure, and configuration |
| 05 | testing | Test structure, fixtures, mocking, and coverage |
| 06 | design-patterns | Reusable code design patterns |
| 07 | quality | Security, reliability, accessibility, and performance |
| 08 | domain | Business entities, invariants, and workflows |
| 09 | other | Rules that fit no narrower category |

- You pick the narrowest accurate category and use its index only when the confirmed project format uses indices.

## Example quality

- You show both good and bad examples when the contrast prevents a common misreading.
- You keep examples tiny, syntactically plausible, and consistent with the rule's scope.
- You never let an example introduce an unconfirmed exception or requirement.

Example:

```text
- You pass process arguments as a validated list.

Good: run("formatter", [validatedFile])
Bad:  runShell("formatter " + userFile)
```

## Split test

You split a draft when two statements have different owners, scopes, lifecycles, categories, or validation methods. You reconfirm new filenames and targets before writing the split files.
