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

echo "Building release binary..."
source ~/.cargo/env
cargo build --release

echo "Setting up rpmbuild structure..."
RPM_ROOT=~/rpmbuild
mkdir -p $RPM_ROOT/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

echo "Preparing build files..."
cp target/release/juanita-banana $RPM_ROOT/BUILD/
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
Release:        1%{?dist}
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
cp %{_topdir}/BUILD/juanita-banana.desktop %{buildroot}/usr/share/applications/
cp %{_topdir}/BUILD/juanita-banana.png %{buildroot}/usr/share/pixmaps/

%files
/usr/bin/juanita-banana
/usr/share/applications/juanita-banana.desktop
/usr/share/pixmaps/juanita-banana.png
EOF

sed -i "s/VERSION_PLACEHOLDER/$NEW_VERSION/" $RPM_ROOT/SPECS/juanita-banana.spec

echo "Building RPM..."
rpmbuild -bb $RPM_ROOT/SPECS/juanita-banana.spec

echo "Done! RPM is available at:"
find $RPM_ROOT/RPMS -name "juanita-banana*.rpm"
