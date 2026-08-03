# Review Rubric

Use severity for impact and confidence for evidence quality. Do not blend them.

## Severity

- **Critical:** Flag an exploitable security failure, unrecoverable data loss, broad outage, or equivalent release-stopping harm with a reachable path.
- **High:** Flag a likely correctness, security, privacy, authorization, or reliability failure that affects important users or data.
- **Medium:** Flag a bounded defect, regression, missing required behavior, or concrete maintainability problem likely to cause future errors.
- **Low:** Flag a small but real issue with limited impact. Do not use this level for personal style preferences.

## Confidence

- **High:** Use direct code, test, rule, or contract evidence with a clear reachable scenario.
- **Medium:** Use strong evidence with one named uncertainty that does not erase the likely impact.
- **Low:** Do not publish the item as a finding. Put it in residual uncertainty only when it materially affects confidence in the review.

## Verdict

- Return `blocked` for any supported critical finding.
- Return `changes-requested` for any supported high or medium finding, or any unmet required acceptance condition.
- Return `approve-with-notes` when only low findings remain and coverage is complete.
- Return `approve` when no actionable finding remains and coverage is complete.
- Return `incomplete` when the target, required source, inaccessible file, or truncation prevents the requested coverage.

Never let a high finding disappear from the verdict because other axes pass.
