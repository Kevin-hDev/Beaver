# Pull Request Content Fallback

Read this reference only when the repository has no usable pull or merge request template.

## Required sections

### Summary

- You explain why the change exists and its observable result in two or three bullets.

### Changes

- You group the main committed changes by behavior or responsibility.

### Verification

- You list exact checks and results actually observed.
- You write `Not run` with a reason for relevant checks that were not executed.

### Risks

- You state compatibility, migration, data, security, deployment, rollback, or user-interface risks that actually apply.
- You write `None identified` when the inspected evidence supports no specific risk.

## Conditional sections

- You include screenshots only for relevant visible changes and only when artifacts exist.
- You include breaking changes and migration instructions only when the committed range proves them.
- You include issue references only when supplied or verified in repository metadata.
- You include follow-up work only when it is deliberately outside this request and already evidenced.

## Title

- You follow repository style when one exists.
- Otherwise you use a concise imperative or outcome-oriented title, normally no longer than 72 characters.
