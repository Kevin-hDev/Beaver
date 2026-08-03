---
name: project-bootstrap
description: Designs and validates a new project stack through a needs checklist, 2–3 audited candidates, user selection, and optional technical-vision documentation. Use for project starts or architecture choices. Not for scaffolding or existing-stack changes.
---

# Project Bootstrap

Design and validate a new project's technical direction without creating its implementation. Support SaaS, web, mobile, desktop, CLI, library, service, data, and internal-tool projects.

## Workflow

~~~mermaid
flowchart LR
    Needs["Gather complete needs"] --> Candidates["Propose 2–3 distinct candidates"]
    Candidates --> Audit["Audit candidates independently in parallel"]
    Audit --> Passed{"Any candidate viable?"}
    Passed -->|No| Rework["Revise needs or candidates"]
    Rework --> Candidates
    Passed -->|Yes| Pick["User selects explicitly"]
    Pick --> Choice{"Verdict?"}
    Choice -->|Warning| Mitigate["Require credible mitigation"]
    Mitigate --> Pick
    Choice -->|Broken| Pick
    Choice -->|Pass| Design["Design tree and validated diagram"]
    Design --> Deliver["Return or write documentation"]
~~~

## Actions

Read every action required by the current workflow step.

| Action | Use it when | Output |
| --- | --- | --- |
| [01-gather-needs](actions/01-gather-needs.md) | You receive a new-project or stack-selection request | A complete, user-confirmed 24-item checklist |
| [02-propose-candidates](actions/02-propose-candidates.md) | The needs checklist is confirmed | Two or three genuinely different evidence-backed candidates |
| [03-audit-candidates](actions/03-audit-candidates.md) | Candidate claims and costs have current evidence | Independent verdicts and an all-fail loop when necessary |
| [04-pick-and-design](actions/04-pick-and-design.md) | At least one candidate is viable | An explicit selection, mitigations, folder tree, and validated Mermaid diagram |
| [05-deliver-document](actions/05-deliver-document.md) | The user confirms the design | Conversation output or one approved Markdown technical-vision artifact |

## Rules

- Complete the full needs checklist before proposing candidates. Accept not applicable only with a project-specific reason.
- Propose two or three candidates per round. Make them differ materially in architecture, runtime, data model, deployment or distribution, or operational ownership.
- Verify unstable compatibility, support, version, licensing, and pricing claims against current official primary sources. Date every pricing estimate and expose assumptions.
- Audit every candidate through an isolated reviewer in the same parallel wave. Do not give reviewers a preferred winner or another reviewer's result.
- Keep the audit adversarial. Challenge preferences that conflict with scale, security, integration, budget, team, deployment, distribution, or performance needs.
- When every candidate fails, return to needs or candidate generation and continue in a new bounded round. Never advance a failed set.
- Let the user select by exact candidate name. Require credible mitigation for every warning and refuse a broken candidate.
- Treat a user-proposed hybrid as a new candidate. Verify and audit it before selection.
- Produce a conceptual folder tree and a Mermaid module diagram, then validate the diagram with an available parser or renderer before calling the design complete.
- Remain documentation-only. Never scaffold code, dependencies, directories, accounts, repositories, services, infrastructure, configuration, or credentials.
- Return the document in conversation by default. Write one Markdown file only when the user requests it or an existing project convention provides an accepted destination.
- Never impose a destination. Validate the chosen path, require an existing parent directory, preserve existing content, and ask before overwriting.
- Bound every scan, source collection, question set, repair attempt, and audit round. Continue later batches until the requested scope is complete or a true blocker occurs.

## Resources

- Copy [needs-checklist.md](assets/needs-checklist.md) during 01-gather-needs.
- Read [decision-heuristics.md](references/decision-heuristics.md) during 02-propose-candidates.
- Read [candidate-audit-rubric.md](references/candidate-audit-rubric.md) during 03-audit-candidates.
- Copy [technical-vision-template.md](assets/technical-vision-template.md) only when the user requests a complete technical-vision or INSTALL-style artifact.
