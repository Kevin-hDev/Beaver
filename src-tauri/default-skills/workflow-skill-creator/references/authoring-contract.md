# Authoring Contract

You apply this contract to every generated workflow skill.

## Router

- You keep `SKILL.md` under 500 lines and retain only the workflow, action selection, transversal rules, and direct resource links.
- You write frontmatter `name` and one single-line `description` of at most 250 characters.
- You make the description state what the skill does, concrete use cases, and concise exclusions without naming unrelated skills as dependencies.
- You include a diagram only when a branch, loop, or state transition materially clarifies the workflow.
- You let every skill run independently and add cross-skill routing only when an external handoff is a real required capability.

## Actions and resources

- You give every action `Input`, `Output`, `Process`, `Stop conditions`, and `Test` sections.
- You start action input bullets with `Accept` or `Use` and output bullets with `Return`.
- You write instructions as direct second-person imperatives and keep one idea per sentence.
- You keep references directly linked from `SKILL.md` or the consuming action and avoid deep reference chains.
- You store copied output scaffolds in `assets/`, executable deterministic helpers in `scripts/`, and facts or rubrics in `references/`.
- You omit empty directories and files that no execution path consumes.

## Evaluations and safety

- You store bounded evaluation cases in `evals/cases.json` with matching `version`, `skill`, unique identifiers, messages, expectations, and prohibited outcomes.
- You include at least three trigger cases, at least two close non-trigger cases for writing skills, invalid or missing input, tool failure, no-change behavior, and an observable-result check.
- You add a semantic `judge` when keywords cannot prove the required reasoning or workflow.
- You validate every external path, bound externally supplied collections, protect secrets, avoid shell interpolation, fail closed, and expose no internal error detail to end users.
- You preserve unrelated user content and report only checks and effects that actually occurred.
