import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  chmod,
  copyFile,
  mkdir,
  readFile,
  rm,
  writeFile,
} from "node:fs/promises";
import { dirname, join, resolve } from "node:path";

export const HELPERS = [
  "Beaver Helper",
  "Beaver Helper (GPU)",
  "Beaver Helper (Renderer)",
  "Beaver Helper (Plugin)",
  "Beaver Helper (Alerts)",
];

async function writeExecutable(path, body) {
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, body, "utf8");
  await chmod(path, 0o755);
}

export function runScript(cwd, script, environment = {}, argumentsList = []) {
  const result = spawnSync("/bin/bash", [script, ...argumentsList], {
    cwd,
    encoding: "utf8",
    env: { ...process.env, ...environment },
    shell: false,
  });
  assert.equal(
    result.status,
    0,
    `${script} failed\nstdout:\n${result.stdout}\nstderr:\n${result.stderr}`,
  );
}

async function createCefSource(tauriDirectory) {
  const framework = join(
    tauriDirectory,
    ".cef-verified",
    "current",
    "Release",
    "Chromium Embedded Framework.framework",
  );
  await mkdir(join(framework, "Libraries"), { recursive: true });
  await Promise.all([
    writeFile(join(framework, "Chromium Embedded Framework"), "framework\n"),
    writeFile(join(framework, "Libraries", "libfixture.dylib"), "library\n"),
    writeFile(
      join(tauriDirectory, ".cef-verified", "current", "LICENSE.txt"),
      "fixture license\n",
    ),
  ]);
}

async function createHelperSkeletons(tauriDirectory) {
  for (const helper of HELPERS) {
    const contents = join(
      tauriDirectory,
      "resources",
      "cef",
      "macos",
      "helpers",
      `${helper}.app`,
      "Contents",
    );
    await mkdir(contents, { recursive: true });
    await writeFile(join(contents, "Info.plist"), `${helper}\n`, "utf8");
  }
  const devApp = join(tauriDirectory, "resources", "cef", "macos", "dev-app");
  await mkdir(devApp, { recursive: true });
  await writeFile(join(devApp, "Info.plist"), "dev app\n", "utf8");
}

async function createTrackedInputs(tauriDirectory) {
  for (const input of [
    "Cargo.toml",
    "Cargo.lock",
    "build.rs",
    "Entitlements.dev.plist",
    join("src", "bin", "cl-go-dash-helper.rs"),
  ]) {
    const path = join(tauriDirectory, input);
    await mkdir(dirname(path), { recursive: true });
    await writeFile(path, `${input}\n`, "utf8");
  }
  for (const targetRoot of ["target", join("target", "e2e")]) {
    const debugRoot = join(tauriDirectory, targetRoot, "debug");
    await mkdir(join(debugRoot, "default-skills"), { recursive: true });
    await writeFile(join(debugRoot, "default-skills", "fixture.md"), "fixture\n");
    await writeExecutable(join(debugRoot, "cl-go-dash"), "#!/bin/bash\nexit 0\n");
  }
}

async function createFakeCommands(fakeBin) {
  await writeExecutable(join(fakeBin, "uname"), "#!/bin/bash\nprintf 'Darwin\\n'\n");
  await writeExecutable(join(fakeBin, "node"), "#!/bin/bash\nexit 0\n");
  await writeExecutable(join(fakeBin, "codesign"), "#!/bin/bash\nexit 0\n");
  await writeExecutable(
    join(fakeBin, "ditto"),
    `#!/bin/bash
set -euo pipefail
if [[ -d "$2" ]]; then
  /bin/cp -R "$1"/. "$2"/
else
  /bin/cp -R "$1" "$2"
fi
`,
  );
  await writeExecutable(
    join(fakeBin, "cargo"),
    `#!/bin/bash
set -euo pipefail
profile="dev"
for argument in "$@"; do
  [[ "$argument" == "e2e" ]] && profile="e2e"
done
if [[ "$profile" == "e2e" ]]; then
  release_root="target/e2e/release"
else
  release_root="target/release"
fi
/bin/mkdir -p "$release_root"
/usr/bin/printf '%s-helper\\n' "$profile" > "$release_root/cl-go-dash-helper"
/bin/chmod 755 "$release_root/cl-go-dash-helper"
`,
  );
}

export async function createFixture(directory) {
  const tauriDirectory = join(directory, "src-tauri");
  const fakeBin = join(directory, "fake-bin");
  await Promise.all([
    mkdir(join(tauriDirectory, "scripts"), { recursive: true }),
    mkdir(join(directory, "scripts", "cef"), { recursive: true }),
    mkdir(fakeBin, { recursive: true }),
  ]);
  for (const script of [
    "cef-runtime-profile.sh",
    "prepare-cef.sh",
    "ensure-cef-dev-runtime.sh",
    "run-cef-dev-app.sh",
  ]) {
    await copyFile(
      resolve("src-tauri", "scripts", script),
      join(tauriDirectory, "scripts", script),
    );
  }
  await writeFile(
    join(directory, "scripts", "cef", "prepare-cef-source.mjs"),
    "// External execution is replaced by the fixture.\n",
    "utf8",
  );
  await Promise.all([
    createCefSource(tauriDirectory),
    createHelperSkeletons(tauriDirectory),
    createTrackedInputs(tauriDirectory),
    createFakeCommands(fakeBin),
  ]);
  return {
    environment: {
      CARGO: join(fakeBin, "cargo"),
      CLGO_CEF_DEV_PREP: "1",
      PATH: `${fakeBin}:${process.env.PATH}`,
    },
    tauriDirectory,
  };
}

export async function assertProfile(tauriDirectory, targetRoot, profile) {
  const runtime = join(tauriDirectory, targetRoot, "cef-runtime", "macos");
  assert.equal(await readFile(join(runtime, ".profile"), "utf8"), `${profile}\n`);
  for (const helper of HELPERS) {
    const binary = join(
      runtime,
      "helpers",
      `${helper}.app`,
      "Contents",
      "MacOS",
      helper,
    );
    assert.equal(await readFile(binary, "utf8"), `${profile}-helper\n`);
  }
}

export async function contaminateRuntime(tauriDirectory, targetRoot, profile) {
  const runtime = join(tauriDirectory, targetRoot, "cef-runtime", "macos");
  await rm(join(runtime, ".profile"), { force: true });
  for (const helper of HELPERS) {
    const binary = join(
      runtime,
      "helpers",
      `${helper}.app`,
      "Contents",
      "MacOS",
      helper,
    );
    await writeFile(binary, `${profile}-helper\n`, "utf8");
  }
}

export async function assertLaunchedProfile(tauriDirectory, targetRoot, profile) {
  for (const helper of HELPERS) {
    const binary = join(
      tauriDirectory,
      targetRoot,
      "cef-dev",
      "Beaver Dev.app",
      "Contents",
      "Frameworks",
      `${helper}.app`,
      "Contents",
      "MacOS",
      helper,
    );
    assert.equal(await readFile(binary, "utf8"), `${profile}-helper\n`);
  }
}
