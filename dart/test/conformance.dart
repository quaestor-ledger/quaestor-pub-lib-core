import 'dart:convert';
import 'dart:io';

import 'package:quaestor_pub_lib_core/quaestor_pub_lib_core.dart';

Never _fail(String message) => throw StateError(message);

void _expectFailure(String caseName, void Function() parse) {
  try {
    parse();
  } on PublicCoreValidationException {
    return;
  }
  _fail('accepted invalid case: $caseName');
}

void main() {
  final fixtureFile = File('../conformance/public-core-v1.json');
  final fixtures =
      jsonDecode(fixtureFile.readAsStringSync()) as Map<String, Object?>;
  if (fixtures['contractVersion'] != contractVersion) {
    _fail('contract version mismatch');
  }

  final valid = fixtures['valid']! as Map<String, Object?>;
  for (final input in valid['clientInfo']! as List<Object?>) {
    final parsed = ClientInfo.fromJson(input);
    if (jsonEncode(parsed.toJson()) != jsonEncode(input)) {
      _fail('ClientInfo round-trip changed a valid fixture');
    }
  }
  for (final input in valid['idempotencyKey']! as List<Object?>) {
    final parsed = IdempotencyKey.fromJson(input);
    if (jsonEncode(parsed.toJson()) != jsonEncode(input)) {
      _fail('IdempotencyKey round-trip changed a valid fixture');
    }
  }

  final invalid = fixtures['invalid']! as Map<String, Object?>;
  for (final item in invalid['clientInfo']! as List<Object?>) {
    final testCase = item! as Map<String, Object?>;
    _expectFailure(testCase['case']! as String,
        () => ClientInfo.fromJson(testCase['value']));
  }
  for (final item in invalid['idempotencyKey']! as List<Object?>) {
    final testCase = item! as Map<String, Object?>;
    _expectFailure(
      testCase['case']! as String,
      () => IdempotencyKey.fromJson(testCase['value']),
    );
  }

  stdout.writeln('Dart public-core conformance passed.');
}
