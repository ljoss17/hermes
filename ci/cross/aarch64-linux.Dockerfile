FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:latest

RUN apt-get update && \
    apt-get install --yes \
      libssl-dev \
      pkg-config