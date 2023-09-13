# Important: This file is provided for demonstration purposes and may NOT be suitable for production use.
# The maintainers of electrs are not deeply familiar with Docker, so you should DYOR.
# If you are not familiar with Docker either it's probably be safer to NOT use it.

FROM debian:bookworm-slim as base
RUN apt-get update -qqy
RUN apt-get install -qqy librocksdb-dev curl git

### Electrum Rust Server ###
FROM base as electrs-build
RUN apt-get install -qqy cargo clang cmake

ARG ELECTRS_NODE_VERSION='v0.10.0'
ARG GIT_URI='https://github.com/romanz/electrs.git'
ARG GIT_BRANCH='v0.10.0'

# Install electrs
WORKDIR /build/electrs
RUN git clone ${GIT_URI} -b ${GIT_BRANCH} .
ENV ROCKSDB_INCLUDE_DIR=/usr/include
ENV ROCKSDB_LIB_DIR=/usr/lib
RUN cargo install --locked --path .

FROM base as result
# Copy the binaries
COPY --from=electrs-build /root/.cargo/bin/electrs /usr/bin/electrs

COPY config.toml /etc/electrs/config.toml

WORKDIR /
