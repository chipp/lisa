FROM ghcr.io/chipp/build.rust.armv7_musl:1.66.0_2 AS builder

ENV HOST=armv7-unknown-linux-musleabihf

COPY ./bin/isabel/install_static_libs.sh ./install_static_libs.sh
RUN ./install_static_libs.sh && rm ./install_static_libs.sh

WORKDIR /home/rust/src
RUN USER=rust \
  cargo new --lib /home/rust/src/lib/alice && \
  cargo new --lib /home/rust/src/lib/elisheba && \
  cargo new --lib /home/rust/src/lib/alzhbeta && \
  cargo new --bin /home/rust/src/bin/lisa && \
  cargo new --bin /home/rust/src/bin/isabel

COPY ./lib/elisheba/Cargo.toml ./lib/elisheba/Cargo.toml
COPY ./lib/alzhbeta/Cargo.toml ./lib/alzhbeta/Cargo.toml
COPY ./bin/isabel/Cargo.toml ./bin/isabel/Cargo.toml

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release -p isabel && \
  cargo clean --release -p elisheba -p isabel -p alzhbeta \
  --target armv7-unknown-linux-musleabihf && \
  rm ./lib/elisheba/src/*.rs ./lib/alzhbeta/src/*.rs ./bin/isabel/src/*.rs

COPY ./lib/elisheba/src ./lib/elisheba/src
COPY ./lib/alzhbeta/src ./lib/alzhbeta/src
COPY ./bin/isabel/src ./bin/isabel/src

RUN cargo build --release -p isabel && \
  mv target/armv7-unknown-linux-musleabihf/release/isabel ./ && \
  rm -rf target/armv7-unknown-linux-musleabihf/release/ target/release/

FROM alpine:3.11
RUN apk --no-cache add ca-certificates && update-ca-certificates

WORKDIR /root/
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs
ENV RUST_BACKTRACE=full

COPY --from=0 /home/rust/src/isabel .
