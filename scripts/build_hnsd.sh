#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="$WORKSPACE_DIR/build_temp"

echo "Creating build directory..."
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

# 1. Clone hnsd if not exists
if [ ! -d "hnsd" ]; then
    echo "Cloning hnsd..."
    git clone --depth 1 https://github.com/handshake-org/hnsd.git hnsd
fi

# 2. Build hnsd linking to system unbound
cd hnsd
echo "Configuring hnsd..."
./autogen.sh
./configure
echo "Compiling hnsd..."
make -j$(nproc)

# 3. Copy hnsd to workspace bin/
mkdir -p "$WORKSPACE_DIR/bin"
cp hnsd "$WORKSPACE_DIR/bin/hnsd"

echo "=========================================================="
echo "hnsd compiled successfully and copied to $WORKSPACE_DIR/bin/hnsd"
echo "=========================================================="
