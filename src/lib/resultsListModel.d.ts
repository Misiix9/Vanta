import type { SearchResult } from './types';

export type GroupHeaderRow = {
  type: 'header';
  key: string;
  groupKey: string;
  label: string;
  count: number;
};

export type ResultRow = {
  type: 'item';
  key: string;
  groupKey: string | null;
  result: SearchResult;
  itemIndex: number;
};

export type VisualRow = GroupHeaderRow | ResultRow;

export function buildVisualRows(
  results: SearchResult[],
  collapsedGroups?: Set<string>,
): { rows: VisualRow[] };
