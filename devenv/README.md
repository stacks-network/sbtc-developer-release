# Docker Based Development Environment

This is a collection of Dockerized services to provide a simple 
standalone development environment to sBTC developers. It also 
includes some helper scripts to make it easier to operate.

## Docker and Docker Compose

To use this environment, you must install:

- [Docker](https://docs.docker.com/engine/install/)
- [Docker Compose](https://docs.docker.com/compose/install/)

## Building Containers

In order to deploy the environment, you must first build the images.

```
cd devenv 
./build.sh
```

If you prefer only to build a specific container:

```
cd devenv/bitcoin/
./build.sh
```

## Running Containers

To start the entire deployment simply run:

```
cd devenv
./up.sh
```

By default, this will start a BTC node on regtest, 
STX node on mocknet, stacks-api and database.

If you prefer to run a specific container:

```
cd devenv/bitcoin/
./up.sh
```

## Logging Containers

There is a helper script at the top level directory to facilitate logging:

```
./log bitcoin
./log bitcoin-explorer
./log stacks
./log stacks-api
./log stacks-explorer
./log postgres
./log miner
./log sbtc
```
## Services

### Miner
There is a BTC mining service which will create a legacy wallet, 
importaddress for UXTO monitoring that is defined in the stacks 
Config.toml.

By default it automatically mines 100 blocks initially, and 
generates 1 block every ten seconds from there on.

If you want to customize these values you can, update the 
variables in the docker-compose.yml:

```
INIT_BTC_BLOCKS: <number of blocks to initially mine>
```
```
BTC_BLOCK_GEN_TIME: <number of seconds before the next block is mined>
```
If you need the BTC wallet private key, it is listed in the stacks Config.toml

### Bitcoin
You can access the [Bitcoin Explorer](https://github.com/janoside/btc-rpc-explorer)
explorer at:

```
http://127.0.0.1:3002
```

### Stacks
You can access the [Stacks Explorer](https://github.com/hirosystems/explorer)
at:

```
http://127.0.0.1:3000/?chain=testnet
```
It's important to use the above URL, as it can parse blocks properly.

The Stacks API service is running on port 3999.

### sBTC Bridge App
The sBTC bridge app is running at:

```
http://127.0.0.1:4137/
```

## sBTC Development
If you would like to build sbtc standalone:

```
cd /devenv/sbtc
./build.sh
```

After the deployment is up, generate a new private key:

```
./devenv/sbtc/bin/sbtc generate-from new
```

Take the WIF + mnemonic phrase and update the sbtc config:

```
cd /devenv/sbtc/docker/
vim config.json
```
In order to deploy the sBTC contract you must first fund your STX address listed above in the `config.json` file.

Download a wallet client:

  - [Leather Wallet Browser Extension](https://leather.io/install-extension)
  - [Leather Wallet Desktop Client](https://github.com/leather-wallet/desktop/releases)

If you wish to use the desktop client, you *MUST* download the testnet version of the executable.

You can use any one of the wallet mnemonic phrases in the 
`../romeo/asset-contract/settings/Devnet.toml` file for the client.

Now send 100 STX to your STX address you generated with the `sbtc-cli` above.

Lastly, bring the service down and up:

```
docker compose down sbtc
docker compose up -d sbtc
```

## Stopping Containers

To stop the entire deployment simply run:

```
cd devenv
./down.sh
```

If you prefer to stop a specific container:

```
cd devenv/bitcoin
./down.sh
```

## Persistence 

At the moment, the container data will not persist. However it is 
easy to add persistent storage volumes if needed.

## TODO

- Why does it take stacks so long to start mining blocks?
- Write a wrapper to wait for the stacks API to 
  populate before running romeo
- Investigate Docker Compose Fragment and Extensions.
- Add Bridge webapp and API
