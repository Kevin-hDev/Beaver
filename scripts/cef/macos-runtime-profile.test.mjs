import { mkdtemp, realpath, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  assertLaunchedProfile,
  assertProfile,
  contaminateRuntime,
  createFixture,
  runScript,
} from "./macos-runtime-profile-fixture.mjs";

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
