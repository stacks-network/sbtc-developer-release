import {
  AssetCommand,
  Real,
  Stub,
  shortenString
} from "./asset_CommandModel.ts";

import { Cl } from "@stacks/transactions";

import { expect } from "vitest";

export class GetTotalSupplyCommand implements AssetCommand {
  readonly sender: string;

  constructor(
    sender: string,
  ) {
    this.sender = sender;
  }

  check(_model: Readonly<Stub>): boolean {
    // Can always get total supply.
    return true;
  }

  run(model: Stub, real: Real): void {
    const { result } = real.simnet.callReadOnlyFn(
      "asset",
      "get-total-supply",
      [],
      this.sender,
    );

    let supply = 0;
    model.wallets.forEach((balance) => supply += balance);
    expect(result).toBeOk(Cl.uint(supply));

    console.log(
      `✓ ${shortenString(this.sender).padStart(8, " ")} ${`get-total-supply`.padStart(16, " ")} ${supply.toString().padStart(24, " ")}`,
    );
  }

  toString() {
    // fast-check will call toString() in case of errors, e.g. property failed.
    // It will then make a minimal counterexample, a process called 'shrinking'
    // https://github.com/dubzzz/fast-check/issues/2864#issuecomment-1098002642
    return `${this.sender} get-total-supply`;
  }
}
