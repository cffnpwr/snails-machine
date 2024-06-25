FROM debian:bookworm as mold

ARG MOLD_VERSION=2.32.0

WORKDIR /mold

RUN apt-get update -qq; \
    apt-get install -qq --no-install-recommends curl ca-certificates
RUN curl -L https://github.com/rui314/mold/releases/download/v${MOLD_VERSION}/mold-${MOLD_VERSION}-$(uname -m)-linux.tar.gz --output mold.tar.gz
RUN tar -xzf mold.tar.gz --strip-components 1


FROM debian:bookworm as apple-cross-compiler

ARG MACOS_SDK_URL
ARG MACOS_SDK_FILE_PATH="dummy"

RUN apt-get update -qq; \
    apt-get install -qq --no-install-recommends \
    curl \
    ca-certificates \
    clang \
    cmake \
    make \
    git \
    patch \
    python-is-python3 \
    libssl-dev \
    lzma-dev \
    libxml2-dev \
    xz-utils \
    bzip2 \
    cpio \
    zlib1g-dev
RUN git clone https://github.com/tpoechtrager/osxcross.git /osxcross
RUN if [ -n "${MACOS_SDK_URL}" ]; then curl -fOL "${MACOS_SDK_URL}" --output-dir /osxcross/tarballs/ ; fi

COPY ./${MACOS_SDK_FILE_PATH}* /osxcross/tarballs/

WORKDIR /osxcross

RUN mkdir -p /osxcross/target/bin
RUN if [ -n "${MACOS_SDK_URL}" ] || ( [ -n "${MACOS_SDK_FILE_PATH}" ] && [ "${MACOS_SDK_FILE_PATH}" != "dummy" ] ); \
        then UNATTENDED=1 ./build.sh; \
    fi


FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm as runner

ARG MACOS_SDK_URL
ARG MACOS_SDK_FILE_PATH="dummy"
ARG ENABLE_WINDOWS_TARGET="false"

RUN apt-get update -qq; \
    apt-get install -qq --no-install-recommends \
    clang \
    iputils-ping \
    crossbuild-essential-arm64 \
    crossbuild-essential-amd64; \
    g++-mingw-w64; \
    apt-get clean; \
    rm -rf /var/lib/apt/lists/*; \
    rustup target add aarch64-unknown-linux-gnu; \
    rustup target add aarch64-unknown-linux-musl; \
    rustup target add x86_64-unknown-linux-gnu; \
    rustup target add x86_64-unknown-linux-musl; \
    if [ "${ENABLE_WINDOWS_TARGET}" ]; then \
        rustup target add aarch64-pc-windows-msvc; \
        rustup target add x86_64-pc-windows-gnu; \
        rustup target add x86_64-pc-windows-msvc; \
        cargo install xwin; \
        xwin --accept-license --cache-dir /.xwin-cache --arch=x86_64,aarch64 splat --output /.xwin; \
    fi; \
    if [ -n "${MACOS_SDK_URL}" ] || ( [ -n "${MACOS_SDK_FILE_PATH}" ] && [ "${MACOS_SDK_FILE_PATH}" != "dummy" ] ); then \
        rustup target add x86_64-apple-darwin; \
        rustup target add aarch64-apple-darwin; \
    fi; \
    chmod a+w -R /usr/local/cargo/registry

COPY ./cargo-config.toml /.cargo/config.toml
COPY --from=mold /mold/ /usr/local/
COPY --from=apple-cross-compiler /osxcross/target/ /usr/local/

RUN ldconfig
