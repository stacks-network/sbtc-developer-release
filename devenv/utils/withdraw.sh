#!/bin/bash

dir="$(dirname "$0")"

btc_wif=$(source $dir/get_credentials.sh | jq -r '.credentials["0"].bitcoin.p2wpkh.wif')
btc_address=$(source $dir/get_credentials.sh | jq -r '.credentials["0"].bitcoin.p2wpkh.address')
btc_p2tr_address=$(source $dir/get_credentials.sh | jq -r '.credentials["0"].bitcoin.p2tr.address')
stacks_wif=$(source $dir/get_credentials.sh | jq -r '.credentials["0"].stacks.wif')

amount=$((RANDOM%1000+1000))
fulfillment_fee=$((RANDOM%1000+1000))

json=$(sbtc withdraw \
    -w $btc_wif \
    -n regtest \
    -d $stacks_wif \
    -b $btc_address \
    -a $amount \
    -f $fulfillment_fee \
    -p $btc_p2tr_address \
    -u localhost:60401)

tx=$(echo -n $json | jq -r .hex)

sbtc broadcast localhost:60401 $tx | jq -r .
