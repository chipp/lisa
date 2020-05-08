FROM docker.pkg.github.com/chipp/base-image/build.rust.x86_64.musl:latest AS builder

WORKDIR /home/rust/src
RUN USER=rust cargo init --lib /home/rust/src
RUN USER=rust cargo new --bin /home/rust/src/lisa
RUN USER=rust cargo new --lib /home/rust/src/alice

COPY ./alice/Cargo.toml ./alice/Cargo.toml
COPY ./lisa/Cargo.toml ./lisa/Cargo.toml

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release && \
  cargo clean --release -p lisa && \
  cargo clean --release -p alice

RUN cargo build && \
  cargo clean -p lisa && \
  cargo clean -p alice && \
  rm src/lisa/*.rs && \
  rm src/alice/*.rs

COPY ./src ./src
RUN cargo test && rm -rf target/debug/
RUN cargo build --release

FROM alpine:3.11
RUN apk --no-cache add ca-certificates && update-ca-certificates

WORKDIR /root/
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt \
  SSL_CERT_DIR=/etc/ssl/certs
ENV RUST_BACKTRACE=full

COPY --from=0 /home/rust/src/target/x86_64-unknown-linux-musl/release/lisa .
