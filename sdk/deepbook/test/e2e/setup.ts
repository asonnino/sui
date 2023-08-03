// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { expect } from 'vitest';
import { execSync } from 'child_process';
import tmp from 'tmp';
import { Ed25519Keypair } from '@mysten/sui.js/keypairs/ed25519';
import { retry } from 'ts-retry-promise';
import { FaucetRateLimitError, getFaucetHost, requestSuiFromFaucetV0 } from '@mysten/sui.js/faucet';
import { SuiClient, SuiObjectChangePublished, getFullnodeUrl } from '@mysten/sui.js/client';
import { TransactionBlock } from '@mysten/sui.js/transactions';
import { PoolSummary } from '../../src/types/pool';
import { normalizeSuiObjectId, SUI_FRAMEWORK_ADDRESS } from '@mysten/sui.js/utils';
import { createPool } from '../../src';

const DEFAULT_FAUCET_URL = import.meta.env.VITE_FAUCET_URL ?? getFaucetHost('localnet');
const DEFAULT_FULLNODE_URL = import.meta.env.VITE_FULLNODE_URL ?? getFullnodeUrl('localnet');
const SUI_BIN = import.meta.env.VITE_SUI_BIN ?? 'cargo run --bin sui';
const DEFAULT_TICK_SIZE = 10000000;
const DEFAULT_LOT_SIZE = 10000;

export const DEFAULT_RECIPIENT =
	'0x0c567ffdf8162cb6d51af74be0199443b92e823d4ba6ced24de5c6c463797d46';
export const DEFAULT_RECIPIENT_2 =
	'0xbb967ddbebfee8c40d8fdd2c24cb02452834cd3a7061d18564448f900eb9e66d';
export const DEFAULT_GAS_BUDGET = 10000000;
export const DEFAULT_SEND_AMOUNT = 1000;

export class TestToolbox {
	keypair: Ed25519Keypair;
	client: SuiClient;

	constructor(keypair: Ed25519Keypair, client: SuiClient) {
		this.keypair = keypair;
		this.client = client;
	}

	address() {
		return this.keypair.getPublicKey().toSuiAddress();
	}

	public async getActiveValidators() {
		return (await this.client.getLatestSuiSystemState()).activeValidators;
	}
}

export function getClient(): SuiClient {
	return new SuiClient({
		url: DEFAULT_FULLNODE_URL,
	});
}

// TODO: expose these testing utils from @mysten/sui.js
export async function setupSuiClient() {
	const keypair = Ed25519Keypair.generate();
	const address = keypair.getPublicKey().toSuiAddress();
	const client = getClient();
	await retry(() => requestSuiFromFaucetV0({ host: DEFAULT_FAUCET_URL, recipient: address }), {
		backoff: 'EXPONENTIAL',
		// overall timeout in 60 seconds
		timeout: 1000 * 60,
		// skip retry if we hit the rate-limit error
		retryIf: (error: any) => !(error instanceof FaucetRateLimitError),
		logger: (msg) => console.warn('Retrying requesting from faucet: ' + msg),
	});
	return new TestToolbox(keypair, client);
}

export async function publishPackage(packagePath: string, toolbox?: TestToolbox) {
	// TODO: We create a unique publish address per publish, but we really could share one for all publishes.
	if (!toolbox) {
		toolbox = await setupSuiClient();
	}

	// remove all controlled temporary objects on process exit
	tmp.setGracefulCleanup();

	const tmpobj = tmp.dirSync({ unsafeCleanup: true });

	const { modules, dependencies } = JSON.parse(
		execSync(
			`${SUI_BIN} move build --dump-bytecode-as-base64 --path ${packagePath} --install-dir ${tmpobj.name}`,
			{ encoding: 'utf-8' },
		),
	);
	const tx = new TransactionBlock();
	const cap = tx.publish({
		modules,
		dependencies,
	});

	// Transfer the upgrade capability to the sender so they can upgrade the package later if they want.
	tx.transferObjects([cap], tx.pure(await toolbox.address()));

	const publishTxn = await toolbox.client.signAndExecuteTransactionBlock({
		transactionBlock: tx,
		signer: toolbox.keypair,
		options: {
			showEffects: true,
			showObjectChanges: true,
		},
	});
	expect(publishTxn.effects?.status.status).toEqual('success');

	const packageId = ((publishTxn.objectChanges?.filter(
		(a) => a.type === 'published',
	) as SuiObjectChangePublished[]) ?? [])[0].packageId.replace(/^(0x)(0+)/, '0x') as string;

	expect(packageId).toBeTypeOf('string');

	console.info(`Published package ${packageId} from address ${toolbox.address()}}`);

	return { packageId, publishTxn };
}

export async function setupPool(toolbox: TestToolbox): Promise<PoolSummary> {
	const packagePath = __dirname + '/./data/test_coin';
	const { packageId } = await publishPackage(packagePath, toolbox);
	const baseAsset = `${normalizeSuiObjectId(SUI_FRAMEWORK_ADDRESS)}::sui::SUI`;
	const quoteAsset = `${packageId}::test::TEST`;
	const txb = createPool(baseAsset, quoteAsset, DEFAULT_TICK_SIZE, DEFAULT_LOT_SIZE);
	const resp = await toolbox.client.signAndExecuteTransactionBlock({
		signer: toolbox.keypair,
		transactionBlock: txb,
		options: {
			showEffects: true,
			showEvents: true,
		},
	});
	expect(resp.effects?.status.status).toEqual('success');

	const event = resp.events?.find((e) => e.type.includes('PoolCreated')) as any;
	return {
		poolId: event.parsedJson.pool_id,
		baseAsset,
		quoteAsset,
	};
}
