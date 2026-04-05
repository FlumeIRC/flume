#!/bin/bash
# Generate distinfo for the FreeBSD port.
# Downloads the source tarball and all crate tarballs, computes SHA256 + SIZE.
# Usage: ./generate-distinfo.sh [version]
# Run from the packaging/freebsd directory.

set -e

VERSION="${1:-1.2.4}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DISTINFO="$SCRIPT_DIR/distinfo"
TMPDIR=$(mktemp -d)

echo "Generating distinfo for v${VERSION}..."
echo "TIMESTAMP = $(date +%s)" > "$DISTINFO"

# Source tarball
echo "Fetching source tarball..."
TARBALL="FlumeIRC-flume-v${VERSION}_GH0.tar.gz"
curl -sL "https://codeload.github.com/FlumeIRC/flume/tar.gz/v${VERSION}" -o "$TMPDIR/$TARBALL"
SHA=$(shasum -a 256 "$TMPDIR/$TARBALL" | awk '{print $1}')
SIZE=$(wc -c < "$TMPDIR/$TARBALL" | tr -d ' ')
echo "SHA256 ($TARBALL) = $SHA" >> "$DISTINFO"
echo "SIZE ($TARBALL) = $SIZE" >> "$DISTINFO"

# Crate tarballs
echo "Fetching crate tarballs..."
python3 -c "
import re
with open('$REPO_ROOT/Cargo.lock') as f:
    content = f.read()
packages = re.findall(r'\[\[package\]\]\nname = \"(.+?)\"\nversion = \"(.+?)\"', content)
excluded = {'flume-core', 'flume-tui'}
for name, ver in sorted(set(packages)):
    if name not in excluded:
        print(f'{name} {ver}')
" | while read -r NAME VER; do
    CRATE_FILE="${NAME}-${VER}.crate"
    URL="https://crates.io/api/v1/crates/${NAME}/${VER}/download"
    curl -sL "$URL" -o "$TMPDIR/$CRATE_FILE"
    SHA=$(shasum -a 256 "$TMPDIR/$CRATE_FILE" | awk '{print $1}')
    SIZE=$(wc -c < "$TMPDIR/$CRATE_FILE" | tr -d ' ')
    echo "SHA256 ($CRATE_FILE) = $SHA" >> "$DISTINFO"
    echo "SIZE ($CRATE_FILE) = $SIZE" >> "$DISTINFO"
    echo "  $CRATE_FILE"
done

rm -rf "$TMPDIR"
echo "Generated $DISTINFO with $(grep -c SHA256 "$DISTINFO") entries"
