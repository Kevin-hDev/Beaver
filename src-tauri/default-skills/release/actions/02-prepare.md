# 02 - Prepare

You update the exact version and release-note artifacts without publishing.

## Input

- Accept the verified target, complete change range, artifact map, project rules or fallback, note overrides, and notes format.

## Output

- Return a release diff containing only required version and note artifacts plus the complete draft notes.

## Process

1. **Update versions.** Use the project's version mechanism and keep every declared source of truth consistent.
2. **Update derived artifacts.** Run only documented deterministic commands required to synchronize lockfiles or generated version metadata.
3. **Draft notes.** Classify verified changes by user impact and include breaking changes, migrations, security, and platform notes only when evidenced.
4. **Apply overrides.** Preserve supplied title or body overrides when they remain accurate and project-compliant.
5. **Update structured notes.** Preserve required languages, schemas, ordering, and prior history.
6. **Check completeness.** Compare prepared artifacts with the project checklist or SemVer fallback and complete change range.
7. **Inspect diff.** Exclude unrelated files, secrets, generated noise, and unsupported claims.

## Stop conditions

- Stop when a required artifact cannot be updated consistently or a generation command changes unrelated files.
- Never tag, commit, push, publish, deploy, or change provider state during preparation.
- Do not summarize a change whose user impact cannot be established from evidence.

## Test

- Confirm that every required version source contains the exact target and every notes format validates.
- Confirm that the diff contains only expected artifacts and preserves prior release history.
