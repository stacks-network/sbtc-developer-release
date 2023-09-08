/*!
Exposes tools to create and manage Stacks credentials.
*/

use std::str::FromStr;

use bdk::bitcoin::secp256k1::Secp256k1;
use bdk::bitcoin::util::bip32::{ChildNumber, DerivationPath, ExtendedPrivKey};
use bdk::keys::bip39::Mnemonic;

use crate::address::{AddressVersion, StacksAddress};
use crate::{Network, PrivateKey, PublicKey, StacksResult};

/// Derivation path used by the Stacks ecosystem to derive keys
pub const STACKS_DERIVATION_PATH: &str = "m/44'/5757'/0'/0";

/// Wallet of credentials
pub struct Wallet {
    network: Network,
    seed_key: ExtendedPrivKey,
}

impl Wallet {
    /// Creates a wallet from the network, mnemonic, and optional passphrase
    pub fn new(
        network: Network,
        mnemonic: impl AsRef<str>,
        passphrase: Option<impl AsRef<str>>,
    ) -> StacksResult<Self> {
        let mnemonic = mnemonic.as_ref();
        let passphrase = passphrase
            .map(|p| p.as_ref().to_string())
            .unwrap_or_default();

        let master_seed = Mnemonic::from_str(mnemonic)?.to_seed(passphrase);

        let bitcoin_network = match network {
            Network::Mainnet => bdk::bitcoin::Network::Bitcoin,
            Network::Testnet => bdk::bitcoin::Network::Testnet,
        };

        let master_key = ExtendedPrivKey::new_master(bitcoin_network, &master_seed)?;

        let key = master_key
            .derive_priv(
                &mut Secp256k1::new(),
                &DerivationPath::from_str(STACKS_DERIVATION_PATH).unwrap(),
            )
            .unwrap();

        Ok(Self {
            network,
            seed_key: key,
        })
    }

    /// Returns the credentials at the given index
    pub fn credentials(&self, index: u32) -> StacksResult<Credentials> {
        let key = self
            .seed_key
            .ckd_priv(&mut Secp256k1::new(), ChildNumber::Normal { index })?
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
pub struct Credentials {
    network: Network,
    key: PrivateKey,
}

impl Credentials {
    /// Creates credentials from the network and private key
    pub fn new(network: Network, key: PrivateKey) -> Self {
        Self { network, key }
    }

    /// Returns the private key
    pub fn private_key(&self) -> PrivateKey {
        self.key
    }

    /// Returns the public key
    pub fn public_key(&self) -> PublicKey {
        self.key.public_key(&mut Secp256k1::new())
    }

    /// Returns the P2PKH address
    pub fn address(&self) -> StacksAddress {
        let version = match self.network {
            Network::Mainnet => AddressVersion::MainnetSingleSig,
            Network::Testnet => AddressVersion::TestnetSingleSig,
        };

        StacksAddress::p2pkh(version, &self.public_key())
    }
}
