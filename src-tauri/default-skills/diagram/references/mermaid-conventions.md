# Mermaid Conventions

Use the project's configured Mermaid syntax and version first. Otherwise target Mermaid 10.8.0 or newer and use syntax supported by a compatible available validator. Avoid version-specific features that the validator cannot prove.

## Global conventions

- You add a title through Mermaid frontmatter. When an explicitly configured older project renderer cannot parse title frontmatter, you report the compatibility conflict rather than silently removing the title.
- You default flow diagrams to `LR` unless the source implies another direction.
- You keep each label on one line and shorten it rather than inserting a literal `\n` or HTML break.
- You use descriptive alphanumeric identifiers and quoted bracket labels, such as `OrderService["Order Service"]`.
- You keep identifiers, labels, terminology, and direction consistent.
- You define elements before their relationships.
- You add no styling unless the user confirms styling as part of the plan.
- You create no empty or isolated element unless the confirmed source explicitly contains one.

## Type selection

| Source structure | Prefer |
| --- | --- |
| Components, dependencies, or process steps | `flowchart` |
| Ordered messages between participants | `sequenceDiagram` |
| Lifecycle states and guarded transitions | `stateDiagram-v2` |
| Entities, attributes, and cardinalities | `erDiagram` |
| Classes, members, inheritance, or composition | `classDiagram` |
| Time-bound work, dependencies, and milestones | `gantt` |
| User journey stages and satisfaction | `journey` |

Do not force a source into one diagram when it contains two unrelated semantic views. Ask whether the user wants separate diagrams.

## Elements and relationships

- You represent confirmed groups, parents, and children with the diagram type's native grouping syntax.
- You use fork and join for confirmed parallel state paths and a choice for confirmed conditional state paths.
- You declare state pseudostates with the confirmed form `state ForkPoint <<fork>>`, `state JoinPoint <<join>>`, or `state ChoicePoint <<choice>>`.
- You use directed links and messages that match the source direction.
- You use labeled, dashed, thick, self, optional, and bidirectional relationships only when their meaning is confirmed.
- You write a confirmed labeled flow link as `SourceNode -- label --> TargetNode`.
- You write a confirmed dashed flow link as `SourceNode -.-> TargetNode`, a confirmed emphasized link as `SourceNode ==> TargetNode`, and a confirmed self-loop as `SourceNode --> SourceNode`.
- You preserve exact cardinalities, guards, message order, and dependency direction.
- You keep `:` out of state identifiers and place the human description in the supported label syntax.

## Gantt

- You use `active`, `done`, `crit`, and `milestone` only when the source confirms the corresponding status.
- You combine Gantt tags when the source confirms multiple statuses for the same task.
- You preserve explicit dates, durations, dependencies, and sections without inventing scheduling detail.

## Validation

- You validate the exact final source after title frontmatter and every repair.
- You treat parser warnings that change or drop meaning as failures.
- You confirm compatibility with the project's renderer rather than targeting a newer unsupported syntax.
