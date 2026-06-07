#!/bin/sh
# MemoryKnight installer — downloads the latest prebuilt Linux x86_64 binary.
#
#   curl -fsSL https://raw.githubusercontent.com/FranciscoJBrito/mknight/main/install.sh | sh
#
# Installs to ~/.local/bin by default. Override with MKNIGHT_INSTALL_DIR=/some/dir.
# For macOS or other platforms, use: cargo install mknight
set -eu

REPO="FranciscoJBrito/mknight"
BIN="mknight"
TARGET="x86_64-unknown-linux-gnu"
ASSET="${BIN}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
BINDIR="${MKNIGHT_INSTALL_DIR:-$HOME/.local/bin}"

# --- Platform check ---------------------------------------------------------
os="$(uname -s)"
arch="$(uname -m)"
if [ "$os" != "Linux" ]; then
    echo "error: this installer only supports Linux." >&2
    echo "  On $os, install with:  cargo install mknight" >&2
    exit 1
fi
case "$arch" in
    x86_64 | amd64) ;;
    *)
        echo "error: unsupported architecture '$arch' (only x86_64 is prebuilt)." >&2
        echo "  Build from source with:  cargo install mknight" >&2
        exit 1
        ;;
esac

# --- Download ---------------------------------------------------------------
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo "Downloading $ASSET ..."
if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$URL" -o "$tmp/$ASSET"
elif command -v wget >/dev/null 2>&1; then
    wget -qO "$tmp/$ASSET" "$URL"
else
    echo "error: need either curl or wget installed." >&2
    exit 1
fi

# --- Install ----------------------------------------------------------------
tar -xzf "$tmp/$ASSET" -C "$tmp"
if [ ! -f "$tmp/$BIN" ]; then
    echo "error: '$BIN' not found in the downloaded archive." >&2
    exit 1
fi

mkdir -p "$BINDIR"
install -m 0755 "$tmp/$BIN" "$BINDIR/$BIN"
echo "Installed $BIN to $BINDIR/$BIN"

# --- PATH hint --------------------------------------------------------------
case ":$PATH:" in
    *":$BINDIR:"*) ;;
    *)
        echo ""
        echo "note: $BINDIR is not in your PATH. Add it with:"
        echo "  echo 'export PATH=\"$BINDIR:\$PATH\"' >> ~/.bashrc   # or ~/.zshrc"
        echo "then restart your shell."
        ;;
esac

echo ""
echo "Done. Try:  $BIN --help"
