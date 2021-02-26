FROM ghcr.io/chipp/build.rust.x86_64.musl:latest AS builder

WORKDIR /home/rust/src
RUN USER=rust cargo new --bin /home/rust/src/lisa
RUN USER=rust cargo new --lib /home/rust/src/alice

COPY ./lisa/Cargo.toml ./lisa/Cargo.toml
COPY ./alice/Cargo.toml ./alice/Cargo.toml

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release && \
  cargo clean --release -p lisa -p alice --target x86_64-unknown-linux-musl

RUN cargo build && \
  cargo clean -p alice -p lisa --target x86_64-unknown-linux-musl && \
  rm ./lisa/src/*.rs ./alice/src/*.rs

COPY ./lisa/src ./lisa/src
COPY ./alice/src ./alice/src

RUN cargo test && rm -rf target/debug/
RUN cargo build --release

FROM alpine:3.11
RUN apk --no-cache add ca-certificates && update-ca-certificates

WORKDIR /root/
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs
ENV RUST_BACKTRACE=full

COPY --from=0 /home/rust/src/target/x86_64-unknown-linux-musl/release/lisa .
