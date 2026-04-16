import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import process from "node:process";

const root = process.cwd();

const themeFiles = [
  "src-tauri/resources/themes/base.css",
  "src-tauri/resources/themes/default.css",
  "src-tauri/resources/themes/universal.css",
];

// Phase 52: budgets account for performance-tier fallback rules and hotspot guardrails
const budgets = {
  maxInlineStyleUses: 16,
  maxTransitionAllUses: 0,
  maxLayoutStyleDirectiveUses: 2,
  maxCombinedThemeBytes: 155_000,
  maxBaseThemeBytes: 75_000,
  maxDefaultThemeBytes: 50_000,
  maxUniversalThemeBytes: 35_000,
};

function countInlineStyles(content) {
  return (content.match(/style=/g) || []).length;
}

function countTransitionAll(content) {
  return (content.match(/transition\s*:\s*all\b/g) || []).length;
}

function countLayoutStyleDirectives(content) {
  return (
    content.match(/style:(top|left|right|bottom|width|height|margin|padding)\b/g) || []
  ).length;
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
let totalLayoutStyleDirectiveUses = 0;
const svelteFiles = collectSvelteFiles(join(root, "src"))
  .map((absPath) => absPath.slice(root.length + 1))
  .sort();

for (const relPath of svelteFiles) {
  const absPath = join(root, relPath);
  const content = readFileSync(absPath, "utf8");
  const count = countInlineStyles(content);
  const layoutDirectiveCount = countLayoutStyleDirectives(content);
  totalInlineStyleUses += count;
  totalLayoutStyleDirectiveUses += layoutDirectiveCount;
  if (count > 0) {
    console.log(`visual-budget: ${relPath} inline-style-uses=${count}`);
  }
  if (layoutDirectiveCount > 0) {
    console.log(`visual-budget: ${relPath} layout-style-directives=${layoutDirectiveCount}`);
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

if (totalLayoutStyleDirectiveUses > budgets.maxLayoutStyleDirectiveUses) {
  fail(
    `total layout style directives ${totalLayoutStyleDirectiveUses} exceeds budget ${budgets.maxLayoutStyleDirectiveUses}`,
  );
}

let totalTransitionAllUses = 0;
for (const relPath of themeFiles) {
  const content = readFileSync(join(root, relPath), "utf8");
  const transitionAllCount = countTransitionAll(content);
  totalTransitionAllUses += transitionAllCount;
  if (transitionAllCount > 0) {
    console.log(`visual-budget: ${relPath} transition-all-uses=${transitionAllCount}`);
  }
}

if (totalTransitionAllUses > budgets.maxTransitionAllUses) {
  fail(
    `theme transition-all usage ${totalTransitionAllUses} exceeds budget ${budgets.maxTransitionAllUses}`,
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
