FROM rust:alpine

ARG SBTC_VERSION='Alpha Romeo'
ARG SBTC_GIT_URI='https://github.com/stacks-network/sbtc.git'
ARG SBTC_GIT_BRANCH='master'

WORKDIR /src

RUN apk add --no-cache g++ musl-dev git openssl-dev clang-dev libsecp256k1-dev

RUN git clone ${SBTC_GIT_URI} -b ${SBTC_GIT_BRANCH} .

RUN cargo install --version 0.36.13 cargo-make

RUN rustup component add rustfmt

RUN RUSTFLAGS="-C target-feature=-crt-static" cargo install --path sbtc-cli

RUN RUSTFLAGS="-C target-feature=-crt-static" cargo install --path romeo


