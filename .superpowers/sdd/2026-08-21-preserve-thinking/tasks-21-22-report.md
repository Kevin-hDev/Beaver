# Tasks 21–22 — Chat Completions continuity wiring

## Decision

The common Chat Completions constructor is the single outbound authority for
native continuity. It derives `UserContinuation` or `ToolContinuation` from
the last message for every provider call, then applies the opaque envelope to
the matching assistant message only.

DeepSeek keeps its authenticated replay target at admission even though its
user continuation is forbidden. This preserves the exact scope for a following
tool continuation, where its required contract can be enforced. A forbidden
user continuation never serializes reasoning.

## Closed behavior

- A required contract with a previous assistant missing, partial, compacted,
  wrong-model, wrong-scope, or wrong-mode state stops before the transport.
  The payload test records zero outbound requests, so no retry can discard the
  native state.
- `FixtureCandidate` remains debug-only and bypasses activation only; route,
  model, scope, mode, state, and forbidden policies still block it.
- Kimi uses the shared Chat path while preserving its route-specific target.
  Empty `reasoning_content` is emitted as an empty native field.
- Z.AI and Cerebras add `clear_thinking: false` only after their own approved
  contract has been applied. Gemini, Mistral, and OpenRouter keep their native
  structured payloads. OpenRouter disables fallbacks when `reasoning_details`
  are present.
- The generic `legacy_tool_loop_reasoning` forwarding has been removed from
  the cloud converter. Groq remains a non-replay target with no adapter or
  activation.

## Validation

- Payload tests cover Kimi, DeepSeek user/tool continuations, Z.AI, Cerebras,
  Gemini signatures, Mistral ordered chunks, OpenRouter encrypted details,
  empty fields, partial state, model/scope/mode mismatch, first requests,
  fixture-only activation, and Groq exclusion.
- No live provider call, fixture activation, or registry activation was made.
