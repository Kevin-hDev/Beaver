#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import ts from "typescript";
import { scanRust } from "./check-provider-routing-rust.mjs";

const ROOTS = ["src", "src-tauri/src"];
const MAX_SOURCE_FILES = 10_000;
const ROUTING_NAMES = new Set([
  "provider",
  "providerId",
  "provider_id",
  "chatProviderId",
  "chat_provider_id",
  "canonicalProviderId",
  "canonical_provider_id",
  "connectionId",
  "connection_id",
  "routeId",
  "route_id",
]);

export function isProductionSource(filePath) {
  const normalized = filePath.split(path.sep).join("/");
  if (!/\.(?:rs|tsx?)$/.test(normalized)) return false;
  return !(
    /(?:^|\/)tests?(?:\/|$)/.test(normalized)
    || /(?:^|\/)fixtures?(?:\/|$)/.test(normalized)
    || /(?:^|\/)__tests__(?:\/|$)/.test(normalized)
    || /(?:_tests\.rs|\.test\.tsx?)$/.test(normalized)
    || /(?:^|\/)tests\.rs$/.test(normalized)
    || /(?:^|\/)generated(?:\/|$)/.test(normalized)
    || /(?:^|\/)test-utils(?:\/|$)/.test(normalized)
    || normalized.endsWith(".generated.ts")
    || normalized.startsWith("src-tauri/src/services/forecast/")
    || normalized === "src-tauri/src/commands/forecast.rs"
    || normalized.startsWith("src-tauri/src/services/search/")
    || normalized.startsWith("src/components/forecast/")
  );
}

export function validateAllowlist(value) {
  if (!Array.isArray(value)) throw new Error("allowlist invalide: tableau attendu");
  const paths = new Set();
  for (const entry of value) {
    const valid = entry
      && typeof entry.path === "string"
      && entry.path.length > 0
      && typeof entry.owner === "string"
      && entry.owner.trim().length > 0
      && typeof entry.reason === "string"
      && entry.reason.trim().length > 0
      && Number.isSafeInteger(entry.max_decisions)
      && entry.max_decisions >= 0
      && (entry.remove_after_task === undefined
        || (typeof entry.remove_after_task === "string" && entry.remove_after_task.length > 0));
    if (!valid || paths.has(entry?.path)) {
      throw new Error("allowlist invalide: entrée incomplète ou dupliquée");
    }
    paths.add(entry.path);
  }
  return value;
}

export function compareDecisionCounts(report, allowlist) {
  const byPath = new Map(allowlist.map((entry) => [entry.path, entry]));
  const reportedPaths = new Set(report.map((entry) => entry.path));
  const failures = [];
  for (const item of report) {
    const allowed = byPath.get(item.path);
    if (!allowed) {
      failures.push(`${item.path}: ${item.decisions} décision(s) sans propriétaire`);
    } else if (item.decisions > allowed.max_decisions) {
      failures.push(`${item.path}: ${item.decisions} > ${allowed.max_decisions}`);
    } else if (item.decisions < allowed.max_decisions) {
      failures.push(`${item.path}: borne obsolète ${allowed.max_decisions}, valeur ${item.decisions}`);
    }
  }
  for (const entry of allowlist) {
    if (entry.max_decisions > 0 && !reportedPaths.has(entry.path)) {
      failures.push(`${entry.path}: autorisation périmée`);
    }
  }
  return failures;
}

export function scanSource({ path: filePath, source }) {
  if (!isProductionSource(filePath)) return [];
  return filePath.endsWith(".rs")
    ? scanRust(source)
    : scanTypeScript(filePath, source);
}

function scanTypeScript(filePath, source) {
  const scriptKind = filePath.endsWith(".tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS;
  const root = ts.createSourceFile(filePath, source, ts.ScriptTarget.Latest, true, scriptKind);
  const decisions = [];
  const visit = (node) => {
    if (ts.isBinaryExpression(node) && isEquality(node.operatorToken.kind)) {
      if (containsRoutingIdentity(node.left) || containsRoutingIdentity(node.right)) {
        decisions.push({ kind: "comparison", line: root.getLineAndCharacterOfPosition(node.getStart(root)).line + 1 });
      }
    } else if (ts.isSwitchStatement(node) && containsRoutingIdentity(node.expression)) {
      decisions.push({ kind: "match", line: root.getLineAndCharacterOfPosition(node.getStart(root)).line + 1 });
    } else if (ts.isCallExpression(node) && isNamedModelPredicate(node)) {
      decisions.push({ kind: "named_predicate", line: root.getLineAndCharacterOfPosition(node.getStart(root)).line + 1 });
    }
    ts.forEachChild(node, visit);
  };
  visit(root);
  return decisions;
}

function isEquality(kind) {
  return kind === ts.SyntaxKind.EqualsEqualsEqualsToken
    || kind === ts.SyntaxKind.ExclamationEqualsEqualsToken
    || kind === ts.SyntaxKind.EqualsEqualsToken
    || kind === ts.SyntaxKind.ExclamationEqualsToken;
}

function containsRoutingIdentity(node) {
  if (ts.isIdentifier(node)) return ROUTING_NAMES.has(node.text);
  if (ts.isPropertyAccessExpression(node)) return ROUTING_NAMES.has(node.name.text);
  if (ts.isElementAccessExpression(node) && ts.isStringLiteral(node.argumentExpression)) {
    return ROUTING_NAMES.has(node.argumentExpression.text);
  }
  return false;
}

function isNamedModelPredicate(node) {
  const name = ts.isIdentifier(node.expression)
    ? node.expression.text
    : ts.isPropertyAccessExpression(node.expression)
      ? node.expression.name.text
      : "";
  return name.startsWith("is")
    && node.arguments.some((argument) => ts.isIdentifier(argument) && /^(?:model|modelId|model_id)$/.test(argument.text));
}

function productionFiles(rootDirectory) {
  const files = [];
  const pending = ROOTS.map((root) => path.join(rootDirectory, root));
  while (pending.length > 0) {
    const directory = pending.pop();
    if (!directory || !fs.existsSync(directory)) continue;
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      if (files.length + pending.length >= MAX_SOURCE_FILES) {
        throw new Error("scan provider borné dépassé");
      }
      const absolute = path.join(directory, entry.name);
      if (entry.isDirectory()) pending.push(absolute);
      else files.push(path.relative(rootDirectory, absolute));
    }
  }
  return files.filter(isProductionSource).sort();
}

function run() {
  const rootDirectory = process.cwd();
  const allowlistPath = path.join(rootDirectory, "scripts/providers/provider-branch-allowlist.json");
  const allowlist = validateAllowlist(JSON.parse(fs.readFileSync(allowlistPath, "utf8")));
  const report = [];
  for (const filePath of productionFiles(rootDirectory)) {
    const source = fs.readFileSync(path.join(rootDirectory, filePath), "utf8");
    const decisions = scanSource({ path: filePath, source });
    if (decisions.length === 0) continue;
    report.push({ path: filePath, decisions: decisions.length });
  }
  if (process.argv.includes("--print-baseline")) {
    console.log(JSON.stringify(report, null, 2));
    return;
  }
  const failures = compareDecisionCounts(report, allowlist);
  if (failures.length > 0) {
    console.error(failures.slice(0, 64).join("\n"));
    process.exitCode = 1;
    return;
  }
  console.log(`Provider routing boundaries: ${allowlist.length} fichiers autorisés.`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) run();
