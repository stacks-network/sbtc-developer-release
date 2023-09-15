FROM rust:alpine as build

WORKDIR /romeo

RUN apk add --no-cache g++ musl-dev git openssl-dev clang-dev libsecp256k1-dev

RUN cargo install --version 0.36.13 cargo-make

RUN rustup component add rustfmt

COPY . .

RUN RUSTFLAGS="-C target-feature=-crt-static" cargo install --path sbtc-cli

RUN RUSTFLAGS="-C target-feature=-crt-static" cargo install --path romeo

FROM alpine:latest

RUN apk add --no-cache openssl libsecp256k1 g++

COPY --from=build /usr/local/cargo/bin/sbtc /usr/local/bin
COPY --from=build /usr/local/cargo/bin/romeo /usr/local/bin

ENTRYPOINT ["romeo"]
CMD ["-c", "/romeo/config.json"]


