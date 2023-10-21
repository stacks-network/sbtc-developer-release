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
./log.sh bitcoin
./log.sh stacks
./log.sh stacks-api
./log.sh stacks-explorer
./log.sh postgres
./log.sh mongodb
./log.sh miner
./log.sh sbtc
./log.sh electrs
./log.sh sbtc-bridge-web
./log.sh sbtc-bridge-api
./log.sh mempool-web
./log.sh mempool-api
```
## Services

### Miner
There is a BTC mining service which will create a legacy wallet,
importaddress for UXTO monitoring that is defined in the stacks
Config.toml.

By default it automatically mines 200 blocks initially, and
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
You can access the [Bitcoin Explorer](https://github.com/mempool/mempool)
explorer at:

```
http://127.0.0.1:8083
```

Additionally:
- A Bitcoin Core RPC API is accessible at port 18443 (and proxied at port 18433 for CORS).
- A Blockstream-like API (based on `mempool/electrs`) is accessible at port 3002.

### Stacks
You can access the [Stacks Explorer](https://github.com/hirosystems/explorer)
at:

```
http://127.0.0.1:3020/?chain=testnet&api=http://127.0.0.1:3999
```
It's important to use the above URL, as it can parse blocks properly.

The Stacks API service is running on port 3999.

### sBTC Bridge App
The sBTC bridge app is running at:

```
http://127.0.0.1:8080/
```
### sBTC Bridge API
The sBTC bridge api is running at:

```
http://127.0.0.1:3010/
```

### Electrs (Electrum Rust Server)
The electrs service is running at:

```
http://127.0.0.1:60401
```

Additionally, the electrs service offers an HTTP API at port 3002 (based on `mempool/electrs`).

## sBTC Development
If you would like to build sbtc standalone:

```
cd /devenv/sbtc
./build.sh
```

After the deployment is up, generate a new private key:

```
./devenv/sbtc/bin/sbtc generate-from -b regtest -s testnet new
```

Take the mnemonic phrase and update the sbtc config:

```
cd /devenv/sbtc/docker/
vim config.json
```
In order to deploy the sBTC contract you must first fund your STX address listed above in the `config.json` file.

Download a wallet client:

  - [Leather Wallet Browser Extension](https://leather.io/install-extension)
  - [Leather Wallet Desktop Client](https://github.com/leather-wallet/desktop/releases)

If you wish to use the desktop client, you *MUST* download the testnet version of the executable.

### Prefilled STX Wallets

#### Wallet 0
```
mnemonic:"twice kind fence tip hidden tilt action fragile skin nothing glory cousin green tomorrow spring wrist shed math olympic multiply hip blue scout claw"
secret_key: 753b7cc01a1a2e86221266a154af739463fce51219d97e4f856cd7200c3bd2a601
stx_address: ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM
btc_address: mqVnk6NPRdhntvfm4hh9vvjiRkFDUuSYsHddress
balance: 100000000000000
```
#### Wallet 1
```
mnemonic: "sell invite acquire kitten bamboo drastic jelly vivid peace spawn twice guilt pave pen trash pretty park cube fragile unaware remain midnight betray rebuild"
secret_key: 7287ba251d44a4d3fd9276c88ce34c5c52a038955511cccaf77e61068649c17801
stx_address: ST1SJ3DTE5DN7X54YDH5D64R3BCB6A2AG2ZQ8YPD5
btc_address: mr1iPkD9N3RJZZxXRk7xF9d36gffa6exNC
balance: 100000000000000
```
#### Wallet 2
```
mnemonic: "hold excess usual excess ring elephant install account glad dry fragile donkey gaze humble truck breeze nation gasp vacuum limb head keep delay hospital"
secret_key: 530d9f61984c888536871c6573073bdfc0058896dc1adfe9a6a10dfacadc209101
stx_address: ST2CY5V39NHDPWSXMW9QDT3HC3GD6Q6XX4CFRK9AG
btc_address: muYdXKmX9bByAueDe6KFfHd5Ff1gdN9ErG
amount: 100000000000000
```
#### Wallet 3
```
mnemonic: "cycle puppy glare enroll cost improve round trend wrist mushroom scorpion tower claim oppose clever elephant dinosaur eight problem before frozen dune wagon high"
secret_key: d655b2523bcd65e34889725c73064feb17ceb796831c0e111ba1a552b0f31b3901
stx_address: ST2JHG361ZXG51QTKY2NQCVBPPRRE2KZB1HR05NNC
btc_address: mvZtbibDAAA3WLpY7zXXFqRa3T4XSknBX7
amount: 100000000000000
```
#### Wallet 4
```
mnemonic: "board list obtain sugar hour worth raven scout denial thunder horse logic fury scorpion fold genuine phrase wealth news aim below celery when cabin"
secret_key: f9d7206a47f14d2870c163ebab4bf3e70d18f5d14ce1031f3902fbbc894fe4c701
stx_address: ST2NEB84ASENDXKYGJPQW86YXQCEFEX2ZQPG87ND
btc_address: mg1C76bNTutiCDV3t9nWhZs3Dc8LzUufj8
amount: 100000000000000
```
#### Wallet 5
```
mnemonic: "hurry aunt blame peanut heavy update captain human rice crime juice adult scale device promote vast project quiz unit note reform update climb purchase"
secret_key: 3eccc5dac8056590432db6a35d52b9896876a3d5cbdea53b72400bc9c2099fe801
stx_address: ST2REHHS5J3CERCRBEPMGH7921Q6PYKAADT7JP2VB
btc_address: mweN5WVqadScHdA81aATSdcVr4B6dNokqx
amount: 100000000000000
```
#### Wallet 6
```
mnemonic: "area desk dutch sign gold cricket dawn toward giggle vibrant indoor bench warfare wagon number tiny universe sand talk dilemma pottery bone trap buddy"
secret_key: 7036b29cb5e235e5fd9b09ae3e8eec4404e44906814d5d01cbca968a60ed4bfb01
stx_address: ST3AM1A56AK2C1XAFJ4115ZSV26EB49BVQ10MGCS0
btc_address: mzxXgV6e4BZSsz8zVHm3TmqbECt7mbuErt
amount: 100000000000000
```
#### Wallet 7
```
mnemonic: "prevent gallery kind limb income control noise together echo rival record wedding sense uncover school version force bleak nuclear include danger skirt enact arrow"
secret_key: b463f0df6c05d2f156393eee73f8016c5372caa0e9e29a901bb7171d90dc4f1401
stx_address: ST3PF13W7Z0RRM42A8VZRVFQ75SV1K26RXEP8YGKJ
btc_address: n37mwmru2oaVosgfuvzBwgV2ysCQRrLko7
amount: 100000000000000
```
#### Wallet 8
```
mnemonic: "female adjust gallery certain visit token during great side clown fitness like hurt clip knife warm bench start reunion globe detail dream depend fortune"
secret_key: 6a1a754ba863d7bab14adbbc3f8ebb090af9e871ace621d3e5ab634e1422885e01
stx_address: ST3NBRSFKX28FQ2ZJ1MAKX58HKHSDGNV5N7R21XCP
btc_address: n2v875jbJ4RjBnTjgbfikDfnwsDV5iUByw
amount: 100000000000000
```
#### Wallet 9
```
mnemonic: "shadow private easily thought say logic fault paddle word top book during ignore notable orange flight clock image wealth health outside kitten belt reform"
secret_key: de433bdfa14ec43aa1098d5be594c8ffb20a31485ff9de2923b2689471c401b801
stx_address: STNHKEPYEPJ8ET55ZZ0M5A34J0R3N5FM2CMMMAZ6
btc_address: mjSrB3wS4xab3kYqFktwBzfTdPg367ZJ2d
amount: 100000000000000
```

Using one of the wallets above, send 100 STX to your STX address you generated with the `sbtc-cli` above.

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
