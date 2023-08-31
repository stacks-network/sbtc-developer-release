
export const warningText = `// Code generated using \`clarinet run ./scripts/tests.ts\`
// Manual edits will be lost.`;


export const defaultDeps = `import { Clarinet, Tx, Chain, Account, Block, types } from 'https://deno.land/x/clarinet@v1.7.0/index.ts';
import { assertEquals } from 'https://deno.land/std@0.170.0/testing/asserts.ts';

export { Clarinet, Tx, Chain, types, assertEquals };
export type { Account };

const dirOptions = { strAbbreviateSize: Infinity, depth: Infinity, colors: true };

export function printEvents(block: Block) {
	block.receipts.map(({ events }) => events && events.map(event => console.log(Deno.inspect(event, dirOptions))));
}
`;

// generates ts code for a module with a bootstrap function
// that can be optionally defined at the provided path.
export async function generateBootstrapFile(bootstrapFile?: string) {
	let bootstrapSource = 'export function bootstrap(){}';
	if (bootstrapFile) {
		try {
			bootstrapSource = await Deno.readTextFile(bootstrapFile);
		}
		catch (error) {
			console.error(`Could not read bootstrap file ${bootstrapFile}`, error);
		}
	}
	return `${warningText}\n\n${bootstrapSource}`;
}