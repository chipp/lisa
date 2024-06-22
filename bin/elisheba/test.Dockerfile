FROM ghcr.io/chipp/build.rust.x86_64_musl:1.79.0_1 AS builder

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

COPY ./bin/elisheba/Cargo.toml ./bin/elisheba/Cargo.toml
COPY ./lib/crypto/Cargo.toml ./lib/crypto/Cargo.toml
COPY ./lib/sonoff/Cargo.toml ./lib/sonoff/Cargo.toml
COPY ./lib/str_derive/Cargo.toml ./lib/str_derive/Cargo.toml
COPY ./lib/str_derive/fake_macro.rs ./lib/str_derive/src/lib.rs
COPY ./lib/transport/Cargo.toml ./lib/transport/Cargo.toml

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build \
  -p elisheba \
  -p crypto \
  -p sonoff \
  -p str_derive \
  -p transport && \
  cargo clean \
  -p elisheba \
  -p crypto \
  -p sonoff \
  -p str_derive \
  -p transport \
  --target x86_64-unknown-linux-musl && \
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

RUN cargo test -p elisheba -p crypto -p sonoff -p str_derive -p transport && \
  rm -rf target/x86_64-unknown-linux-musl/debug/ target/debug/
