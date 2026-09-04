import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

const MAX_OUTPUT_CHARS = 64 * 1024;
export const SERVICE_UNAVAILABLE_EXIT_CODE = 75;
const TRANSIENT_PATTERNS = [
  /503 Service Unavailable/iu,
  /audit endpoint returned an error/iu,
  /ECONNRESET/u,
  /EAI_AGAIN/u,
  /ETIMEDOUT/u,
];

export function isTransientAuditFailure(output) {
  return TRANSIENT_PATTERNS.some((pattern) => pattern.test(output));
}

export async function runAuditWithRetry({
  execute = executeNpmAudit,
  report = (message) => process.stderr.write(`${message}\n`),
  wait = (delayMs) => new Promise((resolve) => setTimeout(resolve, delayMs)),
  maxAttempts = 2,
} = {}) {
  for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
    const result = await execute();
    if (result.exitCode === 0) return;
    const transient = isTransientAuditFailure(result.output);
    if (!transient || attempt === maxAttempts) {
      const kind = transient ? "service_unavailable" : "advisory_failure";
      const error = new Error(`npm_audit_${kind}`);
      error.exitCode = transient ? SERVICE_UNAVAILABLE_EXIT_CODE : 1;
      throw error;
    }
    report(`npm advisory service unavailable; retrying (${attempt}/${maxAttempts}).`);
    await wait(2_000);
  }
}

async function executeNpmAudit() {
  return new Promise((resolve, reject) => {
    let output = "";
    const child = spawn("npm", ["audit", "--audit-level=high"], {
      env: {
        ...process.env,
        NPM_CONFIG_FETCH_RETRIES: "0",
        NPM_CONFIG_FETCH_TIMEOUT: "90000",
      },
      stdio: ["ignore", "pipe", "pipe"],
    });
    const forward = (target, chunk) => {
      target.write(chunk);
      output = `${output}${chunk}`.slice(-MAX_OUTPUT_CHARS);
    };
    child.stdout.on("data", (chunk) => forward(process.stdout, chunk));
    child.stderr.on("data", (chunk) => forward(process.stderr, chunk));
    child.once("error", reject);
    child.once("close", (code) => resolve({ exitCode: code ?? 1, output }));
  });
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  runAuditWithRetry().catch((error) => {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = error.exitCode ?? 1;
  });
}
