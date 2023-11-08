#!/usr/bin/env bash

set -x

nginx
bitcoind -chain=${BTC_NETWORK} -txindex=${BTC_TXINDEX} -rpcuser=${BTC_RPCUSER} -rpcpassword=${BTC_RPCPASSWORD} -printtoconsole=${BTC_PRINTTOCONSOLE} -disablewallet=${BTC_DISABLEWALLET} -rpcbind=${BTC_RPCBIND} -rpcallowip=${BTC_RPCALLOWIP}
