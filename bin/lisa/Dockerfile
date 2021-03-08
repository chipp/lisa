FROM ghcr.io/chipp/build.rust.x86_64_musl:1.50.0_4 AS builder

RUN apt-get update && apt-get install -y \
  autoconf \
  automake \
  libtool \
  python2.7 \
  gettext \
  --no-install-recommends && \
  rm -rf /var/lib/apt/lists/*

ARG HOST=x86_64-unknown-linux-musl

ENV FFI_VER="3.3" FFI_SHA="72fba7922703ddfa7a028d513ac15a85c8d54c8d67f55fa5a4802885dc652056"
RUN curl -sSOL https://github.com/libffi/libffi/releases/download/v3.3/libffi-${FFI_VER}.tar.gz && \
  echo "${FFI_SHA}  libffi-${FFI_VER}.tar.gz" | sha256sum -c - && \
  tar xfz libffi-${FFI_VER}.tar.gz && cd libffi-${FFI_VER} && \
  ./configure --host=$HOST --prefix=$PREFIX --disable-shared --enable-static && \
  make -j$(nproc) && make install && cd .. && rm -rf libffi-${FFI_VER}.tar.gz libffi-${FFI_VER}

ENV TEXT_VER="0.3.2" TEXT_SHA="a9a72cfa21853f7d249592a3c6f6d36f5117028e24573d092f9184ab72bbe187"
RUN curl -sSOL https://ftp.barfooze.de/pub/sabotage/tarballs/gettext-tiny-${TEXT_VER}.tar.xz && \
  echo "${TEXT_SHA}  gettext-tiny-${TEXT_VER}.tar.xz" | sha256sum -c - && \
  tar xfJ gettext-tiny-${TEXT_VER}.tar.xz && cd gettext-tiny-${TEXT_VER} && \
  make LIBINT=MUSL && make LIBINT=MUSL prefix=$PREFIX install && \
  cd .. && rm -rf gettext-tiny-${TEXT_VER}.tar.xz gettext-tiny-${TEXT_VER}

ENV GLIB_VER="2.58.3" GLIB_SHA="8f43c31767e88a25da72b52a40f3301fefc49a665b56dc10ee7cc9565cbe7481"
RUN curl -sSOL https://download.gnome.org/sources/glib/2.58/glib-${GLIB_VER}.tar.xz && \
  echo "${GLIB_SHA}  glib-${GLIB_VER}.tar.xz" | sha256sum -c - && \
  tar xfJ glib-${GLIB_VER}.tar.xz && cd glib-${GLIB_VER} && NOCONFIGURE=1 ./autogen.sh && \
  ./configure --host=$HOST --with-pcre=internal --disable-libmount \
  --disable-shared --enable-static --prefix=$PREFIX \
  glib_cv_stack_grows=no glib_cv_uscore=yes ac_cv_func_posix_getpwuid_r=yes \
  ac_cv_func_posix_getgrgid_r=yes PKG_CONFIG_PATH=$PREFIX/lib/pkgconfig && \
  make -j$(nproc) && make install && cd .. && rm -rf glib-${GLIB_VER}.tar.xz glib-${GLIB_VER}

ENV EXPAT_VER="2.2.10" EXPAT_SHA="bf42d1f52371d23684de36cc6d2f0f1acd02de264d1105bdc17792bbeb7e7ceb"
RUN curl -sSOL https://github.com/libexpat/libexpat/releases/download/R_2_2_10/expat-${EXPAT_VER}.tar.gz && \
  echo "${EXPAT_SHA}  expat-${EXPAT_VER}.tar.gz" | sha256sum -c - && \
  tar xfz expat-${EXPAT_VER}.tar.gz && cd expat-${EXPAT_VER} && \
  ./configure --host=$HOST --prefix=$PREFIX --disable-shared --enable-static && \
  make -j$(nproc) && make install && cd .. && rm -rf expat-${EXPAT_VER}.tar.gz expat-${EXPAT_VER}

ENV DBUS_VER="1.12.18" DBUS_SHA="64cf4d70840230e5e9bc784d153880775ab3db19d656ead8a0cb9c0ab5a95306"
RUN curl -sSOL https://dbus.freedesktop.org/releases/dbus/dbus-${DBUS_VER}.tar.gz && \
  echo "${DBUS_SHA}  dbus-${DBUS_VER}.tar.gz" | sha256sum -c - && \
  tar xfz dbus-${DBUS_VER}.tar.gz && cd dbus-${DBUS_VER} && \
  ./configure --host=$HOST --prefix=$PREFIX --disable-shared --enable-static \
  --disable-tests --disable-doxygen-docs --disable-xml-docs && \
  make -j$(nproc) && make install && cd .. && rm -rf dbus-${DBUS_VER}.tar.gz dbus-${DBUS_VER}

ENV BLUEZ_VER="5.50" BLUEZ_SHA="5ffcaae18bbb6155f1591be8c24898dc12f062075a40b538b745bfd477481911"
RUN curl -sSOL https://www.kernel.org/pub/linux/bluetooth/bluez-${BLUEZ_VER}.tar.xz; \
  echo "${BLUEZ_SHA} bluez-${BLUEZ_VER}.tar.xz" | sha256sum -c -; \
  tar xfJ bluez-${BLUEZ_VER}.tar.xz; cd bluez-${BLUEZ_VER}; \
  ./configure --host=armv7-unknown-linux-musleabihf --disable-shared --enable-static \
  --disable-test --disable-monitor --disable-tools --disable-client --disable-systemd \
  --disable-udev --disable-cups --disable-obex --enable-library --prefix=$PREFIX; \
  make && make install

WORKDIR /home/rust/src
RUN USER=rust \
  cargo new --lib /home/rust/src/lib/alice && \
  cargo new --lib /home/rust/src/lib/elisheba && \
  cargo new --lib /home/rust/src/lib/alzhbeta && \
  cargo new --bin /home/rust/src/bin/lisa && \
  cargo new --bin /home/rust/src/bin/isabel

COPY ./lib/alice/Cargo.toml ./lib/alice/Cargo.toml
COPY ./lib/elisheba/Cargo.toml ./lib/elisheba/Cargo.toml
COPY ./lib/alzhbeta/Cargo.toml ./lib/alzhbeta/Cargo.toml
COPY ./bin/lisa/Cargo.toml ./bin/lisa/Cargo.toml
COPY ./bin/isabel/Cargo.toml ./bin/isabel/Cargo.toml

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

ENV RUSTFLAGS=-L$PREFIX/lib

RUN cargo build --release -p lisa && \
  cargo clean --release -p lisa -p alice -p elisheba \
  --target x86_64-unknown-linux-musl

RUN cargo build && \
  cargo clean -p alice -p lisa -p elisheba -p isabel -p alzhbeta \
  --target x86_64-unknown-linux-musl && \
  rm ./bin/lisa/src/*.rs ./bin/isabel/src/*.rs ./lib/alice/src/*.rs \
  ./lib/elisheba/src/*.rs ./lib/alzhbeta/src/*.rs

COPY ./lib/alice/src ./lib/alice/src
COPY ./lib/elisheba/src ./lib/elisheba/src
COPY ./lib/alzhbeta/src ./lib/alzhbeta/src
COPY ./bin/lisa/src ./bin/lisa/src
COPY ./bin/isabel/src ./bin/isabel/src

RUN cargo test && \
  rm -rf target/x86_64-unknown-linux-musl/debug/ target/debug/

RUN cargo build --release -p lisa && \
  mv target/x86_64-unknown-linux-musl/release/lisa ./ && \
  rm -rf target/x86_64-unknown-linux-musl/release/ target/release/

FROM alpine:3.11
RUN apk --no-cache add ca-certificates && update-ca-certificates

WORKDIR /root/
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs
ENV RUST_BACKTRACE=full

COPY --from=0 /home/rust/src/lisa .
