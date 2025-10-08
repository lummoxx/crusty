FROM ubuntu:22.04

# Install only the core utilities and dev tools needed for Rust embedded and flashing Pico W
RUN apt-get update && \
    apt-get install -y \
        coreutils \
        procps \
        curl \
        sudo \
        git \
        ca-certificates \
        build-essential \
        pkg-config \
        libssl-dev \
        libudev-dev \
    && rm -rf /var/lib/apt/lists/*