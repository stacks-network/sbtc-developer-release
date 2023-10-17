FROM rust:bookworm as clarinet

RUN apt update && apt install -y pkg-config libssl-dev clang && \
    rm -rf /var/lib/apt/lists/*

RUN cargo install clarinet-cli --bin clarinet --branch develop --locked --git https://github.com/hirosystems/clarinet.git

FROM rust:bookworm as romeo

RUN apt update && apt install -y g++ libssl-dev clang libsecp256k1-dev && \
    rm -rf /var/lib/apt/lists/*

RUN cargo install --locked cargo-make

RUN rustup component add rustfmt

COPY . .
ENV RUSTFLAGS "-C target-feature=-crt-static"
RUN cargo install --locked --path sbtc-cli
RUN cargo install --locked --path romeo

FROM rust:slim-bookworm

RUN apt update && apt install -y openssl libsecp256k1-dev curl jq && \
    rm -rf /var/lib/apt/lists/*

COPY --from=clarinet /usr/local/cargo/bin/clarinet /usr/local/bin
COPY --from=romeo /usr/local/cargo/bin/sbtc /usr/local/bin
COPY --from=romeo /usr/local/cargo/bin/romeo /usr/local/bin
ADD devenv/sbtc/docker/entrypoint /usr/local/bin
RUN chmod a+x /usr/local/bin/entrypoint
ADD romeo/asset-contract /asset-contract

ENTRYPOINT ["entrypoint"]