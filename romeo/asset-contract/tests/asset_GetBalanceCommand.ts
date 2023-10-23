import {
  AssetCommand,
  Real,
  Stub,
  shortenString
} from "./asset_CommandModel.ts";

import { Cl } from "@stacks/transactions";

import { expect } from "vitest";

export class GetBalanceCommand implements AssetCommand {
  readonly sender: string;
  readonly wallet: string;

  constructor(
    sender: string,
    wallet: string,
  ) {
    this.sender = sender;
    this.wallet = wallet;
  }

  check(_model: Readonly<Stub>): boolean {
    // Can always get balance.
    return true;
  }

  run(model: Stub, real: Real): void {
    const { result } = real.simnet.callReadOnlyFn(
      "asset",
      "get-balance",
      [Cl.standardPrincipal(this.wallet)],
      this.sender,
    );

    const expected = model.wallets.get(this.wallet) ?? 0;
    expect(result).toBeOk(Cl.uint(expected));

    const actual = model.transactions.reduce((sum, [_, amount, wallet]) =>
      (wallet === this.wallet ? sum + amount : sum), 0);
    expect(
      expected === actual,
      `The bitcoin transaction does not match the balance. The bitcoin transaction amount is ${actual} and the balance is ${expected}.`,
    );

    console.log(
      `✓ ${shortenString(this.sender).padStart(8, " ")} ${`get-balance`.padStart(16, " ")} ${shortenString(this.wallet).padStart(8, " ")} ${expected.toString().padStart(12, " ")}`,
    );
  }

  toString() {
    // fast-check will call toString() in case of errors, e.g. property failed.
    // It will then make a minimal counterexample, a process called 'shrinking'
    // https://github.com/dubzzz/fast-check/issues/2864#issuecomment-1098002642
    return `${this.sender} get-balance ${this.wallet}`;
  }
}
