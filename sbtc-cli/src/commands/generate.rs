use std::{collections::BTreeMap, io::stdout};

use bdk::{
    bitcoin::{Address as BitcoinAddress, Network as BitcoinNetwork},
    keys::bip39::Mnemonic,
};
use clap::Parser;
use serde::Serialize;
use stacks_core::{
    address::StacksAddress,
    wallet::{BitcoinCredentials, Credentials, Wallet},
    Network, PrivateKey, PublicKey,
};

#[derive(Parser, Debug, Clone)]
pub struct GenerateArgs {
    /// Specify how to generate the credentials
    #[command(subcommand)]
    subcommand: GenerateSubcommand,
    /// The network to broadcast to
    #[clap(short, long, default_value_t = Network::Testnet)]
    network: Network,
}

#[derive(clap::Subcommand, Debug, Clone)]
enum GenerateSubcommand {
    New,
    Mnemonic { mnemonic: String },
    PrivateKeyHex { private_key: String },
}

pub fn generate(generate_args: &GenerateArgs) -> anyhow::Result<()> {
    match &generate_args.subcommand {
        GenerateSubcommand::New => {
            let wallet = Wallet::random(generate_args.network)?;

            let description = WalletDescription::from_wallet(&wallet, 10);
            serde_json::to_writer_pretty(stdout(), &description)?;
        }
        GenerateSubcommand::Mnemonic { mnemonic } => {
            let wallet = Wallet::new(generate_args.network, mnemonic)?;

            let description = WalletDescription::from_wallet(&wallet, 10);
            serde_json::to_writer_pretty(stdout(), &description)?;
        }
        GenerateSubcommand::PrivateKeyHex { private_key } => {
            let bytes = hex::decode(private_key)?;
            let pk = PrivateKey::from_slice(&bytes)?;

            let credentials = Credentials::new(generate_args.network, pk);

            let bitcoin_network = match generate_args.network {
                Network::Mainnet => BitcoinNetwork::Bitcoin,
                Network::Testnet => BitcoinNetwork::Testnet,
            };
            let bitcoin_credentials = BitcoinCredentials::new(bitcoin_network, pk);

            let description = describe_credentials(credentials, bitcoin_credentials);
            serde_json::to_writer_pretty(stdout(), &description)?;
        }
    };

    Ok(())
}

#[derive(Debug, Clone, Serialize)]
struct WalletDescription {
    mnemonic: Mnemonic,
    seed_private_key: PrivateKey,
    credentials: BTreeMap<u32, CredentialsDescription>,
}

impl WalletDescription {
    fn from_wallet(wallet: &Wallet, credentials_count: usize) -> Self {
        Self {
            mnemonic: wallet.mnemonic(),
            seed_private_key: wallet.master_key(),
            credentials: (0..credentials_count)
                .map(|i| {
                    let credentials = wallet.credentials(i as u32).unwrap();
                    let bitcoin_credentials = wallet.bitcoin_credentials(i as u32).unwrap();

                    (
                        i as u32,
                        describe_credentials(credentials, bitcoin_credentials),
                    )
                })
                .collect(),
        }
    }
}

fn describe_credentials(
    credentials: Credentials,
    bitcoin_credentials: BitcoinCredentials,
) -> CredentialsDescription {
    let stacks_private_key = credentials.private_key();
    let stacks_public_key = credentials.public_key();
    let stacks_address = credentials.address();

    let bitcoin_private_key = bitcoin_credentials.private_key();
    let bitcoin_public_key = bitcoin_credentials.public_key();
    let bitcoin_p2pkh_address = bitcoin_credentials.p2pkh_address();
    let bitcoin_p2wpkh_address = bitcoin_credentials.p2wpkh_address();
    let bitcoin_taproot_address = bitcoin_credentials.taproot_address();

    CredentialsDescription {
        stacks_private_key,
        stacks_public_key,
        bitcoin_private_key,
        bitcoin_public_key,
        stacks_address,
        bitcoin_p2pkh_address,
        bitcoin_p2wpkh_address,
        bitcoin_taproot_address,
    }
}

#[derive(Debug, Clone, Serialize)]
struct CredentialsDescription {
    stacks_private_key: PrivateKey,
    stacks_public_key: PublicKey,
    bitcoin_private_key: PrivateKey,
    bitcoin_public_key: PublicKey,
    stacks_address: StacksAddress,
    bitcoin_p2pkh_address: BitcoinAddress,
    bitcoin_p2wpkh_address: BitcoinAddress,
    bitcoin_taproot_address: BitcoinAddress,
}
