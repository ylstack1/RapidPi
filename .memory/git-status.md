# Git Status — Rapid Pi

## Branches

Local branches:
- `main` (current)
- `feat/ci-runner-settings` (CI runner settings feature, pushed)

Remote branches (`origin/`):
- `HEAD -> origin/main`
- `main`
- `master`
- `pi-bolt`
- `feat/android-github-rest-tools`
- `feat/ci-runner-settings`

## Merge status (all against local `main`)

| Branch | Merged into main? |
|---|---|
| `origin/main` | Yes (local main == origin/main, in sync) |
| `origin/master` | Yes — fully merged |
| `origin/pi-bolt` | Yes — fully merged |
| `origin/feat/android-github-rest-tools` | Yes — fully merged |
| `origin/feat/ci-runner-settings` | No — feature branch, not yet merged to main |

## Recent commits

```
ed9c87ebc fix(android): fix remaining CiRunnerSection.kt compile errors
45151007a fix(android): final fix for CI runner settings compile errors
cdbedc222 fix(android): add missing spend = SpendLimits( in ciRunner decode block
d4b67ada5 fix(android): add missing Box import to CiRunnerSection.kt
dcd50351c fix(android): correct CiRunner import paths
f33094979 fix(android): add getValue/setValue imports for CiTokenField delegate
dd4888f4b fix(android): fix CI runner settings compile errors
afe0b21f8 feat(android): add CI/build runner settings for heavy tasks
72130ee0f docs(agent): add project memory files and wire AGENTS.md to .memory/
864760f22 merge(agent): bring waku-daemon removal work onto main
```

## CI failure (being fixed iteratively)

`android-debug` workflow on `feat/ci-runner-settings` kept failing at step 7 "Tests, lint, debug build" with Kotlin compile errors. All fixes applied:

| Error | File | Fix | Commit |
|---|---|---|---|
| `Expecting a when-condition` line 114 | `SettingsStore.kt` | `when(it)` → `when (result)` with explicit param + null branch | `45151007a` |
| `it.removePrefix("failed:")` on CiTestResult? | `SettingsStore.kt` | `it` was CiTestResult?, not String — fixed with named param | `45151007a` |
| Destructuring Pair<Pair<Pair<...>>> needs component3/4 | `CiRunnerSection.kt:55` | Replaced nested `to` chaining with `Triple` | `45151007a` |
| Argument type mismatch: Color vs ImageVector | `CiRunnerSection.kt:63` | Fixed by fixing destructuring | `45151007a` |
| Unresolved reference 'size'/'width'/'background' | `CiRunnerSection.kt` | Added modifier imports | `45151007a` |
| Unresolved reference 'background' (import) | `CiRunnerSection.kt` | import from `androidx.compose.foundation`, not `layout` | `ed9c87ebc` |
| Unresolved reference 'rememberModalBottomSheetState' | `CiRunnerSection.kt` | Added import from `androidx.compose.material3` | `ed9c87ebc` |
| No parameter 'containerColor' in TextFieldDefaults.colors() | `CiRunnerSection.kt` | Removed `colors` parameter, removed unused import | `ed9c87ebc` |
| Wrong import path `opensec.rapidpi.app.data.CiRunner.<Class>` | 3 files | `CiRunner.kt` is in package `opensec.rapidpi.app.data` (no subpackage) | `dcd50351c` |
| Missing `Box` import | `CiRunnerSection.kt` | Added import | `d4b67ada5` |
| Missing `spend = SpendLimits(` in decode | `SettingsStore.kt` | Restored dropped line | `cdbedc222` |
| `when(it)` spacing | `SettingsStore.kt` | `when(it)` → `when (it)` | `dd4888f4b` |
| Missing `getValue`/`setValue` imports | `CiRunnerSection.kt` | Added for `by remember` delegate | `f33094979` |

All fixes pushed to `feat/ci-runner-settings`. Rerun `android-debug` workflow to verify.

## Version

`VERSION` = `1.1.4`

## Working tree

Clean (as of last check). No uncommitted changes noted.
