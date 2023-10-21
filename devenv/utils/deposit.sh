#!/bin/bash

# Gets the default devnet credentials and makes a deposit

dir="$(dirname "$0")"

# the sbtc wallet (p2tr)
sbtc_wallet_address=$(source $dir/get_credentials.sh | jq -r '.credentials["0"].bitcoin.p2tr.address')
# Alice's btc address
btc_wif=$(source $dir/get_credentials.sh | jq -r '.credentials["1"].bitcoin.p2wpkh.wif')
# Alice's stx address
stacks_address=$(source $dir/get_credentials.sh | jq -r '.credentials["1"].stacks.address')

amount=$((RANDOM%9000+1000))

json=$($dir/../sbtc/bin/sbtc deposit \
    -w $btc_wif \
    -n regtest \
    -r $stacks_address \
    -a $amount \
    -s $sbtc_wallet_address \
    -u electrs:60401)

echo $json

if [ $? -ne 0 ]; then
    echo 'The deposit failed, did you forget to run "mine_btc.sh"?'
    exit 1
fi

tx=$(echo -n $json | jq -r .hex)

echo $tx

$dir/../sbtc/bin/sbtc broadcast electrs:60401 $tx | jq -r .
