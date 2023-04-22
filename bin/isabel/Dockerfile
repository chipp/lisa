FROM ghcr.io/chipp/build.musl.armv7_musl:musl_0.9.9_zlib_1.2.13_ssl_1.1.1s_curl_7.86.0_4 as libs_builder

ENV HOST=armv7-unknown-linux-musleabihf
ENV PREFIX=/bluez
ENV PKG_CONFIG_PATH=$PREFIX/lib/pkgconfig

COPY ./bin/isabel/install_static_libs.sh ./install_static_libs.sh
RUN ./install_static_libs.sh && rm ./install_static_libs.sh

FROM ghcr.io/chipp/build.rust.armv7_musl:1.69.0_2 AS builder

ENV HOST=armv7-unknown-linux-musleabihf

WORKDIR /home/rust/src
RUN USER=rust \
  cargo new --lib /home/rust/src/lib/alice && \
  cargo new --lib /home/rust/src/lib/alisa && \
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

FROM alpine:3.17.1
WORKDIR /root/

COPY --from=1 /home/rust/src/isabel .
