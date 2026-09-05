#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const authoredPath = path.join(root, "contracts", "json-schema", "public-core.schema.json");
const typeSpecPath = path.join(root, "contracts", "typespec", "main.tsp");
const generatedRoot = process.env.QUAESTOR_PUBLIC_TYPESPEC_DIR
  ? path.resolve(process.env.QUAESTOR_PUBLIC_TYPESPEC_DIR)
  : path.join(root, "generated", "typespec", "json-schema");
const artifactPath = path.join(root, "artifacts", "public-authority-agreement.json");
const checkOnly = process.argv.includes("--check");

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function sha256(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

function sortedObject(entries) {
  return Object.fromEntries([...entries].sort(([left], [right]) => left.localeCompare(right)));
}

const authored = readJson(authoredPath);
const authoredDefs = authored.$defs ?? {};
const emitted = new Map();

function emittedSchema(name) {
  if (!emitted.has(name)) {
    const file = path.join(generatedRoot, `${name}.json`);
    if (!fs.existsSync(file)) throw new Error(`missing TypeSpec schema: ${file}`);
    emitted.set(name, readJson(file));
  }
  return emitted.get(name);
}

function resolveAuthored(schema) {
  if (typeof schema?.$ref !== "string") return schema;
  const prefix = "#/$defs/";
  if (!schema.$ref.startsWith(prefix)) throw new Error(`unsupported authored $ref: ${schema.$ref}`);
  const name = schema.$ref.slice(prefix.length);
  if (!Object.hasOwn(authoredDefs, name)) throw new Error(`missing authored $def: ${name}`);
  return authoredDefs[name];
}

function resolveTypeSpec(schema) {
  if (typeof schema?.$ref !== "string") return schema;
  if (!schema.$ref.endsWith(".json") || schema.$ref.includes("/") || schema.$ref.includes("..")) {
    throw new Error(`unsupported TypeSpec $ref: ${schema.$ref}`);
  }
  return emittedSchema(schema.$ref.slice(0, -5));
}

function normalize(schema, resolve) {
  const value = resolve(schema);
  if (value.type === "object") {
    const properties = sortedObject(
      Object.entries(value.properties ?? {}).map(([name, property]) => [name, normalize(property, resolve)]),
    );
    const closed = value.additionalProperties === false
      || (value.unevaluatedProperties?.not
        && Object.keys(value.unevaluatedProperties.not).length === 0);
    return {
      type: "object",
      closed: Boolean(closed),
      required: [...(value.required ?? [])].sort(),
      properties,
    };
  }

  const normalized = { type: value.type };
  for (const key of ["minimum", "maximum", "minLength", "maxLength", "pattern"]) {
    if (value[key] !== undefined) normalized[key] = value[key];
  }
  if (Array.isArray(value.enum)) normalized.enum = [...value.enum].sort();
  return normalized;
}

const compared = ["ClientPlatform", "ClientInfo", "IdempotencyKey"];
const normalized = {};
const mismatches = [];
for (const name of compared) {
  if (!Object.hasOwn(authoredDefs, name)) throw new Error(`missing authored public definition: ${name}`);
  const jsonSchema = normalize(authoredDefs[name], resolveAuthored);
  const typeSpec = normalize(emittedSchema(name), resolveTypeSpec);
  normalized[name] = { jsonSchema, typeSpec };
  if (JSON.stringify(jsonSchema) !== JSON.stringify(typeSpec)) mismatches.push(name);
}

if (mismatches.length > 0) {
  for (const name of mismatches) {
    console.error(`authority mismatch: ${name}`);
    console.error(`  JSON Schema: ${JSON.stringify(normalized[name].jsonSchema)}`);
    console.error(`  TypeSpec:    ${JSON.stringify(normalized[name].typeSpec)}`);
  }
  process.exit(1);
}

const emittedDigests = sortedObject(
  fs.readdirSync(generatedRoot)
    .filter((name) => name.endsWith(".json"))
    .map((name) => [name, sha256(fs.readFileSync(path.join(generatedRoot, name)))]),
);
const evidence = {
  contract: "quaestor.pub-lib-core.v1",
  authorities: {
    jsonSchema: {
      path: "contracts/json-schema/public-core.schema.json",
      sha256: sha256(fs.readFileSync(authoredPath)),
    },
    typeSpec: {
      path: "contracts/typespec/main.tsp",
      sha256: sha256(fs.readFileSync(typeSpecPath)),
    },
  },
  emittedTypeSpecSha256: emittedDigests,
  compared,
  normalized,
};
const serialized = `${JSON.stringify(evidence, null, 2)}\n`;

if (checkOnly) {
  const current = fs.existsSync(artifactPath) ? fs.readFileSync(artifactPath, "utf8") : null;
  if (current !== serialized) {
    console.error("stale: artifacts/public-authority-agreement.json");
    process.exit(1);
  }
  console.log(`Public authorities agree (${compared.length} definitions; evidence current).`);
} else {
  fs.mkdirSync(path.dirname(artifactPath), { recursive: true });
  fs.writeFileSync(artifactPath, serialized);
  console.log(`Public authorities agree (${compared.length} definitions); evidence written.`);
}
