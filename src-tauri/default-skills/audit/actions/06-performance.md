# 06 - Performance

You identify measured bottlenecks and clearly labeled static risk candidates within the selected scope.

## Input

- Use the validated scope, existing profiles or metrics, available read-only analyzers, and relevant hot-path code.

## Output

- Return continuable batches of at most 20 performance findings or unverified candidates with evidence, impact, recommendation, severity, and effort.

## Process

1. **Find budgets and evidence.** You read project performance budgets, existing profiles, benchmarks, query plans, bundle reports, and monitoring summaries.
2. **Use existing runtime data.** You prefer measurements already available and run a profiler or benchmark only when it is configured, bounded, and non-mutating.
3. **Inspect costly patterns.** You look for unbounded reads, N+1 I/O, repeated heavy work, render churn, blocking operations, unnecessary copies, missing batching, and expensive pure computations on a plausible hot path.
4. **Inspect delivered size.** You use an existing bundle report or configured analyzer to find oversized or duplicated client dependencies. Without bundle evidence, you keep size concerns unverified.
5. **Separate certainty.** You call a problem confirmed only with runtime or complexity evidence; you label static patterns as candidates.
6. **Compare baselines.** You avoid claiming regression, slowness, or budget failure without a baseline or explicit threshold.
7. **Rate findings.** You apply severity to demonstrated user or resource impact and keep unverified candidates out of the ranked confirmed list.

## Stop conditions

- You do not start services, generate load against external systems, or run an unbounded benchmark.
- You do not infer a bottleneck solely from missing memoization, loop presence, or file size.

## Test

- Every confirmed performance finding has a measurement, complexity bound, query count, or explicit budget violation.
- Static candidates remain labeled unverified and do not receive inflated severity.
