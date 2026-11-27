# Features and Interfaces

## Core capabilities

- **Same as Pi:** identical commands, sessions, settings, providers, extensions
- **No runtime dependencies:** single executable, no Node.js/Bun/npm needed to run it
- **Fast startup:** ~41 ms ready to type (~10x faster than Pi on Node 24)
- **Low memory:** static heap is file-backed, pages shared with page cache; no JIT code caches or profiling data
- **Long sessions stay fast:** frame cost is constant regardless of message count

## Interfaces

| Interface | Flag | Description |
|---|---|---|
| Interactive CLI | (default) | Terminal UI chat session |
| ACP (Agent Client Protocol) | `--acp` | Runs in editor agent panels: Zed, JetBrains IDEs, VS Code, Neovim, etc. Uses Pi settings, extensions, skills, sessions |
| MCP server | `--mcp` | Model Context Protocol server; other agents give Rapid Pi tasks via its `prompt` tool, continue sessions across calls |

## Built-in tools

- `grep` and `find` tools are on by default, search in-process, follow `.gitignore`, nothing to install
- Can be disabled: `"defaultTools": ["-grep", "-find"]` in `~/.pi/agent/settings.json`
- With `opensec-pi-fff` or `@ff-labs/pi-fff` loaded, FFF's search tools are used instead

## Extensions and plugins

- Pi extensions work unchanged
- `rapid-pi compile` compiles installed extensions into the executable (Linux and macOS) — they run as machine code
- `rapid-pi update` compiles extensions into each new release
- Default build (no JIT): run-time-loaded extensions are interpreted
- JIT-on build (`--jit on`): run-time extensions get baseline JIT -> DFG -> FTL as they warm up (~5% of default speed)
- See `docs/PLUGINS.md` for porting/writing extensions for AOT

## Other features

- **`rapid-pi doctor`** — plain-text installation report for issues (build, signature verification, compiled/loaded/failing extensions, packages vs rapidPi.packages, Herdr/editor setup); `--offline` omits release signature and update check
- **`rapid-pi update` / `--rollback` / `--channel beta`** — release management
- **Herdr** — Rapid Pi shows as `rapid-pi` in a Herdr pane; restart reopens conversation; disable with `"-builtin:herdr"` in extensions setting
- **Optional installer extensions:** `opensec-pi-subagents`, `opensec-pi-todo`, `opensec-pi-fff`

## Provider and settings compatibility

- Uses Pi's agent directory (`~/.pi/agent`) and shared `packages` list
- `rapidPi.packages` extends the shared list; `-source` entries exclude inherited packages from Rapid Pi only
- `rapid-pi install --rapid-pi` / `rapid-pi remove --rapid-pi` change only Rapid Pi's overrides
