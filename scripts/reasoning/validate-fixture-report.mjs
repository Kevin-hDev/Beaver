import { readdir, readFile } from 'node:fs/promises';
import { join } from 'node:path';

const ROOT = new URL('../../docs/providers/reasoning/fixtures/', import.meta.url);
const MAX_BYTES = 256 * 1024;
const MAX_FIXTURE_ID_BYTES = 240;
const PROVIDER_ROUTES = Object.freeze({
  ollama: ['ollama', 'local'], google: ['google', 'api'], mistral: ['mistral', 'api'],
  cerebras: ['cerebras', 'api'], openrouter: ['openrouter', 'api'], openai: ['openai', 'api'],
  deepseek: ['deepseek', 'api'], xai: ['xai', 'api'], 'xai-oauth': ['xai', 'oauth'],
  moonshot: ['moonshot', 'api'], 'moonshot-oauth': ['moonshot', 'oauth'], zai: ['zai', 'api'],
  'codex-oauth': ['codex', 'oauth'], groq: ['groq', 'api'],
});

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
  const providerRoute = PROVIDER_ROUTES[route];
  const date = generatedAt.slice(0, 10);
  if (!providerRoute || !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(region)
    || region.length > 32 || !/^\d{4}-\d{2}-\d{2}T/.test(generatedAt)
    || !new Date(`${date}T00:00:00Z`).toISOString().startsWith(date)) throw new Error('invalid fixture report');
  const id = [...providerRoute, canonicalComponent(model, 'model'), region, date].join('-');
  if (id.length > MAX_FIXTURE_ID_BYTES || !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(id)) throw new Error('invalid fixture report');
  return id;
}

export function validateReport(name, value, bytes) {
  if (bytes > MAX_BYTES) throw new Error('invalid fixture report');
  if (!value || value.schema_version !== 1 || !value.fixture_id || !value.route || !value.model || !value.region || !value.reasoning_mode || !value.generated_at) throw new Error('invalid fixture report');
  const expectedFixtureId = fixtureId(value.route, value.model, value.region, value.generated_at);
  if (`${expectedFixtureId}.json` !== name || value.fixture_id !== expectedFixtureId) throw new Error('invalid fixture report');
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
