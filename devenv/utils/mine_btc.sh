#!/bin/bash

# Mines some BTC to the default BTC address

num_blocks=$1
dir="$(dirname "$0")"

if [[ -z "$num_blocks" ]]; then
    echo "Usage: ./script.sh [num_blocks]"
    exit 1
fi

btc_address=$(source $dir/get_credentials.sh | jq -r '.credentials["0"].bitcoin.p2wpkh.address')

bitcoin-cli -rpcconnect=localhost -rpcport=18443 -rpcuser=devnet -rpcpassword=devnet generatetoaddress $num_blocks $btc_address
