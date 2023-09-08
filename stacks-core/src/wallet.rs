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
/// Derivation path used for mainnet Bitcoin segwit addresses
pub const BITCOIN_SEGWIT_MAINNET_DERIVATION_PATH: &str = "m/84'/0'/0'/0";
/// Derivation path used for testnet Bitcoin segwit addresses
pub const BITCOIN_SEGWIT_TESTNET_DERIVATION_PATH: &str = "m/84'/1'/0'/0";

/// Wallet of credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    network: Network,
    master_key: ExtendedPrivKey,
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

        let master_key = ExtendedPrivKey::new_master(bitcoin_network, &mnemonic.to_seed(""))?;

        Ok(Self {
            network,
            master_key,
            mnemonic,
        })
    }

    /// Creates a random wallet
    pub fn random(network: Network) -> StacksResult<Self> {
        let entropy: [u8; 32] = random();
        let mnemonic = Mnemonic::from_entropy(&entropy)?;

        Self::new(network, mnemonic.to_string())
    }

    /// Returns the network of the wallet
    pub fn network(&self) -> Network {
        self.network
    }

    /// Returns the mnemonic of the wallet
    pub fn mnemonic(&self) -> Mnemonic {
        self.mnemonic.clone()
    }

    /// Returns the master key of the wallet
    pub fn master_key(&self) -> PrivateKey {
        self.master_key.private_key
    }

    /// Returns the Stacks master key of the wallet
    pub fn stacks_master_key(&self) -> StacksResult<ExtendedPrivKey> {
        Ok(self.master_key.derive_priv(
            &Secp256k1::new(),
            &DerivationPath::from_str(STACKS_DERIVATION_PATH).unwrap(),
        )?)
    }

    /// Returns the Bitcoin master key of the wallet
    pub fn bitcoin_master_key(&self) -> StacksResult<ExtendedPrivKey> {
        let derivation_path = match self.network() {
            Network::Mainnet => BITCOIN_SEGWIT_MAINNET_DERIVATION_PATH,
            Network::Testnet => BITCOIN_SEGWIT_TESTNET_DERIVATION_PATH,
        };

        Ok(self.master_key.derive_priv(
            &Secp256k1::new(),
            &DerivationPath::from_str(derivation_path).unwrap(),
        )?)
    }

    /// Returns the credentials at the given index
    pub fn credentials(&self, index: u32) -> StacksResult<Credentials> {
        let key = self
            .stacks_master_key()?
            .ckd_priv(&Secp256k1::new(), ChildNumber::Normal { index })?
            .to_priv()
            .inner;

        Ok(Credentials::new(self.network(), key))
    }

    /// Returns the Bitcoin credentials at the given index
    pub fn bitcoin_credentials(&self, index: u32) -> StacksResult<BitcoinCredentials> {
        let key = self
            .bitcoin_master_key()?
            .ckd_priv(&Secp256k1::new(), ChildNumber::Normal { index })?
            .to_priv()
            .inner;

        let bitcoin_network = match self.network() {
            Network::Mainnet => BitcoinNetwork::Bitcoin,
            Network::Testnet => BitcoinNetwork::Testnet,
        };

        Ok(BitcoinCredentials::new(bitcoin_network, key))
    }
}

/// Credentials that can be used to sign transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    network: Network,
    stacks_private_key: PrivateKey,
}

impl Credentials {
    /// Creates credentials from the network and private key
    pub fn new(network: Network, stacks_private_key: PrivateKey) -> Self {
        Self {
            network,
            stacks_private_key,
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

    /// Returns the private key
    pub fn private_key(&self) -> PrivateKey {
        self.stacks_private_key
    }

    /// Returns the public key
    pub fn public_key(&self) -> PublicKey {
        self.stacks_private_key.public_key(&Secp256k1::new())
    }

    /// Returns the Stacks P2PKH address
    pub fn address(&self) -> StacksAddress {
        let version = match self.network {
            Network::Mainnet => AddressVersion::MainnetSingleSig,
            Network::Testnet => AddressVersion::TestnetSingleSig,
        };

        StacksAddress::p2pkh(version, &self.public_key())
    }
}

/// Bitcoin Credentials that can be used to sign transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinCredentials {
    network: BitcoinNetwork,
    bitcoin_private_key: PrivateKey,
}

impl BitcoinCredentials {
    /// Creates Bitcoin credentials from the Bitcoin network and private key
    pub fn new(network: BitcoinNetwork, bitcoin_private_key: PrivateKey) -> Self {
        Self {
            network,
            bitcoin_private_key,
        }
    }

    /// Creates random Bitcoin credentials
    pub fn random(network: BitcoinNetwork) -> Self {
        let key = Secp256k1::new().generate_keypair(&mut rand::thread_rng()).0;

        Self::new(network, key)
    }

    /// Returns the Bitcoin network
    pub fn network(&self) -> BitcoinNetwork {
        self.network
    }

    /// Returns the Bitcoin private key
    pub fn private_key(&self) -> PrivateKey {
        self.bitcoin_private_key
    }

    /// Returns the Bitcoin public key
    pub fn public_key(&self) -> PublicKey {
        self.bitcoin_private_key.public_key(&Secp256k1::new())
    }

    /// Returns the Bitcoin P2PKH address
    pub fn p2pkh_address(&self) -> BitcoinAddress {
        BitcoinAddress::p2pkh(
            &bdk::bitcoin::PublicKey::new(self.public_key()),
            self.network(),
        )
    }

    /// Returns the Bitcoin P2WPKH address
    pub fn p2wpkh_address(&self) -> BitcoinAddress {
        BitcoinAddress::p2wpkh(
            &bdk::bitcoin::PublicKey::new(self.public_key()),
            self.network(),
        )
        .unwrap()
    }

    /// Returns the Bitcoin taproot address
    pub fn taproot_address(&self) -> BitcoinAddress {
        BitcoinAddress::p2tr(
            &Secp256k1::new(),
            self.public_key().x_only_public_key().0,
            None,
            self.network(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_hiro_addresses() {
        let wallet = Wallet::new(
            Network::Testnet,
            "apart spin rich leader siren foil dish sausage fee pipe ethics bundle",
        )
        .unwrap();

        for i in 0..7 {
            let creds = wallet.credentials(i).unwrap();
            println!("STX address: {}", creds.address());

            let bitcoin_creds = wallet.bitcoin_credentials(i).unwrap();
            println!("Bitcoin P2PKH address: {}", bitcoin_creds.p2pkh_address());
            println!("Bitcoin P2WPKH address: {}", bitcoin_creds.p2wpkh_address());
            println!(
                "Bitcoin Taproot address: {}",
                bitcoin_creds.taproot_address()
            );
        }
    }

    #[test]
    fn taproot() {
        let wallet = Wallet::new(
            Network::Testnet,
            "apart spin rich leader siren foil dish sausage fee pipe ethics bundle",
        )
        .unwrap();

        let creds = wallet.credentials(0).unwrap();
        println!("STX {}", creds.address());
    }
}
