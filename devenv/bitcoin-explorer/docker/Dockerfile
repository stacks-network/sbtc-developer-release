FROM node:16 as builder
ARG BTCEXP_NODE_VERSION='v3.4.0'
ARG GIT_URI='https://github.com/janoside/btc-rpc-explorer.git'
ARG GIT_BRANCH='v3.4.0'
WORKDIR /workspace
RUN git clone https://github.com/janoside/btc-rpc-explorer.git .
RUN npm install

FROM node:16-alpine
WORKDIR /workspace
RUN apk --update add git curl
COPY --from=builder /workspace .
COPY entrypoint.sh /workspace/entrypoint.sh
RUN chmod a+x /workspace/entrypoint.sh
ENTRYPOINT ["/workspace/entrypoint.sh"]
