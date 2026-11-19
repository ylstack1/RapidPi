# Handoff: Rapid Pi Desktop App — Ready to Launch

## Session Summary
Desktop app compiles with 0 errors, builds, and runs successfully. The local `waku-daemon` binary includes the CLI's remote-control (Noise IK WebSocket server at `/rc`) with QR code pairing, binds to LAN (0.0.0.0), detects LAN IP, and removes Origin restrictions for mobile app pairing.

## ✅ Completed

### Desktop Compilation: 0 errors ✅
- `cargo check` → 0 errors
- `cargo build` → builds successfully
- `cargo run --bin rapidpi-desktop` → runs without daemon errors ✅

### waku-daemon Binary with Remote-Control ✅
- `desktop/crates/waku-core/src/bin/daemon.rs` — Waku daemon binary with integrated remote-control
- Binds remote-control to `0.0.0.0:48123` (LAN accessible, not just localhost)
- Detects LAN IP via UDP connect to 8.8.8.8 and prints it in pairing URI
- Removed `--allow-origin` argument (mobile app is not a browser, no Origin checks)
- Removed `--allow-origin` passing from `waku-client::DaemonProcess::spawn_configured`
- Pairing URI uses `ws://<LAN_IP>:<port>/rc` so phones can connect over network
- Prints QR code pairing link on terminal startup
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

# Test daemon standalone (detects LAN IP, binds to 0.0.0.0)
WAKU_DAEMON_TOKEN=test WAKU_APP_EXECUTABLE=test target/debug/waku-daemon.exe --bind 0.0.0.0:0 --rc-port 48123
→ QR code printed, pairing URI uses LAN IP (e.g. ws://192.168.1.130:48123/rc) ✅
→ Listens on ws://0.0.0.0:48123/rc (LAN accessible) ✅
→ {"address":"0.0.0.0:PORT","protocolVersion":7,"pid":...} ✅
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

4. Pair a phone: scan the QR code printed by waku-daemon on startup, or paste the `pibolt://pair#...` link into the Rapid Pi Android app's "Paste pairing link" option. The pairing URI contains the LAN IP so the phone can connect over the network.

## Git Status
- Branch: `master` (pushed to origin)
- Last commit: `5b32a0df0` — fix(agent): bind remote-control to LAN, detect LAN IP, remove Origin restrictions
- Untracked (build artifacts — intentionally not committed): `desktop/tmp.txt`, `packages/out/`

## Files Modified (This Session)
- `desktop/crates/waku-client/src/process.rs` - Removed `--allow-origin` passing from spawn_configured
- `desktop/crates/waku-core/src/bin/daemon.rs` - LAN bind, LAN IP detection, no Origin checks
- `desktop/crates/waku-core/src/lib.rs` - Added `pub mod remote_control`
- `desktop/crates/waku-core/src/remote_control/mod.rs` - Module definition
- `desktop/crates/waku-core/src/remote_control/protocol.rs` - Pairing URI format
- `desktop/crates/waku-core/src/remote_control/noise.rs` - Noise key pair
- `desktop/crates/waku-core/src/remote_control/store.rs` - Paired device store
- `desktop/crates/waku-core/src/remote_control/server.rs` - Noise IK WebSocket server
- `desktop/Cargo.lock` - Updated
