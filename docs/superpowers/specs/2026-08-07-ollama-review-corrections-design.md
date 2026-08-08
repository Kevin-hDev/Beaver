# Ollama Review Corrections Design

## Goal

Resolve the four confirmed review defects without changing provider selection, model capabilities, or the overall Ollama parameter-update workflow.

## Chosen approach

1. Carry the real model context window separately from the retry capsule budget. Capacity diagnostics must always report the configured model window.
2. Centralize conversion of internal capacity errors into public errors. Any exact or encoded capacity error must expose only the stable public code, even when its counters are malformed.
3. Serialize text parameters with Ollama's Modelfile quoting rules rather than JSON escaping. Quotes and Windows backslashes must survive a save/reload cycle byte-for-byte.
4. Make the frontend parameter reader track triple-quoted blocks with the same rule as the backend so prompt text cannot become a setting.
5. Preserve every non-parameter Modelfile directive, including multiple `FROM` sources, adapters, projectors represented as additional sources, drafts, renderers, parsers, licenses, and future directives.

## Alternatives rejected

- Restoring the old `/api/create` payload would reintroduce self-inheritance and make cleared overrides reappear.
- Replacing the complete Modelfile parser is unnecessary for these bounded defects and would increase regression risk.
- Blocking multimodal models is not justified: current Ollama emits model, adapter, draft, and projector blob paths in the generated Modelfile, and the rewrite preserves them.

## Error handling

- Internal encoded counters never reach the frontend or failure journal.
- Valid counters remain available as structured diagnostic details.
- Invalid counters fall back to the public capacity code without details.
- Parameter values containing line breaks remain rejected by the existing backend validation.

## Verification

- Each confirmed defect receives a regression test that is observed failing before implementation.
- Rust tests cover the retry context window, malformed capacity errors, quotes/backslashes, and multi-source Modelfiles.
- Frontend tests cover `PARAMETER` text inside both `SYSTEM` and `TEMPLATE` blocks.
- Final checks cover formatting, lint, TypeScript, Rust compilation, Clippy, targeted tests, and the broad Ollama-related suites.
