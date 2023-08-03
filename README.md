# sbtc

> Note: This repo is still in early development and is not ready for production use.

This repo contains, or will contain, packages that define sBTC primitives, signer components, helper tools such as `sbtc-cli`.

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
