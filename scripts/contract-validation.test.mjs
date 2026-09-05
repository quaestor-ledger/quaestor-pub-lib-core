import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";
import { fileURLToPath } from "node:url";

import Ajv2020 from "ajv/dist/2020.js";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const authoredPath = path.join(root, "contracts", "json-schema", "public-core.schema.json");
const generatedRoot = path.join(root, "generated", "typespec", "json-schema");
const fixtures = JSON.parse(
  fs.readFileSync(path.join(root, "conformance", "public-core-v1.json"), "utf8"),
);

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function validators() {
  const authored = readJson(authoredPath);
  const authoredAjv = new Ajv2020({ allErrors: true, strict: true });
  authoredAjv.addSchema(authored);

  const typeSpecAjv = new Ajv2020({ allErrors: true, strict: true });
  for (const filename of fs.readdirSync(generatedRoot).filter((name) => name.endsWith(".json"))) {
    typeSpecAjv.addSchema(readJson(path.join(generatedRoot, filename)));
  }

  return {
    authored: {
      clientInfo: authoredAjv.compile({ $ref: `${authored.$id}#/$defs/ClientInfo` }),
      idempotencyKey: authoredAjv.compile({ $ref: `${authored.$id}#/$defs/IdempotencyKey` }),
    },
    typeSpec: {
      clientInfo: typeSpecAjv.getSchema("ClientInfo.json"),
      idempotencyKey: typeSpecAjv.getSchema("IdempotencyKey.json"),
    },
  };
}

test("both independent authorities accept and reject every shared case identically", () => {
  const lanes = validators();
  for (const [kind, cases] of Object.entries(fixtures.valid)) {
    for (const value of cases) {
      for (const [lane, byKind] of Object.entries(lanes)) {
        assert.equal(byKind[kind](value), true, `${lane} rejected valid ${kind}: ${JSON.stringify(byKind[kind].errors)}`);
      }
    }
  }

  for (const [kind, cases] of Object.entries(fixtures.invalid)) {
    for (const { case: caseName, value } of cases) {
      for (const [lane, byKind] of Object.entries(lanes)) {
        assert.equal(byKind[kind](value), false, `${lane} accepted invalid ${kind} case: ${caseName}`);
      }
    }
  }
});

test("agreement fails closed when the TypeSpec projection loses a required field", () => {
  const temporaryRoot = fs.mkdtempSync(path.join(os.tmpdir(), "quaestor-public-drift-"));
  fs.cpSync(generatedRoot, temporaryRoot, { recursive: true });
  const clientInfoPath = path.join(temporaryRoot, "ClientInfo.json");
  const clientInfo = readJson(clientInfoPath);
  clientInfo.required = clientInfo.required.filter((name) => name !== "installId");
  fs.writeFileSync(clientInfoPath, `${JSON.stringify(clientInfo, null, 2)}\n`);

  const result = spawnSync(process.execPath, [path.join(root, "scripts", "authority-agreement.mjs")], {
    cwd: root,
    encoding: "utf8",
    env: { ...process.env, QUAESTOR_PUBLIC_TYPESPEC_DIR: temporaryRoot },
  });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /authority mismatch: ClientInfo/);
});
