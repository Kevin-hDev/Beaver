import { copyFile, mkdir, opendir, stat } from "node:fs/promises";
import { join } from "node:path";
import { DEPENDENCY_COPY_LIMITS } from "./runtime-copy-limits.mjs";

export async function copyDirectoryBounded(
  source,
  destination,
  limits = DEPENDENCY_COPY_LIMITS,
) {
  validateLimits(limits);
  const state = { entries: 0, bytes: 0 };
  await copyLevel(source, destination, limits, state, 0);
}

async function copyLevel(source, destination, limits, state, depth) {
  if (depth > limits.maxDepth) throw new Error("Runtime dependency tree is too deep");
  await mkdir(destination, { recursive: true, mode: 0o700 });
  const directory = await opendir(source);
  for await (const entry of directory) {
    state.entries += 1;
    if (state.entries > limits.maxEntries) {
      throw new Error("Runtime dependency tree has too many entries");
    }
    const from = join(source, entry.name);
    const to = join(destination, entry.name);
    if (entry.isDirectory()) {
      await copyLevel(from, to, limits, state, depth + 1);
    } else if (entry.isFile()) {
      const metadata = await stat(from);
      state.bytes += metadata.size;
      if (state.bytes > limits.maxBytes) {
        throw new Error("Runtime dependency tree is too large");
      }
      await copyFile(from, to);
    } else {
      throw new Error("Unsupported runtime dependency entry");
    }
  }
}

function validateLimits(limits) {
  for (const value of [limits.maxEntries, limits.maxBytes, limits.maxDepth]) {
    if (!Number.isSafeInteger(value) || value < 1) {
      throw new Error("Invalid runtime copy limit");
    }
  }
}
