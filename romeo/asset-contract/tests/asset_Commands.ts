import { Account } from "https://deno.land/x/clarinet@v1.7.1/index.ts";

import fc from "https://cdn.skypack.dev/fast-check@3";

import { GetBalanceCommand } from "./asset_GetBalanceCommand.ts";
import { GetTotalSupplyCommand } from "./asset_GetTotalSupplyCommand.ts";
import { MintCommand } from "./asset_MintCommand.ts";
import { TransferCommand } from "./asset_TransferCommand.ts";
import { TransferCommand_NonOwner } from "./asset_TransferCommand_NonOwner.ts";

export function AssetCommands(accounts: Map<string, Account>) {
  const cmds = [
    // GetBalanceCommand
    fc
      .record({
        sender: fc.constantFrom(...accounts.values()),
        wallet: fc.constantFrom(...accounts.values()),
      })
      .map((r: { sender: Account; wallet: Account }) =>
        new GetBalanceCommand(
          r.sender,
          r.wallet,
        )
      ),

    // GetTotalSupplyCommand
    fc
      .record({
        sender: fc.constantFrom(...accounts.values()),
      })
      .map((r: { sender: Account; wallet: Account }) =>
        new GetTotalSupplyCommand(
          r.sender,
        )
      ),

    // MintCommand
    fc
      .record({
        sender: fc.constant(accounts.get("deployer")!),
        amount: fc.integer({ min: 1 }),
        wallet: fc.constantFrom(...accounts.values()).filter((a: Account) =>
          a.address !== accounts.get("deployer")!.address
        ),
        // FIXME: Generate random depositTx, burnChainHeight, merkleProof, txIndex, treeDepth, blockHeader, blockHeaderHash.
        // https://github.com/stacks-network/sbtc/issues/146
        depositTx: fc.constant(
          hexStringToUint8Array(
            "0x0168ee41db8a4766efe02bba1ebc0de320bc1b0abb7304f5f104818a9dd721cf",
          ),
        ),
        burnChainHeight: fc.constant(1),
        merkleProof: fc.constant([
          hexStringToUint8Array(
            "0x582b1900f55dad47d575138e91321c441d174e20a43336780c352a0b556ecc8b",
          ),
        ]),
        txIndex: fc.constant(1),
        treeDepth: fc.constant(1),
        blockHeader: fc.constant(
          hexStringToUint8Array(
            "0x02000000000000000000000000000000000000000000000000000000000000000000000075b8bf903d0153e1463862811283ffbec83f55411c9fa5bd24e4207dee0dc1f1000000000000000000000000",
          ),
        ),
        blockHeaderHash: fc.constant(
          hexStringToUint8Array(
            "0x346993fc64b2a124a681111bb1f381e24dbef3cd362f0a40019238846c7ebf93",
          ),
        ),
      })
      .map((
        r: {
          sender         : Account;
          amount         : number;
          wallet         : Account;
          depositTx      : Uint8Array;
          burnChainHeight: number;
          merkleProof    : Uint8Array[];
          txIndex        : number;
          treeDepth      : number;
          blockHeader    : Uint8Array;
          blockHeaderHash: Uint8Array;
        },
      ) =>
        new MintCommand(
          r.sender,
          r.amount,
          r.wallet,
          r.depositTx,
          r.burnChainHeight,
          r.merkleProof,
          r.txIndex,
          r.treeDepth,
          r.blockHeader,
          r.blockHeaderHash,
        )
      ),

    // TransferCommand
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
        new TransferCommand(
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
        new TransferCommand_NonOwner(
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

function hexStringToUint8Array(hexString: string): Uint8Array {
  if (hexString.startsWith("0x")) {
    hexString = hexString.slice(2);
  }

  const uint8Array = new Uint8Array(hexString.length / 2);

  for (let i = 0; i < hexString.length; i += 2) {
    uint8Array[i / 2] = parseInt(hexString.substring(i, i + 2), 16);
  }

  return uint8Array;
}
