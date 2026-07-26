use serde_json::Value;

/// Git branch tools — optional group `git_branches`, disabled by default.
pub fn git_tool_definitions() -> Vec<Value> {
    use super::tool_definitions::tool_def;
    vec![
        tool_def(
            "create_branch",
            "Create a new git branch from HEAD and switch to it. \
             Operates on the git repo in the working directory. \
             The new branch is created from the current HEAD commit. Fails if a branch with that name already exists. \
             For complex git workflows (rebase, merge, amend, push), use bash instead.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "branch_name": {"type": "string", "description": "Name for the new branch (e.g. feat/my-feature)"}
                },
                "required": ["branch_name"]
            }),
        ),
        tool_def(
            "checkout_branch",
            "Switch to an existing git branch. \
             Operates on the git repo in the working directory. \
             Fails if there are uncommitted changes — commit or stash them first. \
             For listing branches or checking status, use bash (`git branch`, `git status`).",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "branch_name": {"type": "string", "description": "Name of the branch to switch to"}
                },
                "required": ["branch_name"]
            }),
        ),
    ]
}
