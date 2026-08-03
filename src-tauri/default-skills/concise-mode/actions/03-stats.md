# 03 - Stats

You report reproducible concise-mode usage and response-volume statistics from the current conversation.

## Input

- Accept a statistics, usage, token-count, or savings request for concise mode.
- Use only assistant replies and valid state confirmations visible in the current conversation.
- Use an exact token counter or comparable baseline only when it is genuinely available and its scope is known.

## Output

- Return the current mode, counted assistant replies, active replies and ratio, replies per level, response volume by state, counting method, and savings availability.
- Return unavailable fields with a reason instead of a fabricated number.

## Process

1. You read [measurement.md](../references/measurement.md).
2. You scan the visible conversation in chronological order and build state segments from valid confirmation lines.
3. You count only assistant replies after each confirmation; you exclude the confirmation reply itself from active and inactive response-volume comparisons.
4. You report assistant-reply counts and active ratios as measured transcript values when the complete visible scope is known.
5. You use exact token totals only when a real token counter covers the same visible replies.
6. You otherwise report character and word counts when you can inspect the full text, and you may report `estimated tokens` only with the explicit approximation and rounding method.
7. You break active results down by `lite`, `full`, and `ultra` without claiming that the level caused observed length differences.
8. You calculate average saved per comparable reply, total saved, per-level saved totals, and the top-saving level only from paired or otherwise genuinely comparable baseline output measured with the same counter and scope.
9. You mark a per-level result unavailable when that level has no comparable pair instead of ranking incomplete evidence as though it were complete.
10. You return `Savings: unavailable (no comparable baseline).` when no comparable evidence exists.
11. You name omitted, truncated, or inaccessible transcript portions as a scope limitation.

## Stop conditions

- You stop rather than fill a numeric field when its source text, counter, state transition, or baseline is unavailable.
- You return a state-only report with unavailable usage fields when the visible conversation lacks the needed replies.
- You do not use published, assumed, or level-based compression percentages as observed savings.

## Test

- You verify that active reply counts follow the latest state at each reply and exclude state-confirmation replies.
- You verify that every number names its counting method and scope.
- You verify that missing token counters produce labeled text measurements or estimates, never exact-token or savings claims.
