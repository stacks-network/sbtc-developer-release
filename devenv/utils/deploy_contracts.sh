#!/bin/bash

SCRIPT_DIR=$(dirname "$0")

cd $SCRIPT_DIR/../../romeo/asset-contract && \
    clarinet deployments apply -p deployments/default.devnet-plan.yaml && \
    cd -
