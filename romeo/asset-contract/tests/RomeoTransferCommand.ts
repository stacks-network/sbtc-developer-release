import { Real, Stub, RomeoCommand } from "./RomeoCommandModel.ts";

import {
  Account,
  Tx,
  types,
} from "https://deno.land/x/clarinet@v1.7.1/index.ts";

export class RomeoTransferCommand implements RomeoCommand {
  readonly sender: Account;
  readonly amount: number;
  readonly wallet: Account;

  constructor(
    sender: Account,
    amount: number,
    wallet: Account,
  ) {
    this.sender = sender;
    this.amount = amount;
    this.wallet = wallet;
  }

  check(model: Readonly<Stub>): boolean {
    // Can transfer if sender is not the recepient wallet and sender has enough
    // funds.
    if (
      this.sender.address !== this.wallet.address &&
      (model.wallets.get(this.sender.address) ?? 0) >= this.amount
    ) {
      return true;
    } else {
      console.log(
        `! ${this.sender.name.padStart(8, " ")} ${"transfer".padStart(16, " ") } ${this.wallet.name.padStart(8, " ")} ${this.amount.toString().padStart(12, " ") } (discarded)`
      );
      return false;
    }
  }

  run(model: Stub, real: Real): void {
    const block = real.chain.mineBlock([
      Tx.contractCall(
        "asset",
        "transfer",
        [
          types.uint(this.amount),
          types.principal(this.sender.address),
          types.principal(this.wallet.address),
          types.none(), // FIXME
        ],
        this.sender.address,
      ),
    ]);

    block.receipts.map(({ result }) => result.expectOk());

    model.wallets.set(
      this.sender.address,
      (model.wallets.get(this.sender.address) ?? 0) - this.amount,
    );
    model.wallets.set(
      this.wallet.address,
      (model.wallets.get(this.wallet.address) ?? 0) + this.amount,
    );

    console.log(
      `✓ ${this.sender.name.padStart(8, " ")} ${"transfer".padStart(16, " ") } ${this.wallet.name.padStart(8, " ")} ${this.amount.toString().padStart(12, " ") }`
    );
  }

  toString() {
    // fast-check will call toString() in case of errors, e.g. property failed.
    // It will then make a minimal counterexample, a process called 'shrinking'
    // https://github.com/dubzzz/fast-check/issues/2864#issuecomment-1098002642
    return `${this.sender.name} transfer ${this.wallet.name} amount ${this.amount}`;
  }
}
