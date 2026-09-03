import { createHash } from "node:crypto";
import { lstat, realpath, rename, writeFile } from "node:fs/promises";
import { basename, dirname, extname, isAbsolute, relative, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { build } from "esbuild";

const ERROR_MESSAGE = "Extension UI build failed";
const MAX_OUTPUTS = 64;
const MAX_ARTIFACT_BYTES = 4_194_304;
const MAX_STDOUT_BYTES = 1_048_576;
const OUTPUT_TYPES = new Map([
  [".js", "javascript"],
  [".css", "css"],
  [".png", "png"],
  [".jpg", "jpeg"],
  [".jpeg", "jpeg"],
  [".webp", "webp"],
  [".gif", "gif"],
  [".woff2", "woff2"],
]);
const LOADERS = {
  ".png": "file",
  ".jpg": "file",
  ".jpeg": "file",
  ".webp": "file",
  ".gif": "file",
  ".woff2": "file",
  ".svg": "file",
  ".bin": "file",
};

export async function buildExtensionUi(options) {
  try {
    return await buildValidated(options);
  } catch {
    throw new Error(ERROR_MESSAGE);
  }
}

async function buildValidated(options) {
  if (!options || typeof options !== "object") fail();
  const inputRoot = await canonicalDirectory(options.inputRoot);
  const outputRoot = await canonicalDirectory(options.outputRoot);
  const entry = await canonicalInput(inputRoot, options.entry);
  if (inputRoot === outputRoot || inside(inputRoot, outputRoot) || inside(outputRoot, inputRoot)) fail();

  const result = await build({
    absWorkingDir: inputRoot,
    entryPoints: [entry],
    bundle: true,
    format: "esm",
    platform: "browser",
    write: false,
    metafile: true,
    logLevel: "silent",
    outdir: outputRoot,
    entryNames: "[name]-[hash]",
    assetNames: "[name]-[hash]",
    loader: LOADERS,
  });
  const inputs = await validateInputs(inputRoot, Object.keys(result.metafile.inputs));
  const outputs = await validateOutputs(outputRoot, result.outputFiles);
  await writeOutputs(outputRoot, result.outputFiles);
  const unsigned = { version: 1, outputs, inputs };
  const manifest = {
    ...unsigned,
    manifestSha256: sha256(Buffer.from(JSON.stringify(unsigned))),
  };
  if (Buffer.byteLength(JSON.stringify(manifest)) >= MAX_STDOUT_BYTES) fail();
  return manifest;
}

async function canonicalDirectory(value) {
  if (typeof value !== "string" || !value || value.length > 32_768) fail();
  const absolute = resolve(value);
  const [info, canonical] = await Promise.all([lstat(absolute), realpath(absolute)]);
  if (!info.isDirectory() || info.isSymbolicLink() || canonical !== absolute) fail();
  return canonical;
}

async function canonicalInput(root, value) {
  if (typeof value !== "string" || !value || value.length > 4_096 || isAbsolute(value)) fail();
  const candidate = resolve(root, value);
  if (!inside(candidate, root)) fail();
  const [info, canonical] = await Promise.all([lstat(candidate), realpath(candidate)]);
  if (!info.isFile() || info.isSymbolicLink() || canonical !== candidate || !inside(canonical, root)) fail();
  return canonical;
}

async function validateInputs(root, inputNames) {
  if (inputNames.length === 0 || inputNames.length > 8_192) fail();
  const validated = [];
  for (const name of inputNames) {
    const absolute = resolve(root, name);
    if (!inside(absolute, root)) fail();
    const [info, canonical] = await Promise.all([lstat(absolute), realpath(absolute)]);
    if (!info.isFile() || info.isSymbolicLink() || canonical !== absolute || !inside(canonical, root)) fail();
    validated.push(relative(root, canonical).split("\\").join("/"));
  }
  return validated.sort();
}

async function validateOutputs(root, outputFiles) {
  if (!Array.isArray(outputFiles) || outputFiles.length === 0 || outputFiles.length > MAX_OUTPUTS) fail();
  let totalBytes = 0;
  const outputs = [];
  for (const output of outputFiles) {
    const name = basename(output.path);
    if (dirname(output.path) !== root || !/^[A-Za-z0-9_.-]{1,160}$/u.test(name)) fail();
    const type = OUTPUT_TYPES.get(extname(name).toLowerCase());
    if (!type) fail();
    totalBytes = checkedAdd(totalBytes, output.contents.byteLength);
    outputs.push({ name, type, bytes: output.contents.byteLength, sha256: sha256(output.contents) });
  }
  if (totalBytes > MAX_ARTIFACT_BYTES) fail();
  return outputs.sort((left, right) => left.name.localeCompare(right.name));
}

async function writeOutputs(root, outputFiles) {
  for (const output of outputFiles) {
    const name = basename(output.path);
    const destination = resolve(root, name);
    const temporary = resolve(root, `.${name}.${sha256(output.contents)}.tmp`);
    if (!inside(destination, root) || !inside(temporary, root)) fail();
    await writeFile(temporary, output.contents, { flag: "wx", mode: 0o600 });
    await rename(temporary, destination);
  }
}

function inside(candidate, root) {
  const child = relative(root, candidate);
  return child !== "" && !child.startsWith("..") && !isAbsolute(child);
}

function checkedAdd(left, right) {
  const total = left + right;
  if (!Number.isSafeInteger(total) || total < left) fail();
  return total;
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function fail() {
  throw new Error(ERROR_MESSAGE);
}

function parseArguments(values) {
  if (values.length !== 6) fail();
  const parsed = Object.create(null);
  for (let index = 0; index < values.length; index += 2) {
    const flag = values[index];
    if (!new Set(["--input-root", "--output-root", "--entry"]).has(flag) || parsed[flag]) fail();
    parsed[flag] = values[index + 1];
  }
  if (!parsed["--input-root"] || !parsed["--output-root"] || !parsed["--entry"]) fail();
  return { inputRoot: parsed["--input-root"], outputRoot: parsed["--output-root"], entry: parsed["--entry"] };
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  buildExtensionUi(parseArguments(process.argv.slice(2))).then((manifest) => {
    process.stdout.write(`${JSON.stringify(manifest)}\n`);
  }).catch(() => {
    process.stderr.write(`${ERROR_MESSAGE}\n`);
    process.exitCode = 1;
  });
}
