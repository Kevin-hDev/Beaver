# Decision Heuristics

Read these heuristics only while generating candidates. Treat them as starting points and override them when current evidence or the independent audit contradicts them.

## Architecture families

| Signal | Candidate family |
| --- | --- |
| Small team, limited domains, modest scale | Monolith or single-package core |
| Growing domains with one deployment owner | Modular monolith or modular package workspace |
| Independently scaled domains with mature operations | Services with explicit contracts |
| Bursty stateless work and low operations capacity | Managed event or function execution |
| Long-running real-time connections | Stateful service or event-driven core rather than purely short-lived functions |
| Batch, analytics, or model workflows | Staged pipeline with reproducible data and artifact boundaries |
| Extensible desktop, CLI, or library product | Layered core with adapters, commands, or plugin boundaries |

## Project-type concerns

| Type | Preserve these candidate decisions |
| --- | --- |
| Hosted multi-user product | Front end, back end, database, authentication, tenancy, hosting, operations, monthly cost |
| Web site or application | Rendering, client state, server boundary, data, SEO, hosting, browser performance |
| Mobile application | Native or cross-platform UI, local state, sync, back end, stores, signing, device support |
| Desktop application | UI toolkit, local core, storage, updates, signing, OS support, optional cloud service |
| CLI | Command model, core library, configuration, credentials, packaging, shell and OS support |
| Library | Public API, runtime matrix, dependencies, build artifacts, versioning, registry, compatibility |
| Service or API | Protocol, runtime, persistence, identity, observability, scaling, deployment |
| Data or ML project | Ingestion, transformation, orchestration, storage, lineage, compute, serving, reproducibility |
| Internal tool | Identity, permissions, integrations, maintenance ownership, deployment, auditability |

## Constraint priority

1. Prioritize safety, legal, privacy, residency, and license constraints.
2. Prioritize required platforms, protocols, integrations, and offline behavior.
3. Prioritize measurable scale and performance requirements.
4. Prefer team-operable choices over theoretically ideal technology with an unsupported learning burden.
5. Enforce the confirmed operating and distribution budget.
6. Use preferences only after every harder constraint is satisfied.

## Candidate spread

Create genuinely different candidates by varying a high-impact choice such as operational ownership, architecture boundary, runtime family, persistence model, deployment model, or distribution channel. Do not manufacture diversity through minor libraries, cosmetic syntax, or equivalent managed vendors.

Verify every named technology, version, platform, service, license, limit, and price against current official sources before presenting it as suitable.
