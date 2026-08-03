# 02 - Security

You find and fix supported vulnerabilities, prove each fix with regression coverage, and record any intentional behavior change.

## Input

- Accept an optional validated file, directory, or glob scope, defaulting to the current codebase.
- Accept optional pasted or readable security findings from an audit report.

## Output

- Return every supported vulnerability with its file, severity, evidence, fix, intentional behavior change, regression test, documentation update, and final verification state.
- Return every stale, deferred, blocked, or disproven finding with its reason.

## Process

1. **Resolve scope.** You validate the scope and read applicable project security and testing instructions. You protect unrelated changes and sensitive data.
2. **Build the fix list.** You use current audit findings for the security axis when supplied and skip broad discovery; otherwise you use available static analysis and the complete review in [security-checklist.md](../references/security-checklist.md). You confirm each candidate against a reachable trust boundary and concrete evidence before calling it a vulnerability.
3. **Trace inputs.** You follow untrusted data through HTTP, RPC or IPC, CLI, file, message, template, database, and deserialization boundaries. You check type, size, format, encoding, path confinement, and collection limits.
4. **Trace access.** You verify authentication, authorization, tenant or role propagation, least privilege, session handling, and fail-closed error paths at the operation that enforces access.
5. **Trace injection and secrets.** You inspect SQL, system commands, templates, HTML, URLs, redirects, outbound requests, parsing, cryptography, random identifiers, secret storage, comparisons, logs, and memory lifetime.
6. **Design the regression.** You create one focused unit or integration test per fix that demonstrates the unsafe pre-fix behavior and the intended protected behavior. You capture proof that the test fails against the pre-fix state before applying the fix.
7. **Apply minimally.** You prefer parameterized or structural safe APIs, allowlists, explicit authorization, bounded inputs, least privilege, safe secret handling, and closed failure paths over ad-hoc sanitization. You mark every intentionally rejected former behavior.
8. **Verify.** You run each regression test, existing focused tests, type checks, and the configured security linter or scanner on the changed scope. You review the diff for new data exposure, bypasses, fallback paths, and weakened checks.
9. **Document.** You update the project's established docstring, security note, architecture decision, or project-memory location so the protection and its reason survive later changes. You ask for a destination only when no project convention identifies one.

## Stop conditions

- You do not report a suspected vulnerability as confirmed without a reachable path and supporting evidence.
- You report `blocked` rather than exposing secrets, using production credentials, or probing a live external target outside the requested scope.
- You report `deferred` with reason and remaining risk when a supported finding cannot be fixed safely inside the selected scope.
- You report `incomplete` and never claim the vulnerability fixed when its regression test, required security check, or focused verification fails.

## Test

- Every supported finding has a matching fix or an explicit deferral with remaining risk.
- Every applied fix has a regression test proven to fail on the pre-fix state and pass on the final state.
- Existing focused tests, type checks, and configured security checks pass on the changed scope.
- Every intentional behavior change and durable security measure is documented.
