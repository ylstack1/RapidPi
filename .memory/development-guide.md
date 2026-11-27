# Development Guide

## Conversational style (AGENTS.md)

- Short, concise answers; no emojis in commits/issues/PR comments/code; no fluff
- Technical prose only; define unavoidable jargon
- Problem -> concrete example/trace -> solution; state why necessary vs optional complexity
- Answer the user's question first before making edits

## TypeScript rules

- **No `any`** unless absolutely necessary
- **No inline imports** (`await import()`, `import("pkg").Type`) — top-level imports only
- **Erasable TypeScript only** (Node strip-only mode) in `packages/*/src`, `packages/*/test`, `packages/coding-agent/examples`: no parameter properties, `enum`, `namespace`/`module`, `import =`, `export =`. Use explicit fields with constructor assignments.
- **Package assets** in `packages/coding-agent` via helpers in `src/config.ts`, not `__dirname`
- Never downgrade code to fix type errors from outdated deps — upgrade the dep
- Never modify `packages/ai/src/models.generated.ts` directly — update `packages/ai/catalog/` definitions, then regenerate
- `packages/ai/src/providers/data/` is gitignored — never stage it

## Chord v2 code style (`packages/chord/src/next/`)

- One concept per file, named after it; first line is a comment saying what the file is for
- Order: imports, exported types, public entry point first, then helpers in order of use; constants above first user
- Blank line between top-level declarations
- Every export has a one-line doc comment; inline comments explain non-obvious *why*, never *what*
- Short functions (~40 lines max needs a reason); one `switch` over a tagged union is fine
- No boolean flag parameters — use two functions or a tagged value
- Never write the same lines twice
- Context last: required args, then `context`/`ctx` (always required), then optional args
- Tagged unions for every state and result; casts only where TS cannot express an invariant (with short why)
- Every runtime check names the trust boundary it serves
- Caches are `WeakMap`s keyed by owner, each with a GC test; no module-level strong caches
- No defensive code against broken adapters/callers/our own modules

## Commands

```
npm run check          # Full type check + lint + all checks (fix all errors/warnings/infos before committing)
npm run build          # Build all packages (needed for dist/)
./test.sh              # All non-e2e tests from repo root
node vitest/cli.js --run test/specific.test.ts   # Specific vitest test
node --test test/specific.test.ts                # packages/tui (node:test)
```

- Never run `npm test` (e2e tests activate with endpoint/auth env vars)
- If you create/modify a test file, run it and iterate until it passes
- For `packages/coding-agent/test/suite/`: use `test/suite/harness.ts` + faux provider; no real provider APIs/keys/paid tokens
- Regression tests for GitHub issues: add issue number comment next to the test
- Ad-hoc scripts: write to `/tmp`, run, edit, remove; don't embed multi-line scripts in bash commands
- Never commit unless the user asks

## Dependency / install security

- Treat npm dep and lockfile changes as reviewed code
- Direct registry deps (including internal workspace deps) pinned to exact versions
- When updating `undici`, read its changelog first
- Hydrate: `npm install --ignore-scripts`; clean/CI: `npm ci --ignore-scripts`
- Lockfile changes need `PI_ALLOW_LOCKFILE_CHANGE=1` to commit (pre-commit blocks otherwise)

## Git rules

- Only commit files **you** changed in **this** session
- Stage explicit paths (`git add <path>`); never `git add -A` / `git add .`
- Run `git status` before committing; verify only your files are staged
- `packages/ai/src/models.generated.ts` may always be included alongside your files
- Message format: `{feat,fix,docs}[(ai,tui,agent,coding-agent)]: <message>`
- Never: `git reset --hard`, `git checkout .`, `git clean -fd`, `git stash`, `git add -A`, `git add .`, `git commit --no-verify`
- Resolve rebase conflicts only in files you modified; abort and ask if conflict is in a file you didn't modify

## Changelog (packages/*/CHANGELOG.md)

- All new entries under `## [Unreleased]`: `### Breaking Changes`, `### Added`, `### Changed`, `### Fixed`, `### Removed`
- Released version sections are immutable
- Attribution: internal `Fixed foo bar ([#123](...))`; external `Added feature X ([#456](...) by [@user](...))`

## Running tests (packages/coding-agent suite)

Uses `test/suite/harness.ts` + faux provider. No real provider APIs, keys, or paid tokens.
