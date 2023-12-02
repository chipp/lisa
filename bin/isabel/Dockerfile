FROM ghcr.io/chipp/build.rust.armv7_musl:1.74.0_4 as libs_builder

ENV HOST=armv7-unknown-linux-musleabihf
ENV PKG_CONFIG_PATH=$PREFIX/lib/pkgconfig

COPY ./bin/isabel/install_static_libs.sh ./install_static_libs.sh
RUN chmod +x ./install_static_libs.sh && \
  ./install_static_libs.sh && \
  rm ./install_static_libs.sh

FROM ghcr.io/chipp/build.rust.armv7_musl:1.74.0_4 AS builder

COPY --from=0 $PREFIX $PREFIX

WORKDIR /home/rust/src
RUN USER=rust \
  cargo new --lib /home/rust/src/lib/alice && \
  cargo new --lib /home/rust/src/lib/bluetooth && \
  cargo new --lib /home/rust/src/lib/inspinia && \
  cargo new --lib /home/rust/src/lib/str_derive && \
  cargo new --lib /home/rust/src/lib/transport && \
  cargo new --lib /home/rust/src/lib/xiaomi && \
  cargo new --bin /home/rust/src/bin/alisa && \
  cargo new --bin /home/rust/src/bin/elisa && \
  cargo new --bin /home/rust/src/bin/elizabeth && \
  cargo new --bin /home/rust/src/bin/isabel

COPY ./bin/isabel/Cargo.toml ./bin/isabel/Cargo.toml
COPY ./lib/bluetooth/Cargo.toml ./lib/bluetooth/Cargo.toml
COPY ./lib/str_derive/Cargo.toml ./lib/str_derive/Cargo.toml
COPY ./lib/str_derive/fake_macro.rs ./lib/str_derive/src/lib.rs
COPY ./lib/transport/Cargo.toml ./lib/transport/Cargo.toml

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release \
  -p isabel \
  -p bluetooth \
  -p str_derive \
  -p transport && \
  cargo clean --release \
  -p isabel \
  -p bluetooth \
  -p str_derive \
  -p transport \
  --target armv7-unknown-linux-musleabihf && \
  rm ./bin/isabel/src/*.rs \
  ./lib/bluetooth/src/*.rs \
  ./lib/str_derive/src/*.rs \
  ./lib/transport/src/*.rs

COPY ./bin/isabel/src ./bin/isabel/src
COPY ./lib/bluetooth/src ./lib/bluetooth/src
COPY ./lib/bluetooth/build.rs ./lib/bluetooth/build.rs
COPY ./lib/str_derive/src ./lib/str_derive/src
COPY ./lib/transport/src ./lib/transport/src

RUN cargo build --release -p isabel --target armv7-unknown-linux-musleabihf -vv && \
  mv target/armv7-unknown-linux-musleabihf/release/isabel ./ && \
  rm -rf target/armv7-unknown-linux-musleabihf/release/ target/release/

FROM alpine:3.18.4
WORKDIR /root/

COPY --from=builder /home/rust/src/isabel .
