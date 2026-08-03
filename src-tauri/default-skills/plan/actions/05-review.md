# 05 - Review and Approve

You review the complete candidate plan with the user, revise it from feedback, and stop only with explicit approval or a named blocker.

## Input

- Use the candidate plan from `04-compose`, its source, exploration evidence, and approved UI sketch when applicable.

## Output

- Return the complete approved plan and a calibrated confidence assessment that is not written into the plan artifact.

## Process

1. **Reconcile.** You compare the candidate with every confirmed requirement, projected file, risk, migration, acceptance check, and validation gate. You remove unsupported or unrelated work.
2. **Score confidence.** You report a score from 0 to 10, reasons that support it, and concrete risks that lower it. You calibrate the score to evidence: unresolved scope or architecture decisions prevent a high score.
3. **Show all content.** You show the complete plan and every phase across all numbered batches, not a summary or selected excerpt. For a bundle, you reconcile `plan.md` with every linked phase file.
4. **Request review.** You ask for approval or specific feedback and wait. You never infer approval from silence or from an earlier approval of the product source.
5. **Revise.** You apply the feedback to the candidate plan, recheck complete coverage and internal consistency, and show the full revised result with an updated confidence assessment.
6. **Repeat.** You continue the review and revision loop until the user explicitly approves the complete plan.
7. **Finalize.** After approval, you identify the approved result and any written destination. When an artifact exists, you atomically change only its plan status from candidate to approved after rechecking the complete bundle. You keep the confidence assessment in the conversation only.

## Stop conditions

- You stop and ask when feedback introduces a new product or architecture decision that the approved source does not settle.
- You do not self-approve, implement tasks, create branches, commit, or overwrite unrelated content.
- You keep each revision inside the originally validated destination when a plan file was requested.

## Test

- The complete plan was shown after every revision.
- Every phase remains present exactly once after every revision, and every bundle index link resolves.
- The confidence assessment names both supporting evidence and remaining risks and is absent from the plan artifact.
- The final state contains explicit user approval, or the result remains a candidate plan.
