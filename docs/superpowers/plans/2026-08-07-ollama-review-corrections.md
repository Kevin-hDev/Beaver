# Ollama Review Corrections Implementation Plan

**Goal:** Correct every confirmed review defect while preserving all surrounding Ollama behavior.

**Architecture:** Keep the existing request and Modelfile update paths. Separate diagnostic concepts at their source, centralize public error sanitization, match Ollama's own text quoting, and align frontend multiline scanning with the backend.

**Tech Stack:** Rust, TypeScript, React, Vitest, Cargo.

## Global Constraints

- Do not change tool selection, model capability detection, or provider routing.
- Preserve all non-`PARAMETER` Modelfile directives and original line endings.
- Never expose encoded internal diagnostics to the frontend or logs visible to users.
- Add a failing regression test before every production change.

---

### Task 1: Context capacity diagnostics

**Files:**
- Modify: `src-tauri/src/services/agent_local/context_budget_prune.rs`
- Modify: `src-tauri/src/services/agent_local/context_budget.rs`
- Modify: `src-tauri/src/services/agent_local/context_budget_tests.rs`
- Modify: `src-tauri/src/services/agent_local/context_capacity_error.rs`
- Modify: `src-tauri/src/commands/agent_chat.rs`

- [x] Add a retry-path test that requires a capacity error and asserts the real context window.
- [x] Add tests that reject raw encoded errors while retaining valid structured details.
- [x] Run the tests and verify the expected failures.
- [x] Add a separate `context_window` field to `PruneParams` and centralize public error conversion.
- [x] Re-run the diagnostic tests and verify they pass.

### Task 2: Ollama parameter round-trip safety

**Files:**
- Modify: `src-tauri/src/services/agent_local/ollama_modelfile_parameters.rs`
- Modify: `src-tauri/src/services/agent_local/ollama_modelfile_parameters_tests.rs`

- [x] Add failing tests for embedded quotes, Windows backslashes, and multiple model/projector source directives.
- [x] Run the tests and verify the expected failures.
- [x] Replace JSON string escaping with Ollama-compatible quoting while preserving every other directive.
- [x] Re-run the transformation tests and verify they pass.

### Task 3: Frontend multiline parsing

**Files:**
- Modify: `src/components/ollama/modelfile-utils.ts`
- Modify: `src/components/ollama/modelfile-utils.test.ts`

- [x] Add failing tests for `PARAMETER` text inside multiline `SYSTEM` and `TEMPLATE` blocks.
- [x] Run the tests and verify the expected failures.
- [x] Replace the global regular expression with a bounded line scanner that tracks triple-quoted blocks.
- [x] Re-run the frontend parser and parameter-editor tests.

### Task 4: Cross-cutting review and completion

**Files:**
- Review every changed file and all direct callers.
- Update the Graphify code graph.

- [x] Run formatting, lint, TypeScript, Rust compilation, Clippy, targeted tests, and broad Ollama-related tests.
- [x] Inspect the final diff for unrelated changes and files over 230 lines.
- [ ] Commit only the intended files.
- [ ] Attach a Git note covering causes, decisions, tests, and remaining environmental failures.
