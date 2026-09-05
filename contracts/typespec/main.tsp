import "@typespec/json-schema";

namespace Quaestor.PublicCore.V1;

/** Untrusted-device runtime family. */
enum ClientPlatform {
  browser: "browser",
  desktop: "desktop",
  ios: "ios",
  android: "android",
}

/** Printable application version label. */
@minLength(1)
@maxLength(64)
@pattern("^[!-~]+$")
scalar AppVersion extends string;

/** Optional bounded BCP-47-shaped locale hint. */
@minLength(2)
@maxLength(35)
@pattern("^[A-Za-z]{2,3}(?:-[A-Za-z0-9]{2,8})*$")
scalar LocaleHint extends string;

/** Pseudonymous per-installation identifier; never a person or organization identifier. */
@minLength(16)
@maxLength(128)
@pattern("^[A-Za-z0-9_-]+$")
scalar InstallId extends string;

/** Client-minted retry token with no authorization meaning. */
@minLength(16)
@maxLength(128)
@pattern("^[A-Za-z0-9_-]+$")
scalar RetryToken extends string;

/** Bounded metadata supplied by an untrusted client installation. */
model ClientInfo {
  platform: ClientPlatform;
  appVersion: AppVersion;
  locale?: LocaleHint;
  installId: InstallId;
}

/** Client-minted retry identity. It conveys no authorization or permission. */
model IdempotencyKey {
  key: RetryToken;

  /** Unix epoch milliseconds within the exactly portable JSON integer range. */
  @minValue(0)
  @maxValue(9007199254740991)
  mintedAtMs: safeint;
}
