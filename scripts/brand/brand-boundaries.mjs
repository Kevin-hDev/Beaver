import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { COMPATIBILITY_CONTRACTS } from "./brand-boundaries-contracts.mjs";
import {
  loadTrackedEntries,
  MAX_REPOSITORY_FILES,
  MAX_TEXT_FILE_BYTES,
  validateTrackedPath,
} from "./brand-boundaries-repository.mjs";

export { validateTrackedPath };

const BRAND_PATTERN = /CL-GO-DASH|CL-GO|cl-go-dash|cl_go_dash|CLGO|clgo|cl-go/g;
const VISIBLE_VALUES = new Set(["CL-GO-DASH", "CL-GO"]);
const INTERNAL_VALUES = new Set(["cl-go-dash", "cl_go_dash", "CLGO", "clgo"]);
const PACKAGE_COMPATIBILITY_FILES = new Set([
  "install.sh",
  "scripts/release/check-deb-migration.sh",
  "scripts/release/check-nsis-migration.ps1",
  "scripts/release/package-migration.test.mjs",
  "scripts/test-install-sh.sh",
  "src-tauri/tauri.conf.json",
  "src-tauri/windows/nsis-hooks.nsh",
]);
const VISIBLE_COMPATIBILITY_FILES = new Set([
  "scripts/release/check-bridge-metadata.mjs",
  "scripts/release/check-bridge-metadata.test.mjs",
  "scripts/release/publish-bridge-release.mjs",
  "scripts/release/publish-bridge-release.test.mjs",
  "src-tauri/src/services/autostart_migration.rs",
  "src-tauri/src/services/autostart_migration_tests.rs",
  ...PACKAGE_COMPATIBILITY_FILES,
]);
const MAX_OCCURRENCES = 10_000;
const MAX_CONTRACTS = 128;
const MAX_SNIPPETS_PER_CONTRACT = 32;
const DEFAULT_REPORT_ITEMS = 40;

function positiveLimit(value, fallback) {
  const candidate = value ?? fallback;
  if (!Number.isSafeInteger(candidate) || candidate < 1) {
    throw new Error("limite invalide");
  }
  return candidate;
}

export function classifyReference({ value, line, file }) {
  if (VISIBLE_COMPATIBILITY_FILES.has(file) && VISIBLE_VALUES.has(value)) return "internal";
  if (PACKAGE_COMPATIBILITY_FILES.has(file) && value === "cl-go") return "internal";
  if (VISIBLE_VALUES.has(value)) return "visible";
  if (INTERNAL_VALUES.has(value)) return "internal";
  if (value !== "cl-go" || typeof line !== "string") return "unknown";
  if (/["']\/cl-go["']/.test(line)) return "visible";
  if (/lower\.contains\("cl-go[^"]*"\)/.test(line)) return "visible";
  const internalContext = [
    /\.cl-go(?:[-/"'`]|$)/,
    /\.local\/share\/cl-go(?:[/="'`]|$)/,
    /migrated-from-cl-go/,
    /cl-go[-/:@]/,
    /[-/]cl-go(?:["'`)]|$)/,
    /(?:OsStr::new|basename)\(["']cl-go["']\)/,
  ];
  return internalContext.some((pattern) => pattern.test(line)) ? "internal" : "unknown";
}

function pushFinding(report, finding, maxOccurrences, count) {
  if (count >= maxOccurrences) throw new Error("trop d'occurrences de marque");
  report[finding.kind].push(finding);
}

function contextHash(line) {
  return createHash("sha256").update(line, "utf8").digest("hex");
}

export function scanEntries(entries, options = {}) {
  const maxFiles = positiveLimit(options.maxFiles, MAX_REPOSITORY_FILES);
  const maxFileBytes = positiveLimit(options.maxFileBytes, MAX_TEXT_FILE_BYTES);
  const maxOccurrences = positiveLimit(options.maxOccurrences, MAX_OCCURRENCES);
  if (!Array.isArray(entries) || entries.length > maxFiles) {
    throw new Error("trop de fichiers à analyser");
  }
  const report = { visible: [], internal: [], unknown: [] };
  let count = 0;
  for (const entry of entries) {
    const file = validateTrackedPath(entry?.file);
    if (typeof entry?.content !== "string") throw new Error("contenu invalide");
    if (Buffer.byteLength(entry.content, "utf8") > maxFileBytes) {
      throw new Error("fichier texte trop volumineux");
    }
    let start = 0;
    let lineNumber = 1;
    for (let cursor = 0; cursor <= entry.content.length; cursor += 1) {
      if (cursor < entry.content.length && entry.content[cursor] !== "\n") continue;
      const line = entry.content.slice(start, cursor).replace(/\r$/, "");
      BRAND_PATTERN.lastIndex = 0;
      for (let match = BRAND_PATTERN.exec(line); match; match = BRAND_PATTERN.exec(line)) {
        const kind = classifyReference({ value: match[0], line, file });
        pushFinding(
          report,
          {
            kind,
            file,
            line: lineNumber,
            column: match.index + 1,
            value: match[0],
            contextHash: contextHash(line),
          },
          maxOccurrences,
          count,
        );
        count += 1;
      }
      start = cursor + 1;
      lineNumber += 1;
    }
  }
  return report;
}

export function checkContracts(readText, contracts = COMPATIBILITY_CONTRACTS) {
  if (!Array.isArray(contracts) || contracts.length > MAX_CONTRACTS) {
    throw new Error("trop de contrats");
  }
  const failures = [];
  for (const item of contracts) {
    validateTrackedPath(item.file);
    if (!Array.isArray(item.snippets) || item.snippets.length > MAX_SNIPPETS_PER_CONTRACT) {
      throw new Error("contrat invalide");
    }
    let content;
    try {
      content = readText(item.file);
    } catch {
      failures.push(`${item.name} : fichier inaccessible`);
      continue;
    }
    if (typeof content !== "string" || Buffer.byteLength(content, "utf8") > MAX_TEXT_FILE_BYTES) {
      failures.push(`${item.name} : fichier invalide`);
      continue;
    }
    for (const snippet of item.snippets) {
      if (typeof snippet !== "string" || snippet.length === 0 || !content.includes(snippet)) {
        failures.push(`${item.name} : valeur de compatibilité absente`);
      }
    }
  }
  return failures;
}

export function formatReport(report, options = {}) {
  const maxItems = positiveLimit(options.maxItemsPerGroup, DEFAULT_REPORT_ITEMS);
  const groups = [
    ["visible", "VISIBLE À RENOMMER"],
    ["internal", "INTERNE À CONSERVER"],
    ["unknown", "INCONNU ET BLOQUANT"],
  ];
  const lines = [];
  for (const [key, title] of groups) {
    const findings = report[key];
    lines.push(`${title} (${findings.length})`);
    for (const item of findings.slice(0, maxItems)) {
      lines.push(`- ${item.file}:${item.line}:${item.column} — ${item.value}`);
    }
    if (findings.length > maxItems) lines.push(`- … ${findings.length - maxItems} autre(s)`);
    lines.push("");
  }
  return lines.join("\n").trimEnd();
}

function run() {
  const root = process.cwd();
  const readText = (file) => readFileSync(resolve(root, validateTrackedPath(file)), "utf8");
  const failures = checkContracts(readText);
  const report = scanEntries(loadTrackedEntries(root));
  console.log(formatReport(report));
  if (failures.length > 0) {
    console.error(`\nCONTRATS MODIFIÉS (${failures.length})`);
    for (const failure of failures.slice(0, DEFAULT_REPORT_ITEMS)) console.error(`- ${failure}`);
  }
  if (failures.length > 0 || report.unknown.length > 0) process.exitCode = 1;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    run();
  } catch {
    console.error("Contrôle des frontières de marque impossible.");
    process.exitCode = 1;
  }
}
