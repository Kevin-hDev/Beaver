# 03 - Validate

You prove trigger fit, configuration integrity, handler safety, and runtime usability.

## Input

- Accept the confirmed contract, native automation ID or lifecycle paths, and the mutation report from action 02.

## Output

- Return one `passed`, `failed`, `blocked`, or `skipped` verdict for trigger fit, configuration, capabilities, handler, and runtime execution.

## Process

1. **Reload.** You list native automations and match the exact ID, or reread every lifecycle config and handler from disk.
2. **Validate trigger.** You parse the schedule and confirm future time semantics, or confirm that each runtime event is the narrowest supported mapping for the lifecycle moment.
3. **Validate capabilities.** You confirm that every scheduled skill ID and tool is available, necessary, bounded, and permitted. You confirm that hook matchers filter only the intended occurrences.
4. **Validate handler.** You parse or lint the handler, verify executable state when required, feed bounded representative JSON through a non-destructive test, and inspect exit status and output without exposing secrets.
5. **Validate preservation.** You compare the shared configuration before and after and confirm every unrelated sibling and field survived.
6. **Observe carefully.** You run a safe manual or fixture validation only when the runtime supports it and the user authorized any effects. You do not wait for a real future trigger merely to claim success.
7. **Report evidence.** You distinguish registered, parse-valid, handler-tested, runtime-loaded, and trigger-observed states.

## Stop conditions

- Stop with `failed` when the trigger, matcher, stored capabilities, config, handler, or preservation check differs from the confirmed contract.
- Stop with `blocked` when the target runtime or a required non-destructive validator is unavailable.

## Test

- Verify that each claimed runtime has a separate evidence-backed verdict.
- Verify that an unobserved future schedule or lifecycle event is never labeled trigger-observed.
