FROM rust:alpine as clarinet

RUN apk add --no-cache g++ musl-dev git openssl-dev clang-dev

RUN cargo install clarinet-cli --bin clarinet --branch develop --locked --git https://github.com/hirosystems/clarinet.git

FROM rust:alpine as romeo

RUN apk add --no-cache g++ musl-dev git openssl-dev clang-dev libsecp256k1-dev

RUN cargo install --locked cargo-make

RUN rustup component add rustfmt

COPY . .
ENV RUSTFLAGS "-C target-feature=-crt-static"
RUN cargo install --locked --path sbtc-cli
RUN cargo install --locked --path romeo

FROM alpine:latest

RUN apk add --no-cache g++ openssl libsecp256k1 curl jq

COPY --from=clarinet /usr/local/cargo/bin/clarinet /usr/local/bin
COPY --from=romeo /usr/local/cargo/bin/sbtc /usr/local/bin
COPY --from=romeo /usr/local/cargo/bin/romeo /usr/local/bin
ADD devenv/sbtc/docker/entrypoint /usr/local/bin
RUN chmod a+x /usr/local/bin/entrypoint
ADD romeo/asset-contract /asset-contract

ENTRYPOINT ["entrypoint"]
