---
name: cli-create
description: Use when creating a CLI skill, wrapping a CLI tool, building a skill around terminal commands, or teaching an LLM to use a specific tool. Triggers on: cli-create, create cli skill, wrap cli, terminal skill, cli wrapper.
---

# CLI Skill Creator

You create skills that teach LLMs to use CLI tools and terminal commands.
A CLI skill = a SKILL.md that gives an LLM expertise on a specific tool.

<critical_constraints>
- Description = trigger conditions ONLY, max 250 characters. Beyond 250 chars,
  the rest is truncated — the LLM never sees it. If the description explains
  HOW instead of WHEN, the LLM skips the body. This is the #1 cause of failure.
- All instructions = direct imperative 2nd person ("You check...", "You run...").
  NEVER use descriptive infinitive ("Checking...", "Running..."). Infinitives are
  perceived as suggestions and the agent skips steps.
- Scripts must be stdlib-only — zero pip/npm install.
- SKILL.md must stay under 500 lines. Move excess to references/.
</critical_constraints>

---

## Phase 1 — Identify the Target

Before writing anything, you answer these questions:

| Question | Why it matters |
|----------|---------------|
| Which tool? | Existing binary (`ffmpeg`, `gh`, `docker`) or custom tool to create |
| Installed? | You verify with `which <tool>` or `<tool> --version` |
| What docs exist? | `<tool> --help`, man pages, web docs |
| What scope? | Entire tool or a subset of commands |

```bash
# You discover available commands
<tool> --help
<tool> <subcommand> --help
man <tool>
```

You also search the web for up-to-date documentation. Local man pages
can be outdated. Online docs often have examples and undocumented options.

If the tool doesn't exist yet, you will create it in `scripts/`.

---

## Phase 2 — Choose the Level

Five levels of sophistication. You choose the right one for the use case.

| Level | Name | When to use | Example |
|-------|------|-------------|---------|
| 1 | Command wrapper | Tool is installed, commands cover the need | ffmpeg, gh, docker |
| 2 | Scripts as tools | Need custom processing, validation, or calculations | Data analyzer, report generator |
| 3 | Runtime executor | Library API is too rich for pre-built commands | Playwright, data viz libraries |
| 4 | Dynamic injection | Skill needs live context at load time | PR review, system state |
| 5 | Multi-script orchestrator | Complex pipeline with intermediate results | Parallel scanners, ETL |

Levels combine. A skill can be Level 2 + Level 4 (custom scripts + dynamic context).

### Level 1 — Command Wrapper

You document the commands of an existing tool. The LLM calls them directly.

```yaml
allowed-tools: Bash(ffmpeg:*)
```

### Level 2 — Scripts as Tools

Python/Bash scripts in `scripts/` do the heavy deterministic work.
The LLM calls them — only the output enters context, never the script code.

```yaml
allowed-tools: Bash(python ${CLAUDE_SKILL_DIR}/scripts/*)
```

### Level 3 — Runtime Executor

A runner script (`run.js`, `run.py`) executes code that the LLM writes.
The LLM has access to the full library API, not just pre-built commands.

```yaml
allowed-tools: Bash(node ${CLAUDE_SKILL_DIR}/run.js:*)
```

For runner templates, see [references/executor-template.md](references/executor-template.md).

### Level 4 — Dynamic Injection

The skill executes shell commands at load time and injects the output
directly into the content. The LLM receives real data, not a static template.

For syntax, examples, and security rules, see
[references/dynamic-injection.md](references/dynamic-injection.md).

### Level 5 — Multi-Script Orchestrator

Multiple coordinated scripts, results written to disk,
the LLM orchestrates and interprets.

---

## Phase 3 — Discover the Commands

### For an existing tool

```bash
# You extract the command structure
<tool> --help 2>&1
<tool> <subcommand> --help 2>&1
man <tool> | head -200
```

You group commands by functional category (not alphabetical).
Natural categories emerge from usage: core, navigation, configuration,
debug, administration.

### For a custom tool

You define the commands the tool should have, starting from the user's need.
Each command = one atomic action. You write the corresponding scripts.

---

## Phase 4 — Write the SKILL.md

### Frontmatter

```yaml
---
name: my-cli-skill
description: [Trigger conditions ONLY — no instructions]
allowed-tools: [Most restrictive whitelist possible]
---
```

#### `allowed-tools` reference

| Need | Syntax |
|------|--------|
| Single binary | `Bash(tool:*)` |
| Binary + subcommands | `Bash(tool *)` |
| Skill's Python scripts | `Bash(python ${CLAUDE_SKILL_DIR}/scripts/*)` |
| Specific script only | `Bash(python ${CLAUDE_SKILL_DIR}/scripts/analyze.py:*)` |
| JS runner | `Bash(node ${CLAUDE_SKILL_DIR}/run.js:*)` |
| Read-only | `Read, Grep, Glob` |
| Git only | `Bash(git:*)` |

### Body structure

```markdown
# Title — what the skill enables

## Quick Start
[3-5 essential commands to start immediately — covers 80% of cases]

## Commands by category
### Category 1
[Commands with concrete examples]

### Category 2
[Commands with concrete examples]

## Common workflows
### Workflow A
[Full command sequence for a complete use case]

## Advanced
* [Link to references/advanced.md](references/advanced.md) — when X
* [Link to references/troubleshoot.md](references/troubleshoot.md) — when Y
```

<important>
The Quick Start is mandatory. The first 3-5 commands the LLM sees must cover
80% of use cases. If the LLM has to read 200 lines before knowing what to do,
the skill fails.
</important>

### XML tags for structure

You use XML tags to separate data boundaries, enforcement rules, and output formats:

```xml
<task_context>
You are a CLI expert helping users process video files.
</task_context>

<critical_constraints>
- You NEVER delete source files unless explicitly asked.
- You ALWAYS validate input file exists before processing.
</critical_constraints>

<output_format>
You wrap your final command in <command> tags.
</output_format>
```

### Analysis order for multi-source data

When the skill processes multiple inputs (e.g., config file + log file + metrics),
you specify the exact order the LLM must follow:

```markdown
## Analysis order
1. You read the config file FIRST — understand the expected setup.
2. You examine the logs, comparing against the expected config.
3. You check metrics LAST, correlating anomalies with log findings.
```

Structured context before ambiguous content. Facts before interpretation.

---

## Phase 5 — Write Scripts (Level 2+)

### Script pattern

All scripts follow: **validate → process → output**.

```python
#!/usr/bin/env python3
"""What this script does. Stdlib only."""
import json, argparse, sys

def validate(data):
    """You validate BEFORE any processing. Fail closed."""
    if "required_field" not in data:
        print("Error: 'required_field' missing", file=sys.stderr)
        sys.exit(1)

def process(data):
    """Business logic. Returns a dict."""
    return {"result": "..."}

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("input_file")
    parser.add_argument("--format", choices=["text", "json"], default="text")
    args = parser.parse_args()
    with open(args.input_file) as f:
        data = json.load(f)
    validate(data)
    result = process(data)
    if args.format == "json":
        print(json.dumps(result, indent=2, default=str))
    else:
        for k, v in result.items():
            print(f"{k}: {v}")

if __name__ == "__main__":
    main()
```

### Script rules

| Rule | Why |
|------|-----|
| **Stdlib only** (zero pip) | Works everywhere without installation |
| **Errors to stderr, results to stdout** | LLM parses stdout, stderr = diagnostics |
| **Validate first** | Fail closed — invalid input blocks, no partial output |
| **`--format json` supported** | LLM can parse structured JSON |
| **Non-zero exit on error** | LLM detects failure |
| **No hardcoded secrets** | Environment variables only |

For full script and runner templates, see:
- [references/script-templates.md](references/script-templates.md)
- [references/executor-template.md](references/executor-template.md)

---

## Phase 6 — Final Structure

```
my-skill/
├── SKILL.md                  ← Under 500 lines
├── scripts/                  ← Deterministic scripts (0 tokens if not run)
│   ├── analyze.py
│   └── generate.py
├── references/               ← Detailed docs (0 tokens if not read)
│   ├── api-reference.md
│   └── troubleshooting.md
└── examples/                 ← Expected outputs (optional)
    └── sample-output.json
```

### Size management

| Content | Keep in SKILL.md | Move to references/ |
|---------|-----------------|-------------------|
| Quick Start | Always | |
| Essential commands (top 20) | Yes | |
| Advanced commands | | api-reference.md |
| Common workflows (top 3) | Yes | |
| Specialized workflows | | workflows.md |
| Troubleshooting | | troubleshooting.md |
| Full parameter lists | | options.md |

---

## Phase 7 — Anti-patterns

| Anti-pattern | Problem | Fix |
|-------------|---------|-----|
| **Encyclopedia** — list ALL options of ALL commands | SKILL.md exceeds 500 lines, LLM loses focus | Quick Start + top 20, rest in references/ |
| **No examples** — just command syntax | LLM doesn't know how to combine commands | Add 2-3 complete workflows |
| **Explanatory description** | LLM reads description, thinks it knows, skips body | Description = triggers only |
| **Infinitive instructions** | Agent treats as suggestion, skips steps | Imperative 2nd person |
| **Too many choices** — "use X OR Y OR Z" | LLM hesitates and picks wrong | One default + documented exception |
| **Commands without context** — just `tool cmd --flag` | LLM doesn't know WHEN to use which command | Group by use case, not alphabet |
| **Scripts with pip install** | Fails on machines without dependencies | Stdlib only |
| **No edge case handling** | Agent stops silent | Cover "nothing to do" explicitly |

---

## Phase 8 — STOP CHECK

You run this checklist before delivering. Every item must pass.

- [ ] Description = trigger conditions only, max 250 chars, no instructions
- [ ] Body instructions = imperative 2nd person ("You..."), never infinitive
- [ ] "Nothing to do" case handled explicitly
- [ ] Quick Start with 3-5 commands covering 80% of cases
- [ ] `allowed-tools` = most restrictive whitelist possible
- [ ] SKILL.md under 500 lines (advanced content in references/)
- [ ] Commands grouped by use case with concrete examples
- [ ] 2-3 complete workflows (command sequences)
- [ ] XML tags for data boundaries and enforcement
- [ ] Analysis order specified when multiple data sources
- [ ] Scripts validate first (fail closed)
- [ ] Scripts stdlib-only (zero external dependencies)
- [ ] Zero hardcoded secrets (env vars only)
- [ ] Links to `references/` for advanced content (one level only)
- [ ] Dynamic injection without `$ARGUMENTS` or uncontrolled URLs
