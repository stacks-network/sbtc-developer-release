import {
  AssetCommand,
  BitcoinTxData,
  Real,
  Stub,
} from "./asset_CommandModel.ts";

import { tx } from "@hirosystems/clarinet-sdk";
import { Cl } from "@stacks/transactions";

import { expect } from "vitest";

export class MintCommand implements AssetCommand {
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
    // Can mint if sender is the deployer.
    //
    // Note that this is filtered at the generator level. So you don't need to
    // check here.
    //
    // If you don't filter at the generator level, you can check here but then
    // if you return false from here the command is 'discarded'.
    //
    // What discard means is that if you are generating 1000 commands, and 100
    // of them are filtered out here, then you end up running 900 commands. If
    // you filter at the generator level, however, you will run 1000 commands.
    const btcTxHex = uint8ArrayToHexString(this.params.depositTx);
    if (model.transactions.find(([tx, _amount, _wallet]) => tx === btcTxHex)) {
      return false;
    }

    return true;
  }

  run(model: Stub, real: Real): void {
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
    expect(block[1].result).toBeOk(Cl.bool(true));

    const balance = model.wallets.get(this.wallet) ?? 0;
    model.wallets.set(this.wallet, balance + this.amount);

    model.transactions.push([
      uint8ArrayToHexString(this.params.depositTx),
      this.amount,
      this.wallet,
    ]);

    console.log(
      `✓ ${this.sender.padStart(8, " ")} ${"mint".padStart(16, " ") } ${this.wallet.padStart(8, " ")} ${this.amount.toString().padStart(12, " ")} bitcoin tx ${uint8ArrayToHexString(this.params.depositTx).padStart(12, " ")}`
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

