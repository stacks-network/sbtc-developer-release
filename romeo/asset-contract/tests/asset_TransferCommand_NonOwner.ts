import {
  AssetCommand,
  Real,
  Stub,
  shortenString
} from "./asset_CommandModel.ts";

import { tx } from "@hirosystems/clarinet-sdk";
import { Cl } from "@stacks/transactions";

import { expect } from "vitest";

export class TransferCommand_NonOwner implements AssetCommand {
  readonly sender: string;
  readonly amount: number;
  readonly holder: string;
  readonly wallet: string;

  constructor(
    sender: string,
    amount: number,
    holder: string,
    wallet: string,
  ) {
    this.sender = sender;
    this.amount = amount;
    this.holder = holder;
    this.wallet = wallet;
  }

  check(model: Readonly<Stub>): boolean {
    return this.sender !== this.holder &&
           this.sender !== this.wallet &&
           (model.wallets.get(this.holder) ?? 0) >= this.amount;
  }

  run(_: Stub, real: Real): void {
    const block = real.simnet.mineBlock([
      tx.callPublicFn(
        "asset",
        "transfer",
        [
          Cl.uint(this.amount),
          Cl.standardPrincipal(this.holder),
          Cl.standardPrincipal(this.wallet),
          Cl.none(), // FIXME
        ],
        this.sender,
      ),
    ]);

    expect(block[0].result).toBeErr(Cl.uint(2));

    console.log(
      `! ${shortenString(this.sender).padStart(8, " ")} ${"transfer".padStart(16, " ") } ${shortenString(this.wallet).padStart(8, " ")} ${this.amount.toString().padStart(12, " ") } (expected, non-owner)`
    );
  }

  toString() {
    // fast-check will call toString() in case of errors, e.g. property failed.
    // It will then make a minimal counterexample, a process called 'shrinking'
    // https://github.com/dubzzz/fast-check/issues/2864#issuecomment-1098002642
    return `${this.sender} transfer ${this.wallet} amount ${this.amount}`;
  }
}
