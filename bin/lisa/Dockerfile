FROM ghcr.io/chipp/build.rust.x86_64_musl:1.51.0_3 AS builder

WORKDIR /home/rust/src
RUN USER=rust \
  cargo new --lib /home/rust/src/lib/alice && \
  cargo new --lib /home/rust/src/lib/elisheba && \
  cargo new --lib /home/rust/src/lib/alzhbeta && \
  cargo new --bin /home/rust/src/bin/lisa && \
  cargo new --bin /home/rust/src/bin/isabel

COPY ./lib/alice/Cargo.toml ./lib/alice/Cargo.toml
COPY ./lib/elisheba/Cargo.toml ./lib/elisheba/Cargo.toml
COPY ./bin/lisa/Cargo.toml ./bin/lisa/Cargo.toml

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release -p lisa && \
  cargo clean --release -p lisa -p alice -p elisheba \
  --target x86_64-unknown-linux-musl && \
  rm ./lib/elisheba/src/*.rs ./lib/alice/src/*.rs ./bin/lisa/src/*.rs

COPY ./lib/alice/src ./lib/alice/src
COPY ./lib/elisheba/src ./lib/elisheba/src
COPY ./bin/lisa/src ./bin/lisa/src

RUN cargo build --release -p lisa && \
  mv target/x86_64-unknown-linux-musl/release/lisa ./ && \
  rm -rf target/x86_64-unknown-linux-musl/release/ target/release/

FROM alpine:3.11
RUN apk --no-cache add ca-certificates && update-ca-certificates

WORKDIR /root/
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs
ENV RUST_BACKTRACE=full

COPY --from=0 /home/rust/src/lisa .
