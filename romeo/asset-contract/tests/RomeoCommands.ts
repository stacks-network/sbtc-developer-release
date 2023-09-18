import { Account } from "https://deno.land/x/clarinet@v1.7.1/index.ts";

import fc from "https://cdn.skypack.dev/fast-check@3";

import { RomeoGetBalanceCommand } from "./RomeoGetBalanceCommand.ts";
import { RomeoGetTotalSupplyCommand } from "./RomeoGetTotalSupplyCommand.ts";

export function RomeoCommands(accounts: Map<string, Account>) {
  const cmds = [
    // RomeoGetBalanceCommand
    fc
      .record({
        sender: fc.constantFrom(...accounts.values()),
        wallet: fc.constantFrom(...accounts.values()),
      })
      .map((r: { sender: Account; wallet: Account }) =>
        new RomeoGetBalanceCommand(
          r.sender,
          r.wallet,
        )
      ),

    // RomeoGetTotalSupplyCommand
    fc
      .record({
        sender: fc.constantFrom(...accounts.values()),
      })
      .map((r: { sender: Account; wallet: Account }) =>
        new RomeoGetTotalSupplyCommand(
          r.sender,
        )
      ),
  ];
  // More on size: https://github.com/dubzzz/fast-check/discussions/2978
  // More on cmds: https://github.com/dubzzz/fast-check/discussions/3026
  return fc.commands(cmds, { size: "large" });
}
