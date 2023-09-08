#!/bin/bash

./build.sh
docker compose -f docker-compose-miner.yml up -d
