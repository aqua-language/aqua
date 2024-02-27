FROM ubuntu:20.04

# Set the noninteractive timezone (prevents configuration prompts)
ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Europe/London

# Set a working directory
WORKDIR /aqua

# Install apt packages
RUN apt-get update && apt-get install -y \
  curl wget software-properties-common

# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Cleanup
RUN apt-get clean

# Copy the Rust code into the image
COPY Cargo.toml /aqua/
COPY Cargo.lock /aqua/
COPY .cargo /aqua/.cargo
COPY crates /aqua/crates
