# 05 - Deliver

You return the ready backlog and persist it only at an explicitly requested destination.

## Input

- Accept ranked stories, destination, document rules, and optional configured tracker.

## Output

- Return the complete conversation backlog, requested file, or verified tracker batches.

## Process

1. **Gate readiness.** You keep blocked stories separate and never export them as ready.
2. **Return by default.** You present the ordered backlog in the conversation when no destination is explicit.
3. **Write conditionally.** You validate an explicit file path and preserve unrelated existing content.
4. **Resolve tracker conditionally.** You use one configured connector only when tracker creation was explicit.
5. **Check duplicates.** You search for matching existing stories before each external creation.
6. **Confirm external creation.** You show the complete ready ranked backlog and wait for explicit user approval immediately before the first tracker write.
7. **Create bounded batches.** You create at most 20 ready stories per batch and continue with the next ordered batch after every result is verified. You stop on the first failure, preserve a continuation cursor, and report successful, failed, and not-yet-created items separately so a later retry can search remote state first.

## Stop conditions

- You never create tracker items from an implicit save request or without a configured destination.
- You do not demand a redundant confirmation after an already explicit, complete tracker creation request.
- You never retry an ambiguous create before searching remote state.

## Test

- Every exported story is ready and matches the conversation or file body.
- Every created tracker story has one verified identifier and URL and no duplicate was created.
- The union of completed batches covers every ready story unless a reported failure stopped continuation.
- No tracker write occurs before explicit approval of the complete ready backlog.
