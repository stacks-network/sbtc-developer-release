import {
  AssetCommand,
  Real,
  Stub,
  shortenString
} from "./asset_CommandModel.ts";

import { Cl } from "@stacks/transactions";

import { expect } from "vitest";

export class GetBitcoinWalletPublicKeyCommand implements AssetCommand {
  readonly sender: string;

  constructor(
    sender: string,
  ) {
    this.sender = sender;
  }

  check(model: Readonly<Stub>): boolean {
    return model.bitcoinWalletPublicKey !== undefined;
  }

  run(model: Stub, real: Real): void {
    const { result } = real.simnet.callReadOnlyFn(
      "asset",
      "get-bitcoin-wallet-public-key",
      [],
      this.sender,
    );

    const expected = model.bitcoinWalletPublicKey;
    expect(result).toBeSome(Cl.buffer(expected));

    console.log(
      `✓ ${shortenString(this.sender).padStart(8, " ")} ${"get-bitcoin-wallet-public-key".padStart(29, " ") } ${shortenString(uint8ArrayToHexString(expected))}`
    );
  }

  toString() {
    // fast-check will call toString() in case of errors, e.g. property failed.
    // It will then make a minimal counterexample, a process called 'shrinking'
    // https://github.com/dubzzz/fast-check/issues/2864#issuecomment-1098002642
    return `${this.sender} get-bitcoin-wallet-public`;
  }
}

function uint8ArrayToHexString(uint8Array: Uint8Array): string {
  return '0x' + Array.from(uint8Array).map(byte => byte.toString(16).padStart(2, '0')).join('');
}
