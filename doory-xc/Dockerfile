FROM ubuntu:trusty

RUN apt-get update && apt-get install -y \
    build-essential \
    ccache \
    curl \
    gawk \
    gettext \
    git \
    libncurses5-dev \
    libssl-dev \
    python \
    subversion \
    unzip \
    wget \
    xsltproc \
    zlib1g-dev

RUN set -eux; \
    \
    rustupSha256="f5833a64fd549971be80fa42cffc6c5e7f51c4f443cd46e90e4c17919c24481f"; \
    url="https://static.rust-lang.org/rustup/archive/1.6.0/x86_64-unknown-linux-gnu/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --default-toolchain 1.20.0; \
    rm rustup-init

RUN mkdir openwrt-sdk
RUN curl https://downloads.openwrt.org/snapshots/trunk/ramips/rt305x/OpenWrt-SDK-ramips-rt305x_gcc-5.3.0_musl-1.1.16.Linux-x86_64.tar.bz2 | tar --strip-components 1 -C openwrt-sdk -xj

RUN cd /openwrt-sdk && \
    ./scripts/feeds update base && \
    ./scripts/feeds install -a && \
    make package/toolchain/compile

COPY libnfc /openwrt-sdk/package/libnfc
RUN cd /openwrt-sdk && make package/libnfc/compile

ENV PATH="/root/.cargo/bin:/openwrt-sdk/staging_dir/toolchain-mipsel_24kec+dsp_gcc-5.3.0_musl-1.1.16/bin:${PATH}"

RUN rustup default nightly
RUN rustup target add mipsel-unknown-linux-musl
RUN cargo install xargo

COPY cargo.config /root/.cargo/config 

ENV USER=root
