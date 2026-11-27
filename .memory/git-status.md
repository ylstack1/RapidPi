# Git Status — Rapid Pi

## Branches

Local branches:
- `main` (current)

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
| `origin/feat/ci-runner-settings` | Yes — merged into main via fast-forward |

## Recent commits on main

```
7357497b0 fix(ci): resolve engine pack tag excluding pre-releases
b5a7fae59 docs(memory): update git-status with ci-runner merge
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

## Fixes applied

### Engine pack tag resolution (commit 7357497b0)

`.github/workflows/android-debug.yml`: `gh release list --limit 1` was returning `android-v9` (the most recent pre-release with APK assets, no `rapid-pi-engine-pack.zip`), causing `curl 404`. Added `--exclude-pre-releases` so it resolves to `v1.1.2` (the "Packed Engine" release with the engine pack zip) by default.

### CI runner settings (merged)

Feature merged into `main`. Debug APK builds successfully. Debug APK artifact upload is skipped on `workflow_dispatch` triggers due to `if: ${{ github.event_name == 'pull_request' }}` in `.github/workflows/android-debug.yml` — change to `if: always()` if you want artifacts on manual runs too.

## Version

`VERSION` = `1.1.4`

## Working tree

Clean (as of last check). No uncommitted changes noted.
