# 01 - Build

You create a new specification from a request, clarified idea, user stories, or PRD.

## Input

- Accept a free-form request or supplied source document.
- Accept an optional output destination.

## Output

- Return a complete specification draft.
- Return unresolved `TBD` questions.
- Return a handoff to `03-validate`.

## Process

1. **Validate.** You reject an empty request. You read inline input in ordered chunks of at most 100,000 characters and files in ordered chunks of at most 256 KiB until the complete source is covered. When the user supplies a source path, you reject `..`, validate the authorized working area, and stop if the file cannot be read.
2. **Read the rules.** You read [specification-rules.md](../references/specification-rules.md).
3. **Extract.** You process at most 100 requirements, constraints, and completion conditions per batch and continue until the source is covered. You identify one target, hard constraints, non-goals, observable completion conditions, stakeholders, and concise context. You preserve mandated technical constraints, remove secrets and sensitive values, and remove proposed implementation choices that are not requirements.
4. **Split.** When the source contains unrelated primary targets, you propose separate specifications instead of hiding them in one target sentence.
5. **Mark gaps.** You write `TBD: <precise question>` for every missing required decision. You do not guess.
6. **Render.** You follow [spec-template.md](../assets/spec-template.md). You omit an optional section when the source provides no useful content for it.
7. **Persist conditionally.** You return the draft in the conversation by default. When the user requested a file, you validate the destination, preserve an existing file unless replacement was authorized, and write only there.
8. **Validate.** You read `03-validate` and check the draft before you report completion.

## Stop conditions

- You stop when the request is too vague to identify even one target.
- You stop when a supplied source cannot be read or its authority is unclear.
- You stop before overwriting an existing or locked specification without valid authority.

## Test

- The draft contains Target, Hard constraints, Non-goals, and Done-when in the required order.
- Every missing required decision appears as a precise `TBD` question.
- The draft introduces no unrequested implementation choice.
- No file changes when the user did not request persistence.
