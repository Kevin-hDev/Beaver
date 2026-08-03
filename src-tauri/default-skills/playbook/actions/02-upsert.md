# 02 - Upsert a playbook

You create or update one reusable project playbook without overwriting ownership, evidence, or nearby procedures.

## Input

- Accept a playbook topic, its intended outcome, audience, scope, steps, verification needs, and optional destination.
- Use an existing project playbook when the user selects one by current list number, exact slug, title, or unambiguous topic.

## Output

- Return the created or updated project playbook path, the material changes, research sources, validation evidence, and `complete`, `partial`, or `blocked` status.

## Process

1. **Resolve ownership.** You read [locations.md](../references/locations.md), establish a safe project home, derive a kebab-case slug, and resolve any existing project or packaged match.
2. **Complete the authoring contract.** You establish the one-sentence outcome, audience, level, included and excluded scope, actionable steps, verification, recovery needs, and inline references. You ask one focused set of questions for any missing field that would change the artifact.
3. **Research first.** You run the complete [research action](03-research.md) for every new playbook or substantial update. You draft only from its verified results. You may skip fresh research only for a clearly mechanical correction that changes no material claim or procedure.
4. **Check overlap.** For a new slug, you run the list action and compare every near match in `| Existing playbook | Source | Shared scope | Overlap |`, using `none`, `partial`, or `high`. On `high`, you recommend updating the existing playbook and ask `update or create` before writing.
5. **Protect packaged examples.** When only a packaged match exists, you ask whether to copy and adapt it into the resolved project home. You never edit the packaged copy.
6. **Protect project content.** When a project file exists, you show the intended material changes and obtain explicit confirmation before overwriting it. You preserve still-correct examples, rationale, safety notes, and verification steps.
7. **Fill the contract.** You read [playbook-contract.md](../references/playbook-contract.md), scaffold from [playbook-template.md](../assets/playbook-template.md) when needed, fill every placeholder, and keep one action per step.
8. **Verify examples.** You test safe commands in an isolated fixture or documented non-destructive mode when practical. You cite official sources for current syntax and label any unexecuted example.
9. **Validate the artifact.** You confirm its title, opening description, outcome-specific steps heading, continuous numbering, concrete examples, safety boundaries, and observable checks. You do not maintain a separate index unless the existing project convention requires one.

## Stop conditions

- You stop and ask when destination, ownership, overlap, or overwrite intent is unresolved.
- You stop with `blocked` when required research cannot verify a material claim or the only proposed destination is unsafe.
- You do not edit packaged examples, personal or global playbooks, unrelated documentation, or implementation code.

## Test

- Confirm that every new or substantially changed playbook traces to verified research.
- Confirm that no placeholder remains and every step follows the playbook contract.
- Confirm that a high-overlap result receives an update-or-create decision before any write.
- Confirm that an existing project file or packaged example is never overwritten silently.
