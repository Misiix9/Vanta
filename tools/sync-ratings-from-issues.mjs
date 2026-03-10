#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const args = new Map(process.argv.slice(2).map((x) => {
  const [k, v] = x.split('=');
  return [k.replace(/^--/, ''), v ?? ''];
}));

const owner = args.get('owner');
const repo = args.get('repo');
const token = process.env.GITHUB_TOKEN;

if (!owner || !repo || !token) {
  console.error('Usage: node tools/sync-ratings-from-issues.mjs --owner=<owner> --repo=<repo> (requires GITHUB_TOKEN)');
  process.exit(1);
}

const url = `https://api.github.com/repos/${owner}/${repo}/issues?state=open&labels=rating&per_page=100`;
const response = await fetch(url, {
  headers: {
    Authorization: `Bearer ${token}`,
    Accept: 'application/vnd.github+json',
    'User-Agent': 'vanta-ratings-sync'
  }
});

if (!response.ok) {
  throw new Error(`GitHub API failed: ${response.status}`);
}

const issues = await response.json();
const buckets = new Map();

for (const issue of issues) {
  const title = String(issue.title || '');
  const match = title.match(/^Rating:\s+([a-z0-9-_]+)\s+\((\d)\s+stars\)/i);
  if (!match) continue;
  const name = match[1];
  const score = Number(match[2]);
  if (score < 1 || score > 5) continue;
  const item = buckets.get(name) || { sum: 0, count: 0 };
  item.sum += score;
  item.count += 1;
  buckets.set(name, item);
}

const ratings = Array.from(buckets.entries())
  .map(([name, data]) => ({
    name,
    average: Number((data.sum / data.count).toFixed(2)),
    count: data.count
  }))
  .sort((a, b) => a.name.localeCompare(b.name));

const out = {
  schema_version: 1,
  source: 'community',
  updated_at: new Date().toISOString(),
  ratings
};

const outPath = path.resolve(process.cwd(), 'vanta-extensions', 'ratings.json');
fs.writeFileSync(outPath, JSON.stringify(out, null, 2) + '\n', 'utf8');
console.log(`Wrote ${ratings.length} rating entries to ${outPath}`);
