# 01 - Capture

You resolve the trigger, action, capabilities, runtime, and scope before any mutation.

## Input

- Accept a free-form automation request, optional existing automation ID or hook files, trigger preference, action, tools, skills, and target runtimes.

## Output

- Return a confirmed automation kind, trigger, action, matcher when applicable, tools, exact skill IDs, active state, targets, scope, resolved destinations, and write boundary.

## Process

1. **Inspect.** You list native automations when the request may update one. You read existing lifecycle configuration and sibling handlers before proposing a hook change.
2. **Choose kind.** You use a native scheduled automation for a time trigger. You use a lifecycle hook for a supported session, prompt, tool, compaction, subagent, turn, or session event.
3. **Choose narrowest trigger.** You resolve one supported schedule or the most specific lifecycle event using [automation-authoring.md](../references/automation-authoring.md).
4. **Define action.** You write one bounded instruction for a scheduled agent or one bounded handler purpose for a hook.
5. **Minimize capabilities.** You select no more than eight exact skill IDs and twelve exact tools for a schedule. You select a precise lifecycle matcher only when the event needs filtering.
6. **Resolve targets.** You confirm the native project and active state for a schedule. You inspect [lifecycle-targets.md](../references/lifecycle-targets.md), detect target evidence, and confirm each runtime, scope, config path, and handler path for a hook.
7. **Confirm mutation.** You state whether you will create, update, merge, deactivate, or delete. You require explicit confirmation before deletion or replacement.

## Stop conditions

- Stop when the requested trigger is unsupported, the action contains several purposes, a skill ID or tool is unavailable, a target runtime lacks the event, or a write boundary remains unconfirmed.
- Stop when the request describes CI, a daemon, an application scheduler feature, or a manually invoked workflow instead of an automation artifact.

## Test

- Verify that one trigger maps to one action and every capability is necessary.
- Verify that every target, path, scope, matcher, active state, and destructive effect is explicit.
