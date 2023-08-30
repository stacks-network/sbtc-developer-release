use std::io::stdout;

use anyhow::{anyhow, Context};
use array_bytes::bytes2hex;
use bdk::{
    bitcoin::{
        schnorr::TweakedPublicKey,
        secp256k1::{rand::random, Secp256k1},
        Address as BitcoinAddress, Network, PrivateKey,
    },
    keys::{bip39::Mnemonic, DerivableKey, ExtendedKey},
    miniscript::BareCtx,
};
use clap::Parser;
use stacks_core::{
    address::{AddressVersion, StacksAddress},
    crypto::{hash160::Hash160Hasher, Hashing},
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
    Wif { wif: String },
    PrivateKeyHex { private_key: String },
    Mnemonic { mnemonic: String },
}

#[derive(serde::Serialize, Debug, Clone)]
struct Credentials {
    mnemonic: String,
    wif: String,
    private_key: String,
    public_key: String,
    stacks_address: String,
    bitcoin_taproot_address_tweaked: String,
    bitcoin_taproot_address_untweaked: String,
    bitcoin_p2pkh_address: String,
}

pub fn generate(generate_args: &GenerateArgs) -> anyhow::Result<()> {
    let (private_key, maybe_mnemonic) = match &generate_args.subcommand {
        GenerateSubcommand::New => {
            let mnemonic = random_mnemonic()?;
            (
                private_key_from_mnemonic(generate_args.network, mnemonic.clone())?,
                Some(mnemonic),
            )
        }
        GenerateSubcommand::Wif { wif } => (private_key_from_wif(wif)?, None),
        GenerateSubcommand::PrivateKeyHex { private_key } => (
            parse_private_key_from_hex(private_key, generate_args.network)?,
            None,
        ),
        GenerateSubcommand::Mnemonic { mnemonic } => {
            let mnemonic = Mnemonic::parse(mnemonic)?;
            (
                private_key_from_mnemonic(generate_args.network, mnemonic.clone())?,
                Some(mnemonic),
            )
        }
    };

    let credentials = generate_credentials(&private_key, maybe_mnemonic)?;

    serde_json::to_writer_pretty(stdout(), &credentials)?;

    Ok(())
}

fn random_mnemonic() -> anyhow::Result<Mnemonic> {
    let entropy: Vec<u8> = std::iter::from_fn(|| Some(random())).take(32).collect();
    Mnemonic::from_entropy(&entropy).context("Could not create mnemonic from entropy")
}

fn private_key_from_wif(wif: &str) -> anyhow::Result<PrivateKey> {
    Ok(PrivateKey::from_wif(wif)?)
}

fn parse_private_key_from_hex(private_key: &str, network: Network) -> anyhow::Result<PrivateKey> {
    let slice = array_bytes::hex2bytes(private_key)
        .map_err(|_| anyhow::anyhow!("Failed to parse hex string: {}", private_key,))?;
    Ok(PrivateKey::from_slice(&slice, network)?)
}

fn private_key_from_mnemonic(network: Network, mnemonic: Mnemonic) -> anyhow::Result<PrivateKey> {
    let extended_key: ExtendedKey<BareCtx> = mnemonic.into_extended_key()?;
    let private_key = extended_key
        .into_xprv(network)
        .ok_or(anyhow!("Could not create an extended private key"))?;

    Ok(private_key.to_priv())
}

fn generate_credentials(
    private_key: &PrivateKey,
    maybe_mnemonic: Option<Mnemonic>,
) -> anyhow::Result<Credentials> {
    let secp = Secp256k1::new();
    let public_key = private_key.public_key(&secp);

    let stacks_address_version = match private_key.network {
        Network::Testnet => AddressVersion::TestnetSingleSig,
        Network::Bitcoin => AddressVersion::MainnetSingleSig,
        _ => panic!("Not supported"),
    };
    let public_key_hash = Hash160Hasher::from_bytes(&public_key.pubkey_hash().as_hash())?;
    let stacks_address = StacksAddress::new(stacks_address_version, public_key_hash);
    let bitcoin_taproot_address_tweaked =
        BitcoinAddress::p2tr(&secp, public_key.inner.into(), None, private_key.network).to_string();

    let bitcoin_taproot_address_untweaked = BitcoinAddress::p2tr_tweaked(
        TweakedPublicKey::dangerous_assume_tweaked(public_key.inner.into()),
        private_key.network,
    )
    .to_string();

    Ok(Credentials {
        mnemonic: maybe_mnemonic
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default(),
        wif: private_key.to_wif(),
        private_key: bytes2hex("0x", private_key.to_bytes()),
        public_key: bytes2hex("0x", public_key.to_bytes()),
        stacks_address: stacks_address.to_string(),
        bitcoin_taproot_address_tweaked,
        bitcoin_taproot_address_untweaked,
        bitcoin_p2pkh_address: BitcoinAddress::p2pkh(&public_key, private_key.network).to_string(),
    })
}
