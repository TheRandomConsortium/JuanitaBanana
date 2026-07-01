#!/bin/bash
set -e

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
Exec=juanita-banana
Icon=juanita-banana
Terminal=false
Type=Application
Categories=Network;WebBrowser;
EOF

cat > $RPM_ROOT/SPECS/juanita-banana.spec << 'EOF'
Name:           juanita-banana
Version:        0.1.0
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

echo "Building RPM..."
rpmbuild -bb $RPM_ROOT/SPECS/juanita-banana.spec

echo "Done! RPM is available at:"
find $RPM_ROOT/RPMS -name "juanita-banana*.rpm"
