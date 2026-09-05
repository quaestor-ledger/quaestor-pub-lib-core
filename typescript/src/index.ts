/** Public, transport-neutral values that are safe to ship to untrusted devices. */

export const CONTRACT_VERSION = "quaestor.pub-lib-core.v1" as const;
export const CLIENT_PLATFORMS = ["browser", "desktop", "ios", "android"] as const;

export type ClientPlatform = (typeof CLIENT_PLATFORMS)[number];

export interface ClientInfo {
  readonly platform: ClientPlatform;
  readonly appVersion: string;
  readonly locale?: string;
  readonly installId: string;
}

export interface IdempotencyKey {
  readonly key: string;
  readonly mintedAtMs: number;
}

export type PublicCoreErrorCode =
  | "expected_object"
  | "unexpected_field"
  | "invalid_platform"
  | "invalid_app_version"
  | "invalid_locale"
  | "invalid_install_id"
  | "invalid_idempotency_key"
  | "invalid_minted_at";

export class PublicCoreValidationError extends Error {
  readonly code: PublicCoreErrorCode;

  constructor(code: PublicCoreErrorCode, message: string) {
    super(message);
    this.name = "PublicCoreValidationError";
    this.code = code;
  }
}

function record(value: unknown): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new PublicCoreValidationError("expected_object", "value must be an object");
  }
  return value as Record<string, unknown>;
}

function exactKeys(value: Record<string, unknown>, allowed: readonly string[]): void {
  const accepted = new Set(allowed);
  const unexpected = Object.keys(value).find((key) => !accepted.has(key));
  if (unexpected !== undefined) {
    throw new PublicCoreValidationError("unexpected_field", `unexpected field: ${unexpected}`);
  }
}

function requiredString(value: unknown, code: PublicCoreErrorCode, message: string): string {
  if (typeof value !== "string") throw new PublicCoreValidationError(code, message);
  return value;
}

export function parseClientInfo(input: unknown): Readonly<ClientInfo> {
  const value = record(input);
  exactKeys(value, ["platform", "appVersion", "locale", "installId"]);

  if (typeof value.platform !== "string" || !CLIENT_PLATFORMS.includes(value.platform as ClientPlatform)) {
    throw new PublicCoreValidationError("invalid_platform", "platform is not supported");
  }
  const appVersion = requiredString(
    value.appVersion,
    "invalid_app_version",
    "appVersion must be a printable ASCII string",
  );
  if (!/^[!-~]{1,64}$/.test(appVersion)) {
    throw new PublicCoreValidationError("invalid_app_version", "appVersion must be 1-64 printable ASCII characters");
  }
  const installId = requiredString(
    value.installId,
    "invalid_install_id",
    "installId must be a bounded opaque identifier",
  );
  if (!/^[A-Za-z0-9_-]{16,128}$/.test(installId)) {
    throw new PublicCoreValidationError("invalid_install_id", "installId must match the public contract");
  }

  if (value.locale === undefined) {
    return Object.freeze({ platform: value.platform as ClientPlatform, appVersion, installId });
  }
  const locale = requiredString(value.locale, "invalid_locale", "locale must be a string");
  if (!/^[A-Za-z]{2,3}(?:-[A-Za-z0-9]{2,8})*$/.test(locale) || locale.length > 35) {
    throw new PublicCoreValidationError("invalid_locale", "locale must match the public contract");
  }
  return Object.freeze({ platform: value.platform as ClientPlatform, appVersion, locale, installId });
}

export function parseIdempotencyKey(input: unknown): Readonly<IdempotencyKey> {
  const value = record(input);
  exactKeys(value, ["key", "mintedAtMs"]);
  const key = requiredString(
    value.key,
    "invalid_idempotency_key",
    "key must be a bounded opaque identifier",
  );
  if (!/^[A-Za-z0-9_-]{16,128}$/.test(key)) {
    throw new PublicCoreValidationError("invalid_idempotency_key", "key must match the public contract");
  }
  if (
    typeof value.mintedAtMs !== "number"
    || !Number.isSafeInteger(value.mintedAtMs)
    || value.mintedAtMs < 0
  ) {
    throw new PublicCoreValidationError(
      "invalid_minted_at",
      "mintedAtMs must be a non-negative, exactly portable JSON integer",
    );
  }
  return Object.freeze({ key, mintedAtMs: value.mintedAtMs });
}
