import {
  Account,
  Chain,
  Clarinet,
  Tx,
} from "https://deno.land/x/clarinet@v1.7.1/index.ts";

import fc from "https://cdn.skypack.dev/fast-check@3";

Clarinet.test({
  name: "asset.clar: Can get token URI",
  fn(chain: Chain, accounts: Map<string, Account>) {
    fc.assert(fc.property(
      fc.record({
        sender: fc.constantFrom(...accounts.values()),
      }),
      (r: { sender: Account }) => {
        // Arrange
        const sender = r.sender;

        // Act
        const block = chain.mineBlock([
          Tx.contractCall(
            "asset",
            "get-token-uri",
            [],
            sender.address,
          ),
        ]);

        // Assert
        block.receipts.map(({ result }) =>
          result.expectOk().expectSome().expectUtf8(
            "https://gateway.pinata.cloud/ipfs/Qma5P7LFGQAXt7gzkNZGxet5qJcVxgeXsenDXwu9y45hpr?_gl=1*1mxodt*_ga*OTU1OTQzMjE2LjE2OTQwMzk2MjM.*_ga_5RMPXG14TE*MTY5NDA4MzA3OC40LjEuMTY5NDA4MzQzOC42MC4wLjA",
          )
        );
      },
    ));
  },
});

Clarinet.test({
  name: "asset.clar: Can get symbol",
  fn(chain: Chain, accounts: Map<string, Account>) {
    fc.assert(fc.property(
      fc.record({
        sender: fc.constantFrom(...accounts.values()),
      }),
      (r: { sender: Account }) => {
        // Arrange
        const sender = r.sender;

        // Act
        const block = chain.mineBlock([
          Tx.contractCall(
            "asset",
            "get-symbol",
            [],
            sender.address,
          ),
        ]);

        // Assert
        block.receipts.map(({ result }) =>
          result.expectOk().expectAscii("sBTC")
        );
      },
    ));
  },
});

Clarinet.test({
  name: "asset.clar: Can get name",
  fn(chain: Chain, accounts: Map<string, Account>) {
    fc.assert(fc.property(
      fc.record({
        sender: fc.constantFrom(...accounts.values()),
      }),
      (r: { sender: Account }) => {
        // Arrange
        const sender = r.sender;

        // Act
        const block = chain.mineBlock([
          Tx.contractCall(
            "asset",
            "get-name",
            [],
            sender.address,
          ),
        ]);

        // Assert
        block.receipts.map(({ result }) =>
          result.expectOk().expectAscii("sBTC")
        );
      },
    ));
  },
});

Clarinet.test({
  name: "asset.clar: Can get decimals",
  fn(chain: Chain, accounts: Map<string, Account>) {
    fc.assert(fc.property(
      fc.record({
        sender: fc.constantFrom(...accounts.values()),
      }),
      (r: { sender: Account }) => {
        // Arrange
        const sender = r.sender;

        // Act
        const block = chain.mineBlock([
          Tx.contractCall(
            "asset",
            "get-decimals",
            [],
            sender.address,
          ),
        ]);

        // Assert
        block.receipts.map(({ result }) => result.expectOk().expectUint(8));
      },
    ));
  },
});

import { AssetCommands } from "./asset_Commands.ts";

Clarinet.test({
  name: "asset.clar: Invariant tests",
  fn(chain: Chain, accounts: Map<string, Account>) {
    const initialChain = { chain: chain };
    const initialModel = {
      wallets: new Map<string, number>(),
    };
    fc.assert(
      fc.property(
        AssetCommands(accounts),
        (cmds: []) => {
          const initialState = () => ({
            model: initialModel,
            real : initialChain,
          });
          fc.modelRun(initialState, cmds);
        },
      ),
      { numRuns: 1, verbose: true },
    );
  },
});
