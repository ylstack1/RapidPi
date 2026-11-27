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

`android-debug` workflow on `feat/ci-runner-settings` keeps failing at step 7 "Tests, lint, debug build" with Kotlin compile errors. Fixes applied:

| Error | File | Fix | Commit |
|---|---|---|---|
| `Expecting a when-condition` line 114 | `SettingsStore.kt` | `when(it)` → `when (it)`, removed sed garbage | `dd4888f4b` |
| `@Composable invocations not in context` | `CiRunnerSection.kt` | Cascading from missing delegate imports | `f33094979` |
| `Property delegate getValue/setValue missing` | `CiRunnerSection.kt` | Added `getValue`/`setValue` imports from `androidx.compose.runtime` | `f33094979` |
| `Unresolved reference 'it'` | `CiRunnerSection.kt` | Cascading from missing delegate imports | `f33094979` |
| Wrong import path `opensec.rapidpi.app.data.CiRunner.<Class>` | `EngineApi.kt`, `LocalEngine.kt`, `CiRunnerSection.kt` | `CiRunner.kt` is in package `opensec.rapidpi.app.data` (no subpackage). Fixed to `opensec.rapidpi.app.data.<Class>` | `dcd50351c` |
| Missing `Box` import | `CiRunnerSection.kt` | `Box` used in `CiProviderChoice` but not imported | `d4b67ada5` |
| Missing `spend = SpendLimits(` in decode block | `SettingsStore.kt` | Python replacement dropped the line; `session/day/month/showInLists` were parsed as top-level AppSettings params instead of SpendLimits fields | `cdbedc222` |

Root cause of most `CiRunnerSection.kt` errors: `var text by remember { mutableStateOf(value) }` requires `getValue`/`setValue` extension imports from `androidx.compose.runtime`, matching the pattern in `BackgroundRun.kt` and `SettingsScreen.kt`.

All fixes pushed to `feat/ci-runner-settings`. CI should pass on next run.

## Version

`VERSION` = `1.1.4`

## Working tree

Clean (as of last check). No uncommitted changes noted.
