#!/bin/bash
# Downloads and compiles torsocks locally, installing it under bin/torsocks_env.
# This ensures torsocks is available for HNS-over-Tor without requiring root/system-wide installation.
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="$WORKSPACE_DIR/build_temp"
BIN_DIR="$WORKSPACE_DIR/bin"

echo "=========================================================="
echo "Building torsocks locally..."
echo "This is a one-time build — subsequent builds skip this step."
echo "=========================================================="

mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

if [ ! -d "torsocks" ]; then
    echo "Cloning torsocks..."
    git clone --depth 1 https://gitlab.torproject.org/tpo/core/torsocks.git torsocks
fi

cd torsocks
echo "Configuring torsocks..."
./autogen.sh
./configure --prefix="$BIN_DIR/torsocks_env"
echo "Compiling and installing torsocks..."
make -j$(nproc)
make install

# Create local wrapper script at bin/torsocks
rm -f "$BIN_DIR/torsocks"
cat > "$BIN_DIR/torsocks" << 'EOF'
#!/bin/bash
# Wrapper for local torsocks installation
TORSOCKS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -d "$TORSOCKS_DIR/torsocks_env" ]; then
    ENV_DIR="$TORSOCKS_DIR/torsocks_env"
elif [ -d "/usr/lib/juanita-banana/torsocks_env" ]; then
    ENV_DIR="/usr/lib/juanita-banana/torsocks_env"
else
    # Fallback
    ENV_DIR="$TORSOCKS_DIR/torsocks_env"
fi
export LD_PRELOAD="$ENV_DIR/lib/torsocks/libtorsocks.so"
export TORSOCKS_CONF_FILE="$ENV_DIR/etc/tor/torsocks.conf"
exec "$ENV_DIR/bin/torsocks" "$@"
EOF
chmod +x "$BIN_DIR/torsocks"

echo "=========================================================="
echo "torsocks compiled and installed successfully to bin/torsocks"
echo "=========================================================="
