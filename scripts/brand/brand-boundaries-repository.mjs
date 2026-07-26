import { execFileSync } from "node:child_process";
import { lstatSync, readFileSync } from "node:fs";
import { extname, isAbsolute, resolve, sep } from "node:path";

export const MAX_REPOSITORY_FILES = 5_000;
export const MAX_TEXT_FILE_BYTES = 2 * 1024 * 1024;

const TEXT_EXTENSIONS = new Set([
  ".css", ".desktop", ".html", ".js", ".json", ".md", ".mjs", ".plist",
  ".nsh", ".ps1", ".py", ".rs", ".sh", ".svg", ".toml", ".ts", ".tsx", ".txt",
  ".xml", ".yaml", ".yml",
]);

export function validateTrackedPath(file) {
  if (
    typeof file !== "string" ||
    file.length === 0 ||
    file.length > 4_096 ||
    /[\u0000-\u001f\u007f]/.test(file) ||
    isAbsolute(file) ||
    /^[A-Za-z]:[\\/]/.test(file) ||
    file.split(/[\\/]/).some((part) => part === ".." || part === ".")
  ) {
    throw new Error("chemin suivi invalide");
  }
  return file;
}

function shouldScan(file) {
  return (
    !file.startsWith("graphify-out/") &&
    !file.startsWith("scripts/brand/") &&
    file !== "docs/BEAVER-RENAME-PLAN.md" &&
    file !== "package-lock.json" &&
    file !== "src-tauri/Cargo.lock" &&
    TEXT_EXTENSIONS.has(extname(file).toLowerCase())
  );
}

function parseTrackedFiles(raw) {
  const files = [];
  let start = 0;
  for (let cursor = 0; cursor <= raw.length; cursor += 1) {
    if (cursor < raw.length && raw[cursor] !== "\0") continue;
    if (cursor > start) {
      if (files.length >= MAX_REPOSITORY_FILES) throw new Error("trop de fichiers suivis");
      files.push(raw.slice(start, cursor));
    }
    start = cursor + 1;
  }
  return files;
}

export function loadTrackedEntries(root) {
  const raw = gitFileList(
    root,
    ["ls-files", "-z", "--cached", "--others", "--exclude-standard"],
  );
  const deleted = new Set(parseTrackedFiles(gitFileList(root, ["ls-files", "-z", "--deleted"])));
  const files = parseTrackedFiles(raw);
  const safeRoot = resolve(root);
  return files.filter((file) => !deleted.has(file)).filter(shouldScan).map((file) => {
    validateTrackedPath(file);
    const absolute = resolve(safeRoot, file);
    if (!absolute.startsWith(`${safeRoot}${sep}`)) throw new Error("chemin suivi invalide");
    const stats = lstatSync(absolute);
    if (!stats.isFile() || stats.isSymbolicLink() || stats.size > MAX_TEXT_FILE_BYTES) {
      throw new Error("fichier texte invalide");
    }
    return { file, content: readFileSync(absolute, "utf8") };
  });
}

function gitFileList(root, args) {
  return execFileSync(
    "git",
    args,
    {
      cwd: root,
      encoding: "utf8",
      maxBuffer: 8 * 1024 * 1024,
      shell: false,
    },
  );
}
