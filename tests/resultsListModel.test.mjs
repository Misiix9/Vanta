import test from 'node:test';
import assert from 'node:assert/strict';
import { buildVisualRows } from '../src/lib/resultsListModel.js';

const baseResults = [
  {
    title: 'Space Terminal',
    subtitle: 'Workspace 1',
    icon: null,
    exec: 'space-terminal',
    score: 10,
    match_indices: [],
    source: 'Window',
    id: 1,
    group: 'Space App',
  },
  {
    title: 'Space Logs',
    subtitle: 'Workspace 1',
    icon: null,
    exec: 'space-logs',
    score: 9,
    match_indices: [],
    source: 'Window',
    id: 2,
    group: 'Space App',
  },
  {
    title: 'Readme.md',
    subtitle: '/home/user/project',
    icon: null,
    exec: '/home/user/project/Readme.md',
    score: 8,
    match_indices: [],
    source: 'File',
    id: 3,
    group: null,
  },
  {
    title: 'Loose Window',
    subtitle: 'Workspace 2',
    icon: null,
    exec: 'loose-window',
    score: 7,
    match_indices: [],
    source: 'Window',
    id: 4,
    group: null,
  },
];

test('groups windows under headers and preserves order', () => {
  const { rows } = buildVisualRows(baseResults, new Set());
  assert.equal(rows.length, 6);
  assert.deepEqual(
    rows.map((r) => r.type),
    ['header', 'item', 'item', 'item', 'header', 'item'],
  );

  const firstHeader = rows[0];
  assert.equal(firstHeader.type, 'header');
  assert.equal(firstHeader.label, 'Space App');
  assert.equal(firstHeader.count, 2);

  const loneHeader = rows[4];
  assert.equal(loneHeader.type, 'header');
  assert.equal(loneHeader.label, 'Loose Window');
  assert.equal(loneHeader.count, 1);
});

test('collapsed groups hide their items but keep the header count', () => {
  const collapsed = new Set(['Space App']);
  const { rows } = buildVisualRows(baseResults, collapsed);

  assert.deepEqual(
    rows.map((r) => r.type),
    ['header', 'item', 'header', 'item'],
  );

  const [firstHeader] = rows;
  assert.equal(firstHeader.count, 2);
});
