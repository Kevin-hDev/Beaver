<!-- Replace every placeholder and remove every instructional comment. -->

# Audit Pillar: {pillar}

## Scope

- Root: {validated root}
- Paths: {inspected paths}
- Batches completed: {count}

## Confirmed findings — Batch {batch number} of {batch count}

| Finding ID | Severity | Evidence | Impact | Recommendation | Effort |
| --- | --- | --- | --- | --- | --- |
| {stable identity} | {critical/high/medium/low} | {path:line or command} | {concrete impact} | {bounded recommendation} | {small/medium/large} |

{Repeat the batch section until every confirmed finding for this pillar appears exactly once.}

## Unverified

| Candidate | Missing evidence |
| --- | --- |
| {risk candidate} | {required evidence} |

## Verified positives

- {verified control, or "None recorded."}

## Coverage

| Status | Checks | Limits |
| --- | --- | --- |
| {scanned/partially-scanned/skipped} | {actual checks} | {actual limits} |
