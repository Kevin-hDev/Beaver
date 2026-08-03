# Evidence Contract

## Finding sources

- You accept pasted findings or a readable report path within the user's selected workspace.
- You validate a supplied path, reject traversal, and never infer a hidden default report directory.
- You map `code-quality` to cleanup and map performance, security, and architecture by name.
- You skip broad discovery when a report supplies the selected axis, but you verify each finding against current code before editing. You mark stale or disproven findings instead of forcing a change.

## Severity

| Level | Meaning |
| --- | --- |
| Critical | You identify exploitable harm, data or access loss, severe operational failure, or a dominant measured bottleneck requiring urgent action. |
| Warning | You identify material risk, recurring cost, boundary erosion, or a significant maintainability or performance issue. |
| Minor | You identify bounded local debt or a small supported improvement with low immediate impact. |

You rate severity from impact and evidence, not from wording in an unverified report.

## Baselines

| Axis | Minimum baseline |
| --- | --- |
| Performance | You capture a repeatable workload, observable behavior, and a relevant metric or counted operation. |
| Security | You capture a reachable trust boundary and a regression that demonstrates the unsafe behavior without exposing real sensitive data. |
| Cleanup | You capture existing focused checks, public surface, and representative behavior. |
| Architecture | You capture documented boundaries, relevant dependency edges, existing checks, and representative behavior. |

## Verification claims

- You label a check `passed` only when you executed it and observed a successful result.
- You label a behavior `preserved` only when comparable evidence covers its relevant inputs, outputs, errors, ordering, and side effects.
- You label a performance gain `measured` only when pre-change and post-change workloads and conditions are comparable; otherwise you use `unverified`.
- You label a security issue `fixed` only when its regression and required changed-scope checks pass.
- You label a boundary `restored` only when the final dependency evidence demonstrates it.
- You retain failing results and report `incomplete`; you never weaken, skip, delete, or silence a required check to obtain success.

## Result record

For every finding, you return:

1. You identify the axis, file or symbol, severity, and evidence.
2. You record the applied diff, or the stale, disproven, deferred, or blocked reason.
3. You record baseline and final checks with their exact observed states.
4. You disclose uncertainty, intentional security behavior changes, and remaining risk.
