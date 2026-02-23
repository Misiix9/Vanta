/**
 * @typedef {import('./types').SearchResult} SearchResult
 * @typedef {{ type: 'header'; key: string; groupKey: string; label: string; count: number }} GroupHeaderRow
 * @typedef {{ type: 'item'; key: string; groupKey: string | null; result: SearchResult; itemIndex: number }} ResultRow
 * @typedef {GroupHeaderRow | ResultRow} VisualRow
 */

/**
 * Build visual rows (headers + items) for search results while preserving incoming order.
 * Collapsed groups still emit their headers but omit their items.
 *
 * @param {SearchResult[]} results
 * @param {Set<string>} [collapsedGroups]
 * @returns {{ rows: VisualRow[] }}
 */
export function buildVisualRows(results, collapsedGroups = new Set()) {
  const rows = /** @type {VisualRow[]} */ ([]);
  const headers = new Map();
  let itemIndex = 0;

  for (const result of results) {
    const rawGroup = typeof result.group === 'string' ? result.group.trim() : '';
    const groupKey = result.source === 'Window' ? rawGroup || result.title || result.exec : null;

    if (groupKey) {
      let header = headers.get(groupKey);
      if (!header) {
        header = {
          type: 'header',
          key: `header-${rows.length}-${groupKey}`,
          groupKey,
          label: rawGroup || groupKey,
          count: 0,
        };
        headers.set(groupKey, header);
        rows.push(header);
      }
      header.count += 1;
    }

    const itemRow = {
      type: 'item',
      key: `item-${groupKey ?? 'solo'}-${result.id ?? result.exec}-${itemIndex}`,
      groupKey,
      result,
      itemIndex,
    };

    if (!groupKey || !collapsedGroups.has(groupKey)) {
      rows.push(itemRow);
    }

    itemIndex += 1;
  }

  return { rows };
}
