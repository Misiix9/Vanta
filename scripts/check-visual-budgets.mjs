import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import process from "node:process";

const root = process.cwd();

const themeFiles = [
  "src-tauri/resources/themes/base.css",
  "src-tauri/resources/themes/default.css",
  "src-tauri/resources/themes/universal.css",
];

// Phase 44: budgets raised to accommodate scoped-to-global CSS consolidation
const budgets = {
  maxInlineStyleUses: 16,
  maxCombinedThemeBytes: 150_000,
  maxBaseThemeBytes: 72_000,
  maxDefaultThemeBytes: 50_000,
  maxUniversalThemeBytes: 35_000,
};

function countInlineStyles(content) {
  return (content.match(/style=/g) || []).length;
}

function collectSvelteFiles(dir, acc = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const abs = join(dir, entry.name);
    if (entry.isDirectory()) {
      collectSvelteFiles(abs, acc);
    } else if (entry.isFile() && entry.name.endsWith(".svelte")) {
      acc.push(abs);
    }
  }
  return acc;
}

function fail(message) {
  console.error(`visual-budget: FAIL - ${message}`);
  process.exitCode = 1;
}

let totalInlineStyleUses = 0;
const svelteFiles = collectSvelteFiles(join(root, "src"))
  .map((absPath) => absPath.slice(root.length + 1))
  .sort();

for (const relPath of svelteFiles) {
  const absPath = join(root, relPath);
  const content = readFileSync(absPath, "utf8");
  const count = countInlineStyles(content);
  totalInlineStyleUses += count;
  if (count > 0) {
    console.log(`visual-budget: ${relPath} inline-style-uses=${count}`);
  }
}

const pageContent = readFileSync(join(root, "src/routes/+page.svelte"), "utf8");
if (countInlineStyles(pageContent) !== 0) {
  fail("src/routes/+page.svelte must not use inline style attributes");
}

if (totalInlineStyleUses > budgets.maxInlineStyleUses) {
  fail(
    `total inline style usage ${totalInlineStyleUses} exceeds budget ${budgets.maxInlineStyleUses}`,
  );
}

let combinedThemeBytes = 0;
for (const relPath of themeFiles) {
  const bytes = statSync(join(root, relPath)).size;
  combinedThemeBytes += bytes;
  console.log(`visual-budget: ${relPath} bytes=${bytes}`);
}

const baseBytes = statSync(join(root, themeFiles[0])).size;
const defaultBytes = statSync(join(root, themeFiles[1])).size;
const universalBytes = statSync(join(root, themeFiles[2])).size;

if (baseBytes > budgets.maxBaseThemeBytes) {
  fail(`base.css size ${baseBytes} exceeds budget ${budgets.maxBaseThemeBytes}`);
}
if (defaultBytes > budgets.maxDefaultThemeBytes) {
  fail(`default.css size ${defaultBytes} exceeds budget ${budgets.maxDefaultThemeBytes}`);
}
if (universalBytes > budgets.maxUniversalThemeBytes) {
  fail(`universal.css size ${universalBytes} exceeds budget ${budgets.maxUniversalThemeBytes}`);
}
if (combinedThemeBytes > budgets.maxCombinedThemeBytes) {
  fail(
    `combined theme size ${combinedThemeBytes} exceeds budget ${budgets.maxCombinedThemeBytes}`,
  );
}

if (process.exitCode) {
  process.exit(process.exitCode);
}

console.log("visual-budget: PASS");
