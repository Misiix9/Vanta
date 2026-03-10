#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(process.cwd(), 'vanta-extensions', 'extensions');
const names = fs.readdirSync(root, { withFileTypes: true }).filter((d) => d.isDirectory()).map((d) => d.name);

let failures = 0;
for (const name of names) {
  const manifestPath = path.join(root, name, 'manifest.json');
  const bundlePath = path.join(root, name, 'dist', 'index.js');
  if (!fs.existsSync(manifestPath)) {
    console.error(`[${name}] missing manifest.json`);
    failures += 1;
    continue;
  }

  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  if (!Array.isArray(manifest.commands) || manifest.commands.length === 0) {
    console.error(`[${name}] has no commands`);
    failures += 1;
  }

  if (!fs.existsSync(bundlePath)) {
    console.error(`[${name}] missing dist/index.js`);
    failures += 1;
  }
}

if (failures > 0) {
  console.error(`Harness failed with ${failures} issue(s).`);
  process.exit(1);
}

console.log(`Harness passed for ${names.length} extension(s).`);
