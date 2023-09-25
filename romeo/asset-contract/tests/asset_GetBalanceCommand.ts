import { AssetCommand, Real, Stub } from "./asset_CommandModel.ts";
import { assert } from "https://deno.land/std@0.202.0/assert/mod.ts";

import {
  Account,
  Tx,
  types,
} from "https://deno.land/x/clarinet@v1.7.1/index.ts";

export class GetBalanceCommand implements AssetCommand {
  readonly sender: Account;
  readonly wallet: Account;

  constructor(
    sender: Account,
    wallet: Account,
  ) {
    this.sender = sender;
    this.wallet = wallet;
  }

  check(_model: Readonly<Stub>): boolean {
    // Can always get balance.
    return true;
  }

  run(model: Stub, real: Real): void {
    const block = real.chain.mineBlock([
      Tx.contractCall(
        "asset",
        "get-balance",
        [types.principal(this.wallet.address)],
        this.sender.address,
      ),
    ]);

    const expected = model.wallets.get(this.wallet.address) ?? 0;
    block.receipts.map(({ result }) => result.expectOk().expectUint(expected));

    // sBTC DR allows several mints or burns with the same Bitcoin transaction.
    const actual = model.transactions.reduce((sum, [_, amount, wallet]) =>
      (wallet.address === this.wallet.address ? sum + amount : sum), 0);
    assert(
      expected === actual,
      `The bitcoin transaction does not match the balance. The bitcoin transaction amount is ${actual} and the balance is ${expected}`,
    );

    console.log(
      `✓ ${this.sender.name.padStart(8, " ")} ${`get-balance`.padStart(16, " ")} ${this.wallet.name.padStart(8, " ")} ${expected.toString().padStart(12, " ")}`,
    );
  }

  toString() {
    // fast-check will call toString() in case of errors, e.g. property failed.
    // It will then make a minimal counterexample, a process called 'shrinking'
    // https://github.com/dubzzz/fast-check/issues/2864#issuecomment-1098002642
    return `${this.sender} get-balance ${this.wallet.name}`;
  }
}
