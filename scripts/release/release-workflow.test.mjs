import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { load as loadYaml } from "js-yaml";
import { normalizeNewlines } from "./text-contracts.mjs";

const workflow = normalizeNewlines(
  readFileSync(".github/workflows/release.yml", "utf8"),
);
const workflowDocument = loadYaml(workflow);

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
  assert.doesNotMatch(
    workflow,
    /Kevin-hDev\/cl-go-dash|cl-go-v1\.0\.2-baseline|Fetch pinned bridge baseline/,
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
  assert.match(workflow, /--bundles=\$\{\{ matrix\.bundles \}\}/u);
});

test("construit les roues SearXNG avec la version Python contrôlée", () => {
  const steps = workflowDocument.jobs.build.steps;
  const checkoutIndex = steps.findIndex(({ uses }) => uses?.startsWith("actions/checkout@"));
  const setup = steps.find(({ name }) => name === "Install SearXNG Python");
  const setupIndex = steps.indexOf(setup);
  const buildIndex = steps.findIndex(({ name }) => name === "Build Tauri app without publishing");

  assert.ok(checkoutIndex >= 0);
  assert.ok(setup);
  assert.ok(setupIndex >= 0);
  assert.ok(buildIndex >= 0);
  assert.equal(setup.uses, "actions/setup-python@ece7cb06caefa5fff74198d8649806c4678c61a1");
  assert.equal(setup.with["python-version-file"], "scripts/build/searxng-python-version.txt");
  assert.ok(checkoutIndex < setupIndex && setupIndex < buildIndex);
});

test("valide les scripts SearXNG avec le Python contrôlé avant la release", () => {
  const steps = workflowDocument.jobs.validate.steps;
  const checkoutIndex = steps.findIndex(({ uses }) => uses?.startsWith("actions/checkout@"));
  const nodeIndex = steps.findIndex(({ uses }) => uses?.startsWith("actions/setup-node@"));
  const setup = steps.find(({ name }) => name === "Install SearXNG test Python");
  const setupIndex = steps.indexOf(setup);
  const tests = steps.find(({ name }) => name === "SearXNG preparation script tests");
  const testsIndex = steps.indexOf(tests);

  assert.ok(setup, "installation du Python de test manquante");
  assert.ok(tests, "tests Python SearXNG manquants");
  assert.equal(setup.uses, "actions/setup-python@ece7cb06caefa5fff74198d8649806c4678c61a1");
  assert.equal(setup.with["python-version-file"], "scripts/build/searxng-python-version.txt");
  assert.equal(tests.run, "npm run test:searxng-scripts");
  assert.ok(checkoutIndex < nodeIndex && nodeIndex < setupIndex && setupIndex < testsIndex);
});

test("vérifie le wheelhouse SearXNG après chaque build avant les artefacts", () => {
  const builds = workflowDocument.jobs.build.strategy.matrix.include;
  const steps = workflowDocument.jobs.build.steps;
  const buildIndex = steps.findIndex(({ name }) => name === "Build Tauri app without publishing");
  const smokeSteps = steps.filter(({ name }) => name === "Verify SearXNG offline runtime");
  const resolveIndex = steps.findIndex(({ name }) => name === "Resolve exact artifact paths");

  assert.equal(smokeSteps.length, 1);
  const smoke = smokeSteps[0];
  assert.equal(smoke["working-directory"], "src-tauri");
  assert.equal(
    smoke.run,
    "cargo test --lib ${{ matrix.searxng_test_features }} services::searxng::runtime_environment_tests::release_wheelhouse_installs_below_the_safety_margin -- --ignored --exact --nocapture",
  );
  assert.deepEqual(
    builds.map(({ os, searxng_test_features: features }) => [os, features]),
    [
      ["macos-latest", ""],
      ["ubuntu-22.04", ""],
      ["windows-latest", "--features windows-tests"],
    ],
  );
  const smokeIndex = steps.indexOf(smoke);
  assert.ok(buildIndex < smokeIndex && smokeIndex < resolveIndex);
});

test("partage la cible Cargo Windows avant le build et sa relecture", () => {
  const configure = workflow.indexOf("Configure Windows Cargo target");
  const build = workflow.indexOf("Build Tauri app without publishing");
  const resolvePaths = workflow.indexOf("Resolve exact artifact paths");
  assert.ok(configure > 0 && configure < build && build < resolvePaths);
  assert.match(
    workflow,
    /Configure Windows Cargo target\n\s+if: runner\.os == 'Windows'[\s\S]*?CARGO_TARGET_DIR=[^\n]*GITHUB_ENV/,
  );
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

test("la vérification Windows possède ses variables et propage son échec natif", () => {
  const step = workflowDocument.jobs.build.steps.find(
    ({ name }) => name === "Inspect and install Windows package",
  );

  assert.ok(step);
  assert.equal(step.env.CARGO_BUILD_TARGET, "${{ matrix.target }}");
  assert.equal(step.env.BEAVER_TAURI_BUNDLE_TYPE, "${{ matrix.bundles }}");
  assert.match(
    step.run,
    /tauri-bundle-marker\.mjs verify[\s\S]*?if \(\$LASTEXITCODE -ne 0\) \{[\s\S]*?throw/,
  );
});

test("le parcours Windows résout et valide sans Bash", () => {
  assert.match(
    workflow,
    /Resolve exact artifact paths[\s\S]*?run: node scripts\/release\/resolve-artifact-path\.mjs/,
  );
  for (const variable of ["RELEASE_TAG", "BUNDLE_TARGET", "BUNDLE_DIR", "ASSET_SUFFIX"]) {
    assert.match(workflow, new RegExp(`${variable}:`));
  }
  const resolverStep = workflow
    .split(/\n      - name:/u)
    .find((step) => step.startsWith(" Resolve exact artifact paths"));
  assert.ok(resolverStep);
  assert.doesNotMatch(resolverStep, /shell: bash/);
  assert.equal(workflow.match(/check-nsis-migration\.test\.ps1/g)?.length, 2);
  assert.match(workflow, /shell: pwsh[\s\S]*?check-nsis-migration\.test\.ps1/);
  assert.match(workflow, /shell: powershell[\s\S]*?check-nsis-migration\.test\.ps1/);
  assert.match(
    workflow,
    /powershell\.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass[\s\S]*?-Mode Installed -InstallerPath \$installer/,
  );

  const bashSteps = workflow
    .split(/\n      - name:/u)
    .filter((step) => step.includes("shell: bash"));
  for (const step of bashSteps) {
    assert.match(step, /if: runner\.os (?:!= 'Windows'|== '(?:Linux|macOS)')/);
  }
});

test("assemble les trois assets puis revérifie le manifeste séparément", () => {
  assert.match(workflow, /\n  manifest:\n    needs: build\n/);
  assert.match(workflow, /\n  verify_release:\n    needs: manifest\n/);
  assert.match(workflow, /\n  publish_release:\n    needs: verify_release\n/);
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

test("publie Beaver uniquement après vérification complète et refuse un état ambigu", () => {
  assert.match(workflow, /^permissions:\n  contents: read$/mu);
  assert.match(
    workflow,
    /\n  publish_release:[\s\S]*?\n    permissions:\n      contents: write\n/,
  );
  assert.match(workflow, /gh release create "\$RELEASE_TAG"/);
  assert.match(workflow, /--verify-tag \\\n\s+--latest/);
  assert.match(workflow, /--title "Beaver \$RELEASE_TAG"/);
  assert.match(workflow, /if \[ "\$STATE" != \$'false\\tfalse' \]/);
  assert.match(workflow, /gh release upload[\s\S]*--clobber/);
  assert.match(workflow, /gh release edit[\s\S]*--draft=false[\s\S]*--prerelease=false[\s\S]*--latest/);
  assert.doesNotMatch(workflow, /\n\s+--draft \\|\n  draft_release:\n/);
  assert.doesNotMatch(workflow, /CL-GO/);
});

test("empêche deux publications concurrentes du même tag", () => {
  assert.match(
    workflow,
    /^concurrency:\n  group: release-\$\{\{ inputs\.version \|\| github\.ref_name \}\}\n  cancel-in-progress: false$/mu,
  );
});
