#!/bin/sh
# Wait until bitcoin RPC is ready
echo "checking if bitcoin node is online"
until curl -f -s -o /dev/null --user devnet:devnet --data-binary '{"jsonrpc": "1.0", "id": "curltest", "method": "getblockcount", "params": []}' -H 'content-type: text/plain;' http://bitcoin:18443/
do
	echo "bitcoin node is not ready, sleep two seconds"
	sleep 2
done
echo "bitcoin node is ready"
npm start
