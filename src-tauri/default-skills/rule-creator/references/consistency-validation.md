# Consistency Validation

You treat target-specific files as renderings of one confirmed canonical meaning.

## Semantic ledger

You extract these fields from the canonical rule and compare them with every target:

- You list each obligation and prohibition.
- You record its strength and any condition.
- You record its file scope and exceptions.
- You record examples that define valid or invalid behavior.
- You record links or project terms required for interpretation.

## Allowed differences

- You allow only confirmed destination syntax such as extensions, metadata names, glob representation, wrapper headings, or generated-file markers.
- You allow a broader scope representation only when the destination cannot express the exact scope, the broader reach is safe, and the user explicitly confirms it.
- You preserve every behavioral requirement even when you restructure formatting.

## Consistency failures

- You fail a target that drops an obligation, prohibition, condition, exception, or meaning-bearing example.
- You fail a target that weakens `must` to advice or broadens an exception.
- You fail a target that changes scope without confirmation.
- You fail the set when a required mirror is absent, stale, unverifiable, or contradictory.
- You report destinations that cannot represent the canonical rule as blocked instead of forcing a lossy rendering.

## Validation record

You report for each target:

| Field | Required evidence |
| --- | --- |
| Boundary | Resolved project-local path and regular-file status |
| Placement | Project convention and confirmed ownership |
| Format | Extension, metadata, scope syntax, and links |
| Content | Focused, imperative, enforceable canonical meaning |
| Preservation | Unrelated existing content retained |
| Equivalence | Complete semantic ledger match |
| Check | Project validator result or explicit unavailable status |

You issue an all-target pass only when every required target passes and the semantic ledger matches across the complete set.
