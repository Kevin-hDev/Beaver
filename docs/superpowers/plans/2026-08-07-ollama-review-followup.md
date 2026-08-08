# Ollama Review Follow-up Implementation Plan

**Goal:** Correct the remaining Modelfile and unknown-context diagnostic defects from the review.

**Architecture:** Exchange semantic values across the frontend/backend boundary, preserve unsupported multiline parameter spans, and keep the existing zero sentinel for an unknown context window while displaying it explicitly as unknown.

**Tech Stack:** Rust, TypeScript, React, Vitest, Cargo.

## Global Constraints

- Keep provider routing, tool selection, model capabilities, and Ollama lifecycle unchanged.
- Preserve non-edited multiline parameter spans and original line endings.
- Keep parameter collections and values bounded.
- Add every behavior through a red-green regression test.

---

### Task 1: Safe semantic parameter values

**Files:**
- Modify: `src/components/ollama/modelfile-utils.ts`
- Modify: `src/components/ollama/modelfile-utils.test.ts`
- Modify: `src-tauri/src/services/agent_local/ollama_modelfile_parameters.rs`
- Modify: `src-tauri/src/services/agent_local/ollama_modelfile_parameters_tests.rs`

- [x] Add tests proving `"`, `""`, embedded quotes, and backslashes are preserved as data.
- [x] Add tests proving single-line quoted values are decoded before entering editor state.
- [x] Run the tests and confirm failures caused by the current syntax/value ambiguity.
- [x] Remove backend quote stripping and render quote-containing text with triple delimiters.
- [x] Re-run the focused tests.

### Task 2: Multiline parameter preservation

**Files:**
- Modify: `src/components/ollama/modelfile-utils.ts`
- Modify: `src/components/ollama/modelfile-utils.test.ts`
- Modify: `src-tauri/src/services/agent_local/ollama_modelfile_parameters.rs`
- Modify: `src-tauri/src/services/agent_local/ollama_modelfile_parameters_tests.rs`

- [x] Add tests with triple-quoted and simple-quoted multiline parameters followed by an old single-line parameter.
- [x] Confirm the backend leaves residue and the frontend exposes an opening delimiter.
- [x] Add a bounded quoted-block scanner that preserves complete multiline parameter spans and omits them from editing.
- [x] Confirm the multiline span survives byte-for-byte and the following old parameter is removed.

### Task 3: Unknown context capacity

**Files:**
- Modify: `src-tauri/src/services/agent_local/context_capacity_error.rs`
- Modify: `src-tauri/src/services/agent_local/context_budget_tests.rs`
- Modify: `src/hooks/agent-context-capacity-error.ts`
- Modify: `src/hooks/__tests__/agent-chat-stream-callbacks-error.test.ts`
- Modify: `src/i18n/{fr,en,es,de,it,zh,ja}.json`

- [x] Add Rust and frontend tests for `contextWindow = 0` with valid known counters.
- [x] Confirm the structured details are currently discarded.
- [x] Keep the zero sentinel internal and add dedicated translated messages that call the window unknown.
- [x] Confirm known-window diagnostics remain unchanged.

### Task 4: Completion

- [x] Remove the unreachable newline-rendering branch.
- [x] Run formatting, TypeScript, lint, Rust compilation, Clippy, targeted tests, and broad suites.
- [x] Update Graphify and inspect the final diff for unrelated changes.
- [x] Prepare only the intended files and a reviewer-oriented Git note.
