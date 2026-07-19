#!/bin/bash
# Downloads and installs the arti binary at build time, placing it alongside hnsd in bin/.
# arti is the Tor Project's official Rust implementation of Tor.
# Source: https://gitlab.torproject.org/tpo/core/arti
#
# Strategy: install via `cargo install arti` into a temporary prefix, then copy the
# resulting binary to bin/. This ensures we always get the correct architecture
# without maintaining pre-built binaries or platform-specific URLs.
#
# NOTE: This is the interim strategy while we use the subprocess approach.
# The target architecture (Phase 4) embeds arti-client in-process and will
# not need this script.
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BIN_DIR="$WORKSPACE_DIR/bin"

mkdir -p "$BIN_DIR"

echo "=========================================================="
echo "Building arti (Tor Project Rust implementation)..."
echo "This is a one-time build — subsequent builds skip this step."
echo "=========================================================="

# Install arti into a temporary cargo root so we don't pollute $CARGO_HOME
ARTI_CARGO_ROOT="$WORKSPACE_DIR/build_temp/arti_cargo_root"
mkdir -p "$ARTI_CARGO_ROOT"

cargo install \
    --root "$ARTI_CARGO_ROOT" \
    --features "tokio,static-sqlite" \
    arti

cp "$ARTI_CARGO_ROOT/bin/arti" "$BIN_DIR/arti"
chmod +x "$BIN_DIR/arti"

echo "=========================================================="
echo "arti installed successfully to $BIN_DIR/arti"
echo "=========================================================="
