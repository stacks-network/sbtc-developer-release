FROM debian:stable-slim
MAINTAINER Tyler Baker <tyler@trustmachines.co>

ARG VERSION=25.0

RUN apt-get update && apt-get install -y \
    wget \
    ca-certificates \
    coreutils \
    file \
  && rm -rf /var/lib/apt/lists/*

RUN file /bin/bash | grep -q x86-64 && echo x86_64-linux-gnu > /tmp/arch || true
RUN file /bin/bash | grep -q aarch64 && echo aarch64-linux-gnu > /tmp/arch || true
RUN file /bin/bash | grep -q EABI5 && echo arm-linux-gnueabihf > /tmp/arch || true

WORKDIR /src

RUN wget https://bitcoincore.org/bin/bitcoin-core-${VERSION}/bitcoin-${VERSION}-$(cat /tmp/arch).tar.gz 

RUN wget https://bitcoincore.org/bin/bitcoin-core-${VERSION}/SHA256SUMS

RUN cat SHA256SUMS | grep bitcoin-${VERSION}-$(cat /tmp/arch).tar.gz | sha256sum -c \
  && tar xzvf bitcoin-${VERSION}-$(cat /tmp/arch).tar.gz \
  && mkdir /root/.bitcoin \
  && mv bitcoin-${VERSION}/bin/* /usr/local/bin/ \
  && rm -rf bitcoin-${VERSION}/ \
  && rm -rf bitcoin-${VERSION}-$(cat /tmp/arch).tar.gz \
  && rm SHA256SUMS

ADD entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod a+x /usr/local/bin/entrypoint.sh

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
