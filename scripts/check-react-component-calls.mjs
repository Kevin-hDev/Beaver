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

function listTypeScriptFiles(dir) {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    const stat = statSync(path);
    if (stat.isDirectory()) return listTypeScriptFiles(path);
    return /\.tsx?$/u.test(path) ? [path] : [];
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

function findExtensionUiBoundaryViolations(file) {
  if (!file.startsWith("src/features/extension-ui/") || file.includes("/__tests__/")) return [];
  const source = readFileSync(file, "utf8");
  const sourceFile = ts.createSourceFile(file, source, ts.ScriptTarget.Latest, true);
  const findings = [];
  function visit(node) {
    if (ts.isImportDeclaration(node) && ts.isStringLiteral(node.moduleSpecifier)) {
      const imported = node.moduleSpecifier.text;
      if (imported.startsWith("@/components/") && !imported.startsWith("@/components/ui/")) {
        findings.push(`${location(sourceFile, node.moduleSpecifier)} ${imported}`);
      }
    }
    ts.forEachChild(node, visit);
  }
  visit(sourceFile);
  return findings;
}

const findings = listTsxFiles(SRC_DIR).flatMap(findDirectComponentCalls);
const boundaryFindings = listTypeScriptFiles("src/features/extension-ui")
  .flatMap(findExtensionUiBoundaryViolations);

if (findings.length > 0) {
  console.error("Direct React component-style calls are forbidden in .tsx files.");
  console.error("Use JSX (<Component />) or rename slot logic to a hook such as useComponentSlots().");
  console.error("");
  for (const finding of findings) console.error(`- ${finding}`);
  process.exit(1);
}

if (boundaryFindings.length > 0) {
  console.error("Extension UI features may import only shared UI primitives.");
  for (const finding of boundaryFindings) console.error(`- ${finding}`);
  process.exit(1);
}
