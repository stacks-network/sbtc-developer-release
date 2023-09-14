#!/bin/bash

# Accept transaction bytes in hex as an argument
transaction_bytes="$1"

if [ -z "$transaction_bytes" ]; then
    echo "Error: No transaction bytes were passed as an argument."
    exit 1
fi

# Broadcast it to a local Stacks node
curl -s -X POST http://localhost:3999/v2/transactions \
  -H "Content-Type: application/octet-stream"\
  --data-binary "$transaction_bytes"
