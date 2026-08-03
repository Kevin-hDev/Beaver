# Documentation Types

Choose the narrowest document that owns the reader's goal. Combine types only when the existing project convention combines them.

| Type | Primary reader goal | Required concerns |
| --- | --- | --- |
| README | Understand the project and reach the first successful use | Purpose, supported scope, prerequisites, setup, first result, validation, next references |
| User guide | Complete a product task safely | Preconditions, ordered actions, visible results, variants, errors, recovery |
| Developer guide | Change or extend the project correctly | Architecture boundary, setup, workflow, tests, conventions, debugging, contribution path |
| API reference | Call a public interface correctly | Authentication, inputs, outputs, errors, limits, examples, version or compatibility state |
| CLI or command reference | Invoke supported operations correctly | Syntax, arguments, defaults, exit states, output shape, examples, platform differences |
| Configuration reference | Set supported behavior safely | Key, type, default, allowed values, scope, sensitivity, reload behavior, examples |
| Data model reference | Understand a supported public or operational data contract | Entities, fields, relations, invariants, lifecycle, compatibility, privacy boundaries |
| Architecture document | Understand current components and boundaries | Responsibilities, data and control flow, trust boundaries, dependencies, constraints, diagrams |
| Migration guide | Move between confirmed states without data loss | Supported start and end, prerequisites, backup, ordered steps, verification, rollback, breaking changes |
| Deployment guide | Deliver a supported build repeatably | Artifacts, environment, configuration, ordered steps, verification, rollback, platform differences |
| Operations guide | Operate a system repeatably | Preconditions, health signals, routine actions, failure response, rollback, escalation |
| Security guide | Apply and maintain confirmed protections | Trust boundaries, supported controls, secret handling, safe procedures, validation, incident boundary |
| Troubleshooting guide | Diagnose known user-visible failures | Symptom, safe checks, confirmed causes, resolution, verification, escalation boundary |

Keep ADRs focused on why a decision was made. Keep project memory focused on durable internal context. Keep specifications focused on behavior to build. Do not turn those artifacts into general documentation substitutes.
