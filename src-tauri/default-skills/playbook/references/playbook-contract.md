# Playbook contract

You apply this contract to every project playbook created or materially updated by the skill.

## File and opening

- You name the file `<kebab-case-slug>.md` inside the resolved project home.
- You open with one H1 title and one plain sentence stating the observable outcome. You do not add a metadata table or a `Goal:` label.
- You follow with `## Why`, `## Steps to <outcome>`, and an optional `## Verify`. You use an optional short conclusion only when it adds a useful next boundary.
- You keep links where they are used rather than adding a detached related-links section.

## Writing

- You keep one idea per sentence, remove filler, use project terminology, and lead each rationale with the practical benefit.
- You lead `## Why` with searchable terms, bold the key concepts, and keep one benefit-focused idea per line.
- You distinguish verified current behavior from options, assumptions, and user-only instructions.
- You state prerequisites, affected scope, irreversible effects, external effects, credentials, and rollback or recovery where relevant.

## Steps

- You write one action per `#### N) <emoji> <title>` heading and number steps continuously.
- You follow each heading with one sentence explaining the benefit and reason, then a numbered action list.
- You give every step one concrete, evidence-backed example: a safe command with actual or explicitly illustrative output, a valid configuration snippet, a real project path, or an official image or short media reference.
- You link a tool to its canonical official location, give its supported installation route when installation is in scope, and show its real invocation. You prefer a short reusable official example or official media when permitted and useful; otherwise you run a safe representative example and record its actual output.
- You use exact current syntax verified from a primary source or safe execution. You never invent commands, output, options, screenshots, versions, or behavior.
- You use a comparison table when preferring one option over another and a small Mermaid diagram when a structural or branching flow would otherwise be ambiguous.
- You group steps under `### 🟢 Beginner`, `### 🟡 Intermediate`, and `### 🔴 Expert` only when multiple levels genuinely help. You include only levels that contain steps.

## Verification

- You write observable checks that prove the promised outcome, including failure interpretation and recovery when useful.
- You never state that a check passed inside the reusable playbook. The `apply` action records actual run evidence.
- You omit `## Verify` only when every step already contains an equally clear observable check and a separate section adds no value.
