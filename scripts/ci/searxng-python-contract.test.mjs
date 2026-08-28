import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { load as loadYaml } from "js-yaml";

const workflow = loadYaml(readFileSync(".github/workflows/ci.yml", "utf8"));
const packageDocument = JSON.parse(readFileSync("package.json", "utf8"));

test("la CI exécute les tests Python SearXNG avec la version supportée", () => {
  const steps = workflow.jobs.backend.steps;
  const checkoutIndex = steps.findIndex(({ uses }) => uses?.startsWith("actions/checkout@"));
  const nodeIndex = steps.findIndex(({ uses }) => uses?.startsWith("actions/setup-node@"));
  const setup = steps.find(({ name }) => name === "Install SearXNG test Python");
  const setupIndex = steps.indexOf(setup);
  const tests = steps.find(({ name }) => name === "SearXNG preparation script tests");
  const testsIndex = steps.indexOf(tests);

  assert.ok(setup, "installation du Python de test manquante");
  assert.ok(tests, "tests Python SearXNG manquants");
  assert.equal(setup.uses, "actions/setup-python@5fda3b95a4ea91299a34e894583c3862153e4b97");
  assert.equal(setup.with["python-version-file"], "scripts/build/searxng-python-version.txt");
  assert.equal(tests.run, "npm run test:searxng-scripts");
  assert.ok(checkoutIndex < nodeIndex && nodeIndex < setupIndex && setupIndex < testsIndex);
  assert.equal(
    packageDocument.scripts["test:searxng-scripts"],
    "node scripts/ci/run-searxng-python-tests.mjs",
  );
});

test("Windows exécute la recette uv puis vérifie Python dans un nouveau shell", () => {
  const steps = workflow.jobs["backend-windows-native"].steps;
  const setupUv = steps.find(({ name }) => name === "Install uv for Windows SearXNG recipe");
  const install = steps.find(({ name }) => name === "Install Windows SearXNG Python recipe");
  const verify = steps.find(({ name }) => name === "Verify Windows SearXNG Python recipe");

  assert.ok(setupUv);
  assert.ok(install);
  assert.ok(verify);
  assert.equal(
    setupUv.uses,
    "astral-sh/setup-uv@20cfd1bf945f4377ade1205e4dbc17946fc9a30d",
  );
  assert.equal(setupUv.with.version, "latest");
  assert.equal(setupUv.with["enable-cache"], false);
  assert.equal(install.shell, "pwsh");
  assert.equal(install.env.UV_PYTHON_INSTALL_BIN, "1");
  assert.match(install.run, /Get-Content -Raw scripts\/build\/searxng-python-version\.txt/u);
  assert.match(install.run, /uv python install \$version/u);
  assert.match(install.run, /uv python update-shell/u);
  assert.match(install.run, /uv python dir --bin/u);
  assert.match(install.run, /\$env:GITHUB_PATH/u);
  assert.equal(verify.shell, "pwsh");
  assert.match(verify.run, /Get-Content -Raw scripts\/build\/searxng-python-version\.txt/u);
  assert.match(verify.run, /Get-Command "python\$expected" -CommandType Application/u);
  assert.ok(steps.indexOf(setupUv) < steps.indexOf(install));
  assert.ok(steps.indexOf(install) < steps.indexOf(verify));
});
