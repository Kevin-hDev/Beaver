# Capability Signals

Confirm a capability only from a repository fact. Treat equivalent frameworks and layouts as valid evidence; never infer a capability from the product domain alone.

| Capability | Capture | Evidence examples |
| --- | --- | --- |
| `core` | purpose, architecture, map, assertions, tests, version control | any grounded non-empty project |
| `ui` | design system, forms, navigation | web UI framework, components, pages, routes, styles, or design tokens |
| `api` | HTTP or RPC surface and integrations | server framework, route/controller/API directory, schema, or RPC definitions |
| `database` | persistent store and data model | driver, ORM, migrations, schema, repository, or database configuration |
| `auth` | authentication and authorization | identity library, auth module, middleware, roles, policies, or session handling |
| `realtime` | live server-to-client updates | WebSocket, server-sent events, subscriptions, presence, or live-update code |
| `messaging` | asynchronous jobs and events | queue, broker, producer, consumer, worker, topic, retry, or dead-letter handling |
| `deployment` | build, release, and runtime environments | continuous-integration workflow, container build, release config, or deploy scripts |
| `infrastructure` | provisioned runtime topology | infrastructure-as-code, orchestration, cluster, network, or state definitions |
| `mobile` | mobile application | mobile platform directories, mobile manifest, cross-platform mobile framework, or store build config |
| `desktop` | desktop application | desktop framework, native desktop project, packaging, updater, or system integration |
| `package` | reusable library distribution | publishable manifest, exports, public entry point, package metadata, or library consumers |
| `cli` | command-line interface | executable manifest entry, argument parser, command tree, help text, or shell completion |
| `data-ml` | data processing or machine learning | notebooks, datasets, pipelines, feature code, model files, experiment tracking, or data versioning |

Record the decisive path, manifest key, dependency, or symbol for every selected non-core capability.
