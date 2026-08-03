# 02 - Research

You build a complete evidence ledger for every material documentation claim.

## Input

- Use the confirmed documentation contract and resolved project root.
- Use relevant code, tests, schemas, configuration, interfaces, current docs, and official external sources.

## Output

- Return an evidence ledger, stale or missing documentation findings, command and example inputs, and a file-level update map.

## Process

1. **Read the evidence contract.** You read [evidence-contract.md](../references/evidence-contract.md) before collecting claims.
2. **Trace the subject.** You follow accepted specifications and public contracts, entry points, public interfaces, configuration, data flow, error states, tests, and user-visible outcomes needed by the chosen document type.
3. **Reconcile contracts and behavior.** You treat accepted specifications and declared public contracts as the intended current contract and executable evidence as implementation proof. You stop on a conflict rather than documenting a regression as the new contract or describing an unimplemented contract as working behavior.
4. **Compare existing prose.** You classify each relevant section as accurate, stale, incomplete, duplicated, generated, or unsupported. You cite direct project evidence for every stale or unsupported finding.
5. **Collect commands and examples.** You resolve prerequisites, trusted executable identity, working directory, inputs, expected output shape, failure behavior, network need, platform differences, cleanup requirements, and safe resource limits.
6. **Verify external facts.** You use current official primary sources for unstable third-party versions, flags, support, limits, or platform behavior. You record access date and uncertainty.
7. **Protect sensitive data.** You replace real credentials, identifiers, endpoints, personal data, and machine paths with clearly marked safe examples.
8. **Cover the full scope.** You inspect at most 100 paths, 100 claims, or 30 examples per numbered batch, keep a stable cursor, and continue until every contracted subject is resolved or explicitly blocked.
9. **Map the update.** You name the smallest documentation files and navigation entries that must change. You separate product mismatches from documentation defects. When every contracted document is already accurate, you record a complete no-change ledger and continue directly to read-only validation.

## Stop conditions

- You stop when required evidence is unreadable, contradictory, or insufficient to distinguish accepted current contracts, implemented behavior, and planned behavior.
- You stop and report a product mismatch when documented behavior would require a code, schema, API, or architecture change.
- You do not invent behavior, infer secrets, modify files, or broaden the requested subject silently.

## Test

- Confirm that every material proposed claim maps to direct evidence or an explicit unresolved state.
- Confirm that commands and examples include prerequisites, inputs, outcomes, and safe execution constraints.
- Confirm that the update map distinguishes documentation work from product work.
- Confirm that an already-accurate scope takes the explicit no-change path instead of producing cosmetic edits.
