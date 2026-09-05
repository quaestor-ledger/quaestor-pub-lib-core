#!/usr/bin/env node

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const outputRoot = path.join(root, "generated", "typespec");
const compiler = path.join(root, "node_modules", "@typespec", "compiler", "cmd", "tsp.js");
const checkOnly = process.argv.includes("--check");
const temporaryRoot = fs.mkdtempSync(path.join(os.tmpdir(), "quaestor-public-typespec-"));

const result = spawnSync(
  process.execPath,
  [compiler, "compile", "contracts/typespec", "--output-dir", temporaryRoot],
  { cwd: root, encoding: "utf8", stdio: "pipe" },
);
if (result.stdout) process.stdout.write(result.stdout);
if (result.stderr) process.stderr.write(result.stderr);
if (result.status !== 0) process.exit(result.status ?? 1);

function filesBelow(directory) {
  if (!fs.existsSync(directory)) return [];
  const files = [];
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const absolute = path.join(directory, entry.name);
    if (entry.isDirectory()) files.push(...filesBelow(absolute));
    else if (entry.isFile()) files.push(absolute);
  }
  return files.sort();
}

const expected = filesBelow(temporaryRoot).map((absolute) => ({
  relative: path.relative(temporaryRoot, absolute),
  content: fs.readFileSync(absolute),
}));
if (expected.length === 0) {
  console.error("error: TypeSpec compiled without producing an artifact");
  process.exit(1);
}

const expectedNames = new Set(expected.map(({ relative }) => relative));
const extra = filesBelow(outputRoot)
  .map((absolute) => path.relative(outputRoot, absolute))
  .filter((relative) => !expectedNames.has(relative));

let stale = 0;
for (const { relative, content } of expected) {
  const target = path.join(outputRoot, relative);
  const current = fs.existsSync(target) ? fs.readFileSync(target) : null;
  if (current?.equals(content)) continue;
  stale += 1;
  if (checkOnly) {
    console.error(`stale: generated/typespec/${relative}`);
  } else {
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, content);
    console.log(`wrote: generated/typespec/${relative}`);
  }
}

for (const relative of extra) {
  stale += 1;
  console.error(`unexpected generated artifact: generated/typespec/${relative}`);
}

if (stale > 0 && (checkOnly || extra.length > 0)) {
  console.error(`${stale} TypeSpec artifact(s) out of date.`);
  process.exit(1);
}

console.log(
  checkOnly
    ? `TypeSpec artifacts are current (${expected.length} files).`
    : `generated ${expected.length} TypeSpec artifacts (${stale} changed).`,
);
