# Handoff: Rapid Pi Desktop App — Ready to Launch

## Session Summary
Desktop app compiles with 0 errors, builds, and runs successfully. The local `waku-daemon` binary includes the CLI's remote-control (Noise IK WebSocket server at `/rc`) with QR code pairing, so paired phones can attach directly to the desktop engine without needing the CLI separately.

## ✅ Completed

### Desktop Compilation: 0 errors ✅
- `cargo check` → 0 errors
- `cargo build` → builds successfully
- `cargo run --bin rapidpi-desktop` → runs without "Waku daemon disconnected" errors ✅

### waku-daemon Binary with Remote-Control ✅
- `desktop/crates/waku-core/src/bin/daemon.rs` — Waku daemon binary with integrated remote-control
- Added `[[bin]]` target to `desktop/crates/waku-core/Cargo.toml`
- Accepts `--bind`, `--parent-pid`, `--rc-port`, `--allow-origin`, `--allow-non-loopback` args
- Reads `WAKU_DAEMON_TOKEN` and `WAKU_APP_EXECUTABLE` env vars
- Starts remote-control server on port 48123 (default)
- Prints QR code pairing link to terminal on startup
- Serves WebSocket connections for the desktop client
- `cargo build -p waku-core --bin waku-daemon` → success ✅

### remote_control Module (integrated CLI protocol) ✅
- `protocol.rs` — PairingPayload, `encode_pairing_uri()`, `decode_pairing_uri()`, `pibolt://pair#` format (matches CLI)
- `noise.rs` — X25519 key pair generation via `snow::Builder` (`Noise_IK_25519_AESGCM_SHA256`)
- `store.rs` — `RemoteControlStore` with `host_key_pair()`, `devices()`, `upsert_device()`, `remove_device()`
- `server.rs` — WebSocket at `/rc`, subprotocol `pibolt-rc.1`, Noise IK handshake, pairing validation, encrypted message loop
- `mod.rs` — module definition

### rapid-pi Engine ✅
- Built from root: `packages/coding-agent/src/cli.ts` → `packages/out/rapid-pi.exe` (149 MB)
- `packages/out/rapid-pi.exe --version` → `0.0.0 (Rapid Pi 0.0.3, win32-x64-jit-off, JIT off)` ✅

## Verification

```bash
# From desktop/
cargo check                    → 0 errors ✅
cargo check -p waku-core       → 0 errors ✅
cargo build -p waku-core --bin waku-daemon → builds ✅
cargo build                    → builds successfully ✅

# Test daemon standalone
WAKU_DAEMON_TOKEN=test WAKU_APP_EXECUTABLE=test target/debug/waku-daemon.exe --bind 127.0.0.1:0 --rc-port 48123
→ QR code printed, pairing URI shown, listens on ws://127.0.0.1:48123/rc ✅
→ {"address":"127.0.0.1:PORT","protocolVersion":7,"pid":...} ✅
```

## How to Launch

1. Build the rapid-pi engine (from root):
   ```bash
   cd packages/coding-agent
   bun build --compile --target bun-windows-x64 --bytecode --format esm --splitting --compile-exec-argv=--smol --define "RAPIDPI_BUILD=\"0.0.3 x64-jit-off\"" ./src/cli.ts --outfile ../../out/rapid-pi.exe
   ```

2. Add `packages/out/` to PATH:
   ```powershell
   $env:PATH += ";E:\ylstack-space\Developments\ai\Agents\RapidPi\packages\out"
   ```

3. Run the desktop (from desktop/):
   ```bash
   cargo run --bin rapidpi-desktop
   ```

4. Pair a phone: scan the QR code printed by waku-daemon on startup, or paste the `pibolt://pair#...` link into the Rapid Pi Android app's "Paste pairing link" option.

## Git Status
- Branch: `master` (pushed to origin)
- Last commit: `55a18b25d` — feat(agent): integrate CLI remote-control into waku-daemon
- Untracked (build artifacts — intentionally not committed): `desktop/tmp.txt`, `packages/out/`

## Files Modified (This Session)
- `desktop/crates/waku-core/Cargo.toml` - Added `qrcode`, `rand`, `snow` dependencies; re-added `[[bin]]` target
- `desktop/crates/waku-core/src/bin/daemon.rs` - waku-daemon with remote-control server + QR code
- `desktop/crates/waku-core/src/lib.rs` - Added `pub mod remote_control`
- `desktop/crates/waku-core/src/remote_control/mod.rs` - NEW: module definition
- `desktop/crates/waku-core/src/remote_control/protocol.rs` - NEW: pairing URI format
- `desktop/crates/waku-core/src/remote_control/noise.rs` - NEW: Noise key pair
- `desktop/crates/waku-core/src/remote_control/store.rs` - NEW: paired device store
- `desktop/crates/waku-core/src/remote_control/server.rs` - NEW: Noise IK WebSocket server
- `desktop/Cargo.lock` - Updated
