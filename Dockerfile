# Build verifier
FROM ubuntu:20.04

RUN apt-get update && ln -fs /usr/share/zoneinfo/America/New_York /etc/localtime

# Install basic packages
RUN apt-get install build-essential curl git pkg-config -y

# Install dev-packages
RUN apt-get install libclang-dev libssl-dev llvm -y

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Add Toolchain
RUN rustup toolchain install nightly-2021-11-17

RUN mkdir -p /root/src
ADD . /root/src
RUN cd /root/src && cargo build --release
