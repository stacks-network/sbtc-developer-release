/*!
Exposes tools to create and manage Stacks credentials.
*/

use std::str::FromStr;

use bdk::bitcoin::secp256k1::Secp256k1;
use bdk::bitcoin::util::bip32::{ChildNumber, DerivationPath, ExtendedPrivKey};
use bdk::bitcoin::{Address as BitcoinAddress, Network as BitcoinNetwork};
use bdk::keys::bip39::Mnemonic;
use rand::random;
use serde::{Deserialize, Serialize};

use crate::address::{AddressVersion, StacksAddress};
use crate::{Network, PrivateKey, PublicKey, StacksResult};

/// Derivation path used by the Stacks ecosystem to derive keys
pub const STACKS_DERIVATION_PATH: &str = "m/44'/5757'/0'/0";

/// Wallet of credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    network: Network,
    seed_private_key: ExtendedPrivKey,
    mnemonic: Mnemonic,
}

impl Wallet {
    /// Creates a wallet from the network, mnemonic, and optional passphrase
    pub fn new(network: Network, mnemonic: impl AsRef<str>) -> StacksResult<Self> {
        let mnemonic = Mnemonic::from_str(mnemonic.as_ref())?;

        let bitcoin_network = match network {
            Network::Mainnet => bdk::bitcoin::Network::Bitcoin,
            Network::Testnet => bdk::bitcoin::Network::Testnet,
        };

        let extended_master_key =
            ExtendedPrivKey::new_master(bitcoin_network, &mnemonic.to_seed(""))?;

        let seed_private_key = extended_master_key
            .derive_priv(
                &Secp256k1::new(),
                &DerivationPath::from_str(STACKS_DERIVATION_PATH).unwrap(),
            )
            .unwrap();

        Ok(Self {
            network,
            seed_private_key,
            mnemonic,
        })
    }

    /// Creates a random wallet
    pub fn random(network: Network) -> StacksResult<Self> {
        let entropy: [u8; 32] = random();
        let mnemonic = Mnemonic::from_entropy(&entropy)?;

        Self::new(network, mnemonic.to_string())
    }

    /// Returns the mnemonic of the wallet
    pub fn mnemonic(&self) -> Mnemonic {
        self.mnemonic.clone()
    }

    /// Returns the seed private key of the wallet
    pub fn private_key(&self) -> PrivateKey {
        self.seed_private_key.to_priv().inner
    }

    /// Returns the credentials at the given index
    pub fn credentials(&self, index: u32) -> StacksResult<Credentials> {
        let key = self
            .seed_private_key
            .ckd_priv(&Secp256k1::new(), ChildNumber::Normal { index })?
            .to_priv()
            .inner;

        Ok(Credentials::new(self.network(), key))
    }

    /// Returns the network of the wallet
    pub fn network(&self) -> Network {
        self.network
    }
}

/// Credentials that can be used to sign transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    network: Network,
    private_key: PrivateKey,
}

impl Credentials {
    /// Creates credentials from the network and private key
    pub fn new(network: Network, key: PrivateKey) -> Self {
        Self {
            network,
            private_key: key,
        }
    }

    /// Creates random credentials
    pub fn random(network: Network) -> Self {
        let key = Secp256k1::new().generate_keypair(&mut rand::thread_rng()).0;

        Self::new(network, key)
    }

    /// Returns the Stacks network
    pub fn network(&self) -> Network {
        self.network
    }

    /// Returns the Bitcoin network
    pub fn bitcoin_network(&self) -> BitcoinNetwork {
        match self.network {
            Network::Mainnet => BitcoinNetwork::Bitcoin,
            Network::Testnet => BitcoinNetwork::Testnet,
        }
    }

    /// Returns the private key
    pub fn private_key(&self) -> PrivateKey {
        self.private_key
    }

    /// Returns the public key
    pub fn public_key(&self) -> PublicKey {
        self.private_key.public_key(&Secp256k1::new())
    }

    /// Returns the Stacks P2PKH address
    pub fn address(&self) -> StacksAddress {
        let version = match self.network {
            Network::Mainnet => AddressVersion::MainnetSingleSig,
            Network::Testnet => AddressVersion::TestnetSingleSig,
        };

        StacksAddress::p2pkh(version, &self.public_key())
    }

    /// Returns the Bitcoin P2PKH address
    pub fn bitcoin_p2pkh_address(&self) -> BitcoinAddress {
        BitcoinAddress::p2pkh(
            &bdk::bitcoin::PublicKey::new(self.public_key()),
            self.bitcoin_network(),
        )
    }

    /// Returns the Bitcoin P2WPKH address
    pub fn bitcoin_p2wpkh_address(&self) -> BitcoinAddress {
        BitcoinAddress::p2wpkh(
            &bdk::bitcoin::PublicKey::new(self.public_key()),
            self.bitcoin_network(),
        )
        .unwrap()
    }

    /// Returns the Bitcoin taproot address
    pub fn bitcoin_taproot_address(&self) -> BitcoinAddress {
        BitcoinAddress::p2tr(
            &Secp256k1::new(),
            self.public_key().x_only_public_key().0,
            None,
            self.bitcoin_network(),
        )
    }
}
