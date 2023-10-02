FROM rust:alpine

RUN apk add --no-cache g++ musl-dev git openssl-dev clang-dev libsecp256k1-dev

RUN rustup component add rustfmt

ENV RUSTFLAGS "-C target-feature=-crt-static"

RUN cargo install cargo-nextest --locked

COPY . .

RUN cargo nextest list

RUN apk --no-cache add curl jq

COPY ./devenv/integration/docker/entrypoint /bin/entrypoint

ENTRYPOINT [ "entrypoint" ]
