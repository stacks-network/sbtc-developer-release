#!/usr/bin/env bash
if ! command -v brew &> /dev/null
then
    echo "brew could not be found, please install it first. See https://brew.sh/"
    exit
fi

brew install clarinet