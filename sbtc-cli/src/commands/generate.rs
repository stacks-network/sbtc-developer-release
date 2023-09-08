use std::{collections::BTreeMap, io::stdout};

use bdk::{bitcoin::Address as BitcoinAddress, keys::bip39::Mnemonic};
use clap::Parser;
use serde::Serialize;
use stacks_core::{
    address::StacksAddress,
    wallet::{Credentials, Wallet},
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
            let description: CredentialsDescription = credentials.into();
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
            seed_private_key: wallet.private_key(),
            credentials: (0..credentials_count)
                .map(|i| (i as u32, wallet.credentials(i as u32).unwrap().into()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct CredentialsDescription {
    private_key: PrivateKey,
    public_key: PublicKey,
    stacks_address: StacksAddress,
    bitcoin_p2pkh_address: BitcoinAddress,
    bitcoin_p2wpkh_address: BitcoinAddress,
    bitcoin_taproot_address: BitcoinAddress,
}

/*
Other way around is a different process and is not needed, so we don't implement
it
*/
#[allow(clippy::from_over_into)]
impl Into<CredentialsDescription> for Credentials {
    fn into(self) -> CredentialsDescription {
        CredentialsDescription {
            private_key: self.private_key(),
            public_key: self.public_key(),
            stacks_address: self.address(),
            bitcoin_p2pkh_address: self.bitcoin_p2pkh_address(),
            bitcoin_p2wpkh_address: self.bitcoin_p2wpkh_address(),
            bitcoin_taproot_address: self.bitcoin_taproot_address(),
        }
    }
}
