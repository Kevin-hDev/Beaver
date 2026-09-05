import { lstat, mkdir } from "node:fs/promises";
import { join } from "node:path";

const ARTIFACT_ERROR = "E2E artifact directory is invalid";
const ARTIFACT_ROOT = ".e2e-artifacts";
const CONFIGURED_DIRECTORY = /^\.e2e-artifacts\/([a-z0-9][a-z0-9-]{0,63})$/u;

export async function prepareE2eArtifactDirectory(repositoryRoot, configured) {
  if (configured === undefined) return undefined;
  if (!validPath(repositoryRoot) || typeof configured !== "string") {
    throw new Error(ARTIFACT_ERROR);
  }
  const match = configured.match(CONFIGURED_DIRECTORY);
  if (!match) throw new Error(ARTIFACT_ERROR);

  const root = join(repositoryRoot, ARTIFACT_ROOT);
  await ensureOwnedDirectory(root, true);
  const destination = join(root, match[1]);
  await ensureOwnedDirectory(destination, false);
  return destination;
}

async function ensureOwnedDirectory(path, recursive) {
  try {
    await mkdir(path, { recursive, mode: 0o700 });
    const metadata = await lstat(path);
    if (!metadata.isDirectory() || metadata.isSymbolicLink()) {
      throw new Error(ARTIFACT_ERROR);
    }
  } catch {
    throw new Error(ARTIFACT_ERROR);
  }
}

function validPath(value) {
  return typeof value === "string"
    && value.length > 0
    && value.length <= 32_768
    && !/[\0\r\n]/u.test(value);
}
