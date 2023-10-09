# sbtc

[![Daily Verification][daily-workflow-badge]][daily-workflow-link]
[![Code Coverage][coverage-badge]][coverage-link]
[![License: MIT][mit-license-badge]][mit-license-link]
[![Discord][discord-badge]][discord-link]


> Note: This repo is still in early development and is not ready for production use.

This repo contains, or will contain, packages that define sBTC primitives, signer components, helper tools such as `sbtc-cli` and `devenv`.

## stacks-core

At the moment this repo also contains Stacks primitives in the `stacks-core` package. The goal is to make this the default way of interfacing with the Stacks blockchain in Rust. At some point it might be moved to a different location.

It contains fundamental types and logic such as:

- crockford32 encoding
- hashing primitives (SHA256 and RIPEMD160)
- StacksAddress
- Uint support
- other types

## sbtc-core

This package contains the core logic for sBTC. For now, most of it is sBTC operation parsing and construction.

## romeo (sBTC Developer Release)

This package contains a version of the sBTC token (SIP-10) for developers on testnet.

Version 0.1 is a custodial solution that supports with deposit and withdrawal transaction on Bitoin only using OP_RETURN. The custodial solution is a rust binary that continuously observes the bitcoin and stacks blockchain.

## sbtc-cli

This package contains a command-line interface for sBTC to create and broadcast deposit and withdraw btc transactions. The cli also has a helper commands for credentials.
## devenv

This folder contains configuration files for docker images to launch services for sBTC in a local environment. Use `up.sh` to launch it, and use utility scripts to deposit and withdraw BTC. This environment can be used for demonstrations and automated testing.
## Contributing

**Before going any further please review our [code of conduct](CODE_OF_CONDUCT.md)**

### Getting Started

This repository uses the task runner cargo-make to manage its build scripts and CI. To install cargo-make, run the following command:

```bash
cargo install --version 0.36.13 cargo-make
```

Also verify that openssl is install on your machine.

[coverage-badge]: https://codecov.io/github/stacks-network/sbtc/branch/main/graph/badge.svg?token=2sbE9YLwT6
[coverage-link]: https://codecov.io/github/stacks-network/sbtc
[discord-badge]: https://img.shields.io/static/v1?logo=discord&label=discord&message=Join&color=blue
[discord-link]: https://discord.gg/WPWZPppr
[mit-license-badge]: https://img.shields.io/badge/License-MIT-yellow.svg
[mit-license-link]: https://opensource.org/licenses/MIT
[daily-workflow-badge]: https://github.com/stacks-network/sbtc/actions/workflows/daily.yml/badge.svg
[daily-workflow-link]: https://github.com/stacks-network/sbtc/actions/workflows/daily.yml
