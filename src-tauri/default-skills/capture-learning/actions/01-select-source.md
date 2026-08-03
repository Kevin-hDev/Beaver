# 01 - Select Source

You identify the smallest complete evidence set for the requested learning.

## Input

- Accept a learning request and an optional conversation, file, diff, or review source hint.
- Accept the validated project root and available read-only source operations.

## Output

- Return ordered source specifications with kind, stable label, bounded slice, continuation cursor, and availability state.

## Process

1. **Read the source contract.** You read [sources.md](../references/sources.md) and classify each requested origin as conversation, file, diff, or review.
2. **Prefer explicit evidence.** You honor an explicit source hint. Without one, you use the current conversation only when it contains the completed work or decision to capture.
3. **Validate access.** You canonicalize project paths, reject traversal and paths outside the project, validate revision syntax and review identifiers, and use only read-only operations.
4. **Slice completely.** You create at most 50 source specifications per batch. You split long conversations, files, diffs, and reviews at the limits in the source contract, preserve stable order, and continue later batches until the full selected scope is represented.
5. **Keep the set narrow.** You select the smallest slices that preserve the decision, evidence, tradeoff, and consequence. You include multiple kinds only when one cannot explain the learning alone.
6. **Resolve ambiguity.** You ask one focused question when plausible source choices would produce materially different learning. You do not choose silently.

## Stop conditions

- You stop when a required source is missing, empty, unreadable, truncated without continuation, or unsafe.
- You stop before reading an unrelated source or changing repository, tracker, or review state.
- You do not choose a destination during source selection.

## Test

- Every specification names one readable source kind, stable label, bounded slice, and continuation state.
- The union of source batches covers the selected evidence exactly once without becoming a total cap.
- No source escapes the validated project or changes external state.
