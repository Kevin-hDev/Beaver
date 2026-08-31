---
name: security-check
description: Use for a fast recurring security checkup of the current change set (diff) while working. Triggers on: security check, security checkup, check my changes, quick security pass, is my diff safe, security routine, pre-commit security.
---

# Security Check

You run a fast, recurring security checkup on the CURRENT change set — the
diff, the modified files, the work in progress. This is the routine skill:
quick to run, easy to repeat. It is NOT a full-codebase audit and NOT a deep
focused review (those are `security-map`, the `security-*` focus skills).

<critical_constraints>
- Scope = the current changes. Never widen to the whole codebase unless the
  user explicitly asks for a broader pass.
- Every verdict is CONFORM (you verified), VIOLATION (you have `file:line`
  evidence), or UNDETERMINED (you could not verify — say why). Never declare
  conform what you did not actually check.
- Read-only on the working tree. You may run the project's existing test
  suites. You never modify code unless the user asks for the fix.
- No exploits, no payloads. You point at the weakness and its consequence.
</critical_constraints>

## Step 1 — Resolve the change set

1. Identify the current changes: uncommitted diff first; if clean, the commits
   of the current branch versus its base. Empty change set = say so and stop.
2. List the modified files and read the diff. Note the project's language and
   test runner from its manifests.

## Step 2 — The ten reflexes on the diff

Apply each reflex from `references/ten-reflexes-checklist.md` to the changed
code. The checklist gives you, per reflex: what to search for, what a
violation looks like, what conform looks like. Record one verdict per reflex
(skip with a reason when the reflex does not apply to the diff — e.g. no
secrets touched).

While reading each modified function, keep these five questions in mind:

1. If an attacker controls this input, what happens?
2. If this operation fails, does it block or does it let through?
3. Is any secret compared with a plain equality operator?
4. Can this collection grow without bound?
5. Does this error message reveal internals (paths, stack, versions)?

## Step 3 — Fast automated scans

1. Run the project's dependency audit when a manifest or lockfile changed
   (`npm audit`, `pip audit`, `cargo audit` — pick the project's ecosystem).
   Skip with a note when dependencies were not touched.
2. Scan the changed files for hardcoded secrets: tokens, API keys, passwords,
   private keys, high-entropy strings. A test fixture with an obviously fake
   value is not a finding — say you classified it as fake.

## Step 4 — Existing protections still hold

1. Look for security tests in the project (XSS batteries, sanitizer tests,
   permission tests, auth tests). Run the ones covering the touched areas.
2. A security test that turns red after the change is a VIOLATION with its
   output quoted — never reword it as a suspicion.

## Step 5 — Report

Answer in the chat, compact:

```
SECURITY CHECK — {date} — {N files changed}

Violations: {n} | Undetermined: {n} | Conform: {n}/{n} reflexes

### Violations
- [reflex] {what} — {file:line} — {consequence in one line}

### Undetermined
- [reflex] {what you could not verify and why}

### Scans
- Dependencies: {clean / findings / skipped — reason}
- Hardcoded secrets: {none / findings / fake fixtures classified}

### Security tests
- {suite}: {green / red with output / none exist for this area}

Verdict: SAFE TO CONTINUE | FIX BEFORE CONTINUING | NEEDS HUMAN REVIEW
```

## Rules

- Speed matters: if a reflex cannot be verified quickly, mark UNDETERMINED
  instead of digging for twenty minutes. Depth belongs to the focus skills.
- One violation is enough to say FIX BEFORE CONTINUING. Never soften a
  violation into a suggestion.
- When the same violation repeats across files, report it once with all its
  `file:line` instances.
- End with the natural follow-up when relevant: which `security-*` focus
  skill should take over on the area that raised findings.
