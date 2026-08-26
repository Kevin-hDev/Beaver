import { readdir, readFile } from 'node:fs/promises';
import { join } from 'node:path';

const ROOT = new URL('../../docs/providers/reasoning/fixtures/', import.meta.url);
const MAX_BYTES = 256 * 1024;

export function validateReport(name, value, bytes) {
  if (!/^[a-z0-9][a-z0-9_-]{0,127}\.json$/.test(name) || bytes > MAX_BYTES) throw new Error('invalid fixture report');
  if (!value || value.schema_version !== 1 || !value.route || !value.model) throw new Error('invalid fixture report');
  if (!Array.isArray(value.scenarios) || value.scenarios.length > 64) throw new Error('invalid fixture report');
  for (const scenario of value.scenarios) {
    if (!['passe', 'echoue', 'bloque'].includes(scenario.status) || !scenario.counters) throw new Error('invalid fixture report');
  }
  const text = JSON.stringify(value);
  if (/(session_id|encrypted_content|api[_-]?key|authorization|\/[A-Za-z0-9_.-]+\/)/i.test(text)) throw new Error('sensitive fixture report');
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
