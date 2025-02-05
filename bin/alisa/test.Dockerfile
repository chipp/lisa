ARG RUST_VERSION=1.79.0_3

FROM ghcr.io/chipp/build.rust.arm64_musl:${RUST_VERSION} AS builder

WORKDIR /home/rust/src
RUN USER=rust \
  cargo new --lib /home/rust/src/lib/alice && \
  cargo new --lib /home/rust/src/lib/bluetooth && \
  cargo new --lib /home/rust/src/lib/crypto && \
  cargo new --lib /home/rust/src/lib/inspinia && \
  cargo new --lib /home/rust/src/lib/str_derive && \
  cargo new --lib /home/rust/src/lib/sonoff && \
  cargo new --lib /home/rust/src/lib/transport && \
  cargo new --lib /home/rust/src/lib/xiaomi && \
  cargo new --bin /home/rust/src/bin/alisa && \
  cargo new --bin /home/rust/src/bin/elisa && \
  cargo new --bin /home/rust/src/bin/elisheba && \
  cargo new --bin /home/rust/src/bin/elizabeth && \
  cargo new --bin /home/rust/src/bin/isabel

COPY ./bin/alisa/Cargo.toml ./bin/alisa/Cargo.toml
COPY ./lib/alice/Cargo.toml ./lib/alice/Cargo.toml
COPY ./lib/str_derive/Cargo.toml ./lib/str_derive/Cargo.toml
COPY ./lib/str_derive/fake_macro.rs ./lib/str_derive/src/lib.rs
COPY ./lib/transport/Cargo.toml ./lib/transport/Cargo.toml

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build \
  -p alisa \
  -p alice \
  -p str_derive \
  -p transport && \
  cargo clean \
  -p alisa \
  -p alice \
  -p str_derive \
  -p transport \
  --target aarch64-unknown-linux-musl && \
  rm \
  ./bin/alisa/src/*.rs \
  ./lib/alice/src/*.rs \
  ./lib/str_derive/src/*.rs \
  ./lib/transport/src/*.rs

COPY ./lib/alice/src ./lib/alice/src
COPY ./lib/str_derive/src ./lib/str_derive/src
COPY ./lib/transport/src ./lib/transport/src
COPY ./bin/alisa/src ./bin/alisa/src

RUN cargo test \
  -p alisa \
  -p alice \
  -p str_derive \
  -p transport && \
  rm -rf target/aarch64-unknown-linux-musl/debug/ target/debug/
