import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

import { readRegularFile } from "../file-system/regular-file.mjs";

const MAX_CATALOG_BYTES = 256 * 1024;

async function readCatalogue(path) {
  return readRegularFile(path, MAX_CATALOG_BYTES);
}

export async function checkPackagedVoiceCatalog(resourceDirectory, sourceFile) {
  const bundledFile = join(resourceDirectory, "resources", "voice-catalog.json");
  const [source, bundled] = await Promise.all([
    readCatalogue(sourceFile),
    readCatalogue(bundledFile),
  ]);
  if (!source.equals(bundled)) throw new Error("packaged voice catalogue differs from source");
}

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) {
  const [, , resourceDirectory, sourceFile] = process.argv;
  if (!resourceDirectory || !sourceFile) {
    console.error("usage: check-packaged-resources.mjs <resource-directory> <source-catalogue>");
    process.exitCode = 2;
  } else {
    checkPackagedVoiceCatalog(resourceDirectory, sourceFile).catch(() => {
      console.error("packaged voice catalogue verification failed");
      process.exitCode = 1;
    });
  }
}
