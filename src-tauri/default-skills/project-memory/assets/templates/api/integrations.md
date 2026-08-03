# Integrations

## External services

| Service | Purpose | Integration point | Failure handling |
| --- | --- | --- | --- |
| {service} | {project use} | `{path}` | {retry, fallback, or fail-closed behavior} |

## Data flow

```mermaid
flowchart LR
  {project boundary} --> {external service}
```

## Contract boundaries

- {Authentication, rate-limit, webhook, idempotency, or data-ownership convention without secret values.}

<!-- Include only verified external services and macro flow. Remove placeholders and this comment. -->
