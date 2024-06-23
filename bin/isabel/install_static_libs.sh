#!/bin/bash

set -e
set -o pipefail

apt-get update && apt-get install -y --no-install-recommends \
  autoconf \
  automake \
  gettext \
  libtool \
  musl \
  ninja-build \
  python3 \
  python3-venv

case $TARGET in
  aarch64-linux-musl)
    CPU_FAMILY="aarch64"
    CPU="aarch64"
    ENDIAN="little"
    ;;
  x86_64-unknown-linux-musl)
    CPU_FAMILY="x86_64"
    CPU="x86_64"
    ENDIAN="little"
    ;;
  *)
    echo "Unknown target: $TARGET" >&2
    exit 1
    ;;
esac

rm -rf /var/lib/apt/lists/*

python3 -m venv python
PATH=/python/bin:$PATH
pip3 install meson

tee meson.cross <<EOF
[binaries]
c = '/musl/bin/${CROSS_PREFIX}gcc'
cpp = '/musl/bin/${CROSS_PREFIX}g++'
ar = '/musl/bin/${CROSS_PREFIX}gcc-ar'
nm = '/musl/bin/${CROSS_PREFIX}gcc-nm'
ld = '/musl/bin/${CROSS_PREFIX}ld'
strip = '/musl/bin/${CROSS_PREFIX}strip'
pkg-config = '/usr/bin/pkg-config'

[host_machine]
system = 'linux'
cpu_family = '${CPU_FAMILY}'
cpu = '${CPU}'
endian = '${ENDIAN}'
EOF

FFI_VER="3.4.6"     FFI_SHA="b0dea9df23c863a7a50e825440f3ebffabd65df1497108e5d437747843895a4e"
EXPAT_VER="2.6.2"   EXPAT_SHA="d4cf38d26e21a56654ffe4acd9cd5481164619626802328506a2869afab29ab3"
GLIB_VER="2.80.3"   GLIB_SHA="3947a0eaddd0f3613d0230bb246d0c69e46142c19022f5c4b1b2e3cba236d417"
DBUS_VER="1.15.8"   DBUS_SHA="84fc597e6ec82f05dc18a7d12c17046f95bad7be99fc03c15bc254c4701ed204"
BLUEZ_VER="5.66"    BLUEZ_SHA="39fea64b590c9492984a0c27a89fc203e1cdc74866086efb8f4698677ab2b574"

curl -sSOL https://github.com/libffi/libffi/releases/download/v${FFI_VER}/libffi-${FFI_VER}.tar.gz
echo "${FFI_SHA}  libffi-${FFI_VER}.tar.gz" | sha256sum -c -
tar xfz libffi-${FFI_VER}.tar.gz
cd libffi-${FFI_VER}

./configure --host=$TARGET --prefix=$PREFIX --disable-shared --enable-static
make -j$(nproc) && make install
cd .. && rm -rf libffi-${FFI_VER}.tar.gz libffi-${FFI_VER}


EXPAT_TAG=R_$(printf ${EXPAT_VER} | tr . _)
curl -sSOL https://github.com/libexpat/libexpat/releases/download/${EXPAT_TAG}/expat-${EXPAT_VER}.tar.gz
echo "${EXPAT_SHA}  expat-${EXPAT_VER}.tar.gz" | sha256sum -c -
tar xfz expat-${EXPAT_VER}.tar.gz
cd expat-${EXPAT_VER}

./configure --host=$TARGET --prefix=$PREFIX --disable-shared --enable-static
make -j$(nproc) && make install
cd .. && rm -rf expat-${EXPAT_VER}.tar.gz expat-${EXPAT_VER}


GLIB_MAJOR_MINOR=$(echo $GLIB_VER | cut -d. -f1-2)
curl -sSOL https://download.gnome.org/sources/glib/${GLIB_MAJOR_MINOR}/glib-${GLIB_VER}.tar.xz
echo "${GLIB_SHA}  glib-${GLIB_VER}.tar.xz" | sha256sum -c -
tar xfJ glib-${GLIB_VER}.tar.xz
cd glib-${GLIB_VER}

pip3 install packaging
meson setup --cross-file ../meson.cross --prefix $PREFIX --pkg-config-path $PKG_CONFIG_PATH \
  --default-library static -Dlibmount=disabled -Dselinux=disabled \
  -Dtests=false _build
meson compile -C _build
meson install -C _build
cd .. && rm -rf glib-${GLIB_VER}.tar.xz glib-${GLIB_VER}


curl -sSOL https://dbus.freedesktop.org/releases/dbus/dbus-${DBUS_VER}.tar.xz
echo "${DBUS_SHA}  dbus-${DBUS_VER}.tar.xz" | sha256sum -c -
tar xfJ dbus-${DBUS_VER}.tar.xz
cd dbus-${DBUS_VER}

meson setup --cross-file ../meson.cross --prefix $PREFIX --pkg-config-path $PKG_CONFIG_PATH \
  --default-library static -Ddoxygen_docs=disabled -Dducktype_docs=disabled \
  -Dmessage_bus=false -Dxml_docs=disabled _build
meson compile -C _build
meson install -C _build
cd .. && rm -rf dbus-${DBUS_VER}.tar.gz dbus-${DBUS_VER}


curl -sSOL https://www.kernel.org/pub/linux/bluetooth/bluez-${BLUEZ_VER}.tar.xz
echo "${BLUEZ_SHA} bluez-${BLUEZ_VER}.tar.xz" | sha256sum -c -
tar xfJ bluez-${BLUEZ_VER}.tar.xz
cd bluez-${BLUEZ_VER}

patch -p0 <<EOF
--- src/shared/util.c 2022-11-10 20:24:03.000000000 +0000
+++ src/shared/util.c 2024-06-23 09:05:23.632007315 +0000
@@ -28,6 +28,11 @@
 #include <sys/random.h>
 #endif

+/* define MAX_INPUT for musl */
+#ifndef MAX_INPUT
+#define MAX_INPUT _POSIX_MAX_INPUT
+#endif
+
 #include "src/shared/util.h"

 void *util_malloc(size_t size)
EOF

./configure --host=$TARGET --prefix=$PREFIX --disable-shared --enable-static \
  --disable-test --disable-monitor --disable-tools --disable-client --disable-systemd \
  --disable-udev --disable-cups --disable-obex --enable-library --disable-manpages
make -j$(nproc) && make install
cd .. && rm -rf bluez-${BLUEZ_VER}.tar.xz bluez-${BLUEZ_VER}
