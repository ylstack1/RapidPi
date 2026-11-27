# Workspace Architecture — Rapid Pi (Pi-Bolt)

## What this repository is

Rapid Pi is an independent fork of [Pi](https://github.com/earendil-works/pi) — the coding agent — compiled ahead of time (AOT) to native machine code. It ships as a single executable with no Node.js, Bun, or JIT required at launch. Same commands, sessions, settings, providers, and extensions as Pi.

- **Upstream Pi tag:** `v1.5.0` (see `sources.json`)
- **Runtime:** Bun on a patched JavaScriptCore (oven-sh/WebKit#743) with an AOT compiler (B3/Air back end)
- **Version:** `1.1.4` (see `VERSION`)
- **License:** MIT (binaries bundle Bun MIT, JavaScriptCore LGPL-2.0/BSD, ICU, Pi MIT — see `THIRD_PARTY_NOTICES.md`)
- **Not affiliated** with Pi's authors or Oven (Bun)

## Monorepo layout (root `package.json` workspaces)

| Package | Name | Purpose |
|---|---|---|
| `packages/agent` | `@earendil-works/pi-agent-core` | Core agent loop: sessions, turns, tool execution, provider calls |
| `packages/ai` | `@earendil-works/pi-ai` | LLM provider integrations, model catalog, OAuth |
| `packages/chord` | `@earendil-works/chord` | Durable execution / durable objects framework (Chord v2 in `src/next/`) |
| `packages/codemode` | `@earendil-works/pi-codemode` | In-editor code transformation (apply/edit model outputs into files) |
| `packages/coding-agent` | `@earendil-works/pi-coding-agent` | Top-level coding agent application: TUI, commands, settings, extensions |
| `packages/durable` | `@earendil-works/pi-durable` | Durable state/persistence layer |
| `packages/env` | `@earendil-works/pi-env` | Environment / process / terminal abstraction |
| `packages/evals` | `@earendil-works/pi-evals` | Evaluation harness |
| `packages/mcp` | `@earendil-works/pi-mcp` | Model Context Protocol server implementation |
| `packages/schema` | `@earendil-works/pi-schema` | JSON Schema validation / type system |
| `packages/telemetry` | `@earendil-works/pi-telemetry` | Telemetry / metrics / tracing |
| `packages/tui` | `@earendil-works/pi-tui` | Terminal UI (Rust native bindings + TS) |

All packages are `@earendil-works/pi-*` (except `chord`) and live under `packages/*/`. Each has `src/`, `dist/` (built), `test/`, `CHANGELOG.md`, `package.json`.

## Path aliases (tsconfig.json root)

Root `tsconfig.json` maps `@earendil-works/<pkg>` to `packages/<pkg>/src/index.ts` and various subpaths. No `__dirname` in `packages/coding-agent` — use helpers in `src/config.ts` instead (account for source checkouts, npm installs, standalone binaries).

## Key non-package directories

| Directory | Purpose |
|---|---|
| `patches/` | `webkit.patch` (~947 KB) and `bun.patch` (~893 KB) — Rapid Pi's changes to WebKit and Bun, applied via `git am` by `scripts/fetch-sources.sh` |
| `profiles/` | Training profiles `pi-1.1.0` … `pi-1.5.0` (each: `bytecode.order` + `regexps.txt`), plus `runtime-darwin-arm64.hints` and `runtime-win32-x64.hints` |
| `scripts/` | Build pipeline: `build-pi.sh`, `build-runtime.sh`, `prepare-pi.sh`, `fetch-sources.sh`, `fetch-runtime.sh`, `train-profile.sh`, `package-release.sh`, `release.mjs`, check scripts, etc. |
| `docs/` | `ARCHITECTURE.md`, `BUILDING.md`, `BENCHMARKS.md`, `PLUGINS.md`, `ACP.md`, `MCP.md`, `RELEASING.md`, `TROUBLESHOOTING.md`, `WINDOWS.md` |
| `tests/` | `aot/`, `runtime/`, `pi/`, `compat/`, `termux/` — engine and integration tests |
| `bench/` | Benchmark results and raw data |
| `desktop/`, `android/` | Desktop and Android (Termux) integration |
| `site/`, `docs-site/` | Website sources |
| `npm/` | npm packaging |
| `keys/` | Release signing keys |
| `.memory/` | Project memory / continuation docs (this directory) |

## Build artifacts (not in git)

- `.work/` — runtime and Pi checkouts, build output
- `out/` — built Pi executables (`out/rapid-pi/pi`, `out/pi-stable/pi`)
- `dist/` — release archives
- `packages/*/dist/` — built package output
- `packages/ai/src/providers/data/` — gitignored model catalog data (never stage)

## Inter-package dependency flow (high level)

```
coding-agent (app: TUI, commands, settings, extensions)
  -> agent-core (agent loop, turns, tools)
    -> ai (providers, model catalog)
    -> mcp (MCP client/server)
    -> durable (persistence)
    -> env (process/terminal)
    -> telemetry (metrics)
    -> schema (validation)
    -> codemode (code transforms)
    -> chord (durable execution, plugins)
  -> tui (terminal UI)
```

## Generated files (do not edit directly)

- `packages/ai/src/models.generated.ts` — regenerate from `packages/ai/catalog/` definitions (`npm run generate:models`)
- `packages/ai/src/providers/<provider>.catalog.generated.ts` — from `packages/ai/src/providers/data/` (`npm run generate:catalog-modules`)
- `packages/ai/src/providers/data/` — gitignored, fetched from providers (`--refresh-models`)
