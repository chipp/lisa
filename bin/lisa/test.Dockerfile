FROM ghcr.io/chipp/build.rust.x86_64_musl:1.66.0_2 AS builder

WORKDIR /home/rust/src
RUN USER=rust \
  cargo new --lib /home/rust/src/lib/alice && \
  cargo new --lib /home/rust/src/lib/alisa && \
  cargo new --lib /home/rust/src/lib/elisheba && \
  cargo new --lib /home/rust/src/lib/alzhbeta && \
  cargo new --bin /home/rust/src/bin/lisa && \
  cargo new --bin /home/rust/src/bin/isabel

COPY ./lib/alice/Cargo.toml ./lib/alice/Cargo.toml
COPY ./lib/alisa/Cargo.toml ./lib/alisa/Cargo.toml
COPY ./lib/elisheba/Cargo.toml ./lib/elisheba/Cargo.toml
COPY ./bin/lisa/Cargo.toml ./bin/lisa/Cargo.toml

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build -p lisa -p alice -p alisa -p elisheba && \
  cargo clean -p alice -p alisa -p lisa -p elisheba \
  --target x86_64-unknown-linux-musl && \
  rm ./bin/lisa/src/*.rs ./lib/alice/src/*.rs ./lib/alisa/src/*.rs ./lib/elisheba/src/*.rs

COPY ./lib/alice/src ./lib/alice/src
COPY ./lib/alisa/src ./lib/alisa/src
COPY ./lib/elisheba/src ./lib/elisheba/src
COPY ./bin/lisa/src ./bin/lisa/src

RUN cargo test -p lisa -p alice -p alisa -p elisheba && \
  rm -rf target/x86_64-unknown-linux-musl/debug/ target/debug/
