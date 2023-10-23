import {
  AssetCommand,
  Real,
  Stub,
  shortenString
} from "./asset_CommandModel.ts";

import { tx } from "@hirosystems/clarinet-sdk";
import { Cl } from "@stacks/transactions";

import { expect } from "vitest";

export class TransferCommand implements AssetCommand {
  readonly sender: string;
  readonly amount: number;
  readonly wallet: string;

  constructor(
    sender: string,
    amount: number,
    wallet: string,
  ) {
    this.sender = sender;
    this.amount = amount;
    this.wallet = wallet;
  }

  check(model: Readonly<Stub>): boolean {
    // Can transfer if sender is not the recepient wallet and sender has enough
    // funds.
    if (
      this.sender !== this.wallet &&
      (model.wallets.get(this.sender) ?? 0) >= this.amount
    ) {
      return true;
    } else {
      console.log(
        `! ${shortenString(this.sender).padStart(8, " ")} ${"transfer".padStart(16, " ") } ${shortenString(this.wallet).padStart(8, " ")} ${this.amount.toString().padStart(12, " ") } (discarded)`
      );
      return false;
    }
  }

  run(model: Stub, real: Real): void {
    const block = real.simnet.mineBlock([
      tx.callPublicFn(
        "asset",
        "transfer",
        [
          Cl.uint(this.amount),
          Cl.standardPrincipal(this.sender),
          Cl.standardPrincipal(this.wallet),
          Cl.none(), // FIXME
        ],
        this.sender,
      ),
    ]);

    expect(block[0].result).toBeOk(Cl.bool(true));

    model.wallets.set(
      this.sender,
      (model.wallets.get(this.sender) ?? 0) - this.amount,
    );
    model.wallets.set(
      this.wallet,
      (model.wallets.get(this.wallet) ?? 0) + this.amount,
    );

    console.log(
      `✓ ${shortenString(this.sender).padStart(8, " ")} ${"transfer".padStart(16, " ") } ${shortenString(this.wallet).padStart(8, " ")} ${this.amount.toString().padStart(12, " ") }`
    );
  }

  toString() {
    // fast-check will call toString() in case of errors, e.g. property failed.
    // It will then make a minimal counterexample, a process called 'shrinking'
    // https://github.com/dubzzz/fast-check/issues/2864#issuecomment-1098002642
    return `${this.sender} transfer ${this.wallet} amount ${this.amount}`;
  }
}
