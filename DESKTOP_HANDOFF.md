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
- Added `[[bin]]` target to `desktop/crates/waku-core/Cargo.toml`
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

### Build Fixes ✅
- Removed `rust-umami` (analytics) which pulls in `reqwest` → `hyper-rustls` → `rustls` → `aws-lc-rs` → `aws-lc-sys` which fails C compilation on Windows with MSVC
- Replaced `rust-umami` with no-op analytics stub (events accepted but never sent)
- Switched `tungstenite` from `rustls-tls-native-roots` to `native-tls` to avoid `aws-lc-sys` C dependency
- Fixed `waku-client/src/client.rs`: `MaybeTlsStream::NativeTls` instead of `Rustls`

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
cargo run --bin rapidpi-desktop → runs without daemon errors ✅

# Test daemon standalone
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

4. Pair a phone: scan the QR code printed by waku-daemon on startup, or paste the `pibolt://pair#...` link into the Rapid Pi Android app's "Paste pairing link" option.

## Git Status
- Branch: `master` (pushed to origin)
- Last commit: `427b57475` — fix(agent): fix desktop launch - remove rust-umami, use native-tls for tungstenite
- Untracked (build artifacts — intentionally not committed): `desktop/tmp.txt`, `packages/out/`

## Files Modified (This Session)
- `desktop/Cargo.toml` - Removed `rust-umami`
- `desktop/crates/waku-client/Cargo.toml` - Changed `tungstenite` to `native-tls`
- `desktop/crates/waku-client/src/client.rs` - `MaybeTlsStream::NativeTls` instead of `Rustls`
- `desktop/src/analytics.rs` - Replaced `rust-umami` with no-op stub
- `desktop/crates/waku-core/Cargo.toml` - Added `qrcode`, `rand`, `snow` dependencies
- `desktop/crates/waku-core/src/bin/daemon.rs` - waku-daemon with remote-control server + QR code
- `desktop/crates/waku-core/src/lib.rs` - Added `pub mod remote_control`
- `desktop/crates/waku-core/src/remote_control/mod.rs` - Module definition
- `desktop/crates/waku-core/src/remote_control/protocol.rs` - Pairing URI format
- `desktop/crates/waku-core/src/remote_control/noise.rs` - Noise key pair
- `desktop/crates/waku-core/src/remote_control/store.rs` - Paired device store
- `desktop/crates/waku-core/src/remote_control/server.rs` - Noise IK WebSocket server
