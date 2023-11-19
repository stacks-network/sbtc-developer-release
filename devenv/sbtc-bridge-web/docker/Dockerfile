# Use an official Node.js runtime as a parent image
FROM node:19 as build

ARG SBTC_VERSION='Alpha'
ARG SBTC_GIT_URI='https://github.com/stacks-network/sbtc-bridge-web.git'
ARG SBTC_GIT_BRANCH='main'

WORKDIR /repo

RUN git clone ${SBTC_GIT_URI} -b ${SBTC_GIT_BRANCH} .

# Install dependencies
RUN npm install

# Build the application using Vite
RUN npm run build

CMD npm run devenv -- --host
