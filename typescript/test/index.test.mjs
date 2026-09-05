import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";

import {
  CONTRACT_VERSION,
  PublicCoreValidationError,
  parseClientInfo,
  parseIdempotencyKey,
} from "../dist/index.js";

const fixtures = JSON.parse(
  fs.readFileSync(new URL("../../conformance/public-core-v1.json", import.meta.url), "utf8"),
);

test("TypeScript accepts every shared valid case as an immutable value", () => {
  assert.equal(CONTRACT_VERSION, fixtures.contractVersion);
  for (const value of fixtures.valid.clientInfo) {
    const parsed = parseClientInfo(value);
    assert.deepEqual(parsed, value);
    assert.equal(Object.isFrozen(parsed), true);
  }
  for (const value of fixtures.valid.idempotencyKey) {
    const parsed = parseIdempotencyKey(value);
    assert.deepEqual(parsed, value);
    assert.equal(Object.isFrozen(parsed), true);
  }
});

test("TypeScript rejects every shared invalid case with a typed error", () => {
  for (const { case: caseName, value } of fixtures.invalid.clientInfo) {
    assert.throws(() => parseClientInfo(value), PublicCoreValidationError, caseName);
  }
  for (const { case: caseName, value } of fixtures.invalid.idempotencyKey) {
    assert.throws(() => parseIdempotencyKey(value), PublicCoreValidationError, caseName);
  }
});
