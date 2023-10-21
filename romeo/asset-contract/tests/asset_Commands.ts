import { BitcoinTxData } from "./asset_CommandModel.ts";

import fc from "fast-check";

import { BurnCommand } from "./asset_BurnCommand.ts";
import { BurnCommand_500 } from "./asset_BurnCommand_500.ts";
import { GetBalanceCommand } from "./asset_GetBalanceCommand.ts";
import { GetTotalSupplyCommand } from "./asset_GetTotalSupplyCommand.ts";
import { MintCommand } from "./asset_MintCommand.ts";
import { MintCommand_500 } from "./asset_MintCommand_500.ts";
import { TransferCommand } from "./asset_TransferCommand.ts";
import { TransferCommand_NonOwner } from "./asset_TransferCommand_NonOwner.ts";

export function AssetCommands(accounts: Map<string, string>) {
  const data = getBitcoinTxData();
  const cmds = [
    // BurnCommand
    fc
      .record({
        sender: fc.constant(accounts.get("deployer")!),
        amount: fc.integer({ min: 1, max: 100 }),
        wallet: fc.constantFrom(...accounts.values()).filter((a: string) => a !== accounts.get("deployer")!),
        params: fc.constantFrom(...data),
      })
      .map((
        r: {
          sender: string;
          amount: number;
          wallet: string;
          params: BitcoinTxData;
        },
      ) =>
        new BurnCommand(
          r.sender,
          r.amount,
          r.wallet,
          r.params,
        )
      ),

    // BurnCommand (err-btc-tx-already-used (err u500))
    fc
      .record({
        sender: fc.constant(accounts.get("deployer")!),
        amount: fc.integer({ min: 1, max: 100 }),
        wallet: fc.constantFrom(...accounts.values()).filter((a: string) => a !== accounts.get("deployer")!),
        params: fc.constantFrom(...data),
      })
      .map((
        r: {
          sender: string;
          amount: number;
          wallet: string;
          params: BitcoinTxData;
        },
      ) =>
        new BurnCommand_500(
          r.sender,
          r.amount,
          r.wallet,
          r.params,
        )
      ),

    // GetBalanceCommand
    fc
      .record({
        sender: fc.constantFrom(...accounts.values()),
        wallet: fc.constantFrom(...accounts.values()),
      })
      .map((
        r: {
          sender: string;
          wallet: string;
        },
      ) =>
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
      .map((
        r: {
          sender: string;
        },
      ) =>
        new GetTotalSupplyCommand(
          r.sender,
        )
      ),

    // MintCommand
    fc
      .record({
        sender: fc.constant(accounts.get("deployer")!),
        amount: fc.integer({ min: 1, max: 100 }),
        wallet: fc.constantFrom(...accounts.values()).filter((a: string) => a !== accounts.get("deployer")!),
        params: fc.constantFrom(...data),
      })
      .map((
        r: {
          sender: string;
          amount: number;
          wallet: string;
          params: BitcoinTxData;
        },
      ) =>
        new MintCommand(
          r.sender,
          r.amount,
          r.wallet,
          r.params,
        )
      ),

    // MintCommand (err-btc-tx-already-used (err u500))
    fc
      .record({
        sender: fc.constant(accounts.get("deployer")!),
        amount: fc.integer({ min: 1, max: 100 }),
        wallet: fc.constantFrom(...accounts.values()).filter((a: string) => a !== accounts.get("deployer")!),
        params: fc.constantFrom(...data),
      })
      .map((
        r: {
          sender: string;
          amount: number;
          wallet: string;
          params: BitcoinTxData;
        },
      ) =>
        new MintCommand_500(
          r.sender,
          r.amount,
          r.wallet,
          r.params,
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
          sender: string;
          amount: number;
          wallet: string;
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
          sender: string;
          amount: number;
          holder: string;
          wallet: string;
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

function getBitcoinTxData(): BitcoinTxData[] {
  return [{
    // https://github.com/stacks-network/sbtc/blob/dbe5209e087ec196181282e64d769844ae8fc2d5/romeo/asset-contract/tests/asset_test.clar#L13-L19
    depositTx: hexStringToUint8Array(
      "0x0168ee41db8a4766efe02bba1ebc0de320bc1b0abb7304f5f104818a9dd721cf",
    ),
    burnChainHeight: 1,
    merkleProof: [
      hexStringToUint8Array(
        "0x582b1900f55dad47d575138e91321c441d174e20a43336780c352a0b556ecc8b",
      ),
    ],
    txIndex: 1,
    treeDepth: 1,
    blockHeader: hexStringToUint8Array(
      "0x02000000000000000000000000000000000000000000000000000000000000000000000075b8bf903d0153e1463862811283ffbec83f55411c9fa5bd24e4207dee0dc1f1000000000000000000000000",
    ),
    blockHeaderHash: hexStringToUint8Array(
      "0x346993fc64b2a124a681111bb1f381e24dbef3cd362f0a40019238846c7ebf93",
    ),
  }, {
    // https://gist.github.com/setzeus/469e747290961c03adb09fbeff2534f3/b159465d49a3d5bfc25633e8189d798ce43e1992#file-merkletests-json-L113-L166
    // https://blockstream.info/testnet/block/0000000000000130de2402cb8ee45b755f1a80370ee30aa0db5ed28de6a75f84
    depositTx: hexStringToUint8Array(
      "0x255dcfd00b04456288b5aacc5275835add15c89067f082952331b7fc1b87a63c",
    ),
    burnChainHeight: 1,
    merkleProof: [
      hexStringToUint8Array(
        "0x12a5ec707a1285569eda4ee92178d82a409a1c00e3a14ec743d522242f1f5434",
      ),
      hexStringToUint8Array(
        "0x28a21919d2dd17fb4021be6f44bce9be34ed813a1794d2c66421f2d3d97acb50",
      ),
      hexStringToUint8Array(
        "0xcf648a05e8d001fd494ed67c80de91ec303baad36ededd967fc4a54ef26b5c31",
      ),
    ],
    txIndex: 1,
    treeDepth: 3,
    blockHeader: hexStringToUint8Array(
      "0x000060204c3f1a8884f4ef56bb6590a8a2237f8f1006543f5bc1a8f9e30000000000000003397473336b570cae8e82cc8d83c6711e01c1777d3dbc43c1dbb932248310a46fa00465fcff031a7f23bf84",
    ),
    blockHeaderHash: hexStringToUint8Array(
      "0x0000000000000130de2402cb8ee45b755f1a80370ee30aa0db5ed28de6a75f84",
    ),
  }];
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
