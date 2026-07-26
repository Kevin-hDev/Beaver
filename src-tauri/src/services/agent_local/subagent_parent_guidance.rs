pub const PARENT_GUIDANCE: &str = "\
# Working with subagents

Use subagents only for independent work that can run in parallel. \
After delegate_task, continue useful independent work without duplicating delegated work. \
Do not repeatedly inspect subagents while they run; terminal reports arrive automatically. \
Use message_subagent only for a concrete correction. \
Use cancel_subagent only for a user request or clearly incorrect direction. \
When a coder report includes a pending change, it has been captured in the coder's isolated \
workspace but is not applied to the parent project. Treat it as not integrated until \
apply_subagent_changes succeeds. Do not infer its state by checking whether files exist in the \
parent project. Use inspect_subagent_changes to review its current status and diff. If it satisfies \
the request, use apply_subagent_changes and only report it as integrated after that succeeds. If \
it does not satisfy the request, choose the most appropriate response: request a correction, use \
discard_subagent_changes and fix it yourself, or discard it and delegate new work. \
An apply failure leaves the isolated change unresolved. Inspect its state before deciding how to \
continue. After integrating or recreating the intended result yourself, call \
discard_subagent_changes to clean up the obsolete isolated change and temporary branch. Do not \
claim completion while the isolated change remains pending or conflicted. \
When no useful independent work remains, give at most one short progress update and \
finish the turn without a tool call.";
