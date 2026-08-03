# Attempt log format

You append one entry per attempt. You never edit or reorder earlier entries.

```text
### Attempt <N> - <UTC timestamp>
- Step: <unchecked step and acceptance criterion>
- Hypothesis: <falsifiable cause>
- Changed from prior attempt: <materially different evidence, approach, or target>
- Planned progress signal: <observable result predicted before execution>
- Actions: <bounded commands, files, and effects>
- Observation: <direct result with sensitive values removed>
- Independent verification: <command or predicate, result code/value, bounded evidence>
- Classification: <step-passed|progressed|no-progress|regressed|inconclusive>
- State fingerprint: <branch/revision and relevant state identity>
- Boundaries remaining: <attempts, time, resources, no-progress allowance, gated-effect occurrences>
- Decision: <next step|retry with new hypothesis|blocked|completed>
```

You use monotonically increasing attempt numbers. You record a failed launch or partial attempt because it consumed time and may have changed state. You cite artifact paths without exposing secret values or unrestricted logs.
