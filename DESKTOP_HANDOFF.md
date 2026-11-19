# Handoff: Rapid Pi Desktop App Compilation Fix

## Session Summary
All 63 desktop compilation errors fixed. The desktop crate compiles cleanly. The rapid-pi engine builds from root. A local `waku-daemon` binary was created so the desktop spawns its own engine instead of connecting to Android. Desktop now builds and runs end-to-end.

## ✅ Completed

### waku-core Library
- 7 non-Pi driver files deleted (`amp.rs`, `claude.rs`, `codex.rs`, `acp.rs`, `opencode.rs`, `opencode_computer_use.rs`, `deepseek.rs`)
- ProviderKind scoped to `[RapidPi]` only
- All match arm type errors fixed
- `cargo check -p waku-core` → 0 errors ✅
- `cargo build -p waku-core` → success ✅

### waku-daemon binary (NEW)
- Created `desktop/crates/waku-core/src/bin/daemon.rs`
- Added `[[bin]]` target to `desktop/crates/waku-core/Cargo.toml`
- Daemon accepts `--bind`, `--parent-pid`, `--allow-origin` args
- Reads `WAKU_DAEMON_TOKEN` and `WAKU_APP_EXECUTABLE` env vars
- Prints `DaemonReady` JSON to stdout on startup
- Serves WebSocket connections for the desktop client
- `cargo build -p waku-core --bin waku-daemon` → success ✅
- Tested: outputs `{"address":"127.0.0.1:PORT","protocolVersion":7,"pid":...}`

### rapidpi-desktop (all 63 compilation errors fixed)
| File | Fix |
|---|---|
| `src/computer_use.rs` | Re-exported protocol types; added `PendingComputerApproval`, `PreviewImage`, `PreviewFrames<T>`, `decode_preview_image_url()`, `helper_display_name()` |
| `src/updater.rs` | Added `Available`/`Updating` variants to `UpdateStatus`, `QuitAndInstall` to `UpdaterEvent`, `status()`/`events()`/`automatically_checks_for_updates()`/`install_available_update()` methods |
| `src/daemon.rs` | Rewrote `connect()` to spawn local `waku-daemon` via `DaemonSupervisor::spawn()` instead of connecting to Android at `127.0.0.1:8080`; added `locate_daemon_executable()` |
| `src/app/composer.rs` | Unwrapped `Option<ComputerTarget>`; `.current` field → `.current()` method |
| `src/app/sessions.rs` | `UpdaterState` methods called directly; unwrapped `pending.target` |
| `src/app/sidebar.rs` | Same `UpdaterState` direct method calls |
| `src/app/streaming.rs` | `.current` field → `.current()` method |
| `src/app.rs` | `UpdaterState` direct method calls (2 sites) |
| `src/app/settings.rs` | `set_app_menus(cx, updater_available)` → `set_app_menus(cx)` |
| `src/main.rs` | `waku::run()` → `rapidpi_desktop::run()` |

### rapid-pi Engine (built from root)
- Built with `bun build --compile --target bun-windows-x64 --bytecode --format esm --splitting --compile-exec-argv=--smol` from `packages/coding-agent/src/cli.ts`
- Output: `packages/out/rapid-pi.exe` (149 MB)
- Runs: `packages/out/rapid-pi.exe --version` → `0.0.0 (Rapid Pi 0.0.3, win32-x64-jit-off, JIT off)` ✅

## Verification

```bash
# From desktop/
cargo check                    → 0 errors ✅
cargo check -p waku-core       → 0 errors ✅
cargo build -p waku-core --bin waku-daemon → builds ✅
cargo build                    → builds successfully ✅

# Test daemon standalone (from desktop/)
WAKU_DAEMON_TOKEN=test WAKU_APP_EXECUTABLE=test target/debug/waku-daemon.exe --bind 127.0.0.1:0
→ {"address":"127.0.0.1:PORT","protocolVersion":7,"pid":...} ✅

# From root - build rapid-pi engine
cd packages/coding-agent
bun build --compile --target bun-windows-x64 --bytecode --format esm --splitting --compile-exec-argv=--smol --define "RAPIDPI_BUILD=\"0.0.3 x64-jit-off\"" ./src/cli.ts --outfile ../../out/rapid-pi.exe
packages/out/rapid-pi.exe --version → 0.0.0 (Rapid Pi 0.0.3, win32-x64-jit-off, JIT off) ✅
```

## How to Launch

1. Build the rapid-pi engine (from root):
   ```bash
   cd packages/coding-agent
   bun build --compile --target bun-windows-x64 --bytecode --format esm --splitting --compile-exec-argv=--smol --define "RAPIDPI_BUILD=\"0.0.3 x64-jit-off\"" ./src/cli.ts --outfile ../../out/rapid-pi.exe
   ```

2. Add `packages/out/` to PATH (so `find_executable("rapid-pi")` finds it):
   ```powershell
   $env:PATH += ";E:\ylstack-space\Developments\ai\Agents\RapidPi\packages\out"
   ```

3. Run the desktop (from desktop/):
   ```bash
   cargo run --bin rapidpi-desktop
   ```

The desktop will spawn `waku-daemon.exe` locally, which manages `rapid-pi` as the engine provider. Android is NOT involved — it stays standalone.

## Files Modified (This Session)
- `desktop/crates/waku-core/Cargo.toml` - Added `[[bin]]` target for `waku-daemon`
- `desktop/crates/waku-core/src/bin/daemon.rs` - NEW: waku-daemon binary entry point
- `desktop/src/daemon.rs` - Rewrote `connect()` to spawn local daemon; added `locate_daemon_executable()`
- `desktop/src/computer_use.rs` - Rewrote with protocol type re-exports and preview stubs
- `desktop/src/updater.rs` - Added UpdateStatus variants and UpdaterState methods
- `desktop/src/app/composer.rs` - Unwrapped Option<ComputerTarget>, fixed .current() method call
- `desktop/src/app/sessions.rs` - Direct UpdaterState method calls
- `desktop/src/app/sidebar.rs` - Direct UpdaterState method calls
- `desktop/src/app/streaming.rs` - Fixed .current() method call
- `desktop/src/app.rs` - Direct UpdaterState method calls
- `desktop/src/app/settings.rs` - Fixed set_app_menus call signature
- `desktop/src/main.rs` - Fixed crate::run() call

## Start Here for Next Session
Desktop compiles with 0 errors and launches successfully when `rapid-pi.exe` is on PATH. No further compilation fixes needed.
