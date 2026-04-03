#!/bin/bash
# Generate a shar archive of the FreeBSD port for submission.
# Usage: ./generate-shar.sh [output_file]
#
# Run from the repo root or the packaging/freebsd directory.

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUTPUT="${1:-flume-port.shar}"

# Find the port files
if [ -f "$SCRIPT_DIR/Makefile" ]; then
    PORT_DIR="$SCRIPT_DIR"
elif [ -f "packaging/freebsd/Makefile" ]; then
    PORT_DIR="packaging/freebsd"
else
    echo "Error: Can't find port Makefile" >&2
    exit 1
fi

# Check for shar (may not be available on macOS)
if command -v shar &>/dev/null; then
    cd "$PORT_DIR"
    shar Makefile pkg-descr pkg-plist > "$OLDPWD/$OUTPUT"
    echo "Created $OUTPUT"
else
    # Fallback: create a self-extracting shell script manually
    echo "shar not available, creating portable archive..."
    cd "$PORT_DIR"

    cat > "$OLDPWD/$OUTPUT" <<'HEADER'
#!/bin/sh
# This is a shell archive. To extract, run: sh flume-port.shar
echo x - Makefile
HEADER

    for f in Makefile pkg-descr pkg-plist; do
        LINES=$(wc -l < "$f" | tr -d ' ')
        cat >> "$OLDPWD/$OUTPUT" <<EOF
cat > '$f' << 'SHAR_EOF_$f'
$(cat "$f")
SHAR_EOF_$f
echo x - $f
EOF
    done

    echo "Created $OUTPUT"
fi
