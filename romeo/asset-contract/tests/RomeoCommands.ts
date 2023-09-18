import { Account } from "https://deno.land/x/clarinet@v1.7.1/index.ts";

import fc from "https://cdn.skypack.dev/fast-check@3";

import { RomeoGetBalanceCommand } from "./RomeoGetBalanceCommand.ts";
import { RomeoGetTotalSupplyCommand } from "./RomeoGetTotalSupplyCommand.ts";
import { RomeoTransferCommand } from "./RomeoTransferCommand.ts";
import { RomeoTransferCommand_NonOwner } from "./RomeoTransferCommand_NonOwner.ts";

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

    // RomeoTransferCommand
    fc
      .record({
        sender: fc.constantFrom(...accounts.values()),
        amount: fc.integer({ min: 1 }),
        wallet: fc.constantFrom(...accounts.values()),
      })
      .map((
        r: {
          sender: Account;
          amount: number;
          wallet: Account;
        },
      ) =>
        new RomeoTransferCommand(
          r.sender,
          r.amount,
          r.wallet,
        )
      ),

      fc
      .record({
        sender: fc.constantFrom(...accounts.values()),
        amount: fc.integer({ min: 1 }),
        holder: fc.constantFrom(...accounts.values()),
        wallet: fc.constantFrom(...accounts.values()),
      })
      .map((
        r: {
          sender: Account;
          amount: number;
          holder: Account;
          wallet: Account;
        },
      ) =>
        new RomeoTransferCommand_NonOwner(
          r.sender,
          r.amount,
          r.holder,
          r.wallet,
        )
      ),
  ];
  // More on size: https://github.com/dubzzz/fast-check/discussions/2978
  // More on cmds: https://github.com/dubzzz/fast-check/discussions/3026
  return fc.commands(cmds, { size: "large" });
}
