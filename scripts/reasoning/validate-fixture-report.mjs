import { readdir, readFile } from 'node:fs/promises';
import { join } from 'node:path';

const ROOT = new URL('../../docs/providers/reasoning/fixtures/', import.meta.url);
const MAX_BYTES = 256 * 1024;

export function validateReport(name, value, bytes) {
  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}\.json$/i.test(name) || bytes > MAX_BYTES) throw new Error('invalid fixture report');
  if (!value || value.schema_version !== 1 || !value.fixture_id || !value.route || !value.model || !value.reasoning_mode || !value.generated_at) throw new Error('invalid fixture report');
  if (`${value.fixture_id}.json` !== name) throw new Error('invalid fixture report');
  if (!Array.isArray(value.scenarios) || value.scenarios.length > 64) throw new Error('invalid fixture report');
  for (const scenario of value.scenarios) {
    if (!['passe', 'echoue', 'bloque'].includes(scenario.status)
      || !Number.isInteger(scenario.request_count)
      || !Number.isInteger(scenario.reasoning_event_count)
      || !Array.isArray(scenario.decisions)) throw new Error('invalid fixture report');
  }
  const text = JSON.stringify(value);
  if (/("(?:session_id|thinking|reasoning_content|encrypted_content|signature|authorization|api[_-]?key|token|body)"|(?:^|[^a-z]):?\/[A-Za-z0-9_.-]+\/)/i.test(text)) throw new Error('sensitive fixture report');
  return true;
}

export async function validateDirectory(root = ROOT) {
  const names = await readdir(root).catch(() => []);
  for (const name of names) {
    const data = await readFile(join(root, name));
    validateReport(name, JSON.parse(data), data.byteLength);
  }
}

if (process.argv[1] === new URL(import.meta.url).pathname) await validateDirectory();
