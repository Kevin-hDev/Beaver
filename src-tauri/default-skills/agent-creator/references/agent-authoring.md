# Agent authoring contract

## Canonical requirements

- You define one specialist and one responsibility.
- You write a third-person description that states what the agent does and when to use it.
- You write the body in English with `# Role` and `# Behavior`.
- You use short imperative sentences and one actionable idea per sentence.
- You add `# Inputs`, `# Outputs`, `# Guardrails`, `# Skills`, readiness gates, decision boundaries, or handoffs only when they change behavior.
- You assume the agent starts with no conversation history beyond its role and mission.

## Capability rules

- You grant read-only capabilities for inspection, research, comparison, and reporting.
- You grant write capabilities only when the role must create or change files.
- You name exact native skills only after confirming their source-qualified IDs.
- You never embed secrets, private configuration values, permission-mode details, host provenance, or unavailable tools.
- You bound externally supplied lists, retries, files, and outputs.

## Quality gate

You reject a draft when another agent could interpret its responsibility, stopping point, output, or forbidden decisions in two materially different ways.
