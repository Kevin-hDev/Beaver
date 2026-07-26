# Dynamic Injection — Syntax and Security

## Syntax

In a SKILL.md, bang-backtick syntax executes a shell command BEFORE the LLM
sees the content. The output replaces the command.

Syntax: exclamation mark + backtick + shell command + backtick

```
!`shell command here`
```

## Examples in a SKILL.md

```yaml
---
name: pr-review
description: Use when reviewing a pull request.
allowed-tools: Bash(gh *)
---

## PR Context

Modified files:
!`gh pr diff --name-only`

Full diff:
!`gh pr diff`

Comments:
!`gh pr view --comments`

## Task
You analyze this PR...
```

When the skill loads, the three `gh` commands execute.
The LLM receives the SKILL.md with real results in place of the commands.

## More examples

```
# Git state
!`git status --short`

# Current branch
!`git branch --show-current`

# Today's date
!`date +%Y-%m-%d`

# With piping and filtering
!`git diff --name-only | grep "\.js$"`

# With error fallback
!`docker ps 2>/dev/null || echo "Docker not running"`

# Limit output size
!`git log --oneline | tail -20`
```

## Security Rules

### Safe

| Pattern | Why |
|---------|-----|
| `!`git status`` | Fixed command, local source |
| `!`gh pr diff`` | Fixed command, controlled source (GitHub) |
| `!`date +%Y-%m-%d`` | System info, no sensitive data |

### Dangerous — NEVER DO THIS

| Pattern | Why |
|---------|-----|
| `!`curl $ARGUMENTS`` | User input injected into shell |
| `!`curl https://unknown-site.com`` | Uncontrolled external URL |
| `!`eval $0`` | Arbitrary code execution |

<critical_constraints>
You NEVER use `$ARGUMENTS` in a dynamic injection.
User arguments must be processed by the LLM AFTER loading,
not injected into the shell preprocessor.
</critical_constraints>

## Combining with other levels

Dynamic injection combines well with Levels 1 and 2:

```yaml
---
name: deploy-status
description: Use when checking deployment status.
allowed-tools: Bash(kubectl:*)
---

## Current cluster state
!`kubectl get pods --no-headers | head -20`

## Available commands
kubectl get pods
kubectl describe pod <name>
kubectl logs <name>
```

The skill loads with real cluster state, then the LLM can execute
additional commands to investigate.
