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
f33094979 fix(android): add getValue/setValue imports for CiTokenField delegate
dd4888f4b fix(android): fix CI runner settings compile errors
afe0b21f8 feat(android): add CI/build runner settings for heavy tasks
72130ee0f docs(agent): add project memory files and wire AGENTS.md to .memory/
864760f22 merge(agent): bring waku-daemon removal work onto main
```

## CI failure (fixed)

`android-debug` workflow run 46071979773 failed on `feat/ci-runner-settings` with compile errors:

| Error | File | Fix | Commit |
|---|---|---|---|
| `Expecting a when-condition` line 114 | `SettingsStore.kt` | `when(it)` → `when (it)`, removed sed garbage | `dd4888f4b` |
| `@Composable invocations not in context` lines 238, 254, 255 | `CiRunnerSection.kt` | Cascading from missing delegate imports | `f33094979` |
| `Property delegate getValue/setValue missing` line 249 | `CiRunnerSection.kt` | Added `getValue`/`setValue` imports from `androidx.compose.runtime` | `f33094979` |
| `Unresolved reference 'it'` lines 253 | `CiRunnerSection.kt` | Cascading from missing delegate imports | `f33094979` |

Root cause of most CiRunnerSection.kt errors: `var text by remember { mutableStateOf(value) }` requires `getValue`/`setValue` extension imports from `androidx.compose.runtime`, matching the pattern in `BackgroundRun.kt` and `SettingsScreen.kt`.

All fixes pushed to `feat/ci-runner-settings`.

## Version

`VERSION` = `1.1.4`

## Working tree

Clean (as of last check). No uncommitted changes noted.
