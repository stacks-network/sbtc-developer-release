import { Real, Stub, RomeoCommand } from "./RomeoCommandModel.ts";

import {
  Account,
  Tx,
  types,
} from "https://deno.land/x/clarinet@v1.7.1/index.ts";

export class RomeoMintCommand implements RomeoCommand {
  readonly sender         : Account;
  readonly amount         : number;
  readonly wallet         : Account;
  readonly depositTx      : Uint8Array;
  readonly burnChainHeight: number;
  readonly merkleProof    : Uint8Array[];
  readonly txIndex        : number;
  readonly treeDepth      : number;
  readonly blockHeader    : Uint8Array;
  readonly blockHeaderHash: Uint8Array;

  constructor(
    sender         : Account,
    amount         : number,
    wallet         : Account,
    depositTx      : Uint8Array,
    burnChainHeight: number,
    merkleProof    : Uint8Array[],
    txIndex        : number,
    treeDepth      : number,
    blockHeader    : Uint8Array,
    blockHeaderHash: Uint8Array
  ) {
    this.sender          = sender;
    this.amount          = amount;
    this.wallet          = wallet;
    this.depositTx       = depositTx
    this.burnChainHeight = burnChainHeight;
    this.merkleProof     = merkleProof;
    this.txIndex         = txIndex;
    this.treeDepth       = treeDepth;
    this.blockHeader     = blockHeader;
    this.blockHeaderHash = blockHeaderHash;
  }

  check(model: Readonly<Stub>): boolean {
    // Can mint if sender is the deployer.
    //
    // Note that this is filtered at the generator level. So there's no need to
    // check here.
    //
    // If you don't filter at the generator level, you can check here. But, if
    // you check here, and don't filter at the generator level, you effectively
    // discard the command.
    return true;
  }

  run(model: Stub, real: Real): void {
    const block = real.chain.mineBlock([
      Tx.contractCall(
        "clarity-bitcoin-mini",
        "debug-insert-burn-header-hash",
        [
          types.buff(this.blockHeaderHash),
          types.uint(this.burnChainHeight),
        ],
        this.sender.address,
      ),
      Tx.contractCall(
        "asset",
        "mint",
        [
          types.uint(this.amount),
          types.principal(this.wallet.address),
          types.buff(this.depositTx),
          types.uint(this.burnChainHeight),
          types.list(this.merkleProof.map((p) => types.buff(p))),
          types.uint(this.txIndex),
          types.uint(this.treeDepth),
          types.buff(this.blockHeader),
        ],
        this.sender.address,
      ),
    ]);

    block.receipts.map(({ result }) => result.expectOk());

    const balance = model.wallets.get(this.wallet.address) ?? 0;
    model.wallets.set(this.wallet.address, balance + this.amount);

    console.log(
      `✓ ${this.sender.name.padStart(8, " ")} ${"mint".padStart(16, " ") } ${this.wallet.name.padStart(8, " ")} ${this.amount.toString().padStart(12, " ")}`
    );
  }

  toString() {
    // fast-check will call toString() in case of errors, e.g. property failed.
    // It will then make a minimal counterexample, a process called 'shrinking'
    // https://github.com/dubzzz/fast-check/issues/2864#issuecomment-1098002642
    return `${this.sender.name} mint ${this.amount} to ${this.wallet.name}`;
  }
}
