#!/bin/sh
set +x
sed -i s/{STACKS_NETWORK}/${STACKS_NETWORK}/g /src/stacks-node/Config.toml
/bin/stacks-node start --config /src/stacks-node/Config.toml
