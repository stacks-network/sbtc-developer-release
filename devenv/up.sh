#!/usr/bin/env bash

set -ueo >/dev/null

FLAGS="-d --remove-orphans"
source ./common.sh

run $@
