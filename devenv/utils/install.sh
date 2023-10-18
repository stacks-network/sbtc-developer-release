#!/bin/bash

DIR="$(dirname "$0")"

for PKG in romeo sbtc-cli
do
	cargo install --path $DIR/../../$PKG
done
