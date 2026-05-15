#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 4 ]; then
  echo "usage: $0 <service> <version> <binary-path> <output-dir>" >&2
  exit 64
fi

service="$1"
version="$2"
binary_path="$3"
output_dir="$4"
arch="${ARCH:-arm64}"

case "$service" in
  elisa|elisheba|isabel) ;;
  *)
    echo "unsupported service: $service" >&2
    exit 64
    ;;
esac

if [ ! -f "$binary_path" ]; then
  echo "binary not found: $binary_path" >&2
  exit 66
fi

depends="ca-certificates, systemd"
if [ "$service" = "isabel" ]; then
  depends="$depends, bluez"
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_dir="$(cd "$script_dir/.." && pwd)"
package_root="$(mktemp -d)"
trap 'rm -rf "$package_root"' EXIT

install -d \
  "$package_root/DEBIAN" \
  "$package_root/usr/bin" \
  "$package_root/lib/systemd/system" \
  "$package_root/etc/lisa"

installed_size="$(du -k "$binary_path" | awk '{ print $1 }')"

cat > "$package_root/DEBIAN/control" <<EOF
Package: $service
Version: $version
Section: utils
Priority: optional
Architecture: $arch
Maintainer: chipp
Depends: $depends
Installed-Size: $installed_size
Description: Lisa $service service
EOF

cat > "$package_root/DEBIAN/postinst" <<EOF
#!/bin/sh
set -e

install -d -m 700 /etc/lisa
install -d -m 755 /var/lib/lisa

if ! getent passwd $service >/dev/null 2>&1; then
  useradd \
    --system \
    --user-group \
    --home-dir /var/lib/lisa/$service \
    --no-create-home \
    --shell /usr/sbin/nologin \
    $service
fi

install -d -m 755 -o $service -g $service /var/lib/lisa/$service

if command -v systemctl >/dev/null 2>&1; then
  systemctl daemon-reload || true
  systemctl enable $service.service || true

  if [ -f /etc/lisa/$service.env ]; then
    systemctl restart $service.service || true
  fi
fi
EOF

cat > "$package_root/DEBIAN/prerm" <<EOF
#!/bin/sh
set -e

if [ "\$1" = "remove" ] || [ "\$1" = "purge" ]; then
  if command -v systemctl >/dev/null 2>&1; then
    systemctl stop $service.service || true
  fi
fi
EOF

cat > "$package_root/DEBIAN/postrm" <<EOF
#!/bin/sh
set -e

if command -v systemctl >/dev/null 2>&1; then
  systemctl daemon-reload || true
fi
EOF

chmod 755 \
  "$package_root/DEBIAN/postinst" \
  "$package_root/DEBIAN/prerm" \
  "$package_root/DEBIAN/postrm"

install -m 755 "$binary_path" "$package_root/usr/bin/$service"
install -m 644 "$repo_dir/packaging/deb/systemd/$service.service" \
  "$package_root/lib/systemd/system/$service.service"
install -m 644 "$repo_dir/packaging/deb/env/$service.env.example" \
  "$package_root/etc/lisa/$service.env.example"

install -d "$output_dir"
dpkg-deb --build --root-owner-group "$package_root" \
  "$output_dir/${service}_${version}_${arch}.deb"
