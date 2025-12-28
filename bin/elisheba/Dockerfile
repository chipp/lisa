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
  cargo new --bin /home/rust/src/bin/alisa && \
  cargo new --bin /home/rust/src/bin/elisa && \
  cargo new --bin /home/rust/src/bin/elisheba && \
  cargo new --bin /home/rust/src/bin/elizabeth && \
  cargo new --bin /home/rust/src/bin/isabel

COPY ./bin/elisheba/Cargo.toml ./bin/elisheba/Cargo.toml
COPY ./lib/crypto/Cargo.toml ./lib/crypto/Cargo.toml
COPY ./lib/sonoff/Cargo.toml ./lib/sonoff/Cargo.toml
COPY ./lib/str_derive/Cargo.toml ./lib/str_derive/Cargo.toml
COPY ./lib/str_derive/fake_macro.rs ./lib/str_derive/src/lib.rs
COPY ./lib/transport/Cargo.toml ./lib/transport/Cargo.toml

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release \
  -p elisheba \
  -p crypto \
  -p sonoff \
  -p str_derive \
  -p transport && \
  cargo clean --release \
  -p elisheba \
  -p crypto \
  -p sonoff \
  -p str_derive \
  -p transport \
  --target aarch64-unknown-linux-musl && \
  rm ./bin/elisheba/src/*.rs \
  ./lib/crypto/src/*.rs \
  ./lib/sonoff/src/*.rs \
  ./lib/str_derive/src/*.rs \
  ./lib/transport/src/*.rs

COPY ./lib/crypto/src ./lib/crypto/src
COPY ./lib/sonoff/src ./lib/sonoff/src
COPY ./lib/str_derive/src ./lib/str_derive/src
COPY ./lib/transport/src ./lib/transport/src
COPY ./bin/elisheba/src ./bin/elisheba/src

ARG VERSION=0.1.0
RUN sed -i "s/version = \"0.1.0\"/version = \"${VERSION}\"/g" bin/elisheba/Cargo.toml

RUN cargo build --release -p elisheba --target aarch64-unknown-linux-musl && \
  mv target/aarch64-unknown-linux-musl/release/elisheba ./ && \
  rm -rf target/aarch64-unknown-linux-musl/release/ target/release/

FROM alpine:3.18.4
WORKDIR /root/

COPY --from=builder /home/rust/src/elisheba .
