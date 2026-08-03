# Synchronization Contract

Own exactly one bounded marked section in each chosen project instruction file.

## Markers

```markdown
<!-- project-memory:start -->
## Project Memory

Read the following project memory files when their subject is relevant.

### Files 1-100

- `relative/path/to/memory-file.md`
<!-- project-memory:end -->
```

Use numbered `### Files X-Y` subsections with at most 100 sorted references each. Keep all subsections inside the same marker pair and continue until every current Markdown memory file appears once.

## Update

- Insert the section at the end with the target file's existing newline style when no marker exists.
- Replace only bytes from the opening marker through the closing marker when one balanced pair exists.
- When multiple complete sections exist, keep the first section's position, replace it with the current section, and remove only the additional owned sections.
- Stop on nested, reversed, missing, or unmatched markers.
- Preserve all bytes outside owned sections and never add broad behavioral instructions.

## Verification

- Compare the sorted bank inventory with the sorted referenced paths.
- Require exact equality, no duplicate, one marker pair, and no stale reference.
- Hash text outside owned sections before and after; require equality.
- Snapshot staged paths before and after; require equality.
