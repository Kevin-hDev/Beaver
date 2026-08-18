import assert from "node:assert/strict";
import { join, resolve } from "node:path";
import test from "node:test";
import {
  preparePackagedApp,
  windowsInstallerDirectory,
} from "./packaged-app.mjs";

test("Windows packaged smoke installs the single NSIS artifact into the isolated profile", async () => {
  const calls = [];
  const cargoTargetDir = resolve("target", "e2e");
  const profilePath = resolve("temp", "beaver-e2e-Ab12");
  const installer = join(windowsInstallerDirectory(cargoTargetDir), "Beaver_1.1.3_x64-setup.exe");
  const packaged = await preparePackagedApp({
    platform: "win32",
    cargoTargetDir,
    profilePath,
    listFiles: async () => [installer],
    isRegularFile: async () => true,
    run: async (command, args) => {
      calls.push([command, args]);
      return 0;
    },
  });

  const installDir = join(profilePath, "packaged-app");
  assert.equal(packaged.binaryPath, join(installDir, "cl-go-dash.exe"));
  assert.deepEqual(calls, [[installer, ["/S", `/D=${installDir}`]]]);

  await packaged.cleanup();
  assert.deepEqual(calls[1], [join(installDir, "uninstall.exe"), ["/S"]]);
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

test("macOS packaged smoke uses the binary inside the built app bundle", async () => {
  const cargoTargetDir = resolve("target", "e2e");
  const packaged = await preparePackagedApp({
    platform: "darwin",
    cargoTargetDir,
    profilePath: resolve("temp", "beaver-e2e-Ab12"),
    listFiles: async () => [],
    isRegularFile: async () => true,
    run: async () => 0,
  });

  assert.equal(
    packaged.binaryPath,
    join(
      cargoTargetDir,
      "debug", "bundle", "macos", "Beaver.app", "Contents", "MacOS", "cl-go-dash",
    ),
  );
  await packaged.cleanup();
});
