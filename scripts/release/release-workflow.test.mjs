import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const workflow = readFileSync(".github/workflows/release.yml", "utf8");

test("valide tout le projet et les métadonnées Beaver avant les builds", () => {
  assert.match(workflow, /\n  gate:\n/);
  assert.match(workflow, /\n  validate:\n    needs: gate\n/);
  assert.match(workflow, /\n  validate:\n/);
  assert.match(workflow, /\n  build:\n    needs: validate\n/);
  assert.match(
    workflow,
    /\n  validate:[\s\S]*?ref: refs\/tags\/\$\{\{ env\.RELEASE_TAG \}\}\n          fetch-depth: 0\n/,
  );
  assert.match(
    workflow,
    /check-brand-artifacts\.mjs source "\$RELEASE_TAG" \./,
  );
  assert.match(
    workflow,
    /git fetch --no-tags --depth=2[\s\S]*?refs\/tags\/v1\.0\.2:refs\/tags\/cl-go-v1\.0\.2-baseline/,
  );
  assert.doesNotMatch(workflow, /check-bridge-metadata\.mjs/);
  for (const command of [
    "npm test",
    "npm run build",
    "npm run lint",
    "npm run test:install",
    "npm run test:brand-boundaries",
    "npm run test:persistence-migration",
    "npm run test:release-workflow",
    "npm run test:bridge-release",
    "npm run test:update-manifest",
    "cargo fmt --check",
    "cargo check",
    "cargo clippy --all-targets -- -D warnings",
    "cargo test",
  ]) {
    assert.ok(workflow.includes(command), `commande manquante : ${command}`);
  }
});

test("accepte uniquement un tag exact sans laisser de jeton Git persistant", () => {
  assert.match(
    workflow,
    /\^v\(0\|\[1-9\]\[0-9\]\*\)\\\.\(0\|\[1-9\]\[0-9\]\*\)\\\.\(0\|\[1-9\]\[0-9\]\*\)\$/,
  );
  assert.match(workflow, /ref: refs\/tags\/\$\{\{ env\.RELEASE_TAG \}\}/g);
  const checkouts = workflow.match(/actions\/checkout@[a-f0-9]{40}/g) ?? [];
  const disabledCredentials = workflow.match(/persist-credentials: false/g) ?? [];
  assert.equal(disabledCredentials.length, checkouts.length);
});

test("les trois machines construisent sans toucher à une release", () => {
  assert.match(workflow, /Build Tauri app without publishing/);
  assert.match(
    workflow,
    /tauri-apps\/tauri-action@[a-f0-9]{40} # v0/,
  );
  assert.doesNotMatch(workflow, /^\s+(?:tagName|releaseName|releaseDraft):/mu);
  for (const value of [
    "bundles: app,dmg",
    "bundles: deb",
    "bundles: nsis",
    "beaver-macos-arm64",
    "beaver-linux-x64",
    "beaver-windows-x64",
    "_aarch64.dmg",
    "_amd64.deb",
    "_x64-setup.exe",
  ]) {
    assert.ok(workflow.includes(value), `build incomplet : ${value}`);
  }
});

test("fige chaque action tierce sur une empreinte Git exacte", () => {
  const uses = [...workflow.matchAll(/^\s*-?\s*uses:\s+([^#\s]+)(?:\s+#.*)?$/gmu)].map(
    (match) => match[1],
  );
  assert.ok(uses.length >= 10);
  for (const action of uses) {
    assert.match(action, /^[\w.-]+\/[\w.-]+@[a-f0-9]{40}$/u);
  }
});

test("inspecte chaque bundle avec son outil natif", () => {
  for (const value of [
    "check-brand-artifacts.mjs macos",
    "check-brand-artifacts.mjs linux",
    "check-deb-migration.sh",
    "scripts/test-install-ps1.ps1",
    "check-brand-artifacts.mjs windows",
    "check-nsis-migration.ps1 -Mode Source",
    "check-nsis-migration.ps1 -Mode Installed",
    'Start-Process -FilePath $installer -ArgumentList @("/S", "/D=$installDir")',
  ]) {
    assert.ok(workflow.includes(value), `inspection absente : ${value}`);
  }
  assert.match(workflow, /if-no-files-found: error/g);
});

test("assemble les trois assets puis revérifie le manifeste séparément", () => {
  assert.match(workflow, /\n  manifest:\n    needs: build\n/);
  assert.match(workflow, /\n  verify_release:\n    needs: manifest\n/);
  assert.match(workflow, /\n  draft_release:\n    needs: verify_release\n/);
  assert.match(
    workflow,
    /create-update-manifest\.mjs "\$RELEASE_TAG" release-candidate/,
  );
  assert.ok(
    workflow.match(
      /check-brand-artifacts\.mjs release "\$RELEASE_TAG" release-candidate/g,
    )?.length >= 3,
  );
  assert.match(workflow, /name: beaver-release-candidate/);
  assert.match(workflow, /Independently verify every SHA-256/);
});

test("crée uniquement un brouillon Beaver et refuse une release déjà publiée", () => {
  assert.match(workflow, /^permissions:\n  contents: read$/mu);
  assert.match(
    workflow,
    /\n  draft_release:[\s\S]*?\n    permissions:\n      contents: write\n/,
  );
  assert.match(workflow, /gh release create "\$RELEASE_TAG"/);
  assert.match(workflow, /--draft \\\n\s+--verify-tag/);
  assert.match(workflow, /--title "Beaver \$RELEASE_TAG"/);
  assert.match(workflow, /if \[ "\$STATE" != "true" \]/);
  assert.match(workflow, /gh release upload[\s\S]*--clobber/);
  assert.doesNotMatch(workflow, /--draft=false|releaseDraft: false|\n  publish:\n/);
  assert.doesNotMatch(workflow, /CL-GO/);
});

test("empêche deux brouillons concurrents du même tag", () => {
  assert.match(
    workflow,
    /^concurrency:\n  group: release-\$\{\{ inputs\.version \|\| github\.ref_name \}\}\n  cancel-in-progress: false$/mu,
  );
});
