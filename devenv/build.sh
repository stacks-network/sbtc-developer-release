#!/usr/bin/env bash

DIR=$(dirname "$0")

docker compose -f $DIR/docker-compose.yml build
