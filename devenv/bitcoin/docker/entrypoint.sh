#!/bin/bash

set -x

BTC_DIR=/root/.bitcoin
BITCOIN_CONF=${BITCOIN_DIR}/bitcoin.conf

#-rpcuser=${BTC_RPCUSER} -rpcpassword=${BTC_RPCPASSWORD}

# bitcoind needs creds set in the conf file for remote RPC auth
#echo '[regtest]' > ${BITCOIN_CONF}
echo 'rpcuser=devnet' >> ${BITCOIN_CONF}
echo 'rpcpassword=devnet' >> ${BITCOIN_CONF}

bitcoind -chain=${BTC_NETWORK} -conf=${BITCOIN_CONF} -datadir=${BTC_DIR} -txindex=${BTC_TXINDEX} -rpcuser=${BTC_RPCUSER} -rpcpassword=${BTC_RPCPASSWORD} -printtoconsole=${BTC_PRINTTOCONSOLE} -disablewallet=${BTC_DISABLEWALLET} -rpcbind=${BTC_RPCBIND} -rpcallowip=${BTC_RPCALLOWIP}  
