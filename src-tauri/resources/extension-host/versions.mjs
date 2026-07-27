import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const jitiPackage = require("jiti/package.json");

export const JITI_VERSION = String(jitiPackage.version);
