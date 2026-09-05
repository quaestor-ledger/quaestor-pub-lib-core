# quaestor-pub-lib-core

Public, transport-neutral client primitives for Quaestor Ledger. Everything in
a distributable path is assumed visible to an untrusted device.

## Boundary

- No SQL, migrations, database drivers, ORM entities, raw database rows,
  environment readers, internal endpoints, session material, provider
  credentials, or administrator-only models.
- Shared Auth establishes identity, realm, and assurance. This package does not
  authorize Quaestor organizations, memberships, roles, billing relationships,
  or resources and must never imply that validated input is authorized input.
- Quaestor Ledger observes, records, and proves. Do not introduce execution,
  settlement, custody, transfer, or provider-instruction behavior here.
- Rust, TypeScript, and Dart implementations must accept and reject the same
  `conformance/public-core-v1.json` cases.
- TypeSpec and JSON Schema are independently human-authored peers. Never
  generate one authority from the other. Update both intentionally and let the
  agreement gate decide whether their normalized semantics converge.
- Add every distributable file to the exact allowlist in
  `tools/public-boundary/src/main.rs`. Do not weaken the policy to admit an
  unexplained file or term.

## Verification

Run `npm ci --ignore-scripts`, `npm run generate`, `npm test`, and
`cargo package --allow-dirty --list`. A package is not ready while any runtime
skips its shared fixtures or the package listing contains anything outside the
root Rust manifest, lockfile, license, README, shared conformance fixture, and
`src/lib.rs`.

## Git and safety

Work from the latest `main`; do not rebase, stash, reset, force-push, or create a
worktree without explicit human permission. Stage explicit paths. Resolve every
conflict conceptually after reading the available history; never select a side
wholesale.

The canonical workspace rules are available locally at
`.ores/agents/AGENTS.md` and `/Users/alexandermills/codes/AGENTS.md`. `.ores/`
is machine-local and must never be committed.

## Repository-local Git worktrees

- Create or use a Git worktree only when the human operator explicitly authorizes it for the current task. Concurrency or a dirty checkout is not permission by itself.
- Put every authorized worktree at `<repository-root>/tmp/worktrees/<name>`; from the repository root, use `./tmp/worktrees/<name>`. Never place worktrees beside repositories or organization directories.
- Keep `tmp`, `temp`, `tmp/worktrees`, and `temp/worktrees` ignored in the repository-root `.gitignore`. Do not commit files from those directories.
- Relocate or remove a worktree only when the operator explicitly requests it. Before removal, preserve and publish intended changes, verify its commit is represented on the target branch, and confirm there are no tracked, untracked, ignored-sensitive, or in-use files that must survive. Remove it with `git worktree remove <path>` without `--force`; never delete a worktree directory with `rm`.
