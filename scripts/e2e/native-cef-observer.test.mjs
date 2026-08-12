import assert from "node:assert/strict";
import test from "node:test";

import {
  assertOwnedCefHelpersSandboxed,
  hasOwnedCefHelper,
  hasOwnedProcess,
  parseLinuxProcessTable,
  parseMacProcessTable,
  parseWindowsProcessJson,
  runtimeRootForBinary,
  waitForOwnedCefHelper,
  waitForOwnedProcessesToExit,
  waitForProcessIdsToExit,
} from "./native-cef-observer.mjs";

const WINDOWS_ROOT = "C:\\build\\target\\e2e\\debug";
const MAC_ROOT = "/build/target/e2e/debug/bundle/macos/Beaver.app";

test("Windows process inventory is bounded and ignores malformed entries", () => {
  const input = JSON.stringify([
    {
      ProcessId: 42,
      ParentProcessId: 21,
      ExecutablePath: `${WINDOWS_ROOT}\\cl_go_dash_lib.exe`,
      CommandLine: `cl_go_dash_lib.exe --type=renderer`,
    },
    { ProcessId: -1, CommandLine: "invalid" },
  ]);

  assert.deepEqual(parseWindowsProcessJson(input), [{
    pid: 42,
    parentPid: 21,
    executable: `${WINDOWS_ROOT}\\cl_go_dash_lib.exe`,
    command: "cl_go_dash_lib.exe --type=renderer",
  }]);
});

test("Linux process inventory preserves bounded pid ancestry", () => {
  const input = "  52  41 WebKitWebProces /usr/lib/webkit2gtk/WebKitWebProcess --type=renderer\n";

  assert.deepEqual(parseLinuxProcessTable(input), [{
    pid: 52,
    parentPid: 41,
    executable: "WebKitWebProces",
    command: "/usr/lib/webkit2gtk/WebKitWebProcess --type=renderer",
  }]);
});

test("macOS process inventory preserves helper commands containing spaces", () => {
  const input = `  84  40 ${MAC_ROOT}/Contents/Frameworks/Beaver Helper.app/Contents/MacOS/Beaver Helper --type=gpu-process\n`;

  assert.deepEqual(parseMacProcessTable(input), [{
    pid: 84,
    parentPid: 40,
    executable: "",
    command: `${MAC_ROOT}/Contents/Frameworks/Beaver Helper.app/Contents/MacOS/Beaver Helper --type=gpu-process`,
  }]);
});

test("only a process contained by the build root counts as an owned CEF helper", () => {
  const owned = parseWindowsProcessJson(JSON.stringify({
    ProcessId: 42,
    ParentProcessId: 21,
    ExecutablePath: `${WINDOWS_ROOT}\\cl_go_dash_lib.exe`,
    CommandLine: "cl_go_dash_lib.exe --type=renderer",
  }));
  const external = [{
    pid: 99,
    parentPid: 1,
    executable: "C:\\other\\cl_go_dash_lib.exe",
    command: `C:\\other\\cl_go_dash_lib.exe --type=renderer ${WINDOWS_ROOT}`,
  }];

  assert.equal(hasOwnedCefHelper(owned, WINDOWS_ROOT, "win32"), true);
  assert.equal(hasOwnedCefHelper(external, WINDOWS_ROOT, "win32"), false);
  assert.equal(hasOwnedProcess(owned, WINDOWS_ROOT, "win32"), true);
});

test("an owned CEF helper carrying no-sandbox fails closed", () => {
  const secure = [{
    pid: 42,
    parentPid: 21,
    executable: `${WINDOWS_ROOT}\\cl_go_dash_lib.exe`,
    command: "cl_go_dash_lib.exe --type=renderer",
  }];
  const insecure = [{
    ...secure[0],
    command: "cl_go_dash_lib.exe --type=renderer --no-sandbox",
  }];

  assert.doesNotThrow(() => assertOwnedCefHelpersSandboxed(secure, WINDOWS_ROOT, "win32"));
  assert.throws(
    () => assertOwnedCefHelpersSandboxed(insecure, WINDOWS_ROOT, "win32"),
    /Native CEF observation failed/u,
  );
});

test("the observed runtime root follows the native build layout", () => {
  assert.equal(
    runtimeRootForBinary("win32", `${WINDOWS_ROOT}\\cl-go-dash.exe`),
    WINDOWS_ROOT,
  );
  assert.equal(
    runtimeRootForBinary("darwin", `${MAC_ROOT}/Contents/MacOS/cl-go-dash`),
    MAC_ROOT,
  );
  assert.throws(
    () => runtimeRootForBinary("darwin", "/tmp/cl-go-dash"),
    /Native CEF observation failed/u,
  );
});

test("helper and exit waits poll bounded process snapshots", async () => {
  const helper = [{
    pid: 42,
    parentPid: 21,
    executable: `${WINDOWS_ROOT}\\cl_go_dash_lib.exe`,
    command: "cl_go_dash_lib.exe --type=renderer",
  }];
  let helperPolls = 0;
  await waitForOwnedCefHelper({
    platform: "win32",
    root: WINDOWS_ROOT,
    timeoutMs: 50,
    pollMs: 1,
    listProcesses: () => (helperPolls++ === 0 ? [] : helper),
  });

  let exitPolls = 0;
  await waitForOwnedProcessesToExit({
    platform: "win32",
    root: WINDOWS_ROOT,
    timeoutMs: 50,
    pollMs: 1,
    listProcesses: () => (exitPolls++ === 0 ? helper : []),
  });
  assert.equal(helperPolls, 2);
  assert.equal(exitPolls, 2);
});

test("a missing real helper fails closed", async () => {
  await assert.rejects(
    waitForOwnedCefHelper({
      platform: "darwin",
      root: MAC_ROOT,
      timeoutMs: 2,
      pollMs: 1,
      listProcesses: () => [],
    }),
    /Native CEF observation failed/u,
  );
});

test("pid exit observation waits only for the bounded requested set", async () => {
  let polls = 0;
  await waitForProcessIdsToExit({
    platform: "linux",
    pids: [52, 53],
    timeoutMs: 50,
    pollMs: 1,
    listProcesses: () => (polls++ === 0
      ? [{ pid: 52, parentPid: 41, executable: "WebKitWebProces", command: "" }]
      : []),
  });
  assert.equal(polls, 2);
});
