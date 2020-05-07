FROM docker.pkg.github.com/chipp/base-image/build.rust.x86_64.musl:latest AS builder

WORKDIR /home/rust/src
RUN USER=rust cargo init --lib /home/rust/src

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release && \
  cargo clean --release -p lisa

RUN cargo build && \
  cargo clean -p lisa && \
  rm src/*.rs

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
