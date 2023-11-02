import {
  AssetCommand,
  Real,
  Stub,
  shortenString
} from "./asset_CommandModel.ts";

import { tx } from "@hirosystems/clarinet-sdk";
import { Cl } from "@stacks/transactions";

import { expect } from "vitest";

export class SetBitcoinWalletPublicKeyCommand_NonOwner implements AssetCommand {
  readonly sender: string;
  readonly pubKey: Uint8Array;

  constructor(
    sender: string,
    pubKey: Uint8Array,
  ) {
    this.sender = sender;
    this.pubKey = pubKey;
  }

  check(_: Readonly<Stub>): boolean {
    // Can run if sender is not the deployer. This is filtered in the generator.
    return true;
  }

  run(_: Stub, real: Real): void {
    const block = real.simnet.mineBlock([
      tx.callPublicFn(
        "asset",
        "set-bitcoin-wallet-public-key",
        [
          Cl.buffer(this.pubKey),
        ],
        this.sender,
      ),
    ]);

    expect(block[0].result).toBeErr(Cl.uint(403));

    console.log(
      `! ${shortenString(this.sender).padStart(8, " ")} ${"set-bitcoin-wallet-public-key".padStart(29, " ") } ${shortenString(uint8ArrayToHexString(this.pubKey))}`
    );
  }

  toString() {
    // fast-check will call toString() in case of errors, e.g. property failed.
    // It will then make a minimal counterexample, a process called 'shrinking'
    // https://github.com/dubzzz/fast-check/issues/2864#issuecomment-1098002642
    return `${this.sender} set-bitcoin-wallet-public-key ${uint8ArrayToHexString(this.pubKey)}`;
  }
}

function uint8ArrayToHexString(uint8Array: Uint8Array): string {
  return '0x' + Array.from(uint8Array).map(byte => byte.toString(16).padStart(2, '0')).join('');
}
