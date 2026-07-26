import assert from "node:assert/strict";
import test from "node:test";

import {
  publishBridgeRelease,
  validateDraftRelease,
} from "./publish-bridge-release.mjs";

const TAG = "v1.0.2";
const REPOSITORY = "Kevin-hDev/CL-GO-DASH";

function asset(name) {
  return {
    digest: `sha256:${"a".repeat(64)}`,
    name,
    size: 1024,
    state: "uploaded",
  };
}

function validRelease() {
  return {
    assets: [
      asset("CL-GO_1.0.2_aarch64.dmg"),
      asset("CL-GO_1.0.2_amd64.deb"),
      asset("CL-GO_1.0.2_x64-setup.exe"),
    ],
    isDraft: true,
    isPrerelease: false,
    name: "CL-GO v1.0.2",
    tagName: TAG,
  };
}

test("accepte uniquement le brouillon CL-GO complet", () => {
  assert.doesNotThrow(() => validateDraftRelease(validRelease(), TAG));
});

test("refuse un état ou un asset de release inattendu", () => {
  const mutations = [
    (value) => (value.isDraft = false),
    (value) => (value.isPrerelease = true),
    (value) => (value.tagName = "v1.0.3"),
    (value) => (value.name = "Beaver v1.0.2"),
    (value) => value.assets.pop(),
    (value) => value.assets.push(asset("extra.zip")),
    (value) => (value.assets[0].name = value.assets[1].name),
    (value) => (value.assets[0].state = "new"),
    (value) => (value.assets[0].size = 0),
    (value) => (value.assets[0].size = 2_147_483_649),
    (value) => (value.assets[0].digest = "sha256:invalid"),
  ];

  for (const mutate of mutations) {
    const release = validRelease();
    mutate(release);
    assert.throws(() => validateDraftRelease(release, TAG));
  }
});

test("appelle gh sans shell et publie seulement après validation", () => {
  const calls = [];
  const run = (program, args, options) => {
    calls.push({ args, options, program });
    return calls.length === 1 ? JSON.stringify(validRelease()) : "";
  };

  publishBridgeRelease({ repository: REPOSITORY, run, tag: TAG });

  assert.deepEqual(calls, [
    {
      program: "gh",
      args: [
        "release",
        "view",
        TAG,
        "--repo",
        REPOSITORY,
        "--json",
        "tagName,name,isDraft,isPrerelease,assets",
      ],
      options: { maxOutputBytes: 524_288, timeoutMs: 30_000 },
    },
    {
      program: "gh",
      args: [
        "release",
        "edit",
        TAG,
        "--repo",
        REPOSITORY,
        "--verify-tag",
        "--draft=false",
        "--latest",
      ],
      options: { maxOutputBytes: 524_288, timeoutMs: 30_000 },
    },
  ]);
});

test("refuse tout autre dépôt ou tag avant d'appeler gh", () => {
  const run = () => assert.fail("gh ne doit pas être appelé");

  for (const repository of ["Kevin-hDev/Beaver", "other/CL-GO-DASH", ""]) {
    assert.throws(() => publishBridgeRelease({ repository, run, tag: TAG }));
  }
  for (const tag of ["v1.0.1", "v1.1.0", "../v1.0.2"]) {
    assert.throws(() => publishBridgeRelease({ repository: REPOSITORY, run, tag }));
  }
});

test("ne publie pas si la lecture distante échoue", () => {
  let calls = 0;
  const run = () => {
    calls += 1;
    throw new Error("remote failure");
  };

  assert.throws(() => publishBridgeRelease({ repository: REPOSITORY, run, tag: TAG }));
  assert.equal(calls, 1);
});
