import { Clarinet, Contract, Account } from 'https://deno.land/x/clarinet@v1.7.1/index.ts';
import { extractTestAnnotations, getContractName } from './utils/clarity-parser.ts';
import { defaultDeps, generateBootstrapFile, warningText } from './utils/generate.ts';

const sourcebootstrapFile = './tests/bootstrap.ts';
const targetFolder = '.test';

function isTestContract(contractName: string) {
	return contractName.substring(contractName.length - 5) === "_test" &&
		contractName.substring(contractName.length - 10) !== "_flow_test";
}

Clarinet.run({
	async fn(accounts: Map<string, Account>, contracts: Map<string, Contract>) {
		Deno.writeTextFile(`${targetFolder}/deps.ts`, defaultDeps);
		Deno.writeTextFile(`${targetFolder}/bootstrap.ts`, await generateBootstrapFile(sourcebootstrapFile));

		for (const [contractId, contract] of contracts) {
			console.log(contractId);
			const contractName = getContractName(contractId);
			if (!isTestContract(contractName))
				continue;

			const hasDefaultPrepareFunction = contract.contract_interface.functions.reduce(
				(a, v) => a || (v.name === 'prepare' && v.access === 'public' && v.args.length === 0),
				false);
			const annotations = extractTestAnnotations(contract.source);

			const code: string[][] = [];
			code.push([
				warningText,
				``,
				`import { Clarinet, Tx, Chain, Account, types, assertEquals, printEvents } from './deps.ts';`,
				`import { bootstrap } from './bootstrap.ts';`,
				``
			]);

			for (const { name, access, args } of contract.contract_interface.functions.reverse()) {
				if (access !== 'public' || name.substring(0, 5) !== 'test-')
					continue;
				if (args.length > 0)
					throw new Error(`Test functions cannot take arguments. (Offending function: ${name})`);
				const functionAnnotations = annotations[name] || {};
				if (hasDefaultPrepareFunction && !functionAnnotations.prepare)
					functionAnnotations.prepare = 'prepare';
				if (functionAnnotations['no-prepare'])
					delete functionAnnotations.prepare;
				code.push([generateTest(contractId, name, functionAnnotations)]);
			}

			Deno.writeTextFile(`${targetFolder}/${contractName}.ts`, code.flat().join("\n"));
		}
	}
});

type FunctionAnnotations = { [key: string]: string | boolean };

// generates contract call ts code for prepare function in mineBlock
function generatePrepareTx(contractPrincipal: string, annotations: FunctionAnnotations) {
	return `Tx.contractCall('${contractPrincipal}', '${annotations['prepare']}', [], deployer.address)`;
}

/**
 * generates a mineBlock ts code containing optional prepare function
 * and the test function call
 */
function generateNormalMineBlock(contractPrincipal: string, testFunction: string, annotations: FunctionAnnotations) {
	return `let block = chain.mineBlock([
		${annotations['prepare'] ? `${generatePrepareTx(contractPrincipal, annotations)},` : ''}
		Tx.contractCall('${contractPrincipal}', '${testFunction}', [], callerAddress)
	]);`;
}

/**
 * Generates a mineBlock ts code containing
 * - optional block with prepare function,
 * - several empty blocks and
 * - the test function call
 *
 * supports the `@print events` annotations
 */
function generateSpecialMineBlock(mineBlocksBefore: number, contractPrincipal: string, testFunction: string, annotations: FunctionAnnotations) {
	let code = ``;
	if (annotations['prepare']) {
		code = `let prepareBlock = chain.mineBlock([${generatePrepareTx(contractPrincipal, annotations)}]);
		prepareBlock.receipts.map(({result}) => result.expectOk());
		`;
		if (annotations['print'] === 'events')
			code += `\n\t\tprintEvents(prepareBlock);\n`;
	}
	if (mineBlocksBefore > 1)
		code += `
		chain.mineEmptyBlock(${mineBlocksBefore - 1});`;
	return `${code}
		let block = chain.mineBlock([Tx.contractCall('${contractPrincipal}', '${testFunction}', [], callerAddress)]);
		${annotations['print'] === 'events' ? 'printEvents(block);' : ''}`;
}

/**
 * Generates the ts code for a unit test
 * @param contractPrincipal
 * @param testFunction
 * @param annotations
 * @returns
 */
function generateTest(contractPrincipal: string, testFunction: string, annotations: FunctionAnnotations) {
	const mineBlocksBefore = parseInt(annotations['mine-blocks-before'] as string) || 0;
	return `Clarinet.test({
	name: "${annotations.name ? testFunction + ': ' + (annotations.name as string).replace(/"/g, '\\"') : testFunction}",
	async fn(chain: Chain, accounts: Map<string, Account>) {
		const deployer = accounts.get("deployer")!;
		bootstrap && bootstrap(chain, deployer);
		const callerAddress = ${annotations.caller ? (annotations.caller[0] === "'" ? `"${(annotations.caller as string).substring(1)}"` : `accounts.get('${annotations.caller}')!.address`) : `accounts.get('deployer')!.address`};
		${mineBlocksBefore >= 1
			? generateSpecialMineBlock(mineBlocksBefore, contractPrincipal, testFunction, annotations)
			: generateNormalMineBlock(contractPrincipal, testFunction, annotations)}
		block.receipts.map(({result}) => result.expectOk());
	}
});
`;
}
