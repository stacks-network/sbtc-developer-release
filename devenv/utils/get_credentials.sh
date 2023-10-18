#!/bin/bash
# Returns the default credentials for the devnet

mnemonic="twice kind fence tip hidden tilt action fragile skin nothing glory cousin green tomorrow spring wrist shed math olympic multiply hip blue scout claw"

sbtc generate-from -s testnet -b regtest --accounts 2 mnemonic "$mnemonic"
