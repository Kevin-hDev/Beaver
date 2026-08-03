# Agent runtime targets

## Native project agent

| Runtime | Path | Format |
| --- | --- | --- |
| Native project delegation | `.beaver/agents/<name>.md` | Markdown with `name`, `description`, and `profile` frontmatter |

You set `profile` to `explorer` or `coder`. You invoke the definition with `delegate_task` by passing its project-relative `agent_path` and the identical `subagent_type`.

## Confirmed external targets

| Runtime | Project path | Format |
| --- | --- | --- |
| Claude Code | `.claude/agents/<name>.md` | Markdown and frontmatter |
| Cursor | `.cursor/agents/<name>.md` | Markdown and frontmatter |
| OpenCode | `.opencode/agents/<name>.md` | Markdown and frontmatter |
| GitHub Copilot | `.github/agents/<name>.agent.md` | Markdown and frontmatter |
| Codex CLI | `.codex/agents/<name>.toml` | TOML |

You confirm current project conventions before writing. You treat a matching directory or instruction file only as evidence to propose a target, never as authorization to write it.

## Conversion rules

- You keep `name` and `description` everywhere.
- You keep a model or tool field only when the target supports it and the user confirmed the value.
- You convert the canonical body to `developer_instructions = '''...'''` for a Codex TOML target and escape frontmatter strings as TOML strings.
- You preserve semantic parity when a target changes syntax.
- You mark a target `blocked` when it cannot represent a required capability without weakening or widening the role.
