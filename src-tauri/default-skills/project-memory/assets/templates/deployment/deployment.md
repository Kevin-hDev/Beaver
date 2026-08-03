# Deployment

## Pipeline

| Stage | Trigger | Definition | Gate |
| --- | --- | --- | --- |
| {build/test/release/deploy} | {event} | `{path}` | {required check} |

## Environments

- {environment}: {purpose, deployment target, and configuration source without secret values.}

## Release and rollback

- Release: {verified versioning, artifact, and promotion flow.}
- Rollback: {verified procedure or "Not documented."}

## Observability

- {Logs, metrics, traces, alerts, and ownership supported by repository evidence.}

<!-- Never copy credentials or environment values. Remove placeholders and this comment. -->
