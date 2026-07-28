#!/bin/bash
# Prepares the I2P router dependency at build time in bin/.
# If the user already has i2p installed on the system or running, we use the user's;
# otherwise, this script performs a headless auto-install of I2P into bin/ so Juanita Banana is self-contained.
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
BIN_DIR="$WORKSPACE_DIR/bin"

mkdir -p "$BIN_DIR"

echo "=========================================================="
echo "Preparing I2P Garlic Routing router dependency (v2.13.0)..."
echo "=========================================================="

if [ -f "$BIN_DIR/i2p.jar" ] || [ -f "$BIN_DIR/i2prouter" ] || [ -f "$BIN_DIR/i2p-rs" ]; then
    echo "I2P binary/jar already present in $BIN_DIR."
    exit 0
fi

# 1. Check existing system locations & PATH
if [ -f "/usr/share/i2p/i2p.jar" ]; then
    echo "Copying system i2p.jar to $BIN_DIR..."
    cp "/usr/share/i2p/i2p.jar" "$BIN_DIR/i2p.jar"
elif [ -f "/usr/bin/i2prouter" ]; then
    echo "Symlinking system i2prouter to $BIN_DIR..."
    ln -sf "/usr/bin/i2prouter" "$BIN_DIR/i2prouter"
elif command -v i2prouter >/dev/null 2>&1; then
    echo "Symlinking system i2prouter from PATH to $BIN_DIR..."
    ln -sf "$(command -v i2prouter)" "$BIN_DIR/i2prouter"
else
    echo "Performing headless automated installation of I2P (v2.13.0)..."
    TEMP_DIR="$(mktemp -d)"
    INSTALLER="$TEMP_DIR/i2pinstall.jar"
    INSTALL_TARGET="$WORKSPACE_DIR/build_temp/i2p_install"
    AUTO_XML="$TEMP_DIR/auto-install.xml"

    echo "Fetching installer for I2P v2.13.0..."
    curl -sSL "https://github.com/i2p/i2p.i2p/releases/download/i2p-2.13.0/i2pinstall_2.13.0.jar" -o "$INSTALLER" || \
    curl -sSL "https://files.i2p.net/2.13.0/i2pinstall_2.13.0.jar" -o "$INSTALLER" || \
    curl -sSL "https://download.i2p2.de/releases/2.13.0/i2pinstall_2.13.0.jar" -o "$INSTALLER" || true

    if [ -s "$INSTALLER" ]; then
        echo "Running IzPack automated headless installation..."
        cat <<EOF > "$AUTO_XML"
<AutomatedInstallation langpack="eng">
  <com.izforge.izpack.panels.hello.HelloPanel id="HelloPanel_0"/>
  <com.izforge.izpack.panels.info.InfoPanel id="InfoPanel_1"/>
  <com.izforge.izpack.panels.target.TargetPanel id="TargetPanel_2">
    <installpath>${INSTALL_TARGET}</installpath>
  </com.izforge.izpack.panels.target.TargetPanel>
  <com.izforge.izpack.panels.packs.PacksPanel id="PacksPanel_3">
    <pack name="Base" index="0" selected="true"/>
  </com.izforge.izpack.panels.packs.PacksPanel>
  <com.izforge.izpack.panels.install.InstallPanel id="InstallPanel_4"/>
  <com.izforge.izpack.panels.shortcut.ShortcutPanel id="ShortcutPanel_5"/>
  <com.izforge.izpack.panels.xinfo.XInfoPanel id="XInfoPanel_6"/>
  <com.izforge.izpack.panels.simplefinish.SimpleFinishPanel id="SimpleFinishPanel_7"/>
</AutomatedInstallation>
EOF

        java -jar "$INSTALLER" "$AUTO_XML" || true

        if [ -f "$INSTALL_TARGET/i2prouter" ]; then
            cp "$INSTALL_TARGET/i2prouter" "$BIN_DIR/i2prouter"
            chmod +x "$BIN_DIR/i2prouter"
        fi
        if [ -f "$INSTALL_TARGET/i2p.jar" ]; then
            cp "$INSTALL_TARGET/i2p.jar" "$BIN_DIR/i2p.jar"
        fi
        if [ -f "$INSTALL_TARGET/router.jar" ]; then
            cp "$INSTALL_TARGET/router.jar" "$BIN_DIR/router.jar"
        fi
        if [ -d "$INSTALL_TARGET/lib" ]; then
            cp -r "$INSTALL_TARGET/lib" "$BIN_DIR/" 2>/dev/null || true
        fi
    else
        echo "ERROR: Failed to download non-empty i2pinstall_2.13.0.jar"
        exit 1
    fi
    rm -rf "$TEMP_DIR"
fi

echo "=========================================================="
echo "I2P router dependency setup complete in $BIN_DIR."
echo "=========================================================="
