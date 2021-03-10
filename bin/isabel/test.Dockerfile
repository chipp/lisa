FROM ghcr.io/chipp/build.rust.x86_64_musl:1.50.0_5 AS builder

ENV HOST=x86_64-unknown-linux-musl

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

RUN cargo build -p isabel -p elisheba -p alzhbeta && \
  cargo clean -p isabel -p elisheba -p alzhbeta \
  --target x86_64-unknown-linux-musl && \
  rm ./lib/elisheba/src/*.rs ./lib/alzhbeta/src/*.rs ./bin/isabel/src/*.rs

COPY ./lib/elisheba/src ./lib/elisheba/src
COPY ./lib/alzhbeta/src ./lib/alzhbeta/src
COPY ./bin/isabel/src ./bin/isabel/src

RUN cargo test -p isabel -p elisheba -p alzhbeta && \
  rm -rf target/x86_64-unknown-linux-musl/debug/ target/debug/
