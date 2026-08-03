# Data and Machine Learning

## Sources and ownership

| Source | Format | Owner or origin | Ingestion point |
| --- | --- | --- | --- |
| {dataset or stream} | {format} | {project or external owner} | `{path}` |

## Pipeline

```mermaid
flowchart LR
  {raw source} --> {processing stage} --> {artifact or serving output}
```

## Models and experiments

- {Model or experiment family}: {purpose, training or inference path, and tracked artifacts.}

## Reproducibility and quality

- {Data versioning, seeds, environment, feature definitions, evaluation, and drift conventions.}

<!-- Never copy datasets, personal data, model secrets, or column-level schemas. Remove placeholders and this comment. -->
