# 01 - Capture

You settle one command objective, its input and output, and its exact destination before writing.

## Input

- Accept the user's one-shot operation request, examples, constraints, optional name, and optional destination.
- Use the available skill catalog and project-local conventions to check overlap and ownership.

## Output

- Return a confirmed contract containing slug, objective, inline input, observable output, exclusions, positive and negative examples, destination, and overwrite boundary.

## Process

1. **Define one objective.** You state the repeatable operation in one sentence and separate requested side effects from the result.
2. **Check the form.** You confirm the operation can be expressed in at most eight direct steps without distinct action files, branching workflows, persistent triggers, or a new external tool connection.
3. **Capture input.** You describe how the operation consumes the user text remaining after its slash invocation and define the behavior when required input is absent or invalid.
4. **Capture output.** You name one observable result and any explicitly authorized file or external effect.
5. **Resolve identity.** You choose a kebab-case slug, compare its triggers with neighboring descriptions, and keep separate commands when method or output differs.
6. **Resolve the destination.** You read [destination-resolution.md](../references/destination-resolution.md) and require confirmation of the exact target root and overwrite boundary.
7. **Confirm cases.** You collect at least three realistic invocation requests and two close non-trigger requests, present the complete contract, and wait for confirmation.

## Stop conditions

- You stop when the request requires multiple distinct jobs, reusable action files, an always-enforced rule, an event trigger, a built-in application command, or a real CLI/API connection.
- You stop before writing when any contract field, destination, or overwrite boundary is unconfirmed.
- You do not invent `$ARGUMENTS`, positional variables, command directories, or a destination from another CLI.

## Test

- You verify that no file changed during capture.
- You verify that the contract has one objective, at most eight planned steps, one input contract, one output contract, three trigger cases, and two non-trigger cases.
- You verify that the exact destination and overwrite boundary are confirmed in writing.
