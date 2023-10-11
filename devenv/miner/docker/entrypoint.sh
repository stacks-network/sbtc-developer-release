#!/bin/sh
# Wait until bitcoin RPC is ready
echo "checking if bitcoin node is online"
until curl -f -s -o /dev/null --user devnet:devnet --data-binary '{"jsonrpc": "1.0", "id": "curltest", "method": "getblockcount", "params": []}' -H 'content-type: text/plain;' http://bitcoin:18443/
do
	echo "bitcoin node is not ready, sleep two seconds"
	sleep 2
done
echo "bitcoin node is ready"
# Create a named "legacy" wallet named "devnet"
curl --user devnet:devnet --data-binary '{"jsonrpc": "1.0", "id": "curltest", "method": "createwallet", "params": {"wallet_name":"devnet","avoid_reuse":true,"descriptors":false,"load_on_startup":true}}' -H 'content-type: text/plain;' http://bitcoin:18443/
echo "devnet bitcoin wallet created"
# Import miner address
curl --user devnet:devnet --data-binary '{"jsonrpc": "1.0", "id": "curltest", "method": "importaddress", "params": ["n3GRiDLKWuKLCw1DZmV75W1mE35qmW2tQm"]}' -H 'content-type: text/plain;' http://bitcoin:18443/
# Mine the first N blocks
curl --user devnet:devnet --data-binary '{"jsonrpc": "1.0", "id": "curltest", "method": "generatetoaddress", "params": ['''${INIT_BTC_BLOCKS}''', "n3GRiDLKWuKLCw1DZmV75W1mE35qmW2tQm"]}' -H 'content-type: text/plain;' http://bitcoin:18443/
echo "mined initial blocks"
# Mine a single block every 10 seconds
while true
do
	curl --user devnet:devnet --data-binary '{"jsonrpc": "1.0", "id": "curltest", "method": "generatetoaddress", "params": [1, "n3GRiDLKWuKLCw1DZmV75W1mE35qmW2tQm"]}' -H 'content-type: text/plain;' http://bitcoin:18443/
	echo "mined a single block"
	sleep ${BTC_BLOCK_GEN_TIME}
done
