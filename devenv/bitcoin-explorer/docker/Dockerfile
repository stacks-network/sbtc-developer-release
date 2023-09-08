FROM node:16 as builder
ARG BTCEXP_NODE_VERSION='v3.4.0'
ARG GIT_URI='https://github.com/janoside/btc-rpc-explorer.git'
ARG GIT_BRANCH='v3.4.0'
WORKDIR /workspace
RUN git clone https://github.com/janoside/btc-rpc-explorer.git .
RUN npm install

FROM node:16-alpine
WORKDIR /workspace
COPY --from=builder /workspace .
RUN apk --update add git
CMD npm start
EXPOSE 3002
