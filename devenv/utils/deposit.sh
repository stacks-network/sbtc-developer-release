#!/bin/bash

# Gets the default devnet credentials and makes a deposit

dir="$(dirname "$0")"

# the sbtc wallet
btc_p2tr_address=$(source $dir/get_credentials.sh | jq -r '.credentials["0"].bitcoin.p2tr.address')
# Alice's btc address
btc_wif=$(source $dir/get_credentials.sh | jq -r '.credentials["1"].bitcoin.p2wpkh.wif')
# Alice's stx address
stacks_address=$(source $dir/get_credentials.sh | jq -r '.credentials["1"].stacks.address')

amount=$((RANDOM%9000+1000))

json=$(sbtc deposit \
    -w $btc_wif \
    -n regtest \
    -r $stacks_address \
    -a $amount \
    -d $btc_p2tr_address \
    -u localhost:60401)

if [ $? -ne 0 ]; then
    echo 'The deposit failed, did you forget to run "mine_btc.sh"?'
    exit 1
fi

tx=$(echo -n $json | jq -r .hex)

sbtc broadcast localhost:60401 $tx | jq -r .
