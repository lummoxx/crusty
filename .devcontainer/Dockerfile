FROM ubuntu:22.04

# Install core utilities and common dev tools for Codespaces compatibility
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
		libgtk-3-dev \
		libpango1.0-dev \
		libcairo2-dev \
		libgdk-pixbuf2.0-dev \
		libssl-dev \
		libudev-dev \
	&& rm -rf /var/lib/apt/lists/*
