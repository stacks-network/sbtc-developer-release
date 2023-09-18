import { Real, Stub, RomeoCommand } from "./RomeoCommandModel.ts";

import {
  Account,
  Tx,
  types,
} from "https://deno.land/x/clarinet@v1.7.1/index.ts";

export class RomeoTransferCommand_NonOwner implements RomeoCommand {
  readonly sender: Account;
  readonly amount: number;
  readonly holder: Account;
  readonly wallet: Account;

  constructor(
    sender: Account,
    amount: number,
    holder: Account,
    wallet: Account,
  ) {
    this.sender = sender;
    this.amount = amount;
    this.holder = holder;
    this.wallet = wallet;
  }

  check(model: Readonly<Stub>): boolean {
    return this.sender.address !== this.holder.address &&
           this.sender.address !== this.wallet.address &&
           (model.wallets.get(this.holder.address) ?? 0) >= this.amount;
  }

  run(_: Stub, real: Real): void {
    const block = real.chain.mineBlock([
      Tx.contractCall(
        "asset",
        "transfer",
        [
          types.uint(this.amount),
          types.principal(this.holder.address),
          types.principal(this.wallet.address),
          types.none(), // FIXME
        ],
        this.sender.address,
      ),
    ]);

    block.receipts.map(({ result }) => result.expectErr().expectUint(2));

    console.log(
      `! ${this.sender.name.padStart(8, " ")} ${"transfer".padStart(16, " ") } ${this.wallet.name.padStart(8, " ")} ${this.amount.toString().padStart(12, " ") } (expected, non-owner)`
    );
  }

  toString() {
    // fast-check will call toString() in case of errors, e.g. property failed.
    // It will then make a minimal counterexample, a process called 'shrinking'
    // https://github.com/dubzzz/fast-check/issues/2864#issuecomment-1098002642
    return `${this.sender.name} transfer ${this.wallet.name} amount ${this.amount}`;
  }
}
