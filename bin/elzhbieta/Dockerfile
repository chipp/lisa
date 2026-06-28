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
  cargo new --lib /home/rust/src/lib/roborock && \
  cargo new --lib /home/rust/src/lib/tuya && \
  cargo new --bin /home/rust/src/bin/alisa && \
  cargo new --bin /home/rust/src/bin/elisa && \
  cargo new --bin /home/rust/src/bin/elisheba && \
  cargo new --bin /home/rust/src/bin/elzhbieta && \
  cargo new --bin /home/rust/src/bin/elizabeth && \
  cargo new --bin /home/rust/src/bin/isabel

COPY ./bin/elzhbieta/Cargo.toml ./bin/elzhbieta/Cargo.toml
COPY ./lib/crypto/Cargo.toml ./lib/crypto/Cargo.toml
COPY ./lib/str_derive/Cargo.toml ./lib/str_derive/Cargo.toml
COPY ./lib/str_derive/fake_macro.rs ./lib/str_derive/src/lib.rs
COPY ./lib/transport/Cargo.toml ./lib/transport/Cargo.toml
COPY ./lib/tuya/Cargo.toml ./lib/tuya/Cargo.toml

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release \
  -p elzhbieta \
  -p crypto \
  -p str_derive \
  -p transport \
  -p tuya && \
  cargo clean --release \
  -p elzhbieta \
  -p crypto \
  -p str_derive \
  -p transport \
  -p tuya \
  --target aarch64-unknown-linux-musl && \
  rm ./bin/elzhbieta/src/*.rs \
  ./lib/crypto/src/*.rs \
  ./lib/str_derive/src/*.rs \
  ./lib/transport/src/*.rs \
  ./lib/tuya/src/*.rs

COPY ./lib/crypto/src ./lib/crypto/src
COPY ./lib/str_derive/src ./lib/str_derive/src
COPY ./lib/transport/src ./lib/transport/src
COPY ./lib/tuya/src ./lib/tuya/src
COPY ./bin/elzhbieta/src ./bin/elzhbieta/src

ARG VERSION=0.1.0
RUN sed -i "s/version = \"0.1.0\"/version = \"${VERSION}\"/g" bin/elzhbieta/Cargo.toml

RUN cargo build --release -p elzhbieta --target aarch64-unknown-linux-musl && \
  mv target/aarch64-unknown-linux-musl/release/elzhbieta ./ && \
  rm -rf target/aarch64-unknown-linux-musl/release/ target/release/

FROM alpine:3.18.4
WORKDIR /root/

COPY --from=builder /home/rust/src/elzhbieta .
