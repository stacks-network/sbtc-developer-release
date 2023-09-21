#!/usr/bin/env bash

./build.sh
if [ $? -eq 0 ] 
then
  docker compose up -d
else
  echo "Build failed, not starting devenv"
fi
