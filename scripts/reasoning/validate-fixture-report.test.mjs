import test from 'node:test';
import assert from 'node:assert/strict';
import { fixtureId, validateReport } from './validate-fixture-report.mjs';

const id = 'ollama-local-qwen-us-east-1-2026-08-26';
const report = {
  schema_version: 1,
  fixture_id: id,
  route: 'ollama',
  model: 'qwen',
  region: 'us-east-1',
  reasoning_mode: 'auto',
  generated_at: '2026-08-26T00:00:00Z',
  scenarios: [{ status: 'passe', request_count: 1, reasoning_event_count: 1, decisions: ['captured'] }],
};

test('accepts a bounded anonymous fixture report', () => assert.equal(validateReport(`${id}.json`, report, 100), true));
test('rejects sensitive payload fields', () => assert.throws(() => validateReport(`${id}.json`, { ...report, session_id: 'secret' }, 100)));
test('rejects missing counters and noncanonical names', () => {
  assert.throws(() => validateReport(`${id}.json`, { ...report, scenarios: [{ status: 'passe' }] }, 100));
  assert.throws(() => validateReport('ollama-qwen.json', report, 100));
});

test('rejects traversal and IDs not derived from exact route model region and date', () => {
  assert.throws(() => fixtureId('ollama', 'qwen', '../escape', report.generated_at));
  assert.throws(() => fixtureId('ollama', 'qwen', 'us-east-1', '2026-13-26T00:00:00Z'));
  assert.throws(() => validateReport(`${id}.json`, { ...report, fixture_id: 'other' }, 100));
});
