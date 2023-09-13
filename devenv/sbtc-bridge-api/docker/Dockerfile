FROM node:19-alpine

ARG SBTC_VERSION='Alpha'
ARG SBTC_GIT_URI='https://github.com/stacks-network/sbtc-bridge-api.git'
ARG SBTC_GIT_BRANCH='main'

# Set the environment variables
ENV PORT 7010

# Create app directory
WORKDIR /app

RUN apk add --no-cache git

# Bundle app source
RUN git clone ${SBTC_GIT_URI} -b ${SBTC_GIT_BRANCH} .

WORKDIR /app/sbtc-bridge-api

RUN npm install 
RUN npm run swagger
RUN npm run build
RUN cp src/swagger.json dist/src/

EXPOSE $PORT

CMD npm run start
