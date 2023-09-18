// @ts-nocheck FIXME
// https://github.com/dubzzz/fast-check/issues/2781
import fc from "https://cdn.skypack.dev/fast-check@3";

import { Chain } from "https://deno.land/x/clarinet@v1.7.1/index.ts";

export type Stub = {
  wallets: Map<string, number>; // string: Address, number: Balance
};

export type Real = {
  chain: Chain;
};

export type RomeoCommand = fc.Command<Stub, Real>;
