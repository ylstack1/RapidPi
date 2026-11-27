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
dd4888f4b fix(android): fix CI runner settings compile errors
afe0b21f8 feat(android): add CI/build runner settings for heavy tasks
72130ee0f docs(agent): add project memory files and wire AGENTS.md to .memory/
864760f22 merge(agent): bring waku-daemon removal work onto main
```

## CI failure (fixed)

`android-debug` workflow run 46071979773 failed on `feat/ci-runner-settings` with compile errors:
- `SettingsStore.kt:114` — leftover sed garbage from ciRunner decode block + `when(it)` spacing
- `CiRunnerSection.kt` — `CiTokenField` not `@Composable`, `remember(key)` delegate syntax invalid, `containerColor` not a `TextFieldDefaults.colors()` param, missing `Row`/`Icon` imports, `settings` vs `appSettings` reference

All fixed in `dd4888f4b` and pushed. CI should pass on next run.

## Version

`VERSION` = `1.1.4`

## Working tree

Clean (as of last check). No uncommitted changes noted.
