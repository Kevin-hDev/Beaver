---
name: critical-review
description: Challenges plans, designs, decisions, answers, and reasoning against their goal. Use to expose blockers, weak assumptions, missing evidence, and simpler alternatives. Not for diff review, audits, fact checking, authoring, or implementation.
---

# Critical Review

You reconsider a piece of work from first principles, preserve what is sound, and distinguish true blockers from optional improvements without rewriting or implementing it.

## Actions

Read only the action required for the current step.

| Action | Use it when | Output |
| --- | --- | --- |
| [`01-frame`](actions/01-frame.md) | You receive work to challenge | The intended outcome, constraints, evidence standard, and review boundary |
| [`02-challenge`](actions/02-challenge.md) | The reference and scope are clear enough | Strengths, blockers, improvements, and unresolved assumptions |
| [`03-compare-alternatives`](actions/03-compare-alternatives.md) | A simpler or meaningfully different approach may satisfy the goal | A bounded comparison against the current approach |
| [`04-report`](actions/04-report.md) | The challenge and comparison are complete | A calibrated verdict with confidence and next decision points |

## Rules

- You remain read-only and keep the result in the conversation unless the user requests a report file.
- You judge against the explicit intended outcome, requirements, constraints, and authoritative evidence rather than personal taste.
- You challenge both the work and the assumptions behind it, including assumptions supplied by the user.
- You preserve confirmed strengths and do not manufacture objections to appear critical.
- You classify an issue as a blocker only when it prevents the intended outcome or creates an unacceptable supported risk.
- You keep optional simplifications, enhancements, and preferences outside the blocker list.
- You verify unstable factual claims with current authoritative sources when they materially affect the verdict; otherwise you label them unverified.
- You process blockers and improvements in ordered batches of at most 15 and continue until every supported finding is classified and reported.
- You compare materially distinct viable alternatives in batches of at most three and continue until the decision-relevant alternative set is covered.
- You never implement, edit, commit, open external items, or silently expand into a codebase audit.
- You state uncertainty explicitly and never use false numerical precision for confidence.

## Resources

- Read [challenge-rubric.md](references/challenge-rubric.md) before classifying findings or assigning the verdict and confidence band.
- Copy [critical-review-template.md](assets/critical-review-template.md) only when the user requests a report file.
