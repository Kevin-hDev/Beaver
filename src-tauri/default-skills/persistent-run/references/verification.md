# Completion verification

You prove completion against the recorded objective, not against effort or confidence.

## Command predicate

You record and verify:

- You record the executable and arguments separately.
- You record the validated working directory and controlled inputs.
- You record the expected exit status and any required output property.
- You bound execution time and captured output.
- You reject a pass caused by skipped tests, empty discovery, stale artifacts, ignored failures, or the wrong target.

## Observable predicate

You use a deterministic observation only when a command cannot represent the outcome. You record the observer, target identity, procedure, expected value, tolerance if relevant, and evidence artifact. You require another executor to reproduce the observation.

## Completion evidence

You retain the predicate text, UTC timestamp, state fingerprint, result code or value, bounded relevant output, and any evidence artifact path. You redact secrets. You set `completion: verified` only in the same evaluation that produced this evidence.
