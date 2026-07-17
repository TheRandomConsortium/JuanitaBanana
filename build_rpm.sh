#!/bin/bash
set -e

BUMP_TYPE=${1:-patch}

CURRENT_VERSION=$(grep -E '^version = ' Cargo.toml | head -n 1 | awk -F '"' '{print $2}')
IFS='.' read -r major minor patch <<< "$CURRENT_VERSION"

if [ "$BUMP_TYPE" == "major" ]; then
    major=$((major + 1))
    minor=0
    patch=0
elif [ "$BUMP_TYPE" == "minor" ]; then
    minor=$((minor + 1))
    patch=0
elif [ "$BUMP_TYPE" == "patch" ]; then
    patch=$((patch + 1))
fi

if [ "$BUMP_TYPE" != "none" ]; then
    NEW_VERSION="$major.$minor.$patch"
    echo "Bumping version from $CURRENT_VERSION to $NEW_VERSION..."
    sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
else
    NEW_VERSION=$CURRENT_VERSION
    echo "Keeping version at $NEW_VERSION..."
fi

# Find highest release number across all built versions in local repo or rpmbuild RPMS
REPO_DIR="$HOME/juanita-repo"
RPM_ROOT=~/rpmbuild
RELEASE=1
EXISTING_RPMS=$(find "$REPO_DIR" "$RPM_ROOT/RPMS" -name "juanita-banana-*.rpm" 2>/dev/null || true)
if [ -n "$EXISTING_RPMS" ]; then
    MAX_RELEASE=0
    for rpm in $EXISTING_RPMS; do
        fname=$(basename "$rpm")
        # Strip "juanita-banana-" prefix
        rel_part="${fname#juanita-banana-}"
        # Strip everything up to the first "-" (which separates version and release)
        rel_part="${rel_part#*-}"
        # Extract the release number (everything before the first ".")
        rel_num="${rel_part%%.*}"
        if [[ "$rel_num" =~ ^[0-9]+$ ]]; then
            if [ "$rel_num" -gt "$MAX_RELEASE" ]; then
                MAX_RELEASE=$rel_num
            fi
        fi
    done
    if [ "$MAX_RELEASE" -gt 0 ]; then
        RELEASE=$((MAX_RELEASE + 1))
    fi
fi
echo "Using Release: $RELEASE"

echo "Building release binary..."
source ~/.cargo/env
cargo build --release

echo "Setting up rpmbuild structure..."
RPM_ROOT=~/rpmbuild
mkdir -p $RPM_ROOT/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

echo "Preparing build files..."
cp target/release/juanita-banana $RPM_ROOT/BUILD/
cp bin/hnsd $RPM_ROOT/BUILD/
cp bin/arti $RPM_ROOT/BUILD/ 2>/dev/null || echo "WARNING: bin/arti not found — Tor transport will not be available in this RPM."
cp assets/icon.png $RPM_ROOT/BUILD/juanita-banana.png

cat > $RPM_ROOT/BUILD/juanita-banana.desktop << 'EOF'
[Desktop Entry]
Name=Juanita Banana
Comment=A browser that fights back
Exec=juanita-banana %U
Icon=juanita-banana
Terminal=false
Type=Application
Categories=Network;WebBrowser;
MimeType=text/html;text/xml;application/xhtml+xml;x-scheme-handler/http;x-scheme-handler/https;x-scheme-handler/juanita;
StartupNotify=true
EOF

cat > $RPM_ROOT/SPECS/juanita-banana.spec << 'EOF'
Name:           juanita-banana
Version:        VERSION_PLACEHOLDER
Release:        RELEASE_PLACEHOLDER%{?dist}
Summary:        A browser that fights back
License:        MPL-2.0
BuildArch:      x86_64

%description
Juanita Banana is a bare-metal browser built entirely in Rust, currently powered by WebKitGTK. It fights back against the surveillance economy.

%install
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/usr/share/applications
mkdir -p %{buildroot}/usr/share/pixmaps

cp %{_topdir}/BUILD/juanita-banana %{buildroot}/usr/bin/
cp %{_topdir}/BUILD/hnsd %{buildroot}/usr/bin/
if [ -f %{_topdir}/BUILD/arti ]; then cp %{_topdir}/BUILD/arti %{buildroot}/usr/bin/; fi
cp %{_topdir}/BUILD/juanita-banana.desktop %{buildroot}/usr/share/applications/
cp %{_topdir}/BUILD/juanita-banana.png %{buildroot}/usr/share/pixmaps/

%files
/usr/bin/juanita-banana
/usr/bin/hnsd
%ghost /usr/bin/arti
/usr/share/applications/juanita-banana.desktop
/usr/share/pixmaps/juanita-banana.png
EOF

sed -i "s/VERSION_PLACEHOLDER/$NEW_VERSION/" $RPM_ROOT/SPECS/juanita-banana.spec
sed -i "s/RELEASE_PLACEHOLDER/$RELEASE/" $RPM_ROOT/SPECS/juanita-banana.spec

echo "Building RPM..."
rpmbuild -bb $RPM_ROOT/SPECS/juanita-banana.spec

REPO_DIR="$HOME/juanita-repo"
mkdir -p "$REPO_DIR"

echo "Copying new RPM to local repository at $REPO_DIR..."
find $RPM_ROOT/RPMS -name "juanita-banana-${NEW_VERSION}-${RELEASE}*.rpm" -exec cp {} "$REPO_DIR/" \;

echo "Cleaning up older RPM versions in local repository (keeping the latest 3)..."
pushd "$REPO_DIR" > /dev/null
ls -t juanita-banana-*.rpm 2>/dev/null | tail -n +4 | xargs -I {} rm -f {}
popd > /dev/null

echo "Cleaning up older RPM versions in rpmbuild RPMS (keeping the latest 3)..."
pushd $RPM_ROOT/RPMS/x86_64 > /dev/null
ls -t juanita-banana-*.rpm 2>/dev/null | tail -n +4 | xargs -I {} rm -f {}
popd > /dev/null

echo "Updating DNF repository metadata..."
if command -v createrepo_c >/dev/null 2>&1; then
    createrepo_c "$REPO_DIR"
elif command -v createrepo >/dev/null 2>&1; then
    createrepo "$REPO_DIR"
else
    echo "Warning: 'createrepo_c' or 'createrepo' is not installed."
    echo "Please install it: sudo dnf install -y createrepo_c"
fi

cat > juanita.repo << EOF
[juanita-banana]
name=Juanita Banana Local Repository
baseurl=file://$REPO_DIR
enabled=1
gpgcheck=0
metadata_expire=0
EOF

echo "Done! RPM is available in the local repository at $REPO_DIR"
echo ""
echo "To use this as a local DNF repository:"
echo "  1. If not already done, install createrepo_c:"
echo "     sudo dnf install -y createrepo_c"
echo "  2. Copy the repo file to your system configuration:"
echo "     sudo cp juanita.repo /etc/yum.repos.d/"
echo "  3. Update/Install via DNF:"
echo "     sudo dnf upgrade --refresh juanita-banana"
