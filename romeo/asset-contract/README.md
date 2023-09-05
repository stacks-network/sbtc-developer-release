# Clarity contracts for sBTC DR 0.1

This folder contains clarity contracts and tools for clarity supporting the sBTC DR 0.1 (Romeo).

## Contract `asset.clar`
sBTC is a wrapped BTC asset on Stacks.

It is a fungible token (SIP-10) that is backed 1:1 by BTC
For this version the wallet is controlled by a centralized entity.
sBTC is minted when BTC is deposited into the wallet and
burned when BTC is withdrawn from the wallet.

Requests for minting and burning are made by the contract owner.

## Getting started for developers
See https://stacks-network.github.io/sbtc-docs/

## Contributing

### Unit tests
Tests are written in clarity. It requires `clarinet`. You can install the lastest version via `./scripts/install_clarinet_*`.

Run the unit tests through the script
```
./scripts/tests.sh
```

### Dependencies
When updating the version of clarinet deno library for tests make sure to update
* deps.ts
* generate-test-ts
* install_clarinet_action.sh