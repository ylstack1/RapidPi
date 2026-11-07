#!/usr/bin/env bash
# Publishes each build of a release to npm as rapid-pi-PLATFORM-VARIANT@VERSION (rapid-pi-linux-x64, rapid-pi-darwin-arm64, ...;
# Windows' as @kushalkhemka/rapid-pi-win32-VARIANT, as npm refuses the unscoped names):
# a package that holds the release's .tar.xz (Windows: .zip), byte for byte. The installers download it from the npm registry,
# a CDN that is fast where GitHub's release downloads are slow, and check it against the release's SHA256SUMS on GitHub, as
# they do a download from GitHub. On Windows the rapid-pi package's install script (npm/install.cjs) downloads it the same
# way, by its URL in the registry: it is not a dependency of rapid-pi, so a package manager never installs it unchecked.
#
# Usage: scripts/publish-npm-builds.sh DIST [--pack OUT] [-- NPM PUBLISH OPTIONS]
#   DIST     dist/<version>, as package-release.sh makes it (the .tar.xz files and SHA256SUMS)
#   --pack   only make the packages (.tgz) in OUT, to test them
# A version that npm already has is skipped, so running it again is safe. With RAPIDPI_NPM_EXISTING_ONLY=1 (the publish workflow:
# trusted publishing can only publish packages that exist), a package npm does not have yet is skipped, with a warning; it is
# published the first time by hand (docs/RELEASING.md), and a build npm refuses is skipped with a warning (the installers then
# download it from GitHub), so that the rapid-pi package is still published.
source "$(dirname "$0")/lib/common.sh"

DIST="${1:?usage: scripts/publish-npm-builds.sh DIST [--pack OUT] [-- npm publish options]}"
shift
PACK=""
if [ "${1:-}" = --pack ]; then
	PACK="$(abspath "${2:?--pack needs a folder}")"
	shift 2
fi
[ "${1:-}" != -- ] || shift
VERSION="$(basename "$(realpath "$DIST")")"
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "$DIST: the folder's name must be the version (dist/X.Y.Z)"
[ -f "$DIST/SHA256SUMS" ] || die "no SHA256SUMS in $DIST"
need npm

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
# Every build in DIST: those of each platform, put together for the release. (Windows' are .zip files: install.ps1 unpacks them
# with what Windows has.)
builds=()
for file in "$DIST"/rapid-pi-*.tar.xz "$DIST"/rapid-pi-win32-*.zip; do
	[ -f "$file" ] && builds+=("$(basename "$file")")
done
[ ${#builds[@]} -gt 0 ] || die "no rapid-pi-*.tar.xz or rapid-pi-win32-*.zip in $DIST"
# A platform's builds come together: all of them, or none (a missing one is a mistake, as it always was on Linux).
for set in "linux-x64 linux-x64-baseline linux-x64-jit" "linux-arm64 linux-arm64-jit" "darwin-arm64 darwin-arm64-jit" "win32-x64 win32-x64-jit"; do
	ext=tar.xz
	case "$set" in win32-*) ext=zip ;; esac
	present=0
	for platform in $set; do [ -f "$DIST/rapid-pi-$platform.$ext" ] && present=$((present + 1)); done
	if [ "$present" -gt 0 ]; then
		for platform in $set; do [ -f "$DIST/rapid-pi-$platform.$ext" ] || die "no rapid-pi-$platform.$ext in $DIST"; done
	fi
done
for file in "${builds[@]}"; do
	name="${file%.tar.xz}"
	name="${name%.zip}"
	platform="${name#rapid-pi-}"
	case "$platform" in
	linux-arm64*) os=linux cpu=arm64 ;;
	linux-*) os=linux cpu=x64 ;;
	darwin-*) os=darwin cpu=arm64 ;;
	win32-*) os=win32 cpu=x64 ;;
	*) die "$file: not a build of a known platform" ;;
	esac
	pkg="$name"
	[ "$os" != win32 ] || pkg="@kushalkhemka/$name"
	check_sum "$DIST" "$file" || die "$file does not match SHA256SUMS"
	if [ -z "$PACK" ] && [ "$(npm view "$pkg@$VERSION" version 2>/dev/null)" = "$VERSION" ]; then
		log "npm already has $pkg@$VERSION"
		continue
	fi
	if [ -z "$PACK" ] && [ "${RAPIDPI_NPM_EXISTING_ONLY:-}" = 1 ] && [ -z "$(npm view "$pkg" name 2>/dev/null)" ]; then
		warn "npm has no package $pkg yet: skipped (publish it once by hand, then give it trusted publishing)"
		continue
	fi
	dir="$STAGE/$name"
	mkdir -p "$dir"
	cp "$DIST/$file" "$dir/"
	cat >"$dir/package.json" <<EOF
{
	"name": "$pkg",
	"version": "$VERSION",
	"description": "The Rapid Pi $VERSION executable ($platform) for its installer. Install Rapid Pi with the rapid-pi package or $(if [ "$os" = win32 ]; then echo install.ps1; else echo install.sh; fi).",
	"homepage": "https://github.com/opensec-git/Rapid Pi",
	"repository": {
		"type": "git",
		"url": "git+https://github.com/opensec-git/Rapid Pi.git"
	},
	"license": "MIT",
	"author": "OpenSec",
	"os": ["$os"],
	"cpu": ["$cpu"],
	"files": ["$file"]
}
EOF
	cat >"$dir/README.md" <<EOF
# $pkg

\`$file\` of the [Rapid Pi $VERSION release](https://github.com/opensec-git/Rapid Pi/releases/tag/bolt-v$VERSION), the same
bytes, for Rapid Pi's installer to download from the npm registry. Nothing to install from here: use

$(if [ "$os" = win32 ]; then printf '```powershell\npowershell -c "irm https://rapid-pi.opensec.in/install.ps1 | iex"\n```'; else printf '```bash\ncurl -fsSL https://rapid-pi.opensec.in/install.sh | sh\n```'; fi)

or \`npm install -g rapid-pi\`. Both check the file against the release's \`SHA256SUMS\` on GitHub, and their signature.
EOF
	if [ -n "$PACK" ]; then
		mkdir -p "$PACK"
		tgz="$(cd "$dir" && npm pack --silent --pack-destination "$PACK")"
		log "packed $PACK/$tgz"
	elif (cd "$dir" && npm publish --access public "$@"); then
		log "published $pkg@$VERSION"
	elif [ "${RAPIDPI_NPM_EXISTING_ONLY:-}" = 1 ]; then
		warn "npm did not publish $pkg@$VERSION: skipped (the installers download it from GitHub)"
		[ -z "${GITHUB_ACTIONS:-}" ] || echo "::warning title=npm::npm did not publish $pkg@$VERSION"
	else
		die "npm did not publish $pkg@$VERSION"
	fi
done
