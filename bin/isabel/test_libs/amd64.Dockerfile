ARG RUST_VERSION

FROM ghcr.io/chipp/build.rust.x86_64_musl:${RUST_VERSION}

COPY ./bin/isabel/install_static_libs.sh ./install_static_libs.sh
RUN chmod +x ./install_static_libs.sh && \
  ./install_static_libs.sh && \
  rm ./install_static_libs.sh
