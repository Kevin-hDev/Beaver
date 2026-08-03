# Challenge Rubric

Read this reference before classifying findings or assigning a verdict and confidence band.

## First-principles tests

- **Outcome:** Does the work directly produce every mandatory result?
- **Logic:** Does each conclusion follow from supported premises without circular reasoning?
- **Evidence:** Are decisive factual claims current, authoritative, and applicable?
- **Feasibility:** Can the stated actors, systems, budget, and timeline perform the work?
- **Failure:** Does the approach remain safe and understandable when required operations fail?
- **Simplicity:** Can a component, state, dependency, or step be removed without losing a mandatory outcome?
- **Reversibility:** Can the decision be corrected or rolled back at a cost appropriate to its uncertainty?

## Classification

| Class | Requirement |
| --- | --- |
| Strength | Direct evidence supports a decision that helps the intended outcome |
| Blocker | Evidence shows the work cannot meet a mandatory outcome or creates an unacceptable supported risk |
| Improvement | The work remains viable, but a change would improve simplicity, clarity, cost, resilience, or maintainability |
| Unresolved | Missing or conflicting evidence could change a blocker, recommendation, or verdict |

## Verdict

- `sound`: no supported blocker remains and unresolved items cannot overturn the core outcome.
- `revise`: one or more contained blockers can be corrected without replacing the core approach.
- `rethink`: the core approach, dependency, or assumption prevents the intended outcome.
- `inconclusive`: missing intent or evidence prevents a defensible judgment.

## Confidence

- `high`: direct evidence covers all decisive claims and plausible alternatives were compared consistently.
- `medium`: the verdict is supported, but bounded assumptions or incomplete secondary evidence remain.
- `low`: missing or conflicting evidence could materially change the verdict.

Confidence describes evidence quality, not probability.
