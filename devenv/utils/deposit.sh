#!/bin/bash

dir="$(dirname "$0")"

btc_wif=$(source $dir/get_credentials.sh | jq -r '.credentials["0"].bitcoin.p2wpkh.wif')
btc_p2tr_address=$(source $dir/get_credentials.sh | jq -r '.credentials["0"].bitcoin.p2tr.address')
stacks_address=$(source $dir/get_credentials.sh | jq -r '.credentials["0"].stacks.address')

amount=$((RANDOM%9000+1000))

json=$(sbtc deposit \
    -w $btc_wif \
    -n regtest \
    -r $stacks_address \
    -a $amount \
    -d $btc_p2tr_address \
    -u localhost:60401)

tx=$(echo -n $json | jq -r .hex)

sbtc broadcast localhost:60401 $tx | jq -r .
