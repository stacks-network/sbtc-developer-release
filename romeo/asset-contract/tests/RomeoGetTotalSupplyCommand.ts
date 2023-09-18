import { Real, Stub, RomeoCommand } from "./RomeoCommandModel.ts";

import { Account, Tx } from "https://deno.land/x/clarinet@v1.7.1/index.ts";

export class RomeoGetTotalSupplyCommand implements RomeoCommand {
  readonly sender: Account;

  constructor(
    sender: Account,
  ) {
    this.sender = sender;
  }

  check(model: Readonly<Stub>): boolean {
    // Can always get total supply.
    return true;
  }

  run(model: Stub, real: Real): void {
    const block = real.chain.mineBlock([
      Tx.contractCall(
        "asset",
        "get-total-supply",
        [],
        this.sender.address,
      ),
    ]);

    let supply = 0;
    model.wallets.forEach((balance) => supply += balance);
    block.receipts.map(({ result }) => result.expectOk().expectUint(supply));

    console.log(
      `✓ ${this.sender.name.padStart(8, " ")} ${`get-total-supply`.padStart(16, " ")} ${supply.toString().padStart(21, " ")}`,
    );
  }

  toString() {
    // fast-check will call toString() in case of errors, e.g. property failed.
    // It will then make a minimal counterexample, a process called 'shrinking'
    // https://github.com/dubzzz/fast-check/issues/2864#issuecomment-1098002642
    return `${this.sender} get-total-supply`;
  }
}
