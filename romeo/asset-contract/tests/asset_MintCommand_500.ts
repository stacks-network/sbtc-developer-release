import {
  AssetCommand,
  BitcoinTxData,
  Real,
  Stub,
  shortenString
} from "./asset_CommandModel.ts";

import { tx } from "@hirosystems/clarinet-sdk";
import { Cl } from "@stacks/transactions";

import { expect } from "vitest";

export class MintCommand_500 implements AssetCommand {
  readonly sender: string;
  readonly amount: number;
  readonly wallet: string;
  readonly params: BitcoinTxData;

  constructor(
    sender: string,
    amount: number,
    wallet: string,
    params: BitcoinTxData,
  ) {
    this.sender = sender;
    this.amount = amount;
    this.wallet = wallet;
    this.params = params;
  }

  check(model: Readonly<Stub>): boolean {
    const btcTxHex = uint8ArrayToHexString(this.params.depositTx);
    const wasTxHexAlreadyUsed = model.transactions.some(([tx]) =>
      tx === btcTxHex
    );
    return wasTxHexAlreadyUsed;
  }

  run(_model: Stub, real: Real): void {
    const block = real.simnet.mineBlock([
      tx.callPublicFn(
        "clarity-bitcoin-mini",
        "debug-insert-burn-header-hash",
        [
          Cl.buffer(this.params.blockHeaderHash),
          Cl.uint(this.params.burnChainHeight),
        ],
        this.sender,
      ),
      tx.callPublicFn(
        "asset",
        "mint",
        [
          Cl.uint(this.amount),
          Cl.standardPrincipal(this.wallet),
          Cl.buffer(this.params.depositTx),
          Cl.uint(this.params.burnChainHeight),
          Cl.list(this.params.merkleProof.map((p) => Cl.buffer(p))),
          Cl.uint(this.params.txIndex),
          Cl.buffer(this.params.blockHeader),
        ],
        this.sender,
      ),
    ]);

    expect(block[0].result).toBeOk(Cl.bool(true));
    expect(block[1].result).toBeErr(Cl.uint(500));

    console.log(
      `! ${shortenString(this.sender).padStart(8, " ")} ${"mint".padStart(16, " ") } ${shortenString(this.wallet).padStart(8, " ")} ${this.amount.toString().padStart(12, " ")} bitcoin tx ${shortenString(uint8ArrayToHexString(this.params.depositTx)).padStart(12, " ")} (expected, same bitcoin tx)`
    );
  }

  toString() {
    // fast-check will call toString() in case of errors, e.g. property failed.
    // It will then make a minimal counterexample, a process called 'shrinking'
    // https://github.com/dubzzz/fast-check/issues/2864#issuecomment-1098002642
    return `${this.sender} mint ${this.amount} to ${this.wallet} (bitcoin tx ${uint8ArrayToHexString(this.params.depositTx).padStart(12, " ")})`;
  }
}

function uint8ArrayToHexString(uint8Array: Uint8Array): string {
  return Array.from(uint8Array).map(byte => byte.toString(16).padStart(2, '0')).join('');
}

