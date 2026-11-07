# Rapid Pi — Rebrand & Integration Notes

## Completed (UI + identity)
- RapidPi repo: `pi-bolt` → `rapid-pi`, `Pi-Bolt` → `Rapid Pi`, `PIBOLT` → `RAPIDPI`, `piBolt` → `rapidPi` (597 files)
- Android app: `pibolt` → `rapidpi` (402 files)
- Waku: added `RapidPi` provider (id `rapidpi`, command `rapid-pi`), detection updated, config dir `~/.waku` → `~/.rapidpi`, README → Rapid Pi Desktop

## Remaining (non-UI, deferred)
### Phase 2.3 — Waku provider internals
- composer-complete.rs, skills.rs, slash_command_catalog.rs still reference "Pi" naming; rename to "Rapid Pi" for consistency
- model_catalog.rs provider entries

### Phase 2.5 — Waku release bundling
- Bundle `rapid-pi` binary in release (like waku bundles daemon)
- Update build.rs / release scripts

### Phase 3 — Android on-device engine (optional)
- Bundle ARM64 `rapid-pi` binary in APK with Termux glibc loader
- Currently Android app is remote-control only

### Phase 1.3 — Android package rename (partial)
- Kotlin package `in.opensec.pibolt` → `in.opensec.rapidpi` (done via sed, verify build)

### Phase 1.4 — Update manifest / release tags
- `android-v*` → `rapid-pi-android-v*`
- Update manifest URL

### Phase 1.5 — CLI help text / doctor output
- Herdr pane name, doctor output strings

## Notes
- Waku already supports Pi protocol; Rapid Pi uses same RPC, so no protocol changes needed
- ACP mode (`rapid-pi --acp`) works with any ACP editor (Zed, JetBrains, VS Code, Neovim)
- MCP mode (`rapid-pi --mcp`) for other agents
