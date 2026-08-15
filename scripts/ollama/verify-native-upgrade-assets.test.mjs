import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { test } from "node:test";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Readable } from "node:stream";
import { fileURLToPath } from "node:url";
import {
  constantTimeHexEqual,
  verifyNativeUpgradeAssets,
  verifyNativeUpgradeManifest,
} from "./verify-native-upgrade-assets.mjs";

const scriptDirectory = fileURLToPath(new URL(".", import.meta.url));
const manifestPath = join(scriptDirectory, "native-upgrade-assets.json");

async function readManifest() {
  return JSON.parse(await readFile(manifestPath, "utf8"));
}

function cloneManifest(manifest) {
  return structuredClone(manifest);
}

function assertManifestRejected(manifest, pattern) {
  assert.throws(() => verifyNativeUpgradeManifest(manifest), pattern);
}

test("the immutable manifest has exactly the three Beaver v1.1.2 assets", async () => {
  const manifest = await readManifest();
  const result = verifyNativeUpgradeManifest(manifest);

  assert.deepEqual(result.platforms, ["macos-aarch64", "linux-x64", "windows-x64"]);
  assert.equal(result.product, "Beaver");
  assert.equal(result.version, "1.1.2");
  assert.deepEqual(manifest.excluded, [
    {
      product: "CL-GO-DASH",
      version: "1.0.2",
      reason: "ancien produit sans pont de mise à jour vers Beaver",
    },
  ]);
  assert.equal(Object.keys(manifest.assets).some((key) => key.includes("CL-GO")), false);
});

test("manifest rejects an extra platform", async () => {
  const manifest = cloneManifest(await readManifest());
  manifest.assets["freebsd-x64"] = structuredClone(manifest.assets["linux-x64"]);
  assertManifestRejected(manifest, /manifest/i);
});

test("manifest rejects a foreign host and non-HTTPS URL", async () => {
  const manifest = cloneManifest(await readManifest());
  manifest.assets["linux-x64"].url = "https://evil.example/beaver.deb";
  assertManifestRejected(manifest, /manifest/i);

  const second = cloneManifest(await readManifest());
  second.assets["linux-x64"].url = second.assets["linux-x64"].url.replace("https://", "http://");
  assertManifestRejected(second, /manifest/i);
});

test("manifest rejects wrong version, name, size, short SHA, and old-product assets", async () => {
  const base = await readManifest();

  const wrongVersion = cloneManifest(base);
  wrongVersion.fromVersion = "1.0.2";
  assertManifestRejected(wrongVersion, /manifest/i);

  const wrongName = cloneManifest(base);
  wrongName.assets["macos-aarch64"].name = "CL-GO-DASH_1.0.2.dmg";
  assertManifestRejected(wrongName, /manifest/i);

  const wrongSize = cloneManifest(base);
  wrongSize.assets["linux-x64"].size = 0;
  assertManifestRejected(wrongSize, /manifest/i);

  const shortSha = cloneManifest(base);
  shortSha.assets["windows-x64"].sha256 = "deadbeef";
  assertManifestRejected(shortSha, /manifest/i);

  const oldAsset = cloneManifest(base);
  oldAsset.assets["cl-go-dash-1.0.2"] = {
    name: "CL-GO-DASH_1.0.2.dmg",
    url: "https://github.com/Kevin-hDev/Beaver/releases/download/v1.0.2/CL-GO-DASH_1.0.2.dmg",
    size: 1,
    sha256: "0".repeat(64),
  };
  assertManifestRejected(oldAsset, /manifest/i);
});

test("SHA comparison is equal-length constant-time and rejects mismatches", () => {
  assert.equal(constantTimeHexEqual("a".repeat(64), "a".repeat(64)), true);
  assert.equal(constantTimeHexEqual("a".repeat(64), "b".repeat(64)), false);
  assert.equal(constantTimeHexEqual("a".repeat(64), "a"), false);
});

function manifestForPayload(base, payload) {
  const digest = createHash("sha256").update(payload).digest("hex");
  const manifest = cloneManifest(base);
  for (const asset of Object.values(manifest.assets)) {
    asset.size = payload.length;
    asset.sha256 = digest;
  }
  return manifest;
}

function responseFor(payload, status = 200, location = null) {
  return {
    status,
    ok: status >= 200 && status < 300,
    headers: { get: (name) => (name.toLowerCase() === "location" ? location : null) },
    body: status === 200 ? Readable.from([payload]) : null,
  };
}

test("asset verifier hashes a bounded stream and removes its temporary files", async () => {
  const base = await readManifest();
  const payload = Buffer.from("small deterministic Beaver asset");
  const manifest = manifestForPayload(base, payload);
  const tempRoot = await mkdtemp(join(tmpdir(), "beaver-native-assets-test-"));
  const calls = [];

  try {
    const result = await verifyNativeUpgradeAssets({
      manifest,
      tempRoot,
      fetchImpl: async (url, options) => {
        calls.push({ url, options });
        return responseFor(payload);
      },
    });

    assert.equal(result.verified.length, 3);
    assert.equal(calls.length, 3);
    assert.deepEqual(await (await import("node:fs/promises")).readdir(tempRoot), []);
    assert.equal(calls.every(({ options }) => options.redirect === "manual"), true);
  } finally {
    await rm(tempRoot, { recursive: true, force: true });
  }
});

test("asset verifier rejects a redirect to a foreign host", async () => {
  const base = await readManifest();
  const payload = Buffer.from("redirect payload");
  const manifest = manifestForPayload(base, payload);
  const tempRoot = await mkdtemp(join(tmpdir(), "beaver-native-assets-redirect-"));

  try {
    await assert.rejects(
      verifyNativeUpgradeAssets({
        manifest,
        tempRoot,
        fetchImpl: async () => responseFor(payload, 302, "https://evil.example/asset"),
      }),
      /asset verification/i,
    );
  } finally {
    await rm(tempRoot, { recursive: true, force: true });
  }
});

test("asset verifier follows a bounded redirect that stays on the GitHub release host", async () => {
  const base = await readManifest();
  const payload = Buffer.from("same host redirect payload");
  const manifest = manifestForPayload(base, payload);
  const tempRoot = await mkdtemp(join(tmpdir(), "beaver-native-assets-same-host-"));
  let firstRequest = true;

  try {
    const result = await verifyNativeUpgradeAssets({
      manifest,
      tempRoot,
      fetchImpl: async (url) => {
        if (firstRequest) {
          firstRequest = false;
          return responseFor(payload, 302, url);
        }
        return responseFor(payload);
      },
    });
    assert.equal(result.verified.length, 3);
  } finally {
    await rm(tempRoot, { recursive: true, force: true });
  }
});
