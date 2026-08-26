import test from 'node:test';
import assert from 'node:assert/strict';
import { validateReport } from './validate-fixture-report.mjs';

const report = { schema_version: 1, route: 'ollama', model: 'qwen', scenarios: [{ status: 'passe', counters: { reasoning_chars: 4 } }] };

test('accepts a bounded anonymous fixture report', () => assert.equal(validateReport('ollama-qwen.json', report, 100), true));
test('rejects sensitive payload fields', () => assert.throws(() => validateReport('ollama-qwen.json', { ...report, session_id: 'secret' }, 100)));
