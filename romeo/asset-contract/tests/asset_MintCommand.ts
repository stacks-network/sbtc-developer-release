import {
  AssetCommand,
  Real,
  Stub,
  TransactionData,
} from "./asset_CommandModel.ts";

import {
  Account,
  Tx,
  types,
} from "https://deno.land/x/clarinet@v1.7.1/index.ts";

export class MintCommand implements AssetCommand {
  readonly sender: Account;
  readonly amount: number;
  readonly wallet: Account;
  readonly params: TransactionData;

  constructor(
    sender: Account,
    amount: number,
    wallet: Account,
    params: TransactionData,
  ) {
    this.sender = sender;
    this.amount = amount;
    this.wallet = wallet;
    this.params = params;
  }

  check(_model: Readonly<Stub>): boolean {
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
          types.buff(this.params.blockHeaderHash),
          types.uint(this.params.burnChainHeight),
        ],
        this.sender.address,
      ),
      Tx.contractCall(
        "asset",
        "mint",
        [
          types.uint(this.amount),
          types.principal(this.wallet.address),
          types.buff(this.params.depositTx),
          types.uint(this.params.burnChainHeight),
          types.list(this.params.merkleProof.map((p) => types.buff(p))),
          types.uint(this.params.txIndex),
          types.uint(this.params.treeDepth),
          types.buff(this.params.blockHeader),
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
