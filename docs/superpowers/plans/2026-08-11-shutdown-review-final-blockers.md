# Shutdown Review Final Blockers Implementation Plan

> **For Codex:** Execute each task in order with test-driven development. Observe the targeted test fail before changing production code, then rerun it green before committing.

**Goal:** Make the restart button survive coordinated cleanup, close the Windows CEF expiration/re-reservation race, and restore the release brand contract.

**Architecture:** Beaver uses an interceptable private exit sentinel only to enter the existing shutdown coordinator. Tauri's reserved restart code remains confined to the unique final action. Windows pending CEF launches release emergency/native resources before their central slot, preserving reverse acquisition order. Existing release scanners and monotonic-clock helpers remain authoritative; only their exact contracts/tests are updated.

**Tech Stack:** Rust, Tauri 2, Windows native IPC, Node.js test runner, GitHub Actions contract tests.

---

### Task 1: Keep the Tauri loop alive until the unique restart action

**Files:**
- Modify: `src-tauri/src/app_exit/coordinator_tests.rs`
- Modify: `src-tauri/src/app_exit.rs`
- Modify: `src-tauri/src/app_exit/request_flow.rs`

1. Add tests that inject the restart exit action, capture Beaver's sentinel, require it to differ from `tauri::RESTART_EXIT_CODE`, and map it to `ExitIntent::Restart`.
2. Add a test proving the final Tauri restart code reaches `ReadyToExit` without starting a second cleanup or replacing the stored intent.
3. Run the focused coordinator tests and record the expected failure against the current direct Tauri restart code.
4. Define one private Beaver restart sentinel, route `request_restart` through an injectable helper, and make `requested_intent` recognize only that sentinel.
5. Rerun focused coordinator and final-action tests green.
6. Commit as `fix: coordinate restart before tauri relaunch` and attach a Git note explaining why the private sentinel exists.

### Task 2: Release expired Windows CEF resources in reverse acquisition order

**Files:**
- Modify: `src-tauri/src/services/browser/cef_supervision/windows/tracker_tests.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/windows/tracker/test_api.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/windows/tracker_pending.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/windows/tracker_loop.rs`

1. Add a synchronized test that fills all 64 slots, pauses after the expired launch's emergency/native resources are destroyed but before its central reservation is released, and makes a competing reservation attempt.
2. Require the competing attempt to fail while the central slot is still held, then succeed after expiration completes, with `tracker.failure() == None`.
3. Run the focused Windows tracker test and record its expected compile/assertion failure before the new ordered-expiration API exists.
4. Give `WindowsPendingLaunch` sole ownership of expiration: destructure it, drop emergency registration, drop publication objects, then call `reservation.expire()`.
5. Route the tracker loop through that method and expose only the minimal test probe under `cfg(test)`.
6. Rerun the focused test and the full Windows-feature Rust suite green.
7. Commit as `fix: close windows cef expiration race` and attach a Git note documenting reverse acquisition order.

### Task 3: Restore the release contract and stabilize the clock proof

**Files:**
- Modify: `scripts/brand/brand-boundaries-contracts.mjs`
- Modify: `src-tauri/src/services/browser/cef_supervision/windows/clock_tests.rs`

1. Run `npm run test:brand-boundaries` and preserve the current red evidence: `cl-go-dash` 229 versus 211 and `cl_go_dash` 46 versus 32, with zero unknown references.
2. Update exactly those two expected internal counts; do not add exclusions or weaken the unknown-reference assertion.
3. Replace the 20 ms immediate clock assertion with a sufficiently future monotonic deadline, then prove it is future and becomes reached before an independent absolute limit.
4. Run the brand test green and repeat the focused Windows clock test enough times to exercise scheduler delays.
5. Commit as `test: refresh release and clock contracts` and attach a Git note listing the two audited count changes.

### Task 4: Verify the branch as shipped

**Files:**
- Update generated graph data through `graphify update .` only.

1. Run `graphify update .` and require a healthy result.
2. Run `npm test` and `npm run test:brand-boundaries`.
3. Run `cargo fmt --check`, `cargo check`, and `cargo clippy --all-targets -- -D warnings` with the configured Windows build environment.
4. Run the full Rust library suite with `--features windows-tests`; report expected ignored tests separately from failures.
5. Run `git diff --check`, confirm the worktree is clean, and verify all new Git notes with `git notes show`.
6. Report the exact commit hashes and CI status truthfully; do not push unless the user asks.
