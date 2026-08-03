---
name: async-development
description: Runs a portable async ticket pipeline for setup, one-item implementation, and bounded review correction. Use to install or operate tracker-to-code automation. Not for ordinary coding, status checks, debugging, or unbounded queues.
---

# Async Development

You drive one asynchronous development pipeline through exactly one of three independent sub-flows: `setup`, `run`, or `review`. You use only adapters whose real interfaces and required capabilities you can inspect. You report an unsupported integration honestly instead of inventing commands, schemas, or success.

## Router

You choose exactly one sub-flow and never switch after selection:

1. You honor an exact `action=setup`, `action=run`, or `action=review` override first.
2. You use a trusted, normalized integration event only when its adapter contract identifies one sub-flow unambiguously.
3. You inspect the configured repository, ticket, and change-request state.
4. You use the user's natural-language intent last.
5. You stop and ask for one choice when signals remain ambiguous.

You surface any conflict between intent, configuration, and observed state before an effect. You never repair a conflict by silently switching sub-flows. You read [the routing contract](references/routing.md) for precedence and edge cases.

## Setup

You run every setup action in order. You skip a conditional action only when its own output records the reason.

You follow `01 → 02 → 03 → 04 → 05 → 06 → 07 → 08 → 09 → 10 → 11` and preserve every completed action for resume.

| # | Action | Purpose |
| --- | --- | --- |
| 01 | [Detect context](actions/setup/01-detect-context.md) | You identify the repository and inspect real adapter capabilities. |
| 02 | [Collect configuration](actions/setup/02-collect-configuration.md) | You record states, limits, execution paths, and exact effect authority. |
| 03 | [Generate integration](actions/setup/03-generate-integration.md) | You create a remote integration only from a verified provider format. |
| 04 | [Generate local runner](actions/setup/04-generate-local-runner.md) | You create a one-cycle local entry point only for a verified runtime interface. |
| 05 | [Write configuration](actions/setup/05-write-configuration.md) | You persist a secret-free, validated configuration atomically. |
| 06 | [Bootstrap states](actions/setup/06-bootstrap-states.md) | You create configured lifecycle states when explicitly authorized. |
| 07 | [Ensure capabilities](actions/setup/07-ensure-capabilities.md) | You detect existing adapters before any optional installation. |
| 08 | [Bind credentials](actions/setup/08-bind-credentials.md) | You verify secure credential references without handling secret values. |
| 09 | [Configure scheduling](actions/setup/09-configure-scheduling.md) | You configure an optional, supported schedule with exact authority. |
| 10 | [Record repository changes](actions/setup/10-record-repository-changes.md) | You commit and push only the generated paths explicitly authorized. |
| 11 | [Run smoke test](actions/setup/11-run-smoke-test.md) | You exercise the pipeline with an isolated disposable ticket. |

## Run

You execute one ticket per invocation. You never turn a resumable queue into an unbounded loop.

You follow `01 → 02 → 03 → 04 → 05 → 06`. When a selected ticket reaches a terminal state before delegation, you skip inapplicable middle actions with recorded reasons and still write action 06's durable audit before exit.

| # | Action | Purpose |
| --- | --- | --- |
| 01 | [Poll one ready item](actions/run/01-poll-one-ready-item.md) | You select one eligible ticket deterministically. |
| 02 | [Resolve dependencies](actions/run/02-resolve-dependencies.md) | You verify native, textual, and state-based blockers. |
| 03 | [Acquire lock](actions/run/03-acquire-lock.md) | You acquire and verify an atomic lifecycle lock. |
| 04 | [Discover lifecycle workflow](actions/run/04-discover-lifecycle-workflow.md) | You resolve one complete development workflow by capability. |
| 05 | [Delegate and observe](actions/run/05-delegate-and-observe.md) | You delegate once and judge only observed repository and tracker state. |
| 06 | [Write durable audit](actions/run/06-write-durable-audit.md) | You persist the cycle result and verify authorized finalization effects. |

## Review

You collect every relevant page within the configured bound, then apply the stop rules before any correction.

You follow `01 → 02 → 03 → 01` while the decision is `continue`; you run `04` when the decision is `stop`. You count every correction attempt against one configured total and never reset it on re-entry.

| # | Action | Purpose |
| --- | --- | --- |
| 01 | [Collect feedback](actions/review/01-collect-feedback.md) | You gather new feedback from every configured discussion surface. |
| 02 | [Evaluate stop conditions](actions/review/02-evaluate-stop-conditions.md) | You apply ordered human-control and convergence rules. |
| 03 | [Apply correction iteration](actions/review/03-apply-correction-iteration.md) | You delegate one correction round and verify every effect. |
| 04 | [Finalize review](actions/review/04-finalize-review.md) | You publish an idempotent result only when authorized and verified. |

## Non-negotiable contracts

- You keep tracker, version-control, integration, scheduler, and development-workflow adapters independent. You use [the adapter contract](references/adapters-and-capabilities.md) before any provider operation.
- You record exact external-effect authority during setup and recheck it against the current request before every effect. You read [the authority and audit contract](references/authority-and-audit.md).
- You never ask for, read, display, paste, log, or persist a secret value. You use opaque references to a secure store and verify names or availability only.
- You fail closed when a lock, state transition, comment, test, audit write, or critical observation cannot be verified.
- You stop immediately when the default branch changes during a delegated run. You preserve evidence and never attempt automated recovery.
- You use bounded, resumable pages and cycles. You record a continuation cursor instead of silently truncating or processing an unlimited collection.
- You preserve idempotency keys, audit history, user changes, and previously verified effects on every retry.

## Resources

- You read [the configuration contract](references/setup/configuration-contract.md) before setup action 02.
- You read [the integration-generation guide](references/setup/integration-generation.md) before setup actions 03, 04, or 09.
- You read [the ordered review stop rules](references/review/stop-conditions.md) before review action 02.
- You use [the configuration template](assets/configuration-template.json), [integration contract template](assets/integration-contract-template.md), [run record template](assets/run-record-template.json), and [review summary template](assets/review-summary-template.md) as output skeletons. You replace every placeholder and validate the resulting artifact before use.
