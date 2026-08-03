# 02 - Write

You register the native schedule or merge the lifecycle hook exactly as confirmed.

## Input

- Accept the confirmed automation kind, trigger, action, capabilities, targets, paths, and mutation boundary from action 01.

## Output

- Return the native automation ID and stored contract, or every lifecycle config and handler path touched.

## Process

1. **Register a schedule.** You call `manage_automation` with `create` or `update`, the confirmed name, description, instruction, schedule, exact skill IDs, exact tool names, and active state. You use the current project and current model supplied by the runtime.
2. **Delete only explicitly.** You call `manage_automation` with `delete`, the exact ID, and `confirm: true` only after the user explicitly requested and confirmed deletion.
3. **Build a handler.** For a lifecycle hook, you copy [hook-handler-template.sh](../assets/hook-handler-template.sh) only when a command handler is required. You bound stdin, avoid `eval`, avoid shell-constructed commands, filter secrets, and return the runtime's documented signal.
4. **Build entries.** You fill [hook-entry-template.json](../assets/hook-entry-template.json) and convert it to each confirmed runtime shape in [lifecycle-targets.md](../references/lifecycle-targets.md).
5. **Merge.** You parse each target configuration, append under the correct event, preserve every sibling and unknown field, and write atomically when the format allows it.
6. **Secure paths.** You reject `..`, unconfirmed global destinations, symlink escapes, relative handler paths that the runtime cannot resolve, and handlers outside the confirmed scope.
7. **Report mutation.** You list the created or updated automation ID, config entries, handlers, preserved siblings, and skipped targets.

## Stop conditions

- Stop before mutation when a schedule needs an unavailable tool or skill, a hook handler can expose secrets, a shared config cannot be parsed, an event mapping is uncertain, or a sibling would be replaced.
- Stop rather than converting an unsupported lifecycle event into a broader event without confirmation.

## Test

- Verify that the native automation returned an ID and the stored trigger, tools, skills, and active state match the confirmation.
- Verify that each hook entry exists once, every previous sibling survives, and every referenced handler exists at its confirmed path.
