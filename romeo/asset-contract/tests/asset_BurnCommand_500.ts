import {
  AssetCommand,
  BitcoinTxData,
  Real,
  Stub,
} from "./asset_CommandModel.ts";

import {
  Account,
  Tx,
  types,
} from "https://deno.land/x/clarinet@v1.7.1/index.ts";

export class BurnCommand_500 implements AssetCommand {
  readonly sender: Account;
  readonly amount: number;
  readonly wallet: Account;
  readonly params: BitcoinTxData;

  constructor(
    sender: Account,
    amount: number,
    wallet: Account,
    params: BitcoinTxData,
  ) {
    this.sender = sender;
    this.amount = amount;
    this.wallet = wallet;
    this.params = params;
  }

  check(model: Readonly<Stub>): boolean {
    const btcTxHex = uint8ArrayToHexString(this.params.depositTx);
    const wasTxHexAlreadyUsed = model.transactions.some(([tx]) =>
      tx === btcTxHex
    );
    const balance = model.wallets.get(this.wallet.address) ?? 0;
    return wasTxHexAlreadyUsed && this.amount <= balance;
  }

  run(_model: Stub, real: Real): void {
    const block = real.chain.mineBlock([
      Tx.contractCall(
        "clarity-bitcoin-mini",
        "debug-insert-burn-header-hash",
        [
          types.buff(this.params.blockHeaderHash),
          types.uint(this.params.burnChainHeight),
        ],
        this.sender.address,
      ),
      Tx.contractCall(
        "asset",
        "burn",
        [
          types.uint(this.amount),
          types.principal(this.wallet.address),
          types.buff(this.params.depositTx),
          types.uint(this.params.burnChainHeight),
          types.list(this.params.merkleProof.map((p) => types.buff(p))),
          types.uint(this.params.txIndex),
          types.buff(this.params.blockHeader),
        ],
        this.sender.address,
      ),
    ]);

    block.receipts[0].result.expectOk();
    block.receipts[1].result.expectErr().expectUint(500);

    console.log(
      `! ${this.sender.name.padStart(8, " ")} ${"burn".padStart(16, " ") } ${this.wallet.name.padStart(8, " ")} ${this.amount.toString().padStart(12, " ")} bitcoin tx ${uint8ArrayToHexString(this.params.depositTx).padStart(12, " ")} (expected, same bitcoin tx)`
    );
  }

  toString() {
    return `${this.sender.name} burn ${this.amount} to ${this.wallet.name} (bitcoin tx ${uint8ArrayToHexString(this.params.depositTx).padStart(12, " ")})`;
  }
}

function uint8ArrayToHexString(uint8Array: Uint8Array): string {
  return Array.from(uint8Array).map((byte) =>
    byte.toString(16).padStart(2, "0")
  ).join("");
}
