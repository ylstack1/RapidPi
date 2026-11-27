# Build Process

## Two-stage build

1. **The runtime** — Bun on patched JavaScriptCore with the AOT compiler. Build once, or download from a release.
2. **Pi** — compiled with that runtime into a single executable (~1 minute). This is where plugins are added.

## Quick start (release runtime)

```bash
git clone https://github.com/ylstack1/RapidPi.git && cd RapidPi
scripts/fetch-runtime.sh     # Rapid Pi runtime from latest release -> .work/runtime/bun
scripts/prepare-pi.sh        # builds Pi in this repository (fork of Pi, tag v1.5.0)
scripts/build-pi.sh          # -> out/rapid-pi/pi
out/rapid-pi/pi --version
```

## Building the runtime from source

Requirements: Linux x86-64/ARM64 or macOS Apple silicon; 32 GB RAM (16 GB with swap on ARM64); ~40 GB disk; clang/LLVM 23.1; CMake 3.30; Ninja; Rust stable; Bun 1.4.2; Docker (Linux only, for portable sysroot).

```bash
scripts/fetch-sources.sh            # oven-sh/WebKit and oven-sh/bun at pinned commits, patches applied
scripts/toolchain/make-sysroot.sh   # Ubuntu 20.04 sysroot + gcc-13 + static ICU 78.3 -> .toolchain/sysroot-glibc
scripts/build-runtime.sh            # release build with ThinLTO -> .work/runtime/bun
```

- `fetch-sources.sh` reads `sources.json` (WebKit branch `claude/sound-types-aot`, Bun commit, Pi tag `v1.5.0`)
- Patches: `patches/webkit.patch`, `patches/bun.patch` (applied with `git am`)
- `--native` skips sysroot, links against host libc/ICU (faster setup, less portable)
- macOS: no sysroot; Xcode SDK, macOS 13+, `-mcpu=apple-m1`, system ICU; LLVM 23 from Homebrew; bash 4.4+

## Building Pi

```bash
scripts/build-pi.sh [options]
```

| Option | Default | Meaning |
|---|---|---|
| `--pi DIR` | this repo | A built Pi tree (`scripts/prepare-pi.sh`) |
| `--out DIR` | `out/rapid-pi` | Output directory |
| `--jit on\|off` | `off` | JIT-compiles code loaded at run time |
| `--cpu native\|baseline` | `native` (ARM64 Linux: `baseline`) | Instruction set target |
| `--profile DIR` | `profiles/pi-<ver>` | Training profile |
| `--plugins FILE` | | Compile plugins in |
| `--keep-bytecode` | | Keep bytecode in prebuilt heap |
| `--stable` | | Use stock Bun instead (comparison build) |

The script checks `image registered: true` — the executable runs compiled code.

## Training profiles

Two files per profile (`profiles/pi-<ver>/`):
- `bytecode.order` — functions in first-run order, for code/heap layout (startup page locality)
- `regexps.txt` — regular expressions built from strings at run time, AOT-compiled

Recorded by `scripts/train-profile.sh` running two scripted interactive sessions. `--with DIR` merges a profile from another platform.

`bench/aot_coverage.py` lists what a build still runs outside compiled code.

## macOS runtime order file

`build-runtime.sh` lays out the runtime's own code by what Pi runs first/most. Made from `profiles/runtime-darwin-arm64.hints` and Bun's orderfile, then re-linked. Part of the runtime stamp — changing it requires a runtime rebuild for release.

## Packaging a release

```bash
scripts/package-release.sh [--pi DIR] [--no-build]
```

Builds Pi targets for the host platform and writes `.tar.xz`/`.tar.gz`, runtime, and `SHA256SUMS` to `dist/<VERSION>/`.

## Test commands

| Command | What it checks |
|---|---|
| `tests/aot/run.sh` | Engine correctness (registers across slow paths, Map/Set, realms, workers, inlining, builtins edge cases) |
| `tests/aot/fuzz/run.sh --count 2000` | Differential fuzzing of loops and guards |
| `tests/runtime/run.sh` | Runtime itself (Error.captureStackTrace, fetch keep-alive) |
| `tests/pi/run.sh` | Pi under crash-prone conditions |
| `./test.sh` | All non-e2e tests (from repo root) |
| `npm run check` | Full type check + lint + all check scripts (no tests) |
| `npm run build` | Build all packages (needed for `dist/`) |

**Do not** run `npm test` (includes e2e tests activated by endpoint/auth env vars). Use `./test.sh` or specific vitest invocations.

## Root npm scripts

- `npm run build` — build all packages in order (telemetry, schema, chord, tui, codemode, mcp, ai, durable, env, agent, coding-agent)
- `npm run build:offline` — same, but `ai` with `build:offline`
- `npm run check` — generate catalog modules, biome check, pinned deps, runtime deps, TS imports, entry graphs, install-lock check, tsc --noEmit, browser smoke
- `npm test` — generate catalog modules, test scripts, all workspace tests
- `./test.sh` — non-e2e tests from repo root
- `npm run release:patch|minor|major` — release via `scripts/release.mjs`
- `npm run version:patch|minor|major` — version bump across workspaces
