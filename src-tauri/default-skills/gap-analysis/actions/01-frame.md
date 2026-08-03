# 01 - Frame

You establish what the artifact must enable before deciding what is missing.

## Input

- Accept one current requirements document, PRD, specification, plan, story set, guide, decision, or equivalent text.
- Accept an optional previous version of the same artifact or a previous gap report for comparison.

## Output

- Return the current artifact purpose, consumer, sources, scan boundary, previous-source type, and input warnings.

## Process

1. **Validate the inputs.** You validate every path and authorized working area before reading. You return a plain warning and no gaps for empty current content. You warn that attribution may be imprecise for non-Markdown content and continue. You reject a gap report as the current primary source unless the user explicitly requests report comparison.
2. **Identify artifact.** You name its type and the decisions or actions it is supposed to support.
3. **Identify consumer.** You name who uses it next and what they must be able to decide, build, test, operate, or understand.
4. **Classify the previous source.** You distinguish a previous artifact version from a previous gap report by structure and content. You do not treat earlier artifact prose as a parsed report.
5. **Load authority.** You read only referenced or applicable project sources that may already resolve apparent omissions. You treat the earlier version as comparison evidence, not as current authority.
6. **Set boundary.** You exclude implementation detail that the artifact intentionally delegates to a later phase.
7. **Set categories.** You select the relevant subset of actors, states, failures, boundaries, data, dependencies, verification, assumptions, and ambiguities. You preserve the same set for both artifact versions.

## Stop conditions

- You stop when the current artifact or intended purpose is unavailable or contradictory.
- You stop without version comparison when the two artifacts serve materially different purposes or consumers and the user cannot resolve the mismatch.
- You stop with a warning and no gaps when the artifact is empty.
- You do not judge a PRD as incomplete for lacking code-level design or a plan for lacking product decisions it explicitly receives as input.

## Test

- The frame names current artifact, consumer, downstream need, sources, previous-source type, and relevant categories.
- Later gaps can be tested against the stated purpose rather than a generic checklist.
