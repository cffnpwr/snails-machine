FROM ubuntu:24.04 as mold

ARG MOLD_VERSION=2.32.0

WORKDIR /mold

RUN apt-get update -qq && apt-get install -qq --no-install-recommends curl ca-certificates
RUN curl -L https://github.com/rui314/mold/releases/download/v${MOLD_VERSION}/mold-${MOLD_VERSION}-$(uname -m)-linux.tar.gz --output mold.tar.gz
RUN tar -xzf mold.tar.gz --strip-components 1


FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm as runner

RUN apt-get update -qq; \
    apt-get install -qq --no-install-recommends \
    clang \
    iputils-ping \
    crossbuild-essential-arm64 \
    crossbuild-essential-amd64; \
    apt-get clean; \
    rm -rf /var/lib/apt/lists/*; \
    rustup target add aarch64-unknown-linux-gnu; \
    rustup target add x86_64-unknown-linux-gnu;

COPY ./cargo-config.toml /.cargo/config.toml
COPY --from=mold /mold/ /usr/local/