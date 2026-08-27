import { readdir, readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('../../src-tauri/test-fixtures/reasoning-reports/', import.meta.url));
const MAX_BYTES = 256 * 1024;
const MAX_FIXTURE_ID_BYTES = 240;
const ROUTE = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

function canonicalComponent(value, fallback) {
  let result = '';
  let separator = false;
  for (const character of value) {
    if (/^[a-z0-9]$/i.test(character)) {
      result += character.toLowerCase();
      separator = false;
    } else if (!separator && result) {
      result += '-';
      separator = true;
    }
    if (result.length >= 96) break;
  }
  return result.replace(/-+$/, '') || fallback;
}

export function fixtureId(route, model, region, generatedAt) {
  if (!ROUTE.test(route)) throw new Error('invalid fixture report');
  const oauth = route.endsWith('-oauth');
  const provider = oauth ? route.slice(0, -6) : route;
  const transport = oauth ? 'oauth' : route === 'ollama' ? 'local' : 'api';
  const date = generatedAt.slice(0, 10);
  if (!provider || !ROUTE.test(region)
    || region.length > 32 || !/^\d{4}-\d{2}-\d{2}T/.test(generatedAt)
    || !new Date(`${date}T00:00:00Z`).toISOString().startsWith(date)) throw new Error('invalid fixture report');
  const id = [provider, transport, canonicalComponent(model, 'model'), region, date].join('-');
  if (id.length > MAX_FIXTURE_ID_BYTES || !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(id)) throw new Error('invalid fixture report');
  return id;
}

export function validateReport(name, value, bytes) {
  if (bytes > MAX_BYTES) throw new Error('invalid fixture report');
  if (!value || value.schema_version !== 1 || !value.fixture_id || !value.route || !value.model || !value.region || !value.reasoning_mode || !value.generated_at) throw new Error('invalid fixture report');
  const expectedFixtureId = fixtureId(value.route, value.model, value.region, value.generated_at);
  if (`${expectedFixtureId}.json` !== name || value.fixture_id !== expectedFixtureId) throw new Error('invalid fixture report');
  if (!Array.isArray(value.scenarios) || value.scenarios.length < 2 || value.scenarios.length > 64) throw new Error('invalid fixture report');
  for (const scenario of value.scenarios) {
    if (!['capture_and_persist', 'replay_and_continue'].includes(scenario.requirement)
      || !['passe', 'echoue', 'bloque'].includes(scenario.status)
      || !Number.isInteger(scenario.request_count) || scenario.request_count < 1
      || !Number.isInteger(scenario.reasoning_event_count)
      || !Array.isArray(scenario.decisions)) throw new Error('invalid fixture report');
    const has = decision => scenario.decisions.some(value => value.includes(`decision=\"${decision}\"`));
    const observed = scenario.requirement === 'capture_and_persist'
      ? has('captured') && has('persisted')
      : has('replayed') && has('captured') && has('persisted');
    if (scenario.status === 'passe' && !observed) throw new Error('unproved fixture scenario');
  }
  for (const requirement of ['capture_and_persist', 'replay_and_continue']) {
    if (!value.scenarios.some(scenario => scenario.requirement === requirement && scenario.status === 'passe')) {
      throw new Error('missing fixture proof');
    }
  }
  const text = JSON.stringify(value);
  if (/("(?:session_id|thinking|reasoning_content|encrypted_content|signature|authorization|api[_-]?key|token|body)"|(?:^|[^a-z]):?\/[A-Za-z0-9_.-]+\/)/i.test(text)) throw new Error('sensitive fixture report');
  return true;
}

export async function validateDirectory(root = ROOT) {
  const entries = await readdir(root, { withFileTypes: true });
  const names = entries.filter(entry => entry.isFile() && entry.name.endsWith('.json')).map(entry => entry.name);
  if (names.length === 0 || names.length !== entries.length) throw new Error('missing fixture reports');
  for (const name of names.sort()) {
    const data = await readFile(join(root, name));
    validateReport(name, JSON.parse(data), data.byteLength);
  }
}

if (process.argv[1] === new URL(import.meta.url).pathname) await validateDirectory();
