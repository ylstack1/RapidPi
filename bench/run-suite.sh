#!/usr/bin/env bash
# Runs the full benchmark suite behind the README and docs/BENCHMARKS.md, then renders the charts.
#
# Usage: bench/run-suite.sh RESULTS_DIR [--cpus LIST]
# Expects the builds of scripts/package-release.sh in out/ (rapid-pi, rapid-pi-jit), the plugin builds
# (scripts/build-pi.sh --plugins examples/plugins/plugins.ts --out out/rapid-pi-plugins, and --jit on --out out/rapid-pi-plugins-jit),
# the stock-Bun build (scripts/build-pi.sh --stable --out out/pi-stable) and, for Node, the Pi of this repository, built
# (scripts/prepare-pi.sh).
# Environment: RAPIDPI_PI (the Pi tree Node runs; default this repository), RAPIDPI_STABLE_PI (the stock-Bun executable; default
# out/pi-stable/pi), RAPIDPI_COMPARE (more builds to run alongside, as name=command, separated by spaces: an earlier Rapid Pi release,
# e.g. "rapid-pi-0.7.0=.work/base-070/rapid-pi-linux-x64/pi"; the report shows them after Rapid Pi). Rapid Pi's tree has changes to Pi of its own: to compare with Pi as released, point both at a build of the
# upstream tag (scripts/build-pi.sh --stable --pi <Pi checkout> --out out/pi-stable-upstream).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$(python3 -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "${1:?results directory}")"; shift
CPUS=40-47
[ "$(uname -s)" = Darwin ] && CPUS="" # (macOS has no pinning to cores: the tools ignore --cpus there)
[ "${1:-}" = --cpus ] && CPUS="$2"
PI="${RAPIDPI_PI:-$ROOT}"
NODE="node $PI/packages/coding-agent/dist/bundle/cli.js"
STABLE="${RAPIDPI_STABLE_PI:-out/pi-stable/pi}"
COMPARE=(); COMPARE_NAMES=""
for spec in ${RAPIDPI_COMPARE:-}; do COMPARE+=(--build "$spec"); COMPARE_NAMES+=",${spec%%=*}"; done
mkdir -p "$OUT"
cd "$ROOT"
{
	echo "date: $(date -u +%Y-%m-%dT%H:%MZ)"
	if [ "$(uname -s)" = Darwin ]; then
		echo "cpu: $(sysctl -n machdep.cpu.brand_string), $(sysctl -n hw.ncpu) cores, not pinned"
		echo "system: macOS $(sw_vers -productVersion) ($(uname -r))"
	else
		echo "cpu: $(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2 | xargs), pinned to $CPUS"
		echo "kernel: $(uname -r)"
	fi
	echo "rapid-pi: $(cat VERSION), Pi $(out/rapid-pi/pi --version)"
	echo "bun: $(${RAPIDPI_STABLE_BUN:-bun} --version)"
	echo "node: $(node --version)"
} >"$OUT/environment.txt"

python3 bench/benchmark.py --runs 21 --warmup 3 ${CPUS:+--cpus "$CPUS"} --out "$OUT/benchmark.jsonl" \
	--build rapid-pi=out/rapid-pi/pi ${COMPARE[@]+"${COMPARE[@]}"} --build bun="$STABLE" --build "node=$NODE" --build rapid-pi-jit=out/rapid-pi-jit/pi --baseline bun
for _ in 1 2 3; do
	python3 bench/long_session.py --prompts 75 --every 25 ${CPUS:+--cpus "$CPUS"} --out "$OUT/long.jsonl" \
		--build rapid-pi=out/rapid-pi/pi ${COMPARE[@]+"${COMPARE[@]}"} --build bun="$STABLE" --build "node=$NODE"
done
python3 bench/tmux_check.py --prompts 4 --rounds 5 ${CPUS:+--cpus "$CPUS"} --out "$OUT/tmux.jsonl" \
	--build rapid-pi=out/rapid-pi/pi ${COMPARE[@]+"${COMPARE[@]}"} --build bun="$STABLE" --build "node=$NODE" >/dev/null
python3 bench/plugin_bench.py --runs 5 ${CPUS:+--cpus "$CPUS"} --out "$OUT/plugins.jsonl" \
	--compiled rapid-pi=out/rapid-pi-plugins/pi --compiled rapid-pi-jit=out/rapid-pi-plugins-jit/pi \
	--runtime rapid-pi=out/rapid-pi/pi --runtime rapid-pi-jit=out/rapid-pi-jit/pi --runtime bun="$STABLE" \
	--none rapid-pi=out/rapid-pi/pi --none bun="$STABLE"
# (The charts of each platform have their own folder: Linux's are the README's.)
IMAGES=docs/images
[ "$(uname -s)" = Darwin ] && IMAGES=docs/images/darwin-arm64
mkdir -p "$IMAGES"
python3 bench/report.py "$OUT" --images "$IMAGES" --builds "rapid-pi$COMPARE_NAMES,bun,node" | tee "$OUT/summary.md"
