#!/usr/bin/env bash
# Builds and packages a release: the Pi executables for each target, the runtime, and SHA256SUMS, in dist/<version>/.
#
# Usage: scripts/package-release.sh [--pi DIR] [--no-build] [--runtime-only]
#   --pi DIR      the built Pi tree (default: this repository)
#   --no-build    package the builds already in out/ instead of building them
#   --runtime-only  only the runtime's archive (.github/workflows/runtime.yml), no Pi and no SHA256SUMS
# Archives (the names stay the same from release to release, so that releases/latest/download/<name> always works), of what
# this machine builds:
#   rapid-pi-linux-x64.tar.gz           JIT off, code for AVX2-class CPUs (falls back to bytecode on others)
#   rapid-pi-linux-x64-baseline.tar.gz  JIT off, code for any x86-64 CPU
#   rapid-pi-linux-x64-jit.tar.gz       JIT on, for code loaded at run time
#   rapid-pi-runtime-linux-x64.tar.gz   the Rapid Pi Bun runtime, to build Pi with plugins (docs/PLUGINS.md)
#   rapid-pi-linux-arm64.tar.gz         on Linux ARM64: JIT off, code for any ARMv8.0 CPU
#   rapid-pi-linux-arm64-jit.tar.gz     on Linux ARM64: JIT on
#   rapid-pi-runtime-linux-arm64.tar.gz
#   rapid-pi-darwin-arm64.tar.gz        on macOS: JIT off, for Apple silicon (M1 and later)
#   rapid-pi-darwin-arm64-jit.tar.gz    on macOS: JIT on
#   rapid-pi-runtime-darwin-arm64.tar.gz
#   rapid-pi-kit-<platform>.tar.gz      the compile kit of this platform's builds, for `rapid-pi compile` (scripts/make-kit.mjs)
# The Pi archives also come as .tar.xz, about 40% smaller, which install.sh prefers where xz is installed.
source "$(dirname "$0")/lib/common.sh"

PI_DIR="$RAPIDPI_PI"; BUILD=1; RUNTIME_ONLY=""
while [ $# -gt 0 ]; do
	case "$1" in
	--pi) PI_DIR="$2"; shift ;;
	--no-build) BUILD="" ;;
	--runtime-only) RUNTIME_ONLY=1; BUILD="" ;;
	-h | --help) sed -n '2,20p' "$0"; exit 0 ;;
	*) die "unknown option $1" ;;
	esac
	shift
done
need xz
VERSION="$(cat "$RAPIDPI_ROOT/VERSION")"
# The npm launcher downloads the release of its own version: the two have to agree.
NPM_VERSION="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' "$RAPIDPI_ROOT/npm/package.json")"
[ "$NPM_VERSION" = "$VERSION" ] || die "npm/package.json is $NPM_VERSION, VERSION is $VERSION"
grep -q "^VERSION=$VERSION\$" "$RAPIDPI_ROOT/npm/bin/rapid-pi" || die "npm/bin/rapid-pi does not download version $VERSION"
DIST="$RAPIDPI_ROOT/dist/$VERSION"
PI_ROOT="$(realpath "$PI_DIR")"
case "$RAPIDPI_PLATFORM" in
linux-x64) TARGETS=("linux-x64:rapid-pi:" "linux-x64-baseline:rapid-pi-baseline:--cpu baseline" "linux-x64-jit:rapid-pi-jit:--jit on") ;;
linux-arm64) TARGETS=("linux-arm64:rapid-pi:" "linux-arm64-jit:rapid-pi-jit:--jit on") ;;
darwin-arm64) TARGETS=("darwin-arm64:rapid-pi:" "darwin-arm64-jit:rapid-pi-jit:--jit on") ;;
*) die "no release is built on $RAPIDPI_PLATFORM" ;;
esac
# Archives whose files belong to nobody in particular, and on macOS without AppleDouble (._*) files of extended attributes.
TAR=(tar --owner=0 --group=0 --numeric-owner)
[ "$RAPIDPI_OS" = darwin ] && TAR=(env COPYFILE_DISABLE=1 tar --uid 0 --gid 0 --numeric-owner --no-xattrs --no-mac-metadata)

# The compile kit comes with the first (the default) build: what it read, and how it was compiled.
KIT="$RAPIDPI_ROOT/out/rapid-pi-kit-$RAPIDPI_PLATFORM"
if [ -n "$BUILD" ]; then
	kit=(--kit "$KIT")
	for target in "${TARGETS[@]}"; do
		IFS=: read -r _ out options <<<"$target"
		# shellcheck disable=SC2086 # options is a list of words
		"$RAPIDPI_ROOT/scripts/build-pi.sh" --pi "$PI_DIR" --out "$RAPIDPI_ROOT/out/$out" "${kit[@]}" $options
		kit=()
	done
fi

rm -rf "$DIST" && mkdir -p "$DIST"
STAGE="$(mktemp -d)"; trap 'rm -rf "$STAGE"' EXIT
notices() {
	cp "$RAPIDPI_ROOT/LICENSE" "$1/LICENSE"
	cp "$RAPIDPI_ROOT/THIRD_PARTY_NOTICES.md" "$1/"
	[ -f "$PI_ROOT/LICENSE" ] && cp "$PI_ROOT/LICENSE" "$1/LICENSE.pi"
	return 0
}

[ -z "$RUNTIME_ONLY" ] || TARGETS=()
for target in "${TARGETS[@]}"; do
	IFS=: read -r name out _ <<<"$target"
	build="$RAPIDPI_ROOT/out/$out"
	[ -x "$build/pi" ] || die "$build/pi not found: build it, or run without --no-build"
	BUN_STATIC_HEAP_VERBOSE=1 "$build/pi" --version 2>&1 | grep -q "image registered: true" || die "$build/pi does not use its compiled code"
	dir="$STAGE/rapid-pi-$name"
	cp -R "$build" "$dir"
	notices "$dir"
	log "rapid-pi-$name.tar.gz (Pi $("$build/pi" --version))"
	"${TAR[@]}" -C "$STAGE" -czf "$DIST/rapid-pi-$name.tar.gz" "rapid-pi-$name"
	# (-T0 writes blocks, which xz -T0 can decompress in parallel.) On x86-64 with xz's x86 filter first, which makes the machine
	# code's call targets relative and so more alike: 2% smaller, and every xz and liblzma since 2009 unpacks it. Not the ARM64
	# filter: xz and libarchive older than 5.4 (2022; Ubuntu 22.04's, macOS's tar) cannot.
	filters=(--lzma2=preset=9e); [ "$RAPIDPI_ARCH" = x64 ] && filters=(--x86 "${filters[@]}")
	"${TAR[@]}" -C "$STAGE" -cf - "rapid-pi-$name" | xz -T0 "${filters[@]}" >"$DIST/rapid-pi-$name.tar.xz"
done

if [ -z "$RUNTIME_ONLY" ]; then
	[ -f "$KIT/kit.json" ] || die "$KIT not found: build, or run without --no-build"
	log "rapid-pi-kit-$RAPIDPI_PLATFORM.tar.gz"
	"${TAR[@]}" -C "$(dirname "$KIT")" -czf "$DIST/rapid-pi-kit-$RAPIDPI_PLATFORM.tar.gz" "$(basename "$KIT")"
fi

runtime="$(runtime_bun)"
dir="$STAGE/rapid-pi-runtime-$RAPIDPI_PLATFORM"
mkdir -p "$dir" && cp "$runtime" "$dir/bun" && notices "$dir"
log "rapid-pi-runtime-$RAPIDPI_PLATFORM.tar.gz (Bun $("$runtime" --version))"
"${TAR[@]}" -C "$STAGE" -czf "$DIST/rapid-pi-runtime-$RAPIDPI_PLATFORM.tar.gz" "rapid-pi-runtime-$RAPIDPI_PLATFORM"
[ -z "$RUNTIME_ONLY" ] || exit 0

# The extensions the installers offer, pinned; and checksums whose first line says which release they are (install.ps1 refuses
# another version's: an older release, signed all the same, served as a newer one).
cp "$RAPIDPI_ROOT/extensions.txt" "$DIST/extensions.txt"
(cd "$DIST" && { echo "# rapid-pi $VERSION"; sha256 -- *.tar.gz *.tar.xz extensions.txt; } >SHA256SUMS)
log "release $VERSION in $DIST:"
(cd "$DIST" && ls -lh -- * | awk '{print "    " $5 "  " $9}')
