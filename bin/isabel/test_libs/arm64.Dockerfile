FROM ghcr.io/chipp/build.rust.arm64_musl:1.79.0_1 as libs_builder

COPY ./bin/isabel/install_static_libs.sh ./install_static_libs.sh
RUN chmod +x ./install_static_libs.sh && \
  ./install_static_libs.sh && \
  rm ./install_static_libs.sh
