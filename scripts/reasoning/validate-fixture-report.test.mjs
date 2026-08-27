import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fixtureId, validateDirectory, validateReport } from './validate-fixture-report.mjs';

const id = 'ollama-local-qwen-us-east-1-2026-08-26';
const report = {
  schema_version: 1,
  fixture_id: id,
  route: 'ollama',
  model: 'qwen',
  region: 'us-east-1',
  reasoning_mode: 'auto',
  generated_at: '2026-08-26T00:00:00Z',
  scenarios: [
    { requirement: 'capture_and_persist', status: 'passe', request_count: 1, reasoning_event_count: 2, decisions: ['reasoning decision="captured"', 'reasoning decision="persisted"'] },
    { requirement: 'replay_and_continue', status: 'passe', request_count: 1, reasoning_event_count: 3, decisions: ['reasoning decision="replayed"', 'reasoning decision="captured"', 'reasoning decision="persisted"'] },
  ],
};

test('accepts a bounded anonymous fixture report', () => assert.equal(validateReport(`${id}.json`, report, 100), true));
test('rejects sensitive payload fields', () => assert.throws(() => validateReport(`${id}.json`, { ...report, session_id: 'secret' }, 100)));
test('rejects missing counters and noncanonical names', () => {
  assert.throws(() => validateReport(`${id}.json`, { ...report, scenarios: [{ status: 'passe' }] }, 100));
  assert.throws(() => validateReport('ollama-qwen.json', report, 100));
});

test('rejects a completed scenario without observed replay', () => {
  const scenarios = report.scenarios.map(scenario => ({ ...scenario, decisions: scenario.decisions.filter(value => !value.includes('replayed')) }));
  assert.throws(() => validateReport(`${id}.json`, { ...report, scenarios }, 100));
});

test('directory validation fails closed when reports are absent', async () => {
  const root = await mkdtemp(join(tmpdir(), 'beaver-reasoning-'));
  try {
    await assert.rejects(() => validateDirectory(root));
    await writeFile(join(root, `${id}.json`), JSON.stringify(report));
    await validateDirectory(root);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test('checked-in reports validate through the default CLI directory', async () => {
  await validateDirectory();
});

test('rejects traversal and IDs not derived from exact route model region and date', () => {
  assert.throws(() => fixtureId('ollama', 'qwen', '../escape', report.generated_at));
  assert.throws(() => fixtureId('ollama', 'qwen', 'us-east-1', '2026-13-26T00:00:00Z'));
  assert.throws(() => validateReport(`${id}.json`, { ...report, fixture_id: 'other' }, 100));
});
