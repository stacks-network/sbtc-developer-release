// @ts-nocheck FIXME
// https://github.com/dubzzz/fast-check/issues/2781
import fc from "https://cdn.skypack.dev/fast-check@3";

import { Chain } from "https://deno.land/x/clarinet@v1.7.1/index.ts";

export type Stub = {
  wallets: Map<string, number>; // string: Address, number: Balance
  transactions: Tuple<string, number, Account>[]; // string: Id, number: Mint/Burn Amount, Account: Stacks Account
};

export type Real = {
  chain: Chain;
};

export type AssetCommand = fc.Command<Stub, Real>;

export interface BitcoinTxData {
  depositTx: Uint8Array;
  burnChainHeight: number;
  merkleProof: Uint8Array[];
  txIndex: number;
  treeDepth: number;
  blockHeader: Uint8Array;
  blockHeaderHash: Uint8Array;
}
