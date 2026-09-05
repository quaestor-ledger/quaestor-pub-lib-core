# quaestor-pub-lib-core

Public, transport-neutral Quaestor primitives for browser, desktop, iOS, and
Android consumers. Everything distributed from this repository is treated as
attacker-visible and attacker-controlled.

Quaestor Ledger observes, records, and proves accounting facts; it never moves
money. Shared Auth establishes identity, realm, and assurance. Quaestor services
still own and enforce organizations, workspaces, memberships, roles, billing
relationships, and resource authorization. A value accepted by this library is
well-formed input, never proof of permission.

## Current bounded surface

Version `quaestor.pub-lib-core.v1` contains only:

- `ClientPlatform` and `ClientInfo`: bounded, pseudonymous installation metadata;
- `IdempotencyKey`: a bounded retry identity and an exactly portable JSON mint
  timestamp.

The first slice is intentionally small. It does not contain database rows, SQL,
migrations, ORM entities, session records, provider credentials, administrator
models, internal endpoints, transport clients, or environment configuration.
Those concerns stay in private service, interface, and ORM repositories.

## Independent contract authorities

The public contract has two human-authored peers:

- [`contracts/json-schema/public-core.schema.json`](contracts/json-schema/public-core.schema.json)
  is JSON Schema Draft 2020-12;
- [`contracts/typespec/main.tsp`](contracts/typespec/main.tsp) is TypeSpec.

Neither file is generated from the other. TypeSpec is compiled with a pinned
official JSON Schema emitter. `scripts/authority-agreement.mjs` independently
normalizes field names, requiredness, closed-object behavior, enum values,
types, ranges, lengths, and patterns; disagreement fails before it can refresh
the content-addressed evidence in
[`artifacts/public-authority-agreement.json`](artifacts/public-authority-agreement.json).

The negative agreement test removes a required TypeSpec-projected field and
proves the comparator fails closed. AJV then executes every positive and
negative case against both authorities.

## Three runtime implementations

Rust, TypeScript, and Dart consume the same checked-in cases from
[`conformance/public-core-v1.json`](conformance/public-core-v1.json). Each
runtime rejects unknown fields and every invalid enum, string constraint, or
integer bound in the shared fixture.

Rust:

```rust
use quaestor_pub_lib_core::{ClientInfo, ClientPlatform};

let info = ClientInfo::new(
    ClientPlatform::Desktop,
    "1.4.0",
    Some("en-US"),
    "desktop_install_01",
)?;
```

TypeScript:

```ts
import { parseClientInfo } from "@quaestor-ledger/pub-lib-core";

const info = parseClientInfo({
  platform: "browser",
  appVersion: "1.4.0",
  locale: "en-US",
  installId: "browser_install_01",
});
```

Dart:

```dart
import 'package:quaestor_pub_lib_core/quaestor_pub_lib_core.dart';

final info = ClientInfo.fromJson({
  'platform': 'android',
  'appVersion': '1.4.0',
  'installId': 'android-install-00000042',
});
```

## Fail-closed publication boundary

`tools/public-boundary` is a dependency-free Rust policy executable. It walks
an exact allowlist of distributable files and rejects:

- any unclassified file or symlink under a public output root;
- credential-, session-, administrator-, database-, or ORM-shaped names;
- database/network/environment dependencies in the Rust crate;
- any TypeScript runtime dependency, any Dart dependency, or any TypeScript
  development dependency except the exact pinned compiler;
- drift from the root-anchored Cargo publication allowlist.

Its tests synthesize a credential-shaped field and prove it is rejected. Cargo
packaging is separately inspected so a broad include glob cannot sweep tooling
or dependency metadata into the public crate.

## Verify

Requires Node.js 22+, Rust 1.85+, and Dart 3.3+:

```sh
npm ci --ignore-scripts
npm run generate
npm test
cargo package --allow-dirty --list
```

`npm test` runs both contract authorities, the negative drift proof, TypeScript,
Rust, Dart, Rustfmt, Clippy with warnings denied, Dart analysis, deterministic
generation checks, and the publication boundary.

This repository is public source and does not carry deployment credentials or a
live provider configuration. The broader Quaestor feature-parity program remains
tracked in [DEN-1143](https://linear.app/denman/issue/DEN-1143/quaestor-ledger-competitive-parity-survey-comparable-ledgerpayment).
