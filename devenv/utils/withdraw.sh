#!/bin/bash

# Gets the default devnet credentials and makes a withdrawal


dir="$(dirname "$0")"

# the sbtc wallet (p2tr)
sbtc_wallet_address=$(source $dir/get_credentials.sh | jq -r '.credentials["0"].bitcoin.p2tr.address')

# Alice's btc credential as wif
btc_wif=$(source $dir/get_credentials.sh | jq -r '.credentials["1"].bitcoin.p2wpkh.wif')
# Alice's btc address
btc_address=$(source $dir/get_credentials.sh | jq -r '.credentials["1"].bitcoin.p2wpkh.address')
# Alice's stx credential as wif
stacks_wif=$(source $dir/get_credentials.sh | jq -r '.credentials["1"].stacks.wif')

amount=$((RANDOM%1000+1000))
fulfillment_fee=$((RANDOM%1000+1000))

json=$(sbtc withdraw \
    -w $btc_wif \
    -n regtest \
    -d $stacks_wif \
    -b $btc_address \
    -a $amount \
    -f $fulfillment_fee \
    -s $sbtc_wallet_address \
    -u localhost:60401)


if [ $? -ne 0 ]; then
    echo 'The withdrawal failed, did you forget to run "mine_btc.sh"?'
    exit 1
fi

tx=$(echo -n $json | jq -r .hex)

sbtc broadcast localhost:60401 $tx | jq -r .
