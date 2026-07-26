import ts from "typescript";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

const SRC_DIR = "src";
const COMPONENT_NAME = /^[A-Z]/;
const ALLOWED_GLOBAL_CALLS = new Set([
  "Array",
  "Blob",
  "Boolean",
  "Date",
  "Error",
  "File",
  "FormData",
  "Intl",
  "Map",
  "MutationObserver",
  "Number",
  "Object",
  "Promise",
  "RegExp",
  "ResizeObserver",
  "Set",
  "String",
  "URL",
  "Uint8Array",
  "WeakMap",
  "WeakSet",
]);

function listTsxFiles(dir) {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    const stat = statSync(path);
    if (stat.isDirectory()) return listTsxFiles(path);
    return path.endsWith(".tsx") ? [path] : [];
  });
}

function location(sourceFile, node) {
  const pos = sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile));
  return `${sourceFile.fileName}:${pos.line + 1}:${pos.character + 1}`;
}

function findDirectComponentCalls(file) {
  const source = readFileSync(file, "utf8");
  const sourceFile = ts.createSourceFile(file, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
  const findings = [];

  function visit(node) {
    if (ts.isCallExpression(node) && ts.isIdentifier(node.expression)) {
      const name = node.expression.text;
      if (COMPONENT_NAME.test(name) && !ALLOWED_GLOBAL_CALLS.has(name)) {
        findings.push(`${location(sourceFile, node.expression)} ${name}(...)`);
      }
    }
    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  return findings;
}

const findings = listTsxFiles(SRC_DIR).flatMap(findDirectComponentCalls);

if (findings.length > 0) {
  console.error("Direct React component-style calls are forbidden in .tsx files.");
  console.error("Use JSX (<Component />) or rename slot logic to a hook such as useComponentSlots().");
  console.error("");
  for (const finding of findings) console.error(`- ${finding}`);
  process.exit(1);
}
