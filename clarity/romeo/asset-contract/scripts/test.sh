#!/bin/sh

# Setup directories.
mkdir -p .test
mkdir -p .coverage
rm -r .test/*

# Exit on next failure.
set -e

# Verify syntax.
clarinet check

# Generate tests.
clarinet run --allow-write --allow-read ext/generate-tests.ts

# Test with coverage.
clarinet test --coverage .coverage/lcov.info .test
