# Automation authoring contract

## Native scheduled automation

- You choose exactly one supported trigger: `once`, `daily`, or `weekly`.
- You express local times as `YYYY-MM-DDTHH:MM` or `HH:MM` and use weekday `0..6` only for weekly schedules.
- You write one bounded instruction with observable completion criteria.
- You select at most eight exact source-qualified skill IDs and twelve exact tools.
- You exclude interactive choice, plan approval, nested subagent, and automation-management tools from scheduled execution.
- You use the project and model captured by the runtime instead of embedding private application paths or provider secrets.

## Lifecycle hook

- You choose the narrowest lifecycle moment that fits: session start, prompt submitted, before tool, after tool, tool failure, before or after compaction, subagent start or stop, turn stop, or session end.
- You confirm the event exists in every target runtime and skip unsupported targets with a reason.
- You use one hook for one purpose and one precise matcher when filtering is necessary.
- You keep executable logic in a handler and configuration in a small merged entry.
- You bound event input, never log secrets, never use `eval`, pass process arguments separately, and fail closed for required checks.
- You block only when the target event explicitly supports blocking and the user confirmed blocking behavior.

## Evidence levels

1. `registered`: the native runtime returned and reloaded an automation ID.
2. `parse-valid`: the stored schedule or hook configuration parses.
3. `handler-tested`: a representative bounded event produced the expected signal.
4. `runtime-loaded`: the target runtime accepted or listed the artifact.
5. `trigger-observed`: the real event fired and produced the expected outcome.

You report only the highest level actually observed for each target.
