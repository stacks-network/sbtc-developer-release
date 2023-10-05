import { Tx, Chain, Account, types } from 'https://deno.land/x/clarinet@v1.7.0/index.ts';

export function bootstrap(chain: Chain, deployer: Account) {
	const { receipts } = chain.mineBlock([
		// Set the asset contract owner to the asset_test contract
		Tx.contractCall(
			`${deployer.address}.asset`,
			'set-contract-owner',
			[types.principal(`${deployer.address}.asset_test`)],
			deployer.address
		)
	]);
	receipts[0].result.expectOk();
}
