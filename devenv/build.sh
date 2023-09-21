#!/usr/bin/env bash
CWD=$(dirname "$0")
docker compose -f $CWD/docker-compose.yml build
