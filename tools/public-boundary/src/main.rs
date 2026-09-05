#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::Path;

#[cfg(test)]
use std::path::PathBuf;

const PUBLISH_ROOTS: &[&str] = &[
    "src",
    "typescript/src",
    "dart/lib",
    "contracts",
    "conformance",
    "generated/typespec",
];

const ALLOWED_FILES: &[&str] = &[
    "conformance/public-core-v1.json",
    "contracts/json-schema/public-core.schema.json",
    "contracts/typespec/main.tsp",
    "contracts/typespec/tspconfig.yaml",
    "dart/lib/quaestor_pub_lib_core.dart",
    "generated/typespec/json-schema/AppVersion.json",
    "generated/typespec/json-schema/ClientInfo.json",
    "generated/typespec/json-schema/ClientPlatform.json",
    "generated/typespec/json-schema/IdempotencyKey.json",
    "generated/typespec/json-schema/InstallId.json",
    "generated/typespec/json-schema/LocaleHint.json",
    "generated/typespec/json-schema/RetryToken.json",
    "src/lib.rs",
    "typescript/src/index.ts",
];

const FORBIDDEN_TERMS: &[&str] = &[
    "access_token",
    "refresh_token",
    "csrf_token",
    "client_secret",
    "private_key",
    "database_url",
    "web_session",
    "admin_audit",
    "admin_role",
    "supabase_service",
    "sea_orm",
    "sea-orm",
    "sqlx",
    "diesel::",
    "postgresql://",
    "postgres://",
];

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("tool must remain under tools/public-boundary");
    if let Err(errors) = check_repository(root) {
        for error in errors {
            eprintln!("public-boundary: {error}");
        }
        std::process::exit(1);
    }
    println!(
        "public boundary passed ({} distributable files)",
        ALLOWED_FILES.len()
    );
}

fn check_repository(root: &Path) -> Result<(), Vec<String>> {
    let expected = ALLOWED_FILES
        .iter()
        .map(|path| (*path).to_owned())
        .collect::<BTreeSet<_>>();
    let mut actual = BTreeSet::new();
    let mut errors = Vec::new();

    for publish_root in PUBLISH_ROOTS {
        collect_files(root, &root.join(publish_root), &mut actual, &mut errors);
    }

    for missing in expected.difference(&actual) {
        errors.push(format!("allowed file is missing: {missing}"));
    }
    for unexpected in actual.difference(&expected) {
        errors.push(format!("unclassified distributable file: {unexpected}"));
    }

    for relative in expected.intersection(&actual) {
        let path = root.join(relative);
        match fs::read_to_string(&path) {
            Ok(content) => {
                if let Some(term) = forbidden_term(&content) {
                    errors.push(format!(
                        "{relative} contains forbidden public term {term:?}"
                    ));
                }
            }
            Err(error) => errors.push(format!("cannot read {relative}: {error}")),
        }
    }

    check_manifests(root, &mut errors);
    check_exported_symbols(root, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn collect_files(
    root: &Path,
    directory: &Path,
    output: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            errors.push(format!(
                "cannot read publish root {}: {error}",
                relative(root, directory)
            ));
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                errors.push(format!("cannot read directory entry: {error}"));
                continue;
            }
        };
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(error) => {
                errors.push(format!(
                    "cannot classify {}: {error}",
                    relative(root, &path)
                ));
                continue;
            }
        };
        if file_type.is_symlink() {
            errors.push(format!(
                "symlink is not allowed in public output: {}",
                relative(root, &path)
            ));
        } else if file_type.is_dir() {
            collect_files(root, &path, output, errors);
        } else if file_type.is_file() {
            output.insert(relative(root, &path));
        } else {
            errors.push(format!(
                "special file is not allowed in public output: {}",
                relative(root, &path)
            ));
        }
    }
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn forbidden_term(content: &str) -> Option<&'static str> {
    let lowered = content.to_ascii_lowercase();
    FORBIDDEN_TERMS
        .iter()
        .copied()
        .find(|term| lowered.contains(term))
}

fn check_manifests(root: &Path, errors: &mut Vec<String>) {
    let cargo = read_manifest(root, "Cargo.toml", errors);
    for dependency in ["sqlx", "diesel", "sea-orm", "postgres", "reqwest", "dotenv"] {
        if cargo.to_ascii_lowercase().contains(dependency) {
            errors.push(format!(
                "Cargo.toml contains forbidden runtime dependency: {dependency}"
            ));
        }
    }
    if !cargo.contains("serde =") {
        errors.push("Cargo.toml must keep serde as its only public runtime dependency".to_owned());
    }
    let anchored_include = "include = [\"/Cargo.toml\", \"/Cargo.lock\", \"/LICENSE\", \"/README.md\", \"/conformance/public-core-v1.json\", \"/src/lib.rs\"]";
    if !cargo.lines().any(|line| line.trim() == anchored_include) {
        errors.push("Cargo.toml must retain the exact root-anchored publish allowlist".to_owned());
    }

    let package_json = read_manifest(root, "typescript/package.json", errors);
    if package_json.contains("\"dependencies\"") {
        errors.push("typescript/package.json must have no runtime dependencies".to_owned());
    }
    if !package_json.contains("\"devDependencies\": {\n    \"typescript\": \"7.0.2\"\n  }") {
        errors.push(
            "typescript/package.json must have only the pinned compiler development dependency"
                .to_owned(),
        );
    }

    let pubspec = read_manifest(root, "dart/pubspec.yaml", errors);
    if pubspec.lines().any(|line| line.trim() == "dependencies:") {
        errors.push("dart/pubspec.yaml must remain dependency-free".to_owned());
    }
}

fn check_exported_symbols(root: &Path, errors: &mut Vec<String>) {
    check_symbols(
        root,
        "src/lib.rs",
        &[
            "pub const ",
            "pub enum ",
            "pub struct ",
            "pub type ",
            "pub fn ",
        ],
        &[
            "CONTRACT_VERSION",
            "ClientInfo",
            "ClientPlatform",
            "IdempotencyKey",
            "ValidationError",
        ],
        errors,
    );
    check_symbols(
        root,
        "typescript/src/index.ts",
        &[
            "export const ",
            "export type ",
            "export interface ",
            "export class ",
            "export function ",
        ],
        &[
            "CLIENT_PLATFORMS",
            "CONTRACT_VERSION",
            "ClientInfo",
            "ClientPlatform",
            "IdempotencyKey",
            "PublicCoreErrorCode",
            "PublicCoreValidationError",
            "parseClientInfo",
            "parseIdempotencyKey",
        ],
        errors,
    );
    check_symbols(
        root,
        "dart/lib/quaestor_pub_lib_core.dart",
        &["const String ", "enum ", "final class ", "class "],
        &[
            "ClientInfo",
            "ClientPlatform",
            "IdempotencyKey",
            "PublicCoreValidationException",
            "contractVersion",
        ],
        errors,
    );
}

fn check_symbols(
    root: &Path,
    relative: &str,
    prefixes: &[&str],
    expected: &[&str],
    errors: &mut Vec<String>,
) {
    let content = read_manifest(root, relative, errors);
    let actual = content
        .lines()
        .filter(|line| !line.starts_with(char::is_whitespace))
        .filter_map(|line| {
            prefixes.iter().find_map(|prefix| {
                line.strip_prefix(prefix).and_then(|rest| {
                    rest.split(|character: char| {
                        character.is_whitespace()
                            || matches!(character, ':' | '=' | '<' | '(' | '{' | ';')
                    })
                    .next()
                })
            })
        })
        .filter(|name| !name.starts_with('_'))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let expected = expected
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<BTreeSet<_>>();
    for missing in expected.difference(&actual) {
        errors.push(format!("{relative} is missing public symbol {missing}"));
    }
    for unexpected in actual.difference(&expected) {
        errors.push(format!(
            "{relative} exports unclassified public symbol {unexpected}"
        ));
    }
}

fn read_manifest(root: &Path, relative: &str, errors: &mut Vec<String>) -> String {
    match fs::read_to_string(root.join(relative)) {
        Ok(content) => content,
        Err(error) => {
            errors.push(format!("cannot read {relative}: {error}"));
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_shaped_field_is_rejected() {
        let unsafe_name = ["encrypted", "access", "token"].join("_");
        let candidate = format!("pub struct LeakedRow {{ pub {unsafe_name}: String }}");
        assert_eq!(forbidden_term(&candidate), Some("access_token"));
    }

    #[test]
    fn bounded_public_values_are_allowed() {
        assert_eq!(
            forbidden_term("pub struct ClientInfo { install_id: String }"),
            None
        );
    }

    #[test]
    fn current_repository_passes_the_exact_allowlist() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf();
        assert_eq!(check_repository(&root), Ok(()));
    }
}
