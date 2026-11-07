#!/usr/bin/env bash
# Records which of the runtime's functions Pi enters, in the order it first enters them, from sessions of a Rapid Pi build
# (bench/orderfile_session.py: Pi started, a headless prompt, a TUI session, against the local fake model). scripts/build-runtime.sh
# lays those functions out first in the runtime's code (a linker order file), so that Pi touches fewer of its pages. The list is
# of names: it holds from one build of the runtime to the next, and is made again when what Pi runs has changed much.
# Usage: scripts/train-runtime-hints.sh [--pi BUILD]   BUILD: a Rapid Pi build made with the runtime in .work (default out/rapid-pi)
source "$(dirname "$0")/lib/common.sh"
BUILD="$RAPIDPI_ROOT/out/rapid-pi"
while [ $# -gt 0 ]; do
	case "$1" in
	--pi) BUILD="$(abspath "$2")"; shift ;;
	*) die "unknown option $1" ;;
	esac
	shift
done
need bun; need python3
BUN_SRC="$RAPIDPI_WORK/bun"; BUILD_DIR="build/pibolt-release"
[ -x "$BUN_SRC/$BUILD_DIR/bun-profile" ] || die "no $BUN_SRC/$BUILD_DIR/bun-profile: run scripts/build-runtime.sh"
EXE="$BUILD/pi"; [ "$RAPIDPI_OS" = darwin ] && EXE="$BUILD/pi-bin" # (a macOS build's pi is a launcher)
[ -x "$EXE" ] || die "no Rapid Pi build at $BUILD: run scripts/build-pi.sh"
cmp -s "$BUN_SRC/$BUILD_DIR/bun" "$(runtime_bun)" || die "$BUILD's runtime is not the one in $BUN_SRC/$BUILD_DIR: build Pi again"
OUT="$RAPIDPI_ROOT/profiles/runtime-$RAPIDPI_PLATFORM.hints"
log "tracing sessions of $EXE -> $OUT"
(cd "$BUN_SRC" && bun scripts/orderfile/hints.ts --build-dir="$BUILD_DIR" --exe="$EXE" --out="$OUT" \
	-- python3 "$RAPIDPI_ROOT/bench/orderfile_session.py" '{}')
