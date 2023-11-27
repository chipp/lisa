FROM ghcr.io/chipp/build.rust.x86_64_musl:1.74.0_3 AS builder

WORKDIR /home/rust/src
RUN USER=rust \
  cargo new --lib /home/rust/src/lib/alice && \
  cargo new --lib /home/rust/src/lib/inspinia && \
  cargo new --lib /home/rust/src/lib/str_derive && \
  cargo new --lib /home/rust/src/lib/transport && \
  cargo new --lib /home/rust/src/lib/xiaomi && \
  cargo new --bin /home/rust/src/bin/alisa && \
  cargo new --bin /home/rust/src/bin/elisa && \
  cargo new --bin /home/rust/src/bin/elizabeth

COPY ./bin/elisa/Cargo.toml ./bin/elisa/Cargo.toml
COPY ./lib/str_derive/Cargo.toml ./lib/str_derive/Cargo.toml
COPY ./lib/str_derive/fake_macro.rs ./lib/str_derive/src/lib.rs
COPY ./lib/transport/Cargo.toml ./lib/transport/Cargo.toml
COPY ./lib/xiaomi/Cargo.toml ./lib/xiaomi/Cargo.toml

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build -p elisa -p str_derive -p transport -p xiaomi && \
  cargo clean -p elisa -p str_derive -p transport -p xiaomi \
  --target x86_64-unknown-linux-musl && \
  rm ./bin/elisa/src/*.rs ./lib/str_derive/src/*.rs \
  ./lib/transport/src/*.rs ./lib/xiaomi/src/*.rs

COPY ./bin/elisa/src ./bin/elisa/src
COPY ./lib/str_derive/src ./lib/str_derive/src
COPY ./lib/transport/src ./lib/transport/src
COPY ./lib/xiaomi/src ./lib/xiaomi/src

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
