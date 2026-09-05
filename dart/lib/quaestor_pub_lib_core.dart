/// Public, transport-neutral values that are safe to ship to untrusted devices.
library;

const String contractVersion = 'quaestor.pub-lib-core.v1';
const int _maximumPortableJsonInteger = 9007199254740991;

/// Untrusted-device runtime family.
enum ClientPlatform {
  browser,
  desktop,
  ios,
  android;

  static ClientPlatform parse(Object? value) {
    if (value is! String) {
      throw const PublicCoreValidationException(
        'invalid_platform',
        'platform is not supported',
      );
    }
    for (final platform in values) {
      if (platform.name == value) return platform;
    }
    throw const PublicCoreValidationException(
      'invalid_platform',
      'platform is not supported',
    );
  }
}

/// Typed failure for malformed public input.
final class PublicCoreValidationException implements Exception {
  const PublicCoreValidationException(this.code, this.message);

  final String code;
  final String message;

  @override
  String toString() => 'PublicCoreValidationException($code): $message';
}

/// Bounded metadata supplied by an untrusted client installation.
final class ClientInfo {
  ClientInfo({
    required this.platform,
    required String appVersion,
    String? locale,
    required String installId,
  })  : appVersion = _appVersion(appVersion),
        locale = _locale(locale),
        installId = _opaqueToken(
          installId,
          'invalid_install_id',
          'installId',
        );

  factory ClientInfo.fromJson(Object? input) {
    final value = _record(input);
    _exactKeys(value, const {'platform', 'appVersion', 'locale', 'installId'});
    return ClientInfo(
      platform: ClientPlatform.parse(value['platform']),
      appVersion:
          _string(value['appVersion'], 'invalid_app_version', 'appVersion'),
      locale: value['locale'] == null
          ? null
          : _string(value['locale'], 'invalid_locale', 'locale'),
      installId: _string(value['installId'], 'invalid_install_id', 'installId'),
    );
  }

  final ClientPlatform platform;
  final String appVersion;
  final String? locale;
  final String installId;

  Map<String, Object?> toJson() => <String, Object?>{
        'platform': platform.name,
        'appVersion': appVersion,
        if (locale != null) 'locale': locale,
        'installId': installId,
      };
}

/// Client-minted retry identity. It conveys no authorization or permission.
final class IdempotencyKey {
  IdempotencyKey({required String key, required int mintedAtMs})
      : key = _opaqueToken(
          key,
          'invalid_idempotency_key',
          'key',
        ),
        mintedAtMs = _mintedAt(mintedAtMs);

  factory IdempotencyKey.fromJson(Object? input) {
    final value = _record(input);
    _exactKeys(value, const {'key', 'mintedAtMs'});
    final mintedAtMs = value['mintedAtMs'];
    if (mintedAtMs is! int) {
      throw const PublicCoreValidationException(
        'invalid_minted_at',
        'mintedAtMs must be an exactly portable JSON integer',
      );
    }
    return IdempotencyKey(
      key: _string(value['key'], 'invalid_idempotency_key', 'key'),
      mintedAtMs: mintedAtMs,
    );
  }

  final String key;
  final int mintedAtMs;

  Map<String, Object?> toJson() => <String, Object?>{
        'key': key,
        'mintedAtMs': mintedAtMs,
      };
}

Map<String, Object?> _record(Object? value) {
  if (value is! Map<String, Object?>) {
    throw const PublicCoreValidationException(
      'expected_object',
      'value must be an object',
    );
  }
  return value;
}

void _exactKeys(Map<String, Object?> value, Set<String> allowed) {
  for (final key in value.keys) {
    if (!allowed.contains(key)) {
      throw PublicCoreValidationException(
        'unexpected_field',
        'unexpected field: $key',
      );
    }
  }
}

String _string(Object? value, String code, String field) {
  if (value is! String) {
    throw PublicCoreValidationException(code, '$field must be a string');
  }
  return value;
}

String _appVersion(String value) {
  final valid = value.isNotEmpty &&
      value.length <= 64 &&
      value.codeUnits.every((unit) => unit >= 0x21 && unit <= 0x7e);
  if (!valid) {
    throw const PublicCoreValidationException(
      'invalid_app_version',
      'appVersion must be 1-64 printable ASCII characters',
    );
  }
  return value;
}

String? _locale(String? value) {
  if (value == null) return null;
  final valid = value.length <= 35 &&
      RegExp(r'^[A-Za-z]{2,3}(?:-[A-Za-z0-9]{2,8})*$').hasMatch(value);
  if (!valid) {
    throw const PublicCoreValidationException(
      'invalid_locale',
      'locale must match the public contract',
    );
  }
  return value;
}

String _opaqueToken(String value, String code, String field) {
  if (!RegExp(r'^[A-Za-z0-9_-]{16,128}$').hasMatch(value)) {
    throw PublicCoreValidationException(
        code, '$field must match the public contract');
  }
  return value;
}

int _mintedAt(int value) {
  if (value < 0 || value > _maximumPortableJsonInteger) {
    throw const PublicCoreValidationException(
      'invalid_minted_at',
      'mintedAtMs must be an exactly portable JSON integer',
    );
  }
  return value;
}
