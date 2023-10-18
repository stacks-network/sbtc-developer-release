#!/usr/bin/env bash

<<<<<<< Updated upstream
if [ $? -eq 0 ]; then
    docker compose up -d
=======
DIR=$(dirname "$0")

if [ $? -eq 0 ]
then
    docker compose -f $DIR/docker-compose.yml up -d
>>>>>>> Stashed changes
else
    echo "Build failed, not starting devenv"
fi
