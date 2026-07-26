import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  checkContracts,
  classifyReference,
  formatReport,
  scanEntries,
  validateTrackedPath,
} from "./brand-boundaries.mjs";
import { COMPATIBILITY_CONTRACTS } from "./brand-boundaries-contracts.mjs";

const PROJECT_ROOT = fileURLToPath(new URL("../../", import.meta.url));

test("classe les noms publics comme visibles", () => {
  assert.equal(
    classifyReference({ value: "CL-GO-DASH", line: 'title: "CL-GO-DASH"' }),
    "visible",
  );
  assert.equal(
    classifyReference({ value: "CL-GO", line: 'tooltip("CL-GO")' }),
    "visible",
  );
  assert.equal(
    classifyReference({ value: "cl-go", line: '"personality": "/cl-go"' }),
    "visible",
  );
  assert.equal(
    classifyReference({ value: "cl-go", line: 'lower.contains("cl-go")' }),
    "visible",
  );
});

test("classe les identifiants compatibles comme internes", () => {
  const samples = [
    ["cl-go-dash", '.join(".local/share/cl-go-dash")'],
    ["cl_go_dash", 'name = "cl_go_dash_lib"'],
    ["CLGO", 'env("CLGO_FORECAST_TOKEN")'],
    ["clgo", 'localStorage.getItem("clgo-theme")'],
    ["cl-go", 'dir.join(".cl-go")'],
    ["cl-go", 'format!("cl-go/subagent/{id}")'],
    ["cl-go", 'format!(".cl-go-transaction-{id}")'],
    ["cl-go", '"id": "cl-go-beaver"'],
    ["cl-go", 'name = "cl-go-windows-private-store-tests"'],
    [
      "CL-GO",
      'const PRODUCT_NAME = "CL-GO";',
      "scripts/release/check-bridge-metadata.mjs",
    ],
    [
      "CL-GO-DASH",
      'const repository = "Kevin-hDev/CL-GO-DASH";',
      "scripts/release/publish-bridge-release.mjs",
    ],
    [
      "CL-GO",
      'const LEGACY_ENTRY_NAME: &str = "CL-GO";',
      "src-tauri/src/services/autostart_migration.rs",
    ],
    [
      "CL-GO",
      '!define BEAVER_OLD_PRODUCT "Software\\clgo\\CL-GO"',
      "src-tauri/windows/nsis-hooks.nsh",
    ],
    [
      "cl-go",
      '"provides": ["cl-go"]',
      "src-tauri/tauri.conf.json",
    ],
    [
      "cl-go",
      'Provides 2>/dev/null)" = "cl-go"',
      "install.sh",
    ],
  ];

  for (const [value, line, file] of samples) {
    assert.equal(classifyReference({ value, line, file }), "internal");
  }
});

test("bloque un contexte minuscule non reconnu", () => {
  assert.equal(
    classifyReference({ value: "cl-go", line: "temporary cl-go branding" }),
    "unknown",
  );
});

test("détecte un contrat supprimé ou modifié", () => {
  const contracts = [
    {
      name: "identité stable",
      file: "config.json",
      snippets: ['"identifier": "com.clgo.dash"', '"service": "cl-go-dash"'],
    },
  ];
  const valid = new Map([
    [
      "config.json",
      '{"identifier": "com.clgo.dash", "service": "cl-go-dash"}',
    ],
  ]);
  const changed = new Map([
    ["config.json", '{"identifier": "com.beaver.app", "service": "beaver"}'],
  ]);

  assert.deepEqual(checkContracts((file) => valid.get(file), contracts), []);
  assert.equal(checkContracts((file) => changed.get(file), contracts).length, 2);
});

test("chaque valeur de compatibilité est réellement verrouillée", () => {
  for (const contract of COMPATIBILITY_CONTRACTS) {
    const original = readFileSync(
      resolve(PROJECT_ROOT, validateTrackedPath(contract.file)),
      "utf8",
    );
    for (const snippet of contract.snippets) {
      const changed = original.split(snippet).join("__renamed_contract__");
      assert.notEqual(changed, original, `${contract.name} absent du fichier source`);
      const failures = checkContracts(
        (file) => (file === contract.file ? changed : undefined),
        [{ ...contract, snippets: [snippet] }],
      );
      assert.equal(failures.length, 1, `${contract.name} n'est pas verrouillé`);
    }
  }
});

test("refuse les chemins absolus, les contrôles et les traversées", () => {
  const invalid = ["/tmp/file", "../secret", "src/../../secret", "C:\\secret", "src/\nfile"];
  for (const path of invalid) {
    assert.throws(() => validateTrackedPath(path), /chemin suivi invalide/i);
  }
  assert.equal(validateTrackedPath("src/file.ts"), "src/file.ts");
});

test("borne le nombre de fichiers et d'occurrences", () => {
  assert.throws(
    () =>
      scanEntries(
        [
          { file: "one.ts", content: "CL-GO" },
          { file: "two.ts", content: "CL-GO" },
        ],
        { maxFiles: 1 },
      ),
    /trop de fichiers/i,
  );
  assert.throws(
    () =>
      scanEntries([{ file: "one.ts", content: "CL-GO CL-GO" }], {
        maxOccurrences: 1,
      }),
    /trop d'occurrences/i,
  );
});

test("produit les trois groupes avec une sortie bornée", () => {
  const report = scanEntries([
    {
      file: "sample.ts",
      content: [
        'const visible = "CL-GO";',
        'const internal = "clgo-theme";',
        "const unknown = `temporary cl-go branding`;",
      ].join("\n"),
    },
  ]);
  const output = formatReport(report, { maxItemsPerGroup: 1 });

  assert.equal(report.visible.length, 1);
  assert.equal(report.internal.length, 1);
  assert.equal(report.unknown.length, 1);
  assert.match(output, /VISIBLE À RENOMMER/);
  assert.match(output, /INTERNE À CONSERVER/);
  assert.match(output, /INCONNU ET BLOQUANT/);
});
