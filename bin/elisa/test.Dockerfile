ARG RUST_VERSION=1.79.0_3

FROM ghcr.io/chipp/build.rust.x86_64_musl:${RUST_VERSION} AS builder

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

COPY ./bin/elisa/Cargo.toml ./bin/elisa/Cargo.toml
COPY ./lib/crypto/Cargo.toml ./lib/crypto/Cargo.toml
COPY ./lib/str_derive/Cargo.toml ./lib/str_derive/Cargo.toml
COPY ./lib/str_derive/fake_macro.rs ./lib/str_derive/src/lib.rs
COPY ./lib/transport/Cargo.toml ./lib/transport/Cargo.toml
COPY ./lib/xiaomi/Cargo.toml ./lib/xiaomi/Cargo.toml

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build \
  -p elisa \
  -p crypto \
  -p str_derive \
  -p transport \
  -p xiaomi && \
  cargo clean \
  -p elisa \
  -p crypto \
  -p str_derive \
  -p transport \
  -p xiaomi \
  --target x86_64-unknown-linux-musl && \
  rm ./bin/elisa/src/*.rs \
  ./lib/crypto/src/*.rs \
  ./lib/str_derive/src/*.rs \
  ./lib/transport/src/*.rs \
  ./lib/xiaomi/src/*.rs

COPY ./lib/crypto/src ./lib/crypto/src
COPY ./lib/str_derive/src ./lib/str_derive/src
COPY ./lib/transport/src ./lib/transport/src
COPY ./lib/xiaomi/src ./lib/xiaomi/src
COPY ./bin/elisa/src ./bin/elisa/src

RUN cargo test -p elisa -p crypto -p str_derive -p transport -p xiaomi && \
  rm -rf target/x86_64-unknown-linux-musl/debug/ target/debug/
