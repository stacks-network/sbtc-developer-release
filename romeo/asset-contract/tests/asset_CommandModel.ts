import fc from "fast-check";

import { Simnet } from "@hirosystems/clarinet-sdk";

export type Stub = {
  wallets: Map<string, number>; // string: Address, number: Balance
  transactions: [string, number, string][]; // string: Id, number: Mint/Burn Amount, string: Stacks Account
};

export type Real = {
  simnet: Simnet;
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

export function shortenString(
  address: string,
  charsToShow: number = 4,
  delimiter: string = "...",
): string {
  if (address.length <= charsToShow * 2 + delimiter.length) {
    return address;
  }
  const start = address.substring(0, charsToShow);
  const end = address.substring(address.length - charsToShow);
  return start + delimiter + end;
}
