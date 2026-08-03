# Lifecycle hook targets

## Target paths

| Runtime | Project configuration | Handler location |
| --- | --- | --- |
| Claude Code | `.claude/settings.json` or confirmed plugin `hooks/hooks.json` | A confirmed `hooks/` directory beside the configuration |
| Codex CLI | Project `.codex/hooks.json` or a confirmed hooks table | A confirmed project hook directory |
| Cursor | `.cursor/hooks.json` | `.cursor/hooks/` or another confirmed project directory |
| GitHub Copilot | `.github/hooks/<name>.json` or confirmed Copilot settings | `.github/hooks/scripts/` |

You do not target a runtime merely because this table names it. You inspect current project conventions and confirm the runtime, event, scope, and path first. You skip OpenCode config-style hooks because its lifecycle extensions require plugin code rather than a config entry plus handler.

## Common lifecycle mappings

| Moment | Claude Code | Codex CLI | Cursor | GitHub Copilot |
| --- | --- | --- | --- | --- |
| Session start | `SessionStart` | `SessionStart` | `sessionStart` | `SessionStart` |
| Prompt submitted | `UserPromptSubmit` | `UserPromptSubmit` | `beforeSubmitPrompt` | `UserPromptSubmit` |
| Before tool | `PreToolUse` | `PreToolUse` | `preToolUse` | `PreToolUse` |
| After tool | `PostToolUse` | `PostToolUse` | `postToolUse` | `PostToolUse` |
| Before compaction | `PreCompact` | `PreCompact` | `preCompact` | `PreCompact` |
| Subagent stop | `SubagentStop` | `SubagentStop` | `subagentStop` | `SubagentStop` |
| Turn stop | `Stop` | `Stop` | `stop` | `Stop` |
| Session end | `SessionEnd` | unsupported | `sessionEnd` | `SessionEnd` |

You verify current runtime documentation or an installed schema before writing because event support and shapes can change. You never infer blocking or output semantics from a similar runtime.

## Merge rules

- You parse the complete target before editing.
- You append one entry under the exact event key and preserve every sibling and unknown field.
- You use a matcher only where the target schema supports it.
- You reference the handler with a runtime-approved variable or a confirmed path resolvable from arbitrary working directories.
- You write atomically and reparse the complete target after writing.
