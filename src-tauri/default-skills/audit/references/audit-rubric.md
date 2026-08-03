# Audit Rubric

Read this reference before rating, deduplicating, or merging findings.

## Finding threshold

Report a confirmed finding only when all fields exist:

- a violated project rule, documented contract, security control, measurable budget, or observable harmful behavior;
- precise file-and-line or command evidence;
- a concrete impact and affected path;
- a scoped recommendation that addresses the cause;
- a confidence level supported by the evidence.

Keep a concern under `Unverified` when runtime evidence, external advisory data, or missing project intent is required.

## Severity

| Severity | Use it when |
| --- | --- |
| Critical | Evidence shows a reachable exploit, secret exposure, data loss, authorization bypass, or broadly broken core correctness requiring immediate action |
| High | Evidence shows significant user, security, reliability, or architectural harm on a reachable path |
| Medium | Evidence shows bounded debt or risk that can cause defects or materially slow safe change |
| Low | Evidence shows a localized maintainability or consistency issue with limited impact |

Do not raise severity for uncertainty. Move uncertain items to `Unverified`.

## Effort

- `S`: a focused change normally contained within a few hours.
- `M`: a multi-file change normally contained within one working day.
- `L`: a cross-module or migration-sized change likely exceeding one working day.

Effort is an estimate, not a promise. Use `unknown` when the inspected evidence cannot support it.

## Health verdict

- `good`: no confirmed Critical or High finding in fully scanned selected pillars.
- `watch`: confirmed Medium findings or meaningful partial coverage.
- `fragile`: at least one confirmed High finding or several connected Medium findings affecting core paths.
- `critical`: at least one confirmed Critical finding with demonstrated reachability.
- `incomplete`: evidence is too limited to assign health honestly.

## Deduplication

- Merge symptoms that share one cause and recommendation.
- Keep separate findings when fixes, owners, trust boundaries, or user impacts differ.
- Preserve the highest supported severity, not the highest imagined severity.
