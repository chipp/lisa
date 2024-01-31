FROM ghcr.io/chipp/build.rust.arm64_musl:1.75.0_1 AS builder

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

COPY ./bin/elisa/Cargo.toml ./bin/elisa/Cargo.toml
COPY ./lib/str_derive/Cargo.toml ./lib/str_derive/Cargo.toml
COPY ./lib/str_derive/fake_macro.rs ./lib/str_derive/src/lib.rs
COPY ./lib/transport/Cargo.toml ./lib/transport/Cargo.toml
COPY ./lib/xiaomi/Cargo.toml ./lib/xiaomi/Cargo.toml

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release \
  -p elisa \
  -p str_derive \
  -p transport \
  -p xiaomi && \
  cargo clean --release \
  -p elisa \
  -p str_derive \
  -p transport \
  -p xiaomi \
  --target aarch64-unknown-linux-musl && \
  rm ./bin/elisa/src/*.rs \
  ./lib/str_derive/src/*.rs \
  ./lib/transport/src/*.rs \
  ./lib/xiaomi/src/*.rs

COPY ./bin/elisa/src ./bin/elisa/src
COPY ./lib/str_derive/src ./lib/str_derive/src
COPY ./lib/transport/src ./lib/transport/src
COPY ./lib/xiaomi/src ./lib/xiaomi/src

ARG VERSION=0.1.0
RUN sed -i "s/version = \"0.1.0\"/version = \"${VERSION}\"/g" bin/elisa/Cargo.toml

RUN cargo build --release -p elisa --target aarch64-unknown-linux-musl && \
  mv target/aarch64-unknown-linux-musl/release/elisa ./ && \
  rm -rf target/aarch64-unknown-linux-musl/release/ target/release/

FROM alpine:3.18.4
WORKDIR /root/

COPY --from=builder /home/rust/src/elisa .
