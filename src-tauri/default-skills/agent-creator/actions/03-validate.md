# 03 - Validate

You prove that every generated agent is focused, valid, and loadable by its claimed runtime.

## Input

- Accept the canonical contract and the complete list of written or supplied agent files.

## Output

- Return one `passed`, `failed`, `blocked`, or `skipped` verdict per target, with checks and remaining risks.

## Process

1. **Check existence.** You confirm each expected file exists at the confirmed destination and no extra target was written.
2. **Parse format.** You parse frontmatter, Markdown, TOML, or target configuration with a compatible parser and reject unknown required fields or duplicate keys.
3. **Compare semantics.** You compare every rendering with the canonical role and flag missing responsibility, behavior, guardrail, output, tool, or skill semantics.
4. **Check focus.** You verify one responsibility, imperative behavior, bounded inputs, observable output, and no hidden orchestration or unrelated work.
5. **Check native loading.** You verify the native path is project-relative, the profile is `explorer` or `coder`, and `delegate_task` can reference the file with the same profile. You do not launch the agent unless the user requests a smoke test.
6. **Check external loading.** You run an available non-mutating validator for each confirmed runtime. You mark unavailable validators `blocked` or `skipped`, never `passed`.
7. **Report.** You distinguish structural validity, semantic parity, and observed runtime loading.

## Stop conditions

- Stop with `failed` when a file does not parse, escapes the project, widens capabilities, or loses a confirmed instruction.
- Stop with `blocked` when the target runtime or validator is unavailable and no equivalent evidence exists.

## Test

- Verify that every claimed target has a separate evidence-backed verdict.
- Verify that no runtime was called compatible solely because its file exists.
