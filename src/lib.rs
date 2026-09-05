#![forbid(unsafe_code)]
//! Public, transport-neutral Quaestor values that are safe to ship to untrusted devices.
//!
//! Identity authentication and assurance belong to Shared Auth. Quaestor services still
//! authorize product-owned organization membership and resource access. Values in this
//! crate are untrusted input and never confer authorization.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

/// Stable contract version shared by the Rust, TypeScript, and Dart slices.
pub const CONTRACT_VERSION: &str = "quaestor.pub-lib-core.v1";

/// Untrusted-device runtime family.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ClientPlatform {
    /// Web browser.
    Browser,
    /// Native desktop application.
    Desktop,
    /// Apple mobile application.
    Ios,
    /// Android mobile application.
    Android,
}

/// Why a public value failed validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationError {
    /// Application version is empty, too long, or not printable ASCII.
    AppVersion,
    /// Locale is not within the bounded BCP-47-shaped subset.
    Locale,
    /// Installation identifier is not a bounded opaque token.
    InstallId,
    /// Idempotency key is not a bounded opaque token.
    IdempotencyKey,
    /// Mint time falls outside the exactly portable JSON integer range.
    MintedAtMs,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::AppVersion => "appVersion must be 1-64 printable ASCII characters",
            Self::Locale => "locale must match the bounded public contract",
            Self::InstallId => "installId must be a 16-128 character opaque token",
            Self::IdempotencyKey => "idempotency key must be a 16-128 character opaque token",
            Self::MintedAtMs => "mintedAtMs must be an exactly portable JSON integer",
        })
    }
}

impl std::error::Error for ValidationError {}

/// Bounded metadata supplied by an untrusted client installation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    platform: ClientPlatform,
    app_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    locale: Option<String>,
    install_id: String,
}

impl ClientInfo {
    /// Validates and constructs client metadata.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] when any bounded public field violates the
    /// contract.
    pub fn new(
        platform: ClientPlatform,
        app_version: impl Into<String>,
        locale: Option<impl Into<String>>,
        install_id: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        let app_version = app_version.into();
        if !printable_ascii(&app_version, 1, 64) {
            return Err(ValidationError::AppVersion);
        }
        let locale = locale.map(Into::into);
        if locale.as_deref().is_some_and(|value| !valid_locale(value)) {
            return Err(ValidationError::Locale);
        }
        let install_id = install_id.into();
        if !opaque_token(&install_id) {
            return Err(ValidationError::InstallId);
        }
        Ok(Self {
            platform,
            app_version,
            locale,
            install_id,
        })
    }

    /// Returns the client platform.
    #[must_use]
    pub const fn platform(&self) -> ClientPlatform {
        self.platform
    }

    /// Returns the validated application version.
    #[must_use]
    pub fn app_version(&self) -> &str {
        &self.app_version
    }

    /// Returns the optional validated locale hint.
    #[must_use]
    pub fn locale(&self) -> Option<&str> {
        self.locale.as_deref()
    }

    /// Returns the pseudonymous installation identifier.
    #[must_use]
    pub fn install_id(&self) -> &str {
        &self.install_id
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawClientInfo {
    platform: ClientPlatform,
    app_version: String,
    locale: Option<String>,
    install_id: String,
}

impl<'de> Deserialize<'de> for ClientInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawClientInfo::deserialize(deserializer)?;
        Self::new(raw.platform, raw.app_version, raw.locale, raw.install_id)
            .map_err(serde::de::Error::custom)
    }
}

/// Client-minted retry identity. It conveys no authorization or permission.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdempotencyKey {
    key: String,
    minted_at_ms: u64,
}

impl IdempotencyKey {
    /// Validates and constructs a retry identity.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] when the key or mint time violates the
    /// portable contract.
    pub fn new(key: impl Into<String>, minted_at_ms: u64) -> Result<Self, ValidationError> {
        let key = key.into();
        if !opaque_token(&key) {
            return Err(ValidationError::IdempotencyKey);
        }
        if minted_at_ms > 9_007_199_254_740_991 {
            return Err(ValidationError::MintedAtMs);
        }
        Ok(Self { key, minted_at_ms })
    }

    /// Returns the validated retry token.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns Unix epoch milliseconds.
    #[must_use]
    pub const fn minted_at_ms(&self) -> u64 {
        self.minted_at_ms
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawIdempotencyKey {
    key: String,
    minted_at_ms: u64,
}

impl<'de> Deserialize<'de> for IdempotencyKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawIdempotencyKey::deserialize(deserializer)?;
        Self::new(raw.key, raw.minted_at_ms).map_err(serde::de::Error::custom)
    }
}

fn printable_ascii(value: &str, minimum: usize, maximum: usize) -> bool {
    (minimum..=maximum).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_graphic())
}

fn opaque_token(value: &str) -> bool {
    (16..=128).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn valid_locale(value: &str) -> bool {
    if !(2..=35).contains(&value.len()) {
        return false;
    }
    let mut segments = value.split('-');
    let Some(language) = segments.next() else {
        return false;
    };
    if !(2..=3).contains(&language.len())
        || !language.bytes().all(|byte| byte.is_ascii_alphabetic())
    {
        return false;
    }
    segments.all(|segment| {
        (2..=8).contains(&segment.len()) && segment.bytes().all(|byte| byte.is_ascii_alphanumeric())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn fixtures() -> Value {
        serde_json::from_str(include_str!("../conformance/public-core-v1.json")).unwrap()
    }

    #[test]
    fn rust_accepts_every_shared_valid_case() {
        let fixtures = fixtures();
        assert_eq!(fixtures["contractVersion"], CONTRACT_VERSION);
        for value in fixtures["valid"]["clientInfo"].as_array().unwrap() {
            let parsed: ClientInfo = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(serde_json::to_value(parsed).unwrap(), *value);
        }
        for value in fixtures["valid"]["idempotencyKey"].as_array().unwrap() {
            let parsed: IdempotencyKey = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(serde_json::to_value(parsed).unwrap(), *value);
        }
    }

    #[test]
    fn rust_rejects_every_shared_invalid_case() {
        let fixtures = fixtures();
        for item in fixtures["invalid"]["clientInfo"].as_array().unwrap() {
            assert!(serde_json::from_value::<ClientInfo>(item["value"].clone()).is_err());
        }
        for item in fixtures["invalid"]["idempotencyKey"].as_array().unwrap() {
            assert!(serde_json::from_value::<IdempotencyKey>(item["value"].clone()).is_err());
        }
    }
}
