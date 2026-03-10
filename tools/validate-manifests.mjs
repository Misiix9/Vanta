#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(process.cwd(), 'vanta-extensions', 'extensions');
const entries = fs.readdirSync(root, { withFileTypes: true }).filter((d) => d.isDirectory());

const required = ['name', 'title', 'schema_version', 'version', 'author', 'publisher', 'safe', 'permissions', 'commands'];
let failures = 0;

for (const dir of entries) {
  const manifestPath = path.join(root, dir.name, 'manifest.json');
  const raw = fs.readFileSync(manifestPath, 'utf8');
  const manifest = JSON.parse(raw);

  for (const key of required) {
    if (!(key in manifest)) {
      console.error(`[${dir.name}] missing field: ${key}`);
      failures += 1;
    }
  }

  if (manifest.publisher !== 'Vanta Team') {
    console.error(`[${dir.name}] publisher must be Vanta Team`);
    failures += 1;
  }

  if (manifest.safe !== true) {
    console.error(`[${dir.name}] safe must be true`);
    failures += 1;
  }

  if (!String(manifest.version || '').match(/^\d+\.\d+\.\d+$/)) {
    console.error(`[${dir.name}] version must be semver`);
    failures += 1;
  }
}

if (failures > 0) {
  console.error(`Manifest validation failed with ${failures} issue(s).`);
  process.exit(1);
}

console.log(`Validated ${entries.length} manifests successfully.`);
