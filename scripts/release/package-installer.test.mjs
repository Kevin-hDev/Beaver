import assert from "node:assert/strict";
import { execFile as execFileCallback } from "node:child_process";
import { createHash } from "node:crypto";
import { promisify } from "node:util";
import {
  chmod,
  link,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { packageInstaller, validateArchiveEntries } from "./package-installer.mjs";

const execFile = promisify(execFileCallback);
const PLIST = `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>com.clgo.dash.installer</string>
<key>CFBundleExecutable</key><string>beaver-installer</string>
</dict></plist>`;

async function fixture() {
  const root = await mkdtemp(join(tmpdir(), "beaver-installer-package-test-"));
  const app = join(root, "Beaver Installer.app");
  const executable = join(app, "Contents", "MacOS", "beaver-installer");
  await mkdir(join(app, "Contents", "Resources"), { recursive: true });
  await mkdir(join(app, "Contents", "MacOS"), { recursive: true });
  await writeFile(join(app, "Contents", "Info.plist"), PLIST);
  await writeFile(executable, "#!/bin/sh\nexit 0\n");
  await chmod(executable, 0o755);
  return { root, app, executable };
}

test("verrouille les chemins de l’archive et son inventaire", () => {
  validateArchiveEntries(["Beaver Installer.app/", "Beaver Installer.app/Contents/"]);
  for (const entries of [
    ["/Beaver Installer.app"],
    ["Beaver Installer.app/../sentinel"],
    ["Beaver Installer.app", "Beaver Installer.app/"],
    ["Wrong.app"],
    [`Beaver Installer.app/${"a".repeat(1_004)}`],
    Array.from({ length: 4_097 }, (_, index) => `Beaver Installer.app/${index}`),
  ]) {
    assert.throws(() => validateArchiveEntries(entries));
  }
});

test("produit deux archives macOS identiques et relit leur bundle", async (t) => {
  if (process.platform !== "darwin") return t.skip("validation plist native macOS");
  const value = await fixture();
  const first = join(value.root, "first");
  const second = join(value.root, "second");
  await mkdir(first);
  await mkdir(second);
  try {
    const left = await packageInstaller({
      tag: "v1.2.3",
      os: "macOS",
      source: value.app,
      suffix: "_installer-aarch64.tar.gz",
      outputDirectory: first,
    });
    const right = await packageInstaller({
      tag: "1.2.3",
      os: "macOS",
      source: value.app,
      suffix: "_installer-aarch64.tar.gz",
      outputDirectory: second,
    });
    assert.equal(left.endsWith("Beaver_1.2.3_installer-aarch64.tar.gz"), true);
    assert.equal(
      createHash("sha256").update(await readFile(left)).digest("hex"),
      createHash("sha256").update(await readFile(right)).digest("hex"),
    );
  } finally {
    await rm(value.root, { recursive: true, force: true });
  }
});

test("refuse suffixes, traversées, liens et fichiers spéciaux sans toucher la sentinelle", async (t) => {
  if (process.platform !== "darwin") return t.skip("validation plist native macOS");
  const value = await fixture();
  const sentinel = join(value.root, "sentinel");
  await writeFile(sentinel, "outside");
  try {
    await assert.rejects(packageInstaller({ tag: "1.2.3", os: "macOS", source: value.app }));
    await assert.rejects(packageInstaller({ tag: "1.2.3", os: "macOS", source: value.app, suffix: "_installer-x64.exe" }));
    await assert.rejects(packageInstaller({
      tag: "1.2.3", os: "macOS", source: `${value.root}/../${value.root.split("/").at(-1)}/Beaver Installer.app`,
      suffix: "_installer-aarch64.tar.gz",
    }));
    const linkPath = join(value.app, "Contents", "sentinel-link");
    await symlink(sentinel, linkPath);
    await assert.rejects(packageInstaller({ tag: "1.2.3", os: "macOS", source: value.app, suffix: "_installer-aarch64.tar.gz" }));
    await rm(linkPath);
    const hardlink = join(value.app, "Contents", "MacOS", "duplicate");
    await link(value.executable, hardlink);
    await assert.rejects(packageInstaller({ tag: "1.2.3", os: "macOS", source: value.app, suffix: "_installer-aarch64.tar.gz" }));
    await rm(hardlink);
    if (process.platform !== "win32") {
      const fifo = join(value.app, "Contents", "fifo");
      await execFile("mkfifo", [fifo]);
      await assert.rejects(packageInstaller({ tag: "1.2.3", os: "macOS", source: value.app, suffix: "_installer-aarch64.tar.gz" }));
      await rm(fifo);
    }
    assert.equal(await readFile(sentinel, "utf8"), "outside");
  } finally {
    await rm(value.root, { recursive: true, force: true });
  }
});

test("copie uniquement un PE Windows régulier et unique", async () => {
  const root = await mkdtemp(join(tmpdir(), "beaver-installer-pe-test-"));
  const output = join(root, "output");
  const source = join(root, "beaver-installer.exe");
  await mkdir(output);
  await writeFile(source, Buffer.from([0x4d, 0x5a, 0, 0]));
  try {
    const result = await packageInstaller({
      tag: "1.2.3",
      os: "Windows",
      source,
      suffix: "_installer-x64.exe",
      outputDirectory: output,
    });
    assert.equal(result.endsWith("Beaver_1.2.3_installer-x64.exe"), true);
    const duplicate = join(root, "duplicate.exe");
    await link(source, duplicate);
    await assert.rejects(packageInstaller({
      tag: "1.2.3",
      os: "Windows",
      source,
      suffix: "_installer-x64.exe",
      outputDirectory: root,
    }));
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
