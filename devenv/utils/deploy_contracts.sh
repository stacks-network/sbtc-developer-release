#!/bin/bash

# Deploys the asset contract to the devnet

SCRIPT_DIR=$(dirname "$0")
API_URL=http://localhost:20443/v2/info

echo "Waiting on Stacks API $API_URL"
while ! curl -s $API_URL >/dev/null; do
    sleep 1
done

# stacks ready to take contracts

STACKS_HEIGHT=1
echo "Waiting on Stacks height $STACKS_HEIGHT"
while [ "$(curl -s $API_URL | jq '.stacks_tip_height')" -lt $STACKS_HEIGHT ]; do
    sleep 2
done

# deploy the contracts

cd $SCRIPT_DIR/../../romeo/asset-contract && \
    clarinet deployments apply -p deployments/default.devnet-plan.yaml && \
    cd -
