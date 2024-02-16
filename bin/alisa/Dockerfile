FROM ghcr.io/chipp/build.rust.arm64_musl:1.76.0_1 AS builder

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

COPY ./bin/alisa/Cargo.toml ./bin/alisa/Cargo.toml
COPY ./lib/alice/Cargo.toml ./lib/alice/Cargo.toml
COPY ./lib/str_derive/Cargo.toml ./lib/str_derive/Cargo.toml
COPY ./lib/str_derive/fake_macro.rs ./lib/str_derive/src/lib.rs
COPY ./lib/transport/Cargo.toml ./lib/transport/Cargo.toml

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release \
  -p alisa \
  -p alice \
  -p str_derive \
  -p transport && \
  cargo clean --release \
  -p alisa \
  -p alice \
  -p str_derive \
  -p transport \
  --target x86_64-unknown-linux-musl && \
  rm \
  ./bin/alisa/src/*.rs \
  ./lib/alice/src/*.rs \
  ./lib/str_derive/src/*.rs \
  ./lib/transport/src/*.rs

COPY ./bin/alisa/src ./bin/alisa/src
COPY ./lib/alice/src ./lib/alice/src
COPY ./lib/str_derive/src ./lib/str_derive/src
COPY ./lib/transport/src ./lib/transport/src

ARG VERSION=0.1.0
RUN sed -i "s/version = \"0.1.0\"/version = \"${VERSION}\"/g" bin/alisa/Cargo.toml

RUN cargo build --release -p alisa && \
  mv target/x86_64-unknown-linux-musl/release/alisa ./ && \
  rm -rf target/x86_64-unknown-linux-musl/release/ target/release/

FROM alpine:3.18.4
RUN apk --no-cache add ca-certificates && update-ca-certificates
RUN apk --no-cache add tzdata && \
  cp /usr/share/zoneinfo/Europe/Vilnius /etc/localtime && \
  echo "Europe/Vilnius" > /etc/timezone && \
  apk del tzdata

WORKDIR /root/
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs
ENV RUST_BACKTRACE=full

COPY --from=0 /home/rust/src/alisa .
