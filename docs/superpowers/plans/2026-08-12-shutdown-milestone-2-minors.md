# Shutdown Milestone 2 Minor Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fermer les 35 obligations mineures validées après la fusion du jalon 2, avec une preuve testable et une git note fidèle pour chaque lot.

**Architecture:** Chaque ressource asynchrone dépend d'un propriétaire unique qui fournit admission, annulation, échéance et moisson. Les décisions scheduler/gateway restent observables même si le stockage optionnel échoue, et les secrets ainsi que les exécutables système passent par leurs autorités communes. Les modifications sont livrées en lots indépendants afin que chaque commit puisse être relu et rejeté séparément.

**Tech Stack:** Rust stable, Tokio, Tauri 2, `subtle`, `zeroize`, React 19, TypeScript, Vitest.

## Global Constraints

- Travailler uniquement dans `codex/shutdown-milestone-2-minors`, jamais directement dans `main`.
- Ne modifier aucun des sept éléments classés **Infos consignées** dans la conception validée.
- Écrire le test avant le correctif, observer l'échec attendu, puis observer le succès.
- Réutiliser les autorités existantes avant d'en créer une nouvelle.
- Tous les fichiers de code touchés restent sous 230 lignes ; les fichiers de tests sont exemptés.
- Toute collection alimentée de l'extérieur reste bornée.
- Les secrets sont comparés en temps constant, conservés dans `Zeroizing` et exclus des traces.
- Les erreurs visibles restent génériques et traduites dans les sept langues lorsqu'un texte est ajouté.
- Chaque commit reçoit une git note : raison, décision, tests réellement exécutés et résultat.

---

### Task 1: Comparaisons et identités de processus sûres

**Obligations:** 1, 2

**Files:**
- Modify: `src-tauri/src/services/terminal/mod.rs`
- Modify: `src-tauri/src/services/terminal/tests.rs`
- Modify: `src-tauri/src/services/process_tree.rs`
- Modify: `src-tauri/src/services/process_tree_unix.rs`
- Modify: `src-tauri/src/services/process_tree_unix_tests.rs`

**Interfaces:**
- Preserve: `fn verify_token(expected: &str, provided: &str) -> Result<(), String>`
- Produce: `UnixProcessIdentity { pid: sysinfo::Pid, start_time: u64 }`
- Produce: `fn collect_children(pid: u32) -> Vec<UnixProcessIdentity>`
- Produce: `fn is_current(identity: UnixProcessIdentity) -> bool`

- [ ] **Step 1: Add the process identity regression test**

```rust
#[test]
fn stale_descendant_identity_is_rejected_before_signal() {
    let identity = UnixProcessIdentity::from_parts(Pid::from_u32(42), 100);
    assert!(!identity.matches(Pid::from_u32(42), 101));
    assert!(identity.matches(Pid::from_u32(42), 100));
}
```

- [ ] **Step 2: Run RED**

Run: `cargo test --quiet --features windows-tests stale_descendant_identity_is_rejected_before_signal -- --test-threads=1`

Expected: FAIL because `UnixProcessIdentity` and its comparison do not exist.

- [ ] **Step 3: Implement the minimal identity capture and primitive comparison**

Use `Process::start_time()` when collecting each descendant. Immediately before an individual `kill`, refresh that PID and require the same start time. Keep the process-group signal protected by the existing root identity check. Replace the terminal XOR fold with:

```rust
use subtle::ConstantTimeEq;

let matches: bool = expected.as_bytes().ct_eq(provided.as_bytes()).into();
matches.then_some(()).ok_or_else(|| "terminal-access-denied".to_string())
```

- [ ] **Step 4: Run GREEN and the terminal tests**

Run:

```powershell
cargo test --quiet --features windows-tests stale_descendant_identity_is_rejected_before_signal -- --test-threads=1
cargo test --quiet --features windows-tests terminal -- --test-threads=1
```

Expected: PASS, with stale identities rejected and terminal token behavior unchanged.

- [ ] **Step 5: Commit and attach a git note**

Commit: `fix(process): revalidate descendants before signaling`

---

### Task 2: Ownership des flux agent et tâches de fond

**Obligations:** 3, 4, 12, 13

**Files:**
- Modify: `src-tauri/src/services/mascot/mod.rs`
- Modify: `src-tauri/src/services/mascot/activity_tests.rs`
- Modify: `src-tauri/src/services/agent_local/eager_dispatch.rs`
- Create: `src-tauri/src/services/agent_local/eager_dispatch_tests.rs`
- Modify: `src-tauri/src/commands/agent_chat.rs`
- Modify: `src-tauri/src/commands/agent_chat_streams.rs`
- Create: `src-tauri/src/commands/agent_chat_streams_tests.rs`
- Modify: `src-tauri/src/runtime_startup.rs`
- Modify: `src-tauri/src/services/file_watcher.rs`
- Modify: `src-tauri/src/services/file_watcher_tests.rs`

**Interfaces:**
- Produce: mascot work spawned through its existing lifecycle owner.
- Produce: eager dispatch accepts the parent `ServiceWorkCancellation` and uses the owned task group.
- Produce: replacement drains/releases the previous admission before requesting the new one.
- Preserve: startup and watcher remain non-fatal, but refusal is logged as a bounded category.

- [ ] **Step 1: Add failing ownership tests**

Add real tests named:

```rust
async fn replacement_reuses_capacity_after_previous_stream_is_drained()
async fn eager_dispatch_stops_when_parent_work_is_cancelled()
async fn mascot_focus_work_is_drained_by_shutdown()
fn watcher_admission_refusal_emits_a_bounded_diagnostic()
```

Each test observes a real admission count, cancellation completion, or captured bounded log category; none asserts a mock invocation.

- [ ] **Step 2: Run RED**

Run each test filter with `cargo test --quiet --features windows-tests <name> -- --test-threads=1`.

Expected: replacement reaches capacity, detached work survives cancellation, or no diagnostic is produced.

- [ ] **Step 3: Implement ownership changes**

Move the agent admission after `replace_active_stream` has completed the old drain. Pass the parent cancellation into eager dispatch and spawn through the same work owner. Replace the mascot's naked Tauri spawn with its lifecycle service. Replace `let _ = admission...` with explicit `match` that records only stable categories such as `closing` or `capacity`.

- [ ] **Step 4: Run GREEN**

Run the four targeted tests and the complete `agent_chat`, `mascot`, `file_watcher` filters.

- [ ] **Step 5: Commit and note**

Commit: `fix(work): keep background tasks under their owners`

---

### Task 3: Cycle de vie complet des extensions

**Obligations:** 5, 6, 27, 28

**Files:**
- Modify: `src-tauri/src/services/extensions/host_process.rs`
- Modify: `src-tauri/src/services/extensions/installer.rs`
- Modify: `src-tauri/src/services/extensions/host_stop_boundary.rs`
- Modify: `src-tauri/src/services/extensions/runtime_lifecycle_tests.rs`
- Create: `src-tauri/src/services/extensions/installer_tests.rs`

**Interfaces:**
- Change: `HostProcess::spawn` consumes a reader admission obtained before `OwnedProcess::spawn_tokio`.
- Change: uninstall observes `ServiceWorkCancellation` before each irreversible filesystem stage.
- Release build contains neither `UninstallBoundary` nor `fail_if_requested`; failure injection lives in test helpers.

- [ ] **Step 1: Add failing real lifecycle tests**

```rust
async fn closed_reader_admission_prevents_node_process_creation()
async fn cancelled_uninstall_keeps_extension_files_and_does_not_restart_host()
async fn stop_and_wait_reaps_host_and_reader_in_order()
```

Use a real short-lived Node fixture and a temporary extension directory. Observe process existence and files, not mock calls.

- [ ] **Step 2: Run RED**

Expected: Node starts before the closed admission is noticed, or uninstall proceeds after cancellation.

- [ ] **Step 3: Implement minimal lifecycle ordering**

Acquire reader admission first, then spawn/publish the child. Thread cancellation through uninstall. Move test failure injection behind `#[cfg(test)]` into the test module while keeping production functions free of test-only arguments.

- [ ] **Step 4: Run GREEN**

Run: `cargo test --quiet --features windows-tests extensions -- --test-threads=1`

- [ ] **Step 5: Commit and note**

Commit: `fix(extensions): close lifecycle admission windows`

---

### Task 4: Gateway fermé, observable et borné

**Obligations:** 10, 14, 16, 18, 19

**Files:**
- Modify: `src-tauri/src/services/gateway/channels/backpressure.rs`
- Modify: `src-tauri/src/services/gateway/channels/discord.rs`
- Modify: `src-tauri/src/services/gateway/channels/slack.rs`
- Modify: `src-tauri/src/services/gateway/refusal_audit.rs`
- Modify: `src-tauri/src/services/gateway/service_audit.rs`
- Create: `src-tauri/src/services/gateway/reconnect_policy.rs`
- Modify: gateway channel and audit tests.

**Interfaces:**
- Add: `#[must_use] enum EnqueueOutcome { Enqueued, Full, Closed }`
- Produce: channel loops break on `Closed`.
- Produce: saturating bounded counters remain observable with audit disabled.
- Produce: `ReconnectPolicy` with capped exponential delay and reset after a stable connection.
- Produce: Discord payload token held in `Zeroizing<String>` until the websocket buffer is dropped.

- [ ] **Step 1: Add failing behavior tests**

```rust
async fn closed_queue_stops_the_network_consumer()
fn disabled_audit_still_counts_dropped_messages()
fn reconnect_delay_grows_and_stops_at_the_shared_cap()
fn sanitizer_keeps_only_the_stable_reason_category()
```

- [ ] **Step 2: Run RED**

Expected: `Closed` is ignored, disabled audit has no count, delays remain fixed, or every reason becomes `operation_failed`.

- [ ] **Step 3: Implement the shared policy and secure payload ownership**

Return and match every `EnqueueOutcome`. Increment a saturating counter before optional audit. Map known reasons to a closed bounded enum. Use one reconnect policy in Discord and Slack. Build the websocket authentication payload directly in a zeroizing buffer and explicitly zeroize the owned buffer after send completion.

- [ ] **Step 4: Run GREEN**

Run: `cargo test --quiet --features windows-tests gateway -- --test-threads=1`

- [ ] **Step 5: Commit and note**

Commit: `fix(gateway): stop closed consumers and preserve bounded diagnostics`

---

### Task 5: Décisions durables et journal scheduler incrémental

**Obligations:** 15, 17, 20, 21

**Files:**
- Modify: `src-tauri/src/services/scheduler/runtime.rs`
- Modify: `src-tauri/src/services/scheduler/due.rs`
- Modify: `src-tauri/src/services/scheduler/log.rs`
- Modify: `src-tauri/src/services/scheduler/log_store.rs`
- Modify: `src-tauri/src/services/scheduler/runtime_tests.rs`
- Modify: `src-tauri/src/services/scheduler/log_tests.rs`
- Modify: frontend wakeup history component tests.
- Modify: `CHANGELOG.md` only if the compatibility decision is user-visible in the next release section.

**Interfaces:**
- Produce: every due occurrence ends as executed, refused, cancelled, inactive, or missed with a persisted cursor/decision.
- Produce: append tracks entry count/offset incrementally and performs one bounded rewrite only at the trim boundary.
- Preserve: legacy entries without `error_code` display a generic translated failure, never raw `error` text.

- [ ] **Step 1: Add failing durability tests**

```rust
async fn failed_refusal_write_keeps_occurrence_reconcilable()
fn restart_grace_records_each_due_occurrence_as_missed()
fn appends_do_not_reread_the_log_before_the_trim_boundary()
```

Add a real component test rendering a legacy history entry and asserting the generic translated failure.

- [ ] **Step 2: Run RED**

Expected: occurrence disappears, grace remains undecided, append reader count grows, or legacy detail is blank.

- [ ] **Step 3: Implement durable ordering and incremental metadata**

Persist a decision before advancing the cursor; if persistence fails, leave the cursor unchanged for reconciliation. Replace the five-minute silent grace with explicit missed decisions. Keep log metadata under the existing log lock and update it after atomic rotation.

- [ ] **Step 4: Run GREEN**

Run scheduler Rust tests and the targeted Vitest component test.

- [ ] **Step 5: Commit and note**

Commit: `fix(scheduler): persist a decision for every occurrence`

---

### Task 6: OAuth attendu par le travail réel

**Obligations:** 22, 23, 35 (partie Codex)

**Files:**
- Modify: `src-tauri/src/services/mcp_oauth/callback_server.rs`
- Modify: `src-tauri/src/services/mcp_oauth/callback_server_tests.rs`
- Create: `src-tauri/src/services/oauth_completion.rs`
- Modify: `src-tauri/src/services/codex_oauth/login.rs`
- Modify: `src-tauri/src/services/llm_oauth/login_registry.rs`
- Modify: corresponding OAuth tests.

**Interfaces:**
- Produce: callback MCP ignores a wrong state and continues until valid callback, cancellation, or deadline.
- Produce: `OAuthCompletion<T>` backed by oneshot/owned join completion, without 10 ms polling.

- [ ] **Step 1: Add failing tests**

```rust
async fn wrong_state_does_not_consume_the_mcp_callback_server()
async fn oauth_login_completes_from_owned_work_without_polling()
async fn codex_login_test_does_not_release_its_own_registry_entry()
```

- [ ] **Step 2: Run RED**, expecting premature callback exit or polling-dependent completion.

- [ ] **Step 3: Implement one completion authority**, route both OAuth registries through it, and remove the test-authored release.

- [ ] **Step 4: Run GREEN** with all `oauth` test filters.

- [ ] **Step 5: Commit and note**

Commit: `fix(oauth): await owned callback completion`

---

### Task 7: MCP stdio signalé et modulaire

**Obligations:** 24, 25, 27 (partie MCP)

**Files:**
- Modify: `src-tauri/src/services/mcp_bridge/stdio.rs`
- Create: `src-tauri/src/services/mcp_bridge/stdio_session.rs`
- Modify: `src-tauri/src/services/mcp_bridge/stdio_transport.rs`
- Modify: `src-tauri/src/services/mcp_bridge/stdio_integration_tests.rs`
- Modify: `src-tauri/src/services/mcp_bridge/process_pool_tests.rs`

**Interfaces:**
- Produce: initialization waits for the first protocol response, child exit, cancellation, or shared deadline; no fixed 500 ms sleep.
- Produce: normal Rust modules, no `include!` boundary.
- Prove: `McpProcessService::stop_and_wait` closes admission, terminates child, then drains work.

- [ ] **Step 1: Add a slow-ready real connector test** that responds after 650 ms and must still initialize, plus a complete stop sequence test.
- [ ] **Step 2: Run RED**, expecting the fixed sleep path to misclassify readiness or the sequence proof to be absent.
- [ ] **Step 3: Move session ownership into `stdio_session.rs`** and make the protocol reader publish readiness through a oneshot channel.
- [ ] **Step 4: Run GREEN** with all `mcp_bridge` tests.
- [ ] **Step 5: Commit and note**

Commit: `fix(mcp): wait for protocol readiness and drain shutdown`

---

### Task 8: Sidecars Forecast et exécutables système

**Obligations:** 7, 8, 9, 11

**Files:**
- Create: `src-tauri/src/services/system_executable.rs`
- Modify: `src-tauri/src/services/mod.rs`
- Modify: `src-tauri/src/services/forecast/sidecar.rs`
- Modify: `src-tauri/src/services/forecast/sidecar_process.rs`
- Modify: `src-tauri/src/services/forecast/sidecar_http.rs`
- Modify: `src-tauri/src/services/searxng/process.rs`
- Modify: Forecast/SearXNG tests.

**Interfaces:**
- Produce: `#[cfg(windows)] fn powershell() -> Result<PathBuf, SystemExecutableError>` from validated absolute `SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe`.
- Produce: spawned Forecast child uses `kill_on_drop(true)` before any await and is published without an unowned window.
- Produce: one identity check before `process_tree::kill`.
- Preserve: auth token remains `Zeroizing<String>` through request creation.

- [ ] **Step 1: Add failing tests** for poisoned `PATH`, sidecar future cancellation before publication, a single identity probe, and token zeroization ownership.
- [ ] **Step 2: Run RED**, expecting PATH resolution or a surviving child.
- [ ] **Step 3: Implement the shared absolute resolver and reorder sidecar publication.** Remove the redundant identity query and pass the zeroizing token by borrowed header value without an ordinary owned copy.
- [ ] **Step 4: Run GREEN** with Forecast and SearXNG process tests.
- [ ] **Step 5: Commit and note**

Commit: `fix(sidecars): close spawn windows and trust system executables`

---

### Task 9: GPU non bloquant et budgets cohérents

**Obligations:** 29, 30, 31

**Files:**
- Modify: `src-tauri/src/services/gpu_vram/linux.rs`
- Modify: `src-tauri/src/services/gpu_vram/owned_probe.rs`
- Modify: `src-tauri/src/services/gpu_vram/owned_probe_tests.rs`
- Modify: `src-tauri/src/services/forecast/sidecar_reuse.rs`
- Modify: `src-tauri/src/services/forecast/sidecar_lifecycle_tests.rs`

**Interfaces:**
- Produce: sysfs reads run inside the owned probe's blocking boundary.
- Produce: reader join is bounded by the same absolute probe deadline after cancellation.
- Produce: reuse deadline is at least the four-second maximum health probe budget and derives from one constant.

- [ ] **Step 1: Add failing timing tests** with a blocked reader and a health probe completing between three and four seconds.
- [ ] **Step 2: Run RED**, expecting an unbounded join or premature reuse failure.
- [ ] **Step 3: Apply deadline-derived waits** and move Linux filesystem work into `spawn_blocking` owned by the probe task.
- [ ] **Step 4: Run GREEN** with `gpu_vram` and sidecar reuse tests.
- [ ] **Step 5: Commit and note**

Commit: `fix(gpu): bound blocking probes by one deadline`

---

### Task 10: Cycle de vie SearXNG à responsabilités séparées

**Obligations:** 32

**Files:**
- Modify: `src-tauri/src/services/searxng/lifecycle.rs`
- Create: `src-tauri/src/services/searxng/start_lifecycle.rs`
- Create: `src-tauri/src/services/searxng/stop_lifecycle.rs`
- Modify: `src-tauri/src/services/searxng/lifecycle_tests.rs`

**Interfaces:**
- Produce: `save_pid` executes after releasing the process mutex.
- Preserve: generation check makes a stale start unable to publish over a newer process.
- Each production file remains below 230 lines and owns one start/stop responsibility.

- [ ] **Step 1: Add a failing concurrency test** that blocks PID persistence and proves a concurrent status/stop operation can still acquire the process lock.
- [ ] **Step 2: Run RED**, expecting the concurrent operation to remain blocked.
- [ ] **Step 3: Extract start and stop lifecycles** and move persistence outside the critical section while retaining generation revalidation.
- [ ] **Step 4: Run GREEN** with all SearXNG lifecycle tests.
- [ ] **Step 5: Commit and note**

Commit: `refactor(searxng): separate start and stop lifecycle ownership`

---

### Task 11: Tests qui exercent la production réelle

**Obligations:** 26, 35 (partie sous-agents)

**Files:**
- Modify: `src-tauri/src/services/model_downloads_store_queue.rs`
- Modify: model download queue tests.
- Modify: `src-tauri/src/services/agent_local/subagent_registry.rs`
- Modify: subagent registry tests.

**Interfaces:**
- Production queue owns the only transition `worker_running: true -> false` and immediately activates the next item.
- Tests use the exact production capacity constant and path, without `cfg(test)` substitution.

- [ ] **Step 1: Add mutation-resistant tests** proving two queued downloads advance through the production worker completion and the production subagent limit rejects exactly the next real admission.
- [ ] **Step 2: Run RED**, expecting the tests to require test-only helpers/constants.
- [ ] **Step 3: Move state transition into the production completion method** and remove `cfg(test)` replacements.
- [ ] **Step 4: Run GREEN** with model download and subagent registry filters.
- [ ] **Step 5: Commit and note**

Commit: `test(runtime): exercise production queue and registry contracts`

---

### Task 12: Traductions et rendu réel des erreurs de réveil

**Obligations:** 33, 34

**Files:**
- Modify: `src/i18n/ja.json`
- Modify: `src/lib/wakeup-run-error.test.ts`
- Modify: the wakeup history component test file.

**Interfaces:**
- Japanese `missed` describes a missed occurrence; `never` describes one that has never run.
- Translation test enumerates the exact six stable error keys.
- Component test renders a real `error_code` and asserts translated text, never the raw code.

- [ ] **Step 1: Add failing Vitest assertions**

```ts
expect(requiredKeys).toEqual([
  'cancelled', 'missed', 'providerUnavailable',
  'capacityReached', 'appClosing', 'operationFailed',
])
expect(screen.queryByText('capacity_reached')).not.toBeInTheDocument()
```

Also assert the Japanese values for `missed` and `never` differ.

- [ ] **Step 2: Run RED** with the exact Vitest files.
- [ ] **Step 3: Correct Japanese copy and render the real component path.**
- [ ] **Step 4: Run GREEN** with the exact Vitest files and `npx tsc --noEmit`.
- [ ] **Step 5: Commit and note**

Commit: `fix(i18n): distinguish wakeup history outcomes`

---

### Task 13: Vérification finale et registre de preuves

**Files:**
- Modify: this plan only to mark completed checkboxes if useful for review.
- Maintain: `graphify-out/` through the project command, without committing unrelated generated noise.

- [ ] **Step 1: Check every obligation** against Tasks 1–12 and record `corrigé`, `déjà conforme`, or `non applicable` with commit evidence in git notes.
- [ ] **Step 2: Run frontend verification**

```powershell
npm test
npx tsc --noEmit
npm run lint
```

- [ ] **Step 3: Run Rust verification**

```powershell
cd src-tauri
cargo check
cargo test --quiet --lib --features windows-tests -- --test-threads=1
cargo clippy --all-targets --features windows-tests -- -D warnings
```

- [ ] **Step 4: Verify structure and graph**

Run a line-count check for every changed production file, then `graphify update .` from the worktree root.

- [ ] **Step 5: Push without merging**

Push `codex/shutdown-milestone-2-minors`, wait for every PR check, and hand the commit range plus git notes to the reviewer. No merge into `main` occurs before explicit user approval.
