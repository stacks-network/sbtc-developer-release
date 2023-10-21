import { it, expect } from "vitest";
import fc from "fast-check";

import { Cl } from "@stacks/transactions";

it("asset.clar: Can get token URI", () => {
  fc.assert(fc.property(
    fc.record({
      sender: fc.constantFrom(...simnet.getAccounts().values()),
    }),
    (r: { sender: string }) => {
      // Arrange
      const sender = r.sender;

      // Act
      const { result } = simnet.callReadOnlyFn(
        "asset",
        "get-token-uri",
        [],
        sender,
      );

      // Assert
      expect(result).toBeOk(
        Cl.some(
          Cl.stringUtf8(
            "https://gateway.pinata.cloud/ipfs/Qma5P7LFGQAXt7gzkNZGxet5qJcVxgeXsenDXwu9y45hpr?_gl=1*1mxodt*_ga*OTU1OTQzMjE2LjE2OTQwMzk2MjM.*_ga_5RMPXG14TE*MTY5NDA4MzA3OC40LjEuMTY5NDA4MzQzOC42MC4wLjA",
          ),
        ),
      );
    },
  ));
});

it("asset.clar: Can get symbol", () => {
  fc.assert(fc.property(
    fc.record({
      sender: fc.constantFrom(...simnet.getAccounts().values()),
    }),
    (r: { sender: string }) => {
      // Arrange
      const sender = r.sender;

      // Act
      const { result } = simnet.callReadOnlyFn(
        "asset",
        "get-symbol",
        [],
        sender,
      );

      // Assert
      expect(result).toBeOk(Cl.stringAscii("sBTC"));
    },
  ));
});

it("asset.clar: Can get name", () => {
  fc.assert(fc.property(
    fc.record({
      sender: fc.constantFrom(...simnet.getAccounts().values()),
    }),
    (r: { sender: string }) => {
      // Arrange
      const sender = r.sender;

      // Act
      const { result } = simnet.callReadOnlyFn(
        "asset",
        "get-name",
        [],
        sender,
      );

      // Assert
      expect(result).toBeOk(Cl.stringAscii("sBTC"));
    },
  ));
});

it("asset.clar: Can get decimals", () => {
  fc.assert(fc.property(
    fc.record({
      sender: fc.constantFrom(...simnet.getAccounts().values()),
    }),
    (r: { sender: string }) => {
      // Arrange
      const sender = r.sender;

      // Act
      const { result } = simnet.callReadOnlyFn(
        "asset",
        "get-decimals",
        [],
        sender,
      );

      // Assert
      expect(result).toBeOk(Cl.uint(8));
    },
  ));
});

import { initSimnet } from "@hirosystems/clarinet-sdk";
import { AssetCommands } from "./asset_Commands.ts";

it("asset.clar: Invariant tests", async () => {
  const initialChain = { simnet: await initSimnet() };
  const initialModel = {
    wallets: new Map<string, number>(),
    transactions: [],
  };
  const accounts = simnet.getAccounts();

  fc.assert(
    fc.property(
      // @ts-ignore
      AssetCommands(accounts),
      (cmds: []) => {
        const initialState = () => ({
          model: initialModel,
          real : initialChain,
        });
        fc.modelRun(initialState, cmds);
      },
    ),
    { numRuns: 10, verbose: true },
  );
});
