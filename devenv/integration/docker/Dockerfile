FROM rust:bookworm as clarinet

RUN apt update && apt install -y pkg-config libssl-dev clang && \
    rm -rf /var/lib/apt/lists/*

RUN rustup update 1.71 && rustup default 1.71
RUN cargo install clarinet-cli --bin clarinet --branch develop --locked --git https://github.com/hirosystems/clarinet.git

FROM rust:bookworm as romeo

RUN apt update && \
    apt install -y libssl-dev libclang-dev libsecp256k1-dev && \
    rm -rf /var/lib/apt/lists/*

RUN rustup component add rustfmt

ENV RUSTFLAGS "-C target-feature=-crt-static"

RUN cargo install cargo-nextest --locked

COPY . .

RUN cargo nextest archive --archive-file integration-tests.tar.zst

FROM rust:bookworm as runtime

RUN apt update && \
    apt install -y openssl ca-certificates

COPY --from=clarinet /usr/local/cargo/bin/clarinet /usr/bin

RUN apt install -y curl jq && \
    rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-nextest --locked

COPY --from=romeo integration-tests.tar.zst .

COPY . .

ENTRYPOINT [ "entrypoint" ]
