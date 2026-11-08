# RapidPi Project Memory

## Overview
RapidPi is a multi-provider AI coding agent desktop app (Waku). It supports many providers: Amp, Claude, Codex, Cursor, DeepSeek, Fx, OpenCode, Grok, Kimi, OhMyPi, RapidPi, Pi.

## Workspace
- Root: `RapidPi/`
- Desktop (Rust + TS): `RapidPi/desktop/`
- Protocol crate: `RapidPi/desktop/crates/waku-protocol/`
- Core crate: `RapidPi/desktop/crates/waku-core/`
- Client crate: `RapidPi/desktop/crates/waku-client/`
- Web app: `RapidPi/desktop/apps/web/`

## Key Types
- `ProviderKind` enum — list of supported providers (11 variants).
- `ProviderResumeCursor` enum — tagged serde enum (`tag = "provider"`) holding per-provider session cursor. Must stay in sync with `ProviderKind`.

## Known Issue Fixed
- `ProviderResumeCursor` referenced `Self::RapidPi` in `from_session_id`, `provider()`, and `native_id()` but the `RapidPi` variant was missing from the enum definition → "no variant found for enum ProviderResumeCursor" build error.
- Fix: added `RapidPi { session_id, session_file }` variant to the enum in `waku-protocol/src/model.rs`.

## Decisions
- RapidPi provider mirrors OhMyPi/Pi (session_id + optional session_file).
- ProviderKind::ALL has 11 entries; keep counts consistent when adding providers.

## Build Notes
- Rust toolchain (cargo) not available in this PRoot environment; builds must run elsewhere.
- Desktop uses cargo workspaces; protocol crate is foundational.

## Conventions
- When adding a new provider, update: ProviderKind enum, ALL array, id/display_name/short_name/command, supports_* methods, ProviderResumeCursor enum + from_session_id/provider/native_id, and any match exhaustiveness sites (daemon.rs, etc.).
- TS bindings generated via ts_rs; enum changes regenerate TS types.
