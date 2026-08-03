# 03 - Draft

You assemble the product decisions into one concise PRD whose sections do not contradict each other.

## Input

- Accept the complete frame, requirements, supplied-story traceability, evidence, assumptions, and questions.

## Output

- Return one complete PRD with every required product section and traceability to supplied stories when present.

## Process

1. **Lead with value.** You summarize the problem, audience, and desired outcome in two or three sentences.
2. **Fill sections.** You include only relevant supported content and preserve required project PRD sections when one exists.
3. **Trace requirements.** You ensure each requirement serves a goal, each concise user story maps to a requirement, each acceptance criterion verifies a requirement or story, and each success measure tests a desired outcome. When stories were supplied, you preserve their identifiers or source anchors in the traceability without copying backlog-management fields into the PRD.
4. **Check boundaries.** You reconcile scope with non-goals, dependencies, risks, and questions.
5. **Remove implementation.** You replace architecture, file, API, schema, framework, and algorithm choices with the outcome they were meant to serve.
6. **Label uncertainty.** You mark assumptions and unresolved decisions without presenting them as approved facts.

## Stop conditions

- You do not hide blocking questions in prose or invent answers to make the PRD appear complete.
- You do not turn user stories, implementation tasks, or technical design into requirements sections.
- You do not change the status, estimate, priority, assignee, or content of a supplied story.

## Test

- The PRD explains what, who, why, scope, and success without describing how to build it.
- Every section is consistent with the stated goals and non-goals.
- User Stories and Acceptance Criteria are complete, concise, and traceable without becoming technical tasks.
