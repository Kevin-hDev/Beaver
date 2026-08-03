# 04 - Prioritize

You add honest relative effort, impact, dependencies, readiness, and a strict delivery order.

## Input

- Accept draft stories, estimation policy, product value, dependencies, and risk evidence.

## Output

- Return effort, confidence, impact, readiness, dependencies, rationale, and unique rank for every story.

## Process

1. **Open a batch.** You process at most 20 stories at a time, keep global dependency and value context, and continue until every drafted story is covered.
2. **Apply team policy.** You use documented points or sizing rules when available.
3. **Estimate honestly.** Without team policy, you use small, medium, or large with low, medium, or high confidence.
4. **Reject oversized stories.** You return any story too uncertain or large to slicing.
5. **Rate impact.** You distinguish isolated, shared-behavior, and critical-path change with evidence.
6. **Check readiness.** You require testable criteria, named dependencies, acceptable size, no blocking product question, and no unaccepted assumption that can change scope, acceptance, dependency, or user value.
7. **Rank globally.** After all batches are assessed, you prioritize the complete backlog by user value and risk reduction against effort, then apply dependency ordering with a stated reason.

## Stop conditions

- You never invent exact person-days, deadlines, velocity, or business value scores.
- You do not rank a blocked story as ready.
- You mark a conditional or assumption-dependent story `not ready` instead of calling it ready under that assumption.

## Test

- Every story has one unique rank, supported estimate, readiness state, and dependency rationale.
- Any dependency override is explicit and preserves a deliverable outcome.
