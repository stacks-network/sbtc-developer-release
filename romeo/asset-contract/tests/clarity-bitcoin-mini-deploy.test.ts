import { beforeEach, describe, expect, it } from "vitest";

beforeEach(async () => {
  await vm.initSession(process.cwd(), "./Clarinet.toml");
});

describe("test deploy version and unit test version of clarity-bitcoin-mini", () => {
  it("ensures the versions differ only in DEBUG flag", () => {
    const sourceForUnitTests = vm.getContractSource("clarity-bitcoin-mini")!;
    const sourceForDeploy = vm.getContractSource(
      "clarity-bitcoin-mini-deploy"
    )!;
    expect(sourceForDeploy).not.toEqual(sourceForUnitTests);
    expect(sourceForDeploy).toEqual(
      sourceForUnitTests.replace(
        "(define-constant DEBUG-MODE true)",
        "(define-constant DEBUG-MODE false)"
      )
    );
  });
});
