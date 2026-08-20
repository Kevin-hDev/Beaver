import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtemp, readFile, realpath, rm, symlink, utimes, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  HELPERS,
  assertLaunchedProfile,
  assertProfile,
  contaminateRuntime,
  createFixture,
  runScript,
  runScriptResult,
} from "./macos-runtime-profile-fixture.mjs";

function plistValue(path, key) {
  const result = spawnSync(
    "/usr/bin/plutil",
    ["-extract", key, "raw", "-o", "-", path],
    { encoding: "utf8", shell: false },
  );
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}

function assertBundleVersion(path, expected) {
  assert.equal(plistValue(path, "CFBundleShortVersionString"), expected);
  assert.equal(plistValue(path, "CFBundleVersion"), expected);
}

async function cargoInvocationCount(cargoLog) {
  const contents = await readFile(cargoLog, "utf8");
  return contents === "" ? 0 : contents.trimEnd().split("\n").length;
}

test(
  "macOS Dev and E2E preparations keep and launch their own CEF helpers",
  { skip: process.platform !== "darwin" },
  async () => {
    const directory = await realpath(
      await mkdtemp(join(tmpdir(), "beaver-cef-runtime-profile-")),
    );
    try {
      const { environment, tauriDirectory } = await createFixture(directory);
      runScript(tauriDirectory, "scripts/prepare-cef.sh", environment);
      runScript(tauriDirectory, "scripts/prepare-cef.sh", {
        ...environment,
        CARGO_TARGET_DIR: join(tauriDirectory, "target", "e2e"),
        CLGO_CEF_CARGO_FEATURES: "e2e",
      });

      await assertProfile(tauriDirectory, "target", "dev");
      await assertProfile(tauriDirectory, join("target", "e2e"), "e2e");

      runScript(
        tauriDirectory,
        "scripts/run-cef-dev-app.sh",
        environment,
        ["target/debug/cl-go-dash"],
      );
      runScript(tauriDirectory, "scripts/run-cef-dev-app.sh", {
        ...environment,
        CARGO_TARGET_DIR: join(tauriDirectory, "target", "e2e"),
        CLGO_CEF_CARGO_FEATURES: "e2e",
      }, ["target/e2e/debug/cl-go-dash"]);

      await assertLaunchedProfile(tauriDirectory, "target", "dev");
      await assertLaunchedProfile(tauriDirectory, join("target", "e2e"), "e2e");
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test(
  "macOS Dev derives every bundle version from the Tauri configuration",
  { skip: process.platform !== "darwin" },
  async () => {
    const directory = await realpath(
      await mkdtemp(join(tmpdir(), "beaver-cef-bundle-version-")),
    );
    try {
      const { environment, tauriDirectory } = await createFixture(directory);
      runScript(tauriDirectory, "scripts/prepare-cef.sh", environment);
      runScript(
        tauriDirectory,
        "scripts/run-cef-dev-app.sh",
        environment,
        ["target/debug/cl-go-dash"],
      );

      const contents = join(
        tauriDirectory,
        "target",
        "cef-dev",
        "Beaver Dev.app",
        "Contents",
      );
      assertBundleVersion(join(contents, "Info.plist"), "9.8.7");
      for (const helper of HELPERS) {
        assertBundleVersion(
          join(contents, "Frameworks", `${helper}.app`, "Contents", "Info.plist"),
          "9.8.7",
        );
      }
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test(
  "macOS Dev refreshes cached helpers after a version change with preserved timestamps",
  { skip: process.platform !== "darwin" },
  async () => {
    const directory = await realpath(
      await mkdtemp(join(tmpdir(), "beaver-cef-version-refresh-")),
    );
    try {
      const { environment, tauriDirectory } = await createFixture(directory);
      runScript(tauriDirectory, "scripts/prepare-cef.sh", environment);

      const config = join(tauriDirectory, "tauri.conf.json");
      await writeFile(config, `${JSON.stringify({ version: "9.8.8" })}\n`, "utf8");
      const preserved = new Date(1_700_000_000_000);
      await utimes(config, preserved, preserved);
      runScript(
        tauriDirectory,
        "scripts/run-cef-dev-app.sh",
        environment,
        ["target/debug/cl-go-dash"],
      );

      const helpers = join(
        tauriDirectory,
        "target",
        "cef-dev",
        "Beaver Dev.app",
        "Contents",
        "Frameworks",
      );
      for (const helper of HELPERS) {
        assertBundleVersion(
          join(helpers, `${helper}.app`, "Contents", "Info.plist"),
          "9.8.8",
        );
      }
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test(
  "macOS Dev rebuilds a runtime with a symlinked bundle version marker",
  { skip: process.platform !== "darwin" },
  async () => {
    const directory = await realpath(await mkdtemp(join(tmpdir(), "beaver-cef-version-link-")));
    try {
      const { cargoLog, environment, tauriDirectory } = await createFixture(directory);
      runScript(tauriDirectory, "scripts/prepare-cef.sh", environment);
      const runtime = join(tauriDirectory, "target", "cef-runtime", "macos");
      const external = join(tauriDirectory, "external-version");
      await writeFile(external, "9.8.7\n", "utf8");
      await rm(join(runtime, ".bundle-version"));
      await symlink(external, join(runtime, ".bundle-version"));

      runScript(tauriDirectory, "scripts/ensure-cef-dev-runtime.sh", environment);

      assert.equal(await cargoInvocationCount(cargoLog), 2);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test(
  "macOS Dev rebuilds a runtime with an oversized bundle version marker",
  { skip: process.platform !== "darwin" },
  async () => {
    const directory = await realpath(await mkdtemp(join(tmpdir(), "beaver-cef-version-size-")));
    try {
      const { cargoLog, environment, tauriDirectory } = await createFixture(directory);
      runScript(tauriDirectory, "scripts/prepare-cef.sh", environment);
      const marker = join(
        tauriDirectory,
        "target",
        "cef-runtime",
        "macos",
        ".bundle-version",
      );
      await writeFile(marker, `9.8.7\n${"x".repeat(64)}\n`, "utf8");

      runScript(tauriDirectory, "scripts/ensure-cef-dev-runtime.sh", environment);

      assert.equal(await cargoInvocationCount(cargoLog), 2);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test(
  "macOS CEF preparation rejects an invalid bundle version before compiling",
  { skip: process.platform !== "darwin" },
  async () => {
    const directory = await realpath(
      await mkdtemp(join(tmpdir(), "beaver-cef-invalid-version-")),
    );
    try {
      const { cargoLog, environment, tauriDirectory } = await createFixture(directory);
      await writeFile(
        join(tauriDirectory, "tauri.conf.json"),
        `${JSON.stringify({ version: "9.8" })}\n`,
        "utf8",
      );

      const result = runScriptResult(
        tauriDirectory,
        "scripts/prepare-cef.sh",
        environment,
      );

      assert.notEqual(result.status, 0);
      assert.equal(await cargoInvocationCount(cargoLog), 0);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test(
  "macOS E2E preparation rejects a missing Cargo target before compiling",
  { skip: process.platform !== "darwin" },
  async () => {
    const directory = await realpath(
      await mkdtemp(join(tmpdir(), "beaver-cef-runtime-missing-target-")),
    );
    try {
      const { cargoLog, environment, tauriDirectory } = await createFixture(directory);
      const result = runScriptResult(tauriDirectory, "scripts/prepare-cef.sh", {
        ...environment,
        CLGO_CEF_CARGO_FEATURES: "e2e",
      });

      assert.notEqual(result.status, 0);
      assert.equal(await cargoInvocationCount(cargoLog), 0);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test(
  "macOS E2E preparation rejects a foreign Cargo target before compiling",
  { skip: process.platform !== "darwin" },
  async () => {
    const directory = await realpath(
      await mkdtemp(join(tmpdir(), "beaver-cef-runtime-foreign-target-")),
    );
    try {
      const { cargoLog, environment, tauriDirectory } = await createFixture(directory);
      const result = runScriptResult(tauriDirectory, "scripts/prepare-cef.sh", {
        ...environment,
        CARGO_TARGET_DIR: join(tauriDirectory, "target", "foreign"),
        CLGO_CEF_CARGO_FEATURES: "e2e",
      });

      assert.notEqual(result.status, 0);
      assert.equal(await cargoInvocationCount(cargoLog), 0);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test(
  "macOS E2E helper compilation does not inherit the Tauri bundle config",
  { skip: process.platform !== "darwin" },
  async () => {
    const directory = await realpath(
      await mkdtemp(join(tmpdir(), "beaver-cef-runtime-tauri-config-")),
    );
    try {
      const { environment, tauriDirectory } = await createFixture(directory);
      runScript(tauriDirectory, "scripts/prepare-cef.sh", {
        ...environment,
        CARGO_TARGET_DIR: join(tauriDirectory, "target", "e2e"),
        CLGO_CEF_CARGO_FEATURES: "e2e",
        FIXTURE_REJECT_TAURI_CONFIG: "1",
        TAURI_CONFIG: '{"bundle":{"macOS":{"frameworks":["missing.framework"]}}}',
      });

      await assertProfile(tauriDirectory, join("target", "e2e"), "e2e");
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test(
  "macOS Dev rebuilds its runtime when the profile authority changes",
  { skip: process.platform !== "darwin" },
  async () => {
    const directory = await realpath(
      await mkdtemp(join(tmpdir(), "beaver-cef-runtime-authority-")),
    );
    try {
      const { cargoLog, environment, tauriDirectory } = await createFixture(directory);
      runScript(tauriDirectory, "scripts/prepare-cef.sh", environment);
      const authority = join(tauriDirectory, "scripts", "cef-runtime-profile.sh");
      const future = new Date(Date.now() + 5_000);
      await utimes(authority, future, future);

      runScript(tauriDirectory, "scripts/ensure-cef-dev-runtime.sh", environment);

      assert.equal(await cargoInvocationCount(cargoLog), 2);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test(
  "macOS Dev rebuilds a legacy cache contaminated by E2E helpers",
  { skip: process.platform !== "darwin" },
  async () => {
    const directory = await realpath(
      await mkdtemp(join(tmpdir(), "beaver-cef-runtime-recovery-")),
    );
    try {
      const { environment, tauriDirectory } = await createFixture(directory);
      runScript(tauriDirectory, "scripts/prepare-cef.sh", environment);
      await contaminateRuntime(tauriDirectory, "target", "e2e");

      runScript(
        tauriDirectory,
        "scripts/run-cef-dev-app.sh",
        environment,
        ["target/debug/cl-go-dash"],
      );

      await assertProfile(tauriDirectory, "target", "dev");
      await assertLaunchedProfile(tauriDirectory, "target", "dev");
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);
