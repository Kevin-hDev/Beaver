# 01 - Resolve

You resolve one tracker destination and one issue boundary.

## Input

- Accept a problem or request, repository context, optional tracker project, issue type, labels, project, milestone, assignee, priority, references, attachments, and provider preference.

## Output

- Return tracker mechanism, project, type, template, required fields, valid metadata options, duplicate candidates, and issue boundary.

## Process

1. **Validate destination.** Use project configuration or one available authenticated connector, CLI, MCP capability, or API. Reject ambiguous or mismatched tracker URLs.
2. **Read rules.** Load applicable issue templates, contribution rules, field requirements, naming conventions, and provider limits.
3. **Classify type.** Select bug, feature, task, documentation, or another configured type only when the request or template supports it.
4. **Define one problem.** Separate unrelated symptoms, outcomes, or owners and keep one issue in scope.
5. **Search duplicates.** Query at most 50 open and recently closed candidates per pass using distinctive behavior, error, component, and title terms. Continue with narrower queries when required.
6. **Compare candidates.** Identify exact duplicates, related work, superseded issues, and distinct scope.
7. **Resolve metadata.** Validate requested labels, project, milestone, assignee, priority, references, and attachment support against current tracker options.

## Stop conditions

- Return an existing URL when an exact usable duplicate covers the request.
- Ask whether to continue when a close candidate overlaps materially but differs in scope.
- Stop when tracker mechanism, project, type, required template, or destination cannot be resolved safely.

## Test

- Confirm one configured destination and one issue boundary.
- Confirm bounded duplicate evidence and valid options for every requested metadata field.
