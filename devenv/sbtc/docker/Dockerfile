FROM rust:alpine

WORKDIR /src

RUN apk add --no-cache g++ musl-dev git openssl-dev clang-dev libsecp256k1-dev

RUN cargo install --version 0.36.13 cargo-make

RUN rustup component add rustfmt

COPY ./../../../ .

RUN RUSTFLAGS="-C target-feature=-crt-static" cargo install --path sbtc-cli

RUN RUSTFLAGS="-C target-feature=-crt-static" cargo install --path romeo

WORKDIR /src/romeo/src

ENTRYPOINT ["/usr/local/cargo/bin/romeo", "-c", "config.json"]


