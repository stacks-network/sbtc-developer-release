# Important: This file is provided for demonstration purposes and may NOT be suitable for production use.
# The maintainers of electrs are not deeply familiar with Docker, so you should DYOR.
# If you are not familiar with Docker either it's probably be safer to NOT use it.

FROM debian:bookworm-slim as base
RUN apt-get update -qqy
RUN apt-get install -qqy librocksdb-dev curl git

### Electrum Rust Server ###
FROM base as electrs-build
RUN apt-get install -qqy cargo clang cmake

ARG GIT_URI='https://github.com/mempool/electrs.git'
ARG GIT_BRANCH='mempool'

# Install electrs
WORKDIR /build/electrs
RUN git clone ${GIT_URI} -b ${GIT_BRANCH} .
RUN cargo install --locked --path .

FROM base as result

# Copy the binaries
COPY --from=electrs-build /root/.cargo/bin/electrs /usr/bin/electrs

COPY entrypoint.sh /entrypoint.sh

RUN chmod a+x /entrypoint.sh

WORKDIR /

ENTRYPOINT ["./entrypoint.sh"]
