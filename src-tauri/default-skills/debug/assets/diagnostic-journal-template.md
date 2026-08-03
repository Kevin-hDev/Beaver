# Debug Journal: {issue}

## Symptom

- Expected: {expected behavior}
- Actual: {actual behavior}

## Action path

```mermaid
flowchart LR
  {evidence-based path}
```

## Why chain

1. {causal level}

## Hypothesis batch {n}

| Hypothesis | Confidence | Evidence | Confirmation check | Status |
| --- | --- | --- | --- | --- |
| {cause} | {1-10} | {evidence} | {check} | {pending/validated/invalidated/blocked} |

## Instrumentation

| File and location | Sanitized diagnostic | Purpose | Result | Removed |
| --- | --- | --- | --- | --- |
| {path:line} | {message shape} | {confirms or refutes} | {observation} | {yes/no} |

## Conclusion

- Root cause: {confirmed cause or "Not confirmed"}
- Next pending action: {action or "None"}
