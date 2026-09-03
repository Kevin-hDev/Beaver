import assert from "node:assert/strict";
import { join, resolve } from "node:path";
import test from "node:test";
import {
  isWindowsInstallerName,
  preparePackagedApp,
  windowsInstallerDirectory,
} from "./packaged-app.mjs";

test("macOS packaged smoke resolves and verifies the app bundle executable", async () => {
  const cargoTargetDir = resolve("target", "e2e");
  const expected = resolve(
    cargoTargetDir,
    "debug", "bundle", "macos", "Beaver.app", "Contents", "MacOS", "cl-go-dash",
  );
  const checked = [];
  const packaged = await preparePackagedApp({
    platform: "darwin",
    cargoTargetDir,
    profilePath: resolve("temp", "beaver-e2e-Ab12"),
    isRegularFile: async (path) => {
      checked.push(path);
      return path === expected;
    },
    run: async () => {
      throw new Error("macOS preparation must not run a command");
    },
  });

  assert.equal(packaged.binaryPath, expected);
  assert.deepEqual(checked, [expected]);
  await packaged.cleanup();
});

test("Linux packaged smoke uses the injected deterministic AppImage listing", async () => {
  const cargoTargetDir = resolve("target", "e2e");
  const directory = resolve(cargoTargetDir, "debug", "bundle", "appimage");
  const appImage = join(directory, "Beaver_1.1.3_amd64.AppImage");
  const listed = [];
  const packaged = await preparePackagedApp({
    platform: "linux",
    cargoTargetDir,
    profilePath: resolve("temp", "beaver-e2e-Ab12"),
    listLinuxFiles: async (path) => {
      listed.push(path);
      return [appImage];
    },
    isRegularFile: async (path) => path === appImage,
    run: async () => {
      throw new Error("Linux preparation must not run a command");
    },
  });

  assert.equal(packaged.binaryPath, appImage);
  assert.deepEqual(listed, [directory]);
  await packaged.cleanup();
});

test("Windows packaged smoke installs the single NSIS artifact into the isolated profile", async () => {
  const calls = [];
  const cargoTargetDir = resolve("target", "e2e");
  const profilePath = resolve("temp", "beaver-e2e-Ab12");
  const installer = join(
    windowsInstallerDirectory(cargoTargetDir),
    "Beaver_E2E_1.1.3_x64-setup.exe",
  );
  const packaged = await preparePackagedApp({
    platform: "win32",
    cargoTargetDir,
    profilePath,
    listFiles: async () => [installer],
    isRegularFile: async () => true,
    run: async (command, args, options) => {
      calls.push([command, args, options]);
      return 0;
    },
  });

  const installDir = join(profilePath, "packaged-app");
  assert.equal(packaged.binaryPath, join(installDir, "cl-go-dash.exe"));
  assert.deepEqual(calls, [[
    installer,
    ["/S", `/D=${installDir}`],
    { windowsVerbatimArguments: true },
  ]]);

  await packaged.cleanup();
  assert.deepEqual(calls[1], [
    join(installDir, "uninstall.exe"),
    ["/S", `_?=${installDir}`],
    { windowsVerbatimArguments: true },
  ]);
});

test("Windows NSIS keeps an isolated install path containing spaces verbatim", async () => {
  const calls = [];
  const cargoTargetDir = resolve("target", "e2e");
  const profilePath = resolve("temp root", "beaver-e2e-Ab12");
  const installer = join(
    windowsInstallerDirectory(cargoTargetDir),
    "Beaver_E2E_1.1.3_x64-setup.exe",
  );
  await preparePackagedApp({
    platform: "win32",
    cargoTargetDir,
    profilePath,
    listFiles: async () => [installer],
    isRegularFile: async () => true,
    run: async (command, args, options) => {
      calls.push([command, args, options]);
      return 0;
    },
  });

  assert.deepEqual(calls[0], [
    installer,
    ["/S", `/D=${join(profilePath, "packaged-app")}`],
    { windowsVerbatimArguments: true },
  ]);
});

test("Windows packaged smoke accepts both Tauri separators for Beaver E2E", async () => {
  for (const name of [
    "Beaver_E2E_1.1.3_x64-setup.exe",
    "Beaver E2E_1.1.3_x64-setup.exe",
  ]) {
    assert.equal(isWindowsInstallerName(name), true);
  }
  assert.equal(isWindowsInstallerName("Beaver_1.1.3_x64-setup.exe"), false);
});

test("packaged smoke fails closed when the NSIS artifact is ambiguous", async () => {
  await assert.rejects(
    preparePackagedApp({
      platform: "win32",
      cargoTargetDir: resolve("target", "e2e"),
      profilePath: resolve("temp", "beaver-e2e-Ab12"),
      listFiles: async () => ["one.exe", "two.exe"],
      isRegularFile: async () => true,
      run: async () => 0,
    }),
    /Packaged E2E preparation failed/u,
  );
});

test("packaged smoke removes a partial Windows installation before failing", async () => {
  const calls = [];
  const cargoTargetDir = resolve("target", "e2e");
  const profilePath = resolve("temp", "beaver-e2e-Ab12");
  const installDir = join(profilePath, "packaged-app");
  const installer = join(
    windowsInstallerDirectory(cargoTargetDir),
    "Beaver_E2E_1.1.3_x64-setup.exe",
  );

  await assert.rejects(
    preparePackagedApp({
      platform: "win32",
      cargoTargetDir,
      profilePath,
      listFiles: async () => [installer],
      isRegularFile: async (path) => path !== join(installDir, "cl-go-dash.exe"),
      run: async (command, args, options) => {
        calls.push([command, args, options]);
        return 0;
      },
    }),
    /Packaged E2E preparation failed/u,
  );
  assert.deepEqual(calls[1], [
    join(installDir, "uninstall.exe"),
    ["/S", `_?=${installDir}`],
    { windowsVerbatimArguments: true },
  ]);
});

test("packaged smoke normalizes an unreadable NSIS directory to one error", async () => {
  await assert.rejects(
    preparePackagedApp({
      platform: "win32",
      cargoTargetDir: resolve("target", "e2e"),
      profilePath: resolve("temp", "beaver-e2e-Ab12"),
      listFiles: async () => {
        throw new Error("ENOENT: internal runner path");
      },
      isRegularFile: async () => false,
      run: async () => 0,
    }),
    (error) => (
      error instanceof Error
      && error.message === "Packaged E2E preparation failed"
    ),
  );
});
