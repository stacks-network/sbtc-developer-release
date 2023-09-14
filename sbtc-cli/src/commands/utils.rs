use std::collections::{BTreeMap, HashMap};

use bdk::{
    bitcoin::{
        blockdata::{opcodes::all::OP_RETURN, script::Builder},
        Network, PrivateKey, Script, TxOut,
    },
    blockchain::{
        rpc::{Auth::UserPass, RpcSyncParams},
        AnyBlockchain, AnyBlockchainConfig, ConfigurableBlockchain, ElectrumBlockchain,
        ElectrumBlockchainConfig, RpcConfig,
    },
    database::MemoryDatabase,
    electrum_client::Client,
    template::P2Wpkh,
    SyncOptions, Wallet,
};

use serde::Serialize;

use crate::config::Config;

pub fn init_blockstream_blockchain() -> anyhow::Result<ElectrumBlockchain> {
    let client = Client::new("ssl://blockstream.info:993")?;
    let blockchain = ElectrumBlockchain::from(client);
    Ok(blockchain)
}

pub fn setup_wallet(private_key: PrivateKey) -> anyhow::Result<Wallet<MemoryDatabase>> {
    let blockchain = init_blockstream_blockchain()?;
    let wallet = Wallet::new(
        P2Wpkh(private_key),
        Some(P2Wpkh(private_key)),
        private_key.network,
        MemoryDatabase::default(),
    )?;

    wallet.sync(&blockchain, SyncOptions::default())?;

    Ok(wallet)
}

pub fn blockchain_config_from_config(config: &Config) -> AnyBlockchainConfig {
    match config.bitcoin_node_url.scheme() {
        "electrum" => {
            let url = config
                .bitcoin_node_url
                .as_str()
                .to_string()
                .replace("electrum", "https");

            AnyBlockchainConfig::Electrum(ElectrumBlockchainConfig {
                url,
                socks5: None,
                retry: 3,
                timeout: Some(10),
                stop_gap: 10,
                validate_domain: true,
            })
        }
        "rpc" => {
            let mut url = config.bitcoin_node_url.clone();

            url.set_username("").unwrap();
            url.set_password(None).unwrap();

            let mut url = url.as_str().to_string().replace("rpc", "http");

            url.push('/');

            AnyBlockchainConfig::Rpc(RpcConfig {
                url,
                auth: UserPass {
                    username: config.bitcoin_node_url.username().to_string(),
                    password: config
                        .bitcoin_node_url
                        .password()
                        .unwrap_or_default()
                        .to_string(),
                },
                network: config.bitcoin_network,
                wallet_name: "sbtc123".to_string(),
                sync_params: Some(RpcSyncParams {
                    start_script_count: 0,
                    start_time: 0,
                    force_start_time: false,
                    poll_rate_sec: 1,
                }),
            })
        }
        scheme => panic!("Unknown bitcoin node url scheme: {}", scheme),
    }
}

pub fn setup_wallet_from_config(
    config: &Config,
    private_key: PrivateKey,
) -> anyhow::Result<Wallet<MemoryDatabase>> {
    let blockchain_config = blockchain_config_from_config(config);
    dbg!(&blockchain_config);
    let blockchain = AnyBlockchain::from_config(&blockchain_config).unwrap();

    let wallet = Wallet::new(
        P2Wpkh(private_key),
        Some(P2Wpkh(private_key)),
        config.bitcoin_network,
        MemoryDatabase::default(),
    )?;

    wallet.sync(&blockchain, SyncOptions::default())?;

    Ok(wallet)
}

pub fn build_op_return_script(data: &[u8]) -> Script {
    Builder::new()
        .push_opcode(OP_RETURN)
        .push_slice(data)
        .into_script()
}

pub fn reorder_outputs(
    outputs: impl IntoIterator<Item = TxOut>,
    order: impl IntoIterator<Item = (Script, u64)>,
) -> Vec<TxOut> {
    let indices: HashMap<(Script, u64), usize> = order
        .into_iter()
        .enumerate()
        .map(|(idx, val)| (val, idx))
        .collect();

    let outputs_ordered: BTreeMap<usize, TxOut> = outputs
        .into_iter()
        .map(|txout| {
            (
                *indices
                    .get(&(txout.script_pubkey.clone(), txout.value))
                    .unwrap_or(&usize::MAX), // Change amount
                txout,
            )
        })
        .collect();

    outputs_ordered.into_values().collect()
}

pub fn magic_bytes(network: &Network) -> [u8; 2] {
    match network {
        Network::Bitcoin => [b'X', b'2'],
        Network::Testnet => [b'T', b'2'],
        _ => [b'i', b'd'],
    }
}

#[derive(Serialize)]
pub struct TransactionData {
    pub tx_id: String,
    pub tx_hex: String,
}
