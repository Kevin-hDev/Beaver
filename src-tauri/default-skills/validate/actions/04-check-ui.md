# 04 - Check UI

You verify changed visible or interactive behavior in a real running interface.

## Input

- Use the confirmed expected behavior, affected user path, and available local runtime or browser target.

## Output

- Return the user journey exercised, observed result, visual evidence when supported, a continuable hypothesis journal, repairs, and final UI facet result.

## Process

1. **Confirm applicability.** You run this facet when the work changes layout, interaction, navigation, visible state, accessibility behavior, or user-facing errors.
2. **Resolve runtime.** You use a user-supplied URL or discover an already running local target from repository configuration and safe local process evidence. You confirm that it responds. You do not start or restart a server unless the user requests it or the established project workflow explicitly authorizes it.
3. **Inspect safely.** You navigate only to the validated local or approved target and avoid submitting real secrets or production data.
4. **Exercise the path.** You test the changed happy path, relevant failure state, boundary state, and keyboard or accessibility behavior required by the project.
5. **Capture evidence.** You record the observed result and capture before-and-after visual evidence when the browser tool supports it.
6. **Open the journal.** You read [ui-hypothesis-journal.md](../references/ui-hypothesis-journal.md). You record three best candidate causes with confidence, evidence, status, and the check that can confirm or refute each one. You keep the journal in the conversation unless a journal file is explicitly requested.
7. **Repair by hypothesis.** You validate one cause at a time, apply the smallest in-scope repair only after evidence supports it, and record every attempt and observed result. You use at most three attempts per candidate in one repair batch.
8. **Continue batches.** When all three candidates are invalidated or their repair batches fail, you preserve the journal and add three fresh candidates that account for prior evidence. You continue numbered batches until the UI passes or a real blocker remains.
9. **Recheck.** You rerun the complete selected UI journey after the last repair and record the final attempt and evidence.

## Stop conditions

- You return incomplete when UI validation is required but no approved runtime is available.
- You stop on authentication, destructive submission, payment, production mutation, or sensitive-data entry.
- You stop only for a real safety, authorization, runtime, or evidence blocker. A three-attempt or three-hypothesis batch boundary alone never ends the validation.

## Test

- A pass names the exact exercised path and its observed result.
- Every UI repair is confirmed in the running interface, not only by static code inspection.
- A screenshot is not claimed when none was captured.
- Every candidate and attempt has a journal status and evidence, and later batches preserve earlier results.
