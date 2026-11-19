# Rapid Pi Desktop Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the vendored Waku tree in `desktop/` into a native Rapid Pi window: one engine (`rapid-pi`), one protocol (Pi RPC), two transports (local stdio, paired `/rc` WebSocket), no other agents.

**Architecture:** The GPUI app and its daemon keep their existing shape. `crates/waku-core/src/driver/pi.rs` already spawns `rapid-pi --mode rpc` and speaks NDJSON; every other driver and every other protocol is removed. Remote access stops using Waku's own `ws://` daemon client and instead pairs with a `rapid-pi` host through the protocol in `packages/coding-agent/docs/remote-control.md`. Android keeps its in-APK engine and stays standalone; it is never a desktop backend.

**Tech Stack:** Rust, GPUI (zed), `waku-protocol` (internal wire types), `waku-daemon` (session process manager), `waku-client` (transport), `snow` (Noise IK, new dependency), Pi RPC mode.

**Spec:** `packages/coding-agent/docs/rpc.md`, `packages/coding-agent/docs/rpc-commands.md`, `packages/coding-agent/docs/rpc-extension-ui.md`, `packages/coding-agent/docs/remote-control.md`, `android/engine/CONTRACT.md`.

## Global Constraints

- Engine: `rapid-pi` only. No other agent CLI is detected, launched, or listed.
- Wire protocol: Pi RPC NDJSON (`rpc.md`). Remote: remote-control protocol v1 (`remote-control.md`).
- Clients: the GPUI desktop only. No web client, no mobile client in `desktop/`.
- Android: standalone. A missing or unpaired phone must never block desktop startup or a local session.
- Addons: Pi extensions, skills, slash commands, MCP servers. No desktop-side plugin system.
- New dependencies: `snow` (Noise `IK_25519_AESGCM_SHA256`) and `base64` are the only additions.
- Desktop is verified with `cargo` from `desktop/`; root `npm run check` covers TypeScript packages only.
- Commits follow root `AGENTS.md`: `{feat,fix,docs}[(ai,tui,agent,coding-agent)]: <message>`, explicit paths only.
- GPUI work follows `desktop/AGENTS.md`: no I/O or subprocess spawn reachable from `render`, and every new control is keyboard-operable.
- Licensing: root is MIT, `desktop/` is GPL-3.0 inherited from Waku, and `THIRD_PARTY_NOTICES.md` must say so.

## Review Focus

- Opening a saved session whose provider was removed: must show a clear "provider not available" error, not panic, not silently reattach to Rapid Pi.
- Provider surfaces after the scope change: composer tabs, settings, command palette and session lists must show exactly one entry, with no empty group or stale selection.
- Pairing links written by shipped Android builds (`pibolt://pair#`) must still parse while the app emits the same scheme.
- Remote frames larger than 65000 bytes and JSON split across frames must reassemble exactly once, never truncate.
- `rapid-pi` dying mid-turn locally, or a paired host dropping: the session must surface an error, never leave the UI waiting forever. The remote case is pinned by Task 5's test; the local case is exercised in the closing manual pass.

---

## Decisions locked by this plan

| Question | Decision |
|---|---|
| Providers | `ProviderKind::ALL` becomes `[RapidPi]`; the other ten variants stay in the enum for reading old rows |
| Daemon | Kept as a session/process manager. Inlining `pi.rs` into the app is deferred; it buys nothing now |
| Remote path | Daemon opens the paired `/rc` connection and re-exposes it; the UI and session model do not change |
| Waku's own remote | Removed: no `apps/web`, no `apps/mobile`, and no "connect a phone to the daemon" path. Desktop keeps its loopback daemon link |
| Crate names | Stay `waku-*` (churn, no user-visible value). User-visible strings and config dirs are rebranded |
| Pairing scheme | Emit and accept `pibolt://pair#`; add `rapidpi://pair#` as an accepted alias |
| Old sessions | Kept readable; opening one under a removed provider fails with a named error |

## What gets deleted

| Path | Size | Why |
|---|---|---|
| `crates/waku-core/src/driver/{amp,claude,codex,acp,opencode,opencode_computer_use,deepseek}.rs` | ~613 KB | Other agents |
| `apps/web/` | — | Browser client of the daemon |
| `apps/mobile/` | — | Expo client of the daemon; collides with the real Android app |
| `crates/waku-protocol/src/bin/export_types.rs` | — | Exists only to feed the two TypeScript clients |

---

### Task 1: Scope the provider surface to RapidPi

**Files:**
- Modify: `desktop/crates/waku-protocol/src/model.rs:12-45` (`enum ProviderKind`, `const ALL`, `#[default]`)
- Test: same file, `#[cfg(test)] mod tests`

**Interfaces:**
- Produces: `ProviderKind::ALL: [Self; 1] == [Self::RapidPi]`, `ProviderKind::default() == RapidPi`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn only_rapid_pi_is_selectable() {
    assert_eq!(ProviderKind::ALL, [ProviderKind::RapidPi]);
}

#[test]
fn rapid_pi_is_the_default_kind() {
    assert_eq!(ProviderKind::default(), ProviderKind::RapidPi);
}

#[test]
fn removed_providers_still_deserialize() {
    // Saved sessions and the task store hold these values; reading must not fail.
    let kind: ProviderKind = serde_json::from_str("\"claude\"").unwrap();
    assert_eq!(kind, ProviderKind::Claude);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p waku-protocol only_rapid_pi_is_selectable` (from `desktop/`)
Expected: FAIL; `ALL` still has 11 entries.

- [ ] **Step 3: Change `ALL` and the default**

Set `pub const ALL: [Self; 1] = [Self::RapidPi];`, move `#[default]` onto `RapidPi`, and put this comment above the remaining variants: `/// Kept so sessions and task rows written by earlier builds still deserialize.`

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p waku-protocol`
Expected: PASS, including the three new tests.

- [ ] **Step 5: Compile the rest and fix what assumed the other ten**

Run: `cargo check -p waku-core -p waku`
Expected: errors only in `desktop/src/app/{composer,settings,command_palette,runtime,sessions}.rs`. Delete each now-unreachable branch. Do not add fallbacks that keep a removed provider visible.

- [ ] **Step 6: Commit**

```bash
git add desktop/crates/waku-protocol/src/model.rs desktop/src/app/composer.rs desktop/src/app/settings.rs desktop/src/app/command_palette.rs desktop/src/app/runtime.rs desktop/src/app/sessions.rs
git commit -m "feat(ai,tui,agent): scope desktop providers to Rapid Pi"
```

### Task 2: Delete the non-Pi drivers

**Files:**
- Delete: `desktop/crates/waku-core/src/driver/{amp,claude,codex,acp,opencode,opencode_computer_use,deepseek}.rs`
- Modify: `desktop/crates/waku-core/src/driver/mod.rs` (dispatch `match`, `pub mod` lines)
- Modify: `desktop/crates/waku-core/src/{model_catalog,command_env,composer_complete,skills,slash_command_catalog}.rs`
- Modify: `desktop/crates/waku-protocol/src/model.rs` (`ProviderResumeCursor` arms)
- Test: `desktop/crates/waku-core/src/driver/mod.rs` (existing inline tests)

**Interfaces:**
- Consumes: `ProviderKind::ALL` from Task 1
- Produces: `start_local` (`crates/waku-core/src/driver/mod.rs:209`) keeps one live arm plus a catch-all that returns `unavailable_provider(provider)`; `fn unavailable_provider(ProviderKind) -> anyhow::Error` is the single place that names the failure

- [ ] **Step 1: Delete the files and the dispatch arms**

Remove the seven files, their `pub mod` declarations, and every `match` arm that names them. Remove `PiFlavor::OhMyPi` too: after Task 1 nothing can select it, so its flavor branch and resume mapping go with the other dead paths. `computer_use.rs`, `activity.rs`, `title_refresh.rs`, `support.rs` stay.

- [ ] **Step 2: Compile and remove the leftovers**

Run: `cargo check -p waku-core`
Expected: errors naming the deleted modules and the resume cursors. Remove each reference. `ProviderResumeCursor` keeps its variants (stored data) and loses only the code that produced them.

- [ ] **Step 3: Prove nothing references them**

Run: `git grep -nE "(amp|claude|codex|opencode|deepseek|acp)::" -- desktop/crates desktop/src`
Expected: no matches outside comment text and rebrand notes.

- [ ] **Step 4: Pin the removed-provider behaviour with a test**

In `driver/mod.rs`, replace the old arms with `ProviderKind::RapidPi => …` and `other => return Err(unavailable_provider(other))`, then test the error alone so the test needs no `DriverStartOptions`:

```rust
#[test]
fn removed_provider_names_itself_in_the_error() {
    assert_eq!(
        unavailable_provider(ProviderKind::Claude).to_string(),
        "provider not available: Claude"
    );
}
```

Run: `cargo test -p waku-core removed_provider_names_itself_in_the_error`
Expected: FAIL, then PASS once `unavailable_provider` uses `format!("provider not available: {provider:?}")`.

- [x] **Step 5: Run the daemon and core tests**

Run: `cargo test -p waku-core -p waku-daemon`
Expected: PASS. Existing Pi driver tests (21 in `driver/pi.rs`) must still pass unchanged.

- [x] **Build debug executable**

Run: `cargo build -p waku-core`
Expected: ✅ Success, debug build produced in `target/debug/`

- [ ] **Step 6: Commit**

```bash
git add desktop/crates/waku-core/src/driver desktop/crates/waku-core/src/model_catalog.rs desktop/crates/waku-core/src/command_env.rs desktop/crates/waku-core/src/composer_complete.rs desktop/crates/waku-core/src/skills.rs desktop/crates/waku-core/src/slash_command_catalog.rs desktop/crates/waku-protocol/src/model.rs
git commit -m "feat(ai,tui,agent): remove non-Rapid-Pi drivers"
```

### Task 3: Remove the Waku web and mobile clients

**Files:**
- Delete: `desktop/apps/web`, `desktop/apps/mobile`
- Delete: `desktop/crates/waku-protocol/src/bin/export_types.rs`
- Modify: `desktop/package.json` (`workspaces`, `protocol:generate`, `protocol:check`)
- Modify: `desktop/scripts/{dev,release}.ts` if they build either app
- Modify: `desktop/Cargo.toml` if `export_types` is a declared `[[bin]]`
- Modify: `desktop/README.md`, `desktop/AGENTS.md` (drop the app references)

- [ ] **Step 1: Find every reference**

Run: `git grep -nE "apps/(web|mobile)|waku-mobile|export_types" -- desktop`
Expected: the two workspace entries, the two npm scripts, the bin declaration, and doc lines.

- [ ] **Step 2: Delete and edit**

Stop the dev watcher first (`desktop/AGENTS.md`): it rebuilds from `package.json` and will fail while the workspace entries point at deleted folders. Remove the directories and `export_types.rs`; drop the workspace entries and both npm scripts; fix the doc lines. Keep `drizzle.config.ts` and `db/` only if `waku-daemon` still uses them, otherwise remove them in the same step.

- [ ] **Step 3: Build the whole desktop workspace**

Run: `cargo build --workspace` (from `desktop/`)
Expected: success, no warnings about missing bins.

- [ ] **Step 4: Prove nothing references them**

Run: `git grep -nE "apps/(web|mobile)|waku-mobile|export_types" -- desktop`
Expected: no matches.

- [ ] **Step 5: Commit**

```bash
git add desktop/apps/web desktop/apps/mobile desktop/crates/waku-protocol/src/bin/export_types.rs desktop/package.json desktop/scripts/dev.ts desktop/scripts/release.ts desktop/Cargo.toml desktop/README.md desktop/AGENTS.md
git commit -m "feat(ai,tui,agent): drop Waku web and mobile clients"
```

### Task 4: Pairing transport (`/rc`) in `waku-client`

**Files:**
- Create: `desktop/crates/waku-client/src/remote/mod.rs` (public surface, link parsing, endpoint order)
- Create: `desktop/crates/waku-client/src/remote/link.rs` (`pairing link -> PairingLink`)
- Create: `desktop/crates/waku-client/src/remote/noise.rs` (`snow` IK handshake, transport messages)
- Create: `desktop/crates/waku-client/src/remote/frames.rs` (`flag || data` chunking, 65000-byte cap)
- Modify: `desktop/crates/waku-client/src/lib.rs`, `desktop/Cargo.toml`, `desktop/crates/waku-client/Cargo.toml`
- Test: `desktop/crates/waku-client/src/remote/{link,frames,noise}.rs` inline `#[cfg(test)]`

**Interfaces:**
- Produces:
  - `pub struct PairingLink { pub version: u8, pub name: String, pub host_key: [u8; 32], pub secret: [u8; 32], pub endpoints: Vec<String>, pub expires_at: u64 }`
  - `pub fn parse_pairing_link(text: &str) -> Result<PairingLink, RemoteError>`
  - `pub struct Chunker { /* … */ }` with `fn encode(&mut self, payload: &[u8]) -> Vec<Vec<u8>>` and `fn decode(&mut self, frame: &[u8]) -> Option<Vec<u8>>`
  - `pub async fn connect(link: &PairingLink, device: &DeviceInfo) -> Result<RemoteSession, RemoteError>`

- [ ] **Step 1: Write the failing tests**

`link.rs`: parse a literal `pibolt://pair#<base64url>` payload with the six fields from `remote-control.md` and assert every field; assert `rapidpi://pair#` parses identically; assert a truncated base64 payload returns `Err`.
`frames.rs`: encode 200000 bytes, assert the first two frames carry flag `0x01` and at most 65000 bytes, the last carries `0x00`, and `decode` of the concatenation returns the original bytes exactly once.

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p waku-client remote::`
Expected: FAIL, module does not exist.

- [ ] **Step 3: Implement the three modules**

Use `base64` for unpadded base64url. Use `snow` with `Builder::new("Noise_IK_25519_AESGCM_SHA256".parse().unwrap())`, prologue `b"pibolt-rc/1"`, client as initiator, and send `{ "device": …, "app": …, "pair": … }` as handshake message 1's payload, exactly as `remote-control.md` §Connection specifies. Subprotocol header: `pibolt-rc.1`.

- [ ] **Step 4: Run them to verify they pass**

Run: `cargo test -p waku-client remote::`
Expected: PASS.

- [ ] **Step 5: Add the gated handshake test**

In `noise.rs`, add one `#[tokio::test]` that spawns `rapid-pi` with remote control enabled when the binary is on `PATH` and skips otherwise. Follow the gating style already used in `crates/waku-core/src/driver/pi.rs` around `find_executable("rapid-pi")`.

- [ ] **Step 6: Commit**

```bash
git add desktop/Cargo.toml desktop/crates/waku-client/Cargo.toml desktop/crates/waku-client/src
git commit -m "feat(ai,tui,agent): add rapid-pi pairing transport"
```

### Task 5: Serve a paired computer through the daemon

**Files:**
- Modify: `desktop/crates/waku-daemon/src/main.rs` (accept a pairing target, keep the local spawn default)
- Modify: `desktop/crates/waku-daemon/src/*` connection setup (a `Source::Local | Source::Remote` selection)
- Modify: `desktop/crates/waku-client/src/lib.rs` (daemon-side handle to the remote session)
- Modify: `desktop/src/app/settings.rs` (paste a pairing link, list paired computers, revoke)
- Test: `desktop/crates/waku-daemon/src/*` inline tests; fixture `desktop/crates/waku-client/test/fixtures/remote_hello.json`

**Interfaces:**
- Consumes: `waku_client::remote::{parse_pairing_link, connect}` from Task 4
- Produces: daemon command line `--pair <link>`; daemon session records carry `source: "local" | "remote"`; `remote_hello` maps 1:1 onto the existing session snapshot the UI already renders

- [ ] **Step 1: Write the failing test**

Feed the `remote_hello` fixture through the mapping function and assert the produced snapshot has the same shape the local path produces: non-empty `messages`, `state.model`, `state.thinkingLevel`, and `endpoints` preserved.

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p waku-daemon remote_hello`
Expected: FAIL, mapping missing.

- [ ] **Step 3: Implement the mapping and the source switch**

Map `remote_hello`, session events, `response` and `extension_ui_request` onto the daemon's existing event types. `list_projects`, `list_sessions`, `switch_session`, `new_session` fill the project/session sidebar exactly like local sessions. Forward `extension_ui_response` back verbatim. Accept only the commands `remote-control.md` lists; everything else becomes a normal failed `response`.

- [ ] **Step 4: Run it to verify it passes**

Run: `cargo test -p waku-daemon`
Expected: PASS.

- [ ] **Step 5: Pin the host-drop behaviour with a test**

Serve the fixture stream, then close the socket mid-turn and assert the daemon forwards the existing failure channel, `DriverEvent::Error`:

```rust
#[tokio::test]
async fn a_dropped_host_ends_the_session_with_an_error() {
    let events = run_against_fixture(Fixture::RemoteHelloThenClose).await;
    let error = events.into_iter().find_map(|event| match event {
        DriverEvent::Error(text) => Some(text),
        _ => None,
    });
    assert_eq!(error.as_deref(), Some("remote host disconnected"));
}
```

Run: `cargo test -p waku-daemon a_dropped_host_ends_the_session_with_an_error`
Expected: FAIL until socket close maps to that message and the turn is released.

- [ ] **Step 6: Wire the settings UI**

`settings.rs` gets a "Pair a computer" row: paste a link, connect, show host name and endpoints, revoke. No QR rendering on desktop; the phone shows the QR.

- [ ] **Step 7: Manual check**

Start `rapid-pi` with `/remote-control copy`, paste the link into the desktop settings, confirm the session list and a live turn render, then revoke and confirm the connection drops.

- [ ] **Step 8: Commit**

```bash
git add desktop/crates/waku-daemon desktop/crates/waku-client desktop/src/app/settings.rs
git commit -m "feat(ai,tui,agent): drive paired Rapid Pi hosts from the desktop"
```

### Task 6: Rebrand sweep and licence notices

**Files:**
- Modify: every remaining `waku` literal in `desktop/src`, `desktop/crates/*/src`: config dirs, `application_support.join("Waku")` in `crates/waku-core/src/computer_use.rs:317`, command-palette text, window titles, `User-Agent` in the GitHub extension
- Modify: `THIRD_PARTY_NOTICES.md` (add Waku, GPL-3.0-only, and what `desktop/` contains)
- Modify: `LICENSE` (state that `desktop/` is GPL-3.0 and the MIT grant covers the rest)
- Modify: `REBRAND_NOTES.md` (replace with the actual remaining list)

- [ ] **Step 1: Inventory**

Run: `git grep -niE "\bwaku\b" -- desktop/src desktop/crates`
Expected: a finite list. Anything user-visible is renamed; crate names stay.

- [ ] **Step 2: Rename the user-visible strings and migrate the config dir**

Rename with a one-time move from the old directory when it exists, so existing installs keep their state.

- [ ] **Step 3: Update the notices and licence**

Add the Waku row (`https://github.com/egoist/waku`, GPL-3.0-only, "the native desktop shell in `desktop/`, redistributed as a derived work") and the licence boundary sentence.

- [ ] **Step 4: Verify**

Run: `cargo test -p waku-core -p waku-protocol -p waku-daemon` and `cargo build --workspace`
Expected: PASS and success. Then `git grep -n "application_support.join(\"Waku\")"` returns nothing.

- [ ] **Step 5: Commit**

Stage exactly the paths Step 1 listed, never a directory: `desktop/` is shared with other work in progress.

```bash
git add <each desktop path from Step 1> THIRD_PARTY_NOTICES.md LICENSE REBRAND_NOTES.md
git commit -m "docs(ai,tui,agent): rebrand desktop strings and record Waku licence"
```

### Task 7: Ship the GitHub extension to both hosts

**Files:**
- Create: `extensions/rapidpi-github.js` (single source of truth, moved from the Android asset; `extensions.txt` already lives here)
- Delete: `android/app/src/main/assets/extensions/pibolt-github.js`
- Modify: `android/app/src/main/kotlin/opensec/pibolt/app/git/github/GithubExtension.kt` (copy the asset Gradle now stages, and the new file name)
- Modify: `android/app/build.gradle.kts` (stage the root file into `assets/extensions/`)
- Modify: `desktop/src/app/settings.rs` (desktop install into `~/.pi/agent/extensions/` plus a token field)
- Modify: `desktop/build.rs` (embed the root file so the app can install it without a checkout)

**Interfaces:**
- Produces: one JS file installed as `<agentDir>/extensions/rapidpi-github.js` on both hosts; token read from `process.env.GITHUB_TOKEN`, which each host sets (Android: Settings; desktop: stored setting or `gh auth token`)

- [ ] **Step 1: Confirm the extension is already host-neutral**

Run: `git grep -n "Android\|pibolt" -- android/app/src/main/assets/extensions/pibolt-github.js`
Expected: only the `User-Agent` string and the comment header. Nothing else may be host-specific; if it is, fix it here.

- [ ] **Step 2: Move the file and update both installers**

Both apps copy the same file to the same relative location. Android keeps its Gradle asset step; desktop copies it on first launch when the target is missing or older.

- [ ] **Step 3: Add the desktop token field**

Store the token outside the repository, put it in the engine's environment only for sessions the user allowed, and never log it.

- [ ] **Step 4: Verify on both hosts**

Android: `./gradlew :app:assembleDebug` then install and confirm `github_read_file` is offered.
Desktop: `cargo build` then `cargo run`, sign in, and confirm the same tool list.

- [ ] **Step 5: Commit**

```bash
git add extensions/rapidpi-github.js android/app/src/main/assets/extensions/pibolt-github.js android/app/src/main/kotlin/opensec/pibolt/app/git/github/GithubExtension.kt android/app/build.gradle.kts desktop/src/app/settings.rs desktop/build.rs
git commit -m "feat(ai,tui,agent): share the GitHub extension across desktop and Android"
```

---

## Milestones

| Milestone | Tasks | Result |
|---|---|---|
| M1 One engine | 1 | Desktop lists and runs Rapid Pi only (Task 1 complete) |
| M2 One client | 3 | No web or mobile client left in the tree |
| M3 Pairing | 4, 5 | Desktop drives a paired computer over `/rc` |
| M4 Identity | 6 | Strings, config dirs and licence match the product |
| M5 Addons | 7 | Same tools on desktop and phone from one extension file |

## Non-goals

- Reimplementing Android's engine on desktop, or shipping arm64 Android binaries anywhere but the APK.
- A desktop plugin system. Extensions are Pi's.
- Multi-agent comparison, provider switching, or model routing across vendors.

## Verification for the whole plan

```bash
cd desktop
cargo build --workspace
cargo test -p waku-core -p waku-protocol -p waku-daemon -p waku-client
```

Expected: clean build, all tests pass. Then a manual pass: launch the app, start a session, send a prompt, open a saved session from before this work (it must report the removed provider), pair a phone-hosted session, revoke it.
