//! Exposes tools to create and manage Stacks credentials.

use std::str::FromStr;

use bdk::{
	bitcoin::{
		secp256k1::Secp256k1,
		util::bip32::{DerivationPath, ExtendedPrivKey},
		Address as BitcoinAddress, AddressType as BitcoinAddressType,
		Network as BitcoinNetwork,
	},
	keys::bip39::Mnemonic,
};
use rand::random;
use serde::{Deserialize, Serialize};

use crate::{
	address::{AddressVersion, StacksAddress},
	crypto::{wif::WIF, PrivateKey, PublicKey},
	Network, StacksError, StacksResult,
};

/// Computes Stacks derivation paths
pub fn stacks_derivation_path(index: u32) -> StacksResult<DerivationPath> {
	Ok(DerivationPath::from_str(&format!(
		"m/44'/5757'/0'/0/{}",
		index
	))?)
}

/// Computes Bitcoin derivation paths
pub fn bitcoin_derivation_path(
	network: BitcoinNetwork,
	kind: BitcoinAddressType,
	index: u32,
) -> StacksResult<DerivationPath> {
	let mut path = "m/".to_string();

	match kind {
		BitcoinAddressType::P2pkh => path.push_str("44'/"),
		BitcoinAddressType::P2wpkh => path.push_str("84'/"),
		BitcoinAddressType::P2tr => path.push_str("86'/"),
		_ => {
			return Err(StacksError::InvalidArguments(
				"Invalid Bitcoin addres type",
			))
		}
	};

	match network {
		BitcoinNetwork::Bitcoin => path.push_str("0'/"),
		_ => path.push_str("1'/"),
	}

	path.push_str(&format!("{}'/0/0", index));

	Ok(DerivationPath::from_str(&path)?)
}

/// Derives a key from a master key and a derivation path
pub fn derive_key(
	master_key: ExtendedPrivKey,
	path: DerivationPath,
) -> ExtendedPrivKey {
	master_key.derive_priv(&Secp256k1::new(), &path).unwrap()
}

/// Wallet of credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
	master_key: ExtendedPrivKey,
	mnemonic: Mnemonic,
}

impl Wallet {
	/// Creates a wallet from the network, mnemonic, and optional passphrase
	pub fn new(mnemonic: impl AsRef<str>) -> StacksResult<Self> {
		let mnemonic = Mnemonic::from_str(mnemonic.as_ref())?;

		// Bitcoin network is irrelevant for extended private keys
		let master_key = ExtendedPrivKey::new_master(
			BitcoinNetwork::Bitcoin,
			&mnemonic.to_seed(""),
		)?;

		Ok(Self {
			master_key,
			mnemonic,
		})
	}

	/// Creates a random wallet
	pub fn random() -> StacksResult<Self> {
		let entropy: [u8; 32] = random();
		let mnemonic = Mnemonic::from_entropy(&entropy)?;

		Self::new(mnemonic.to_string())
	}

	/// Returns the mnemonic of the wallet
	pub fn mnemonic(&self) -> Mnemonic {
		self.mnemonic.clone()
	}

	/// Returns the master key of the wallet
	pub fn master_key(&self) -> PrivateKey {
		self.master_key.private_key
	}

	/// Returns the WIF of the wallet
	pub fn wif(&self, network: Network) -> WIF {
		WIF::new(network, self.master_key())
	}

	/// Returns the credentials at the given index
	pub fn credentials(
		&self,
		network: Network,
		index: u32,
	) -> StacksResult<Credentials> {
		Credentials::new(network, self.master_key, index)
	}

	/// Returns the Bitcoin credentials at the given index
	pub fn bitcoin_credentials(
		&self,
		network: BitcoinNetwork,
		index: u32,
	) -> StacksResult<BitcoinCredentials> {
		BitcoinCredentials::new(network, self.master_key, index)
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
	pub fn new(
		network: Network,
		master_key: ExtendedPrivKey,
		index: u32,
	) -> StacksResult<Self> {
		let private_key =
			derive_key(master_key, stacks_derivation_path(index)?)
				.to_priv()
				.inner;

		Ok(Self {
			network,
			private_key,
		})
	}

	/// Returns the Stacks network
	pub fn network(&self) -> Network {
		self.network
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

	/// Returns the WIF
	pub fn wif(&self) -> WIF {
		WIF::new(self.network(), self.private_key())
	}
}

/// Bitcoin Credentials that can be used to sign transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinCredentials {
	network: BitcoinNetwork,
	private_key_p2pkh: PrivateKey,
	private_key_p2wpkh: PrivateKey,
	private_key_p2tr: PrivateKey,
}

impl BitcoinCredentials {
	/// Creates Bitcoin credentials from the Bitcoin network and private key
	pub fn new(
		network: BitcoinNetwork,
		master_key: ExtendedPrivKey,
		index: u32,
	) -> StacksResult<Self> {
		let private_key_p2pkh = derive_key(
			master_key,
			bitcoin_derivation_path(network, BitcoinAddressType::P2pkh, index)?,
		)
		.to_priv()
		.inner;

		let private_key_p2wpkh = derive_key(
			master_key,
			bitcoin_derivation_path(
				network,
				BitcoinAddressType::P2wpkh,
				index,
			)?,
		)
		.to_priv()
		.inner;

		let private_key_p2tr = derive_key(
			master_key,
			bitcoin_derivation_path(network, BitcoinAddressType::P2tr, index)?,
		)
		.to_priv()
		.inner;

		Ok(Self {
			network,
			private_key_p2pkh,
			private_key_p2wpkh,
			private_key_p2tr,
		})
	}

	/// Returns the Bitcoin network
	pub fn network(&self) -> BitcoinNetwork {
		self.network
	}

	/// Returns the Bitcoin P2PKH private key
	pub fn private_key_p2pkh(&self) -> PrivateKey {
		self.private_key_p2pkh
	}

	/// Returns the Bitcoin P22PKH private key
	pub fn private_key_p2wpkh(&self) -> PrivateKey {
		self.private_key_p2wpkh
	}

	/// Returns the Bitcoin P2TR private key
	pub fn private_key_p2tr(&self) -> PrivateKey {
		self.private_key_p2tr
	}

	/// Returns the Bitcoin P2PKH public key
	pub fn public_key_p2pkh(&self) -> PublicKey {
		self.private_key_p2pkh.public_key(&Secp256k1::new())
	}

	/// Returns the Bitcoin P2WPKH public key
	pub fn public_key_p2wpkh(&self) -> PublicKey {
		self.private_key_p2wpkh.public_key(&Secp256k1::new())
	}

	/// Returns the Bitcoin P2TR public key
	pub fn public_key_p2tr(&self) -> PublicKey {
		self.private_key_p2tr.public_key(&Secp256k1::new())
	}

	/// Returns the Bitcoin P2PKH address
	pub fn address_p2pkh(&self) -> BitcoinAddress {
		BitcoinAddress::p2pkh(
			&bdk::bitcoin::PublicKey::new(self.public_key_p2pkh()),
			self.network(),
		)
	}

	/// Returns the Bitcoin P2WPKH address
	pub fn address_p2wpkh(&self) -> BitcoinAddress {
		BitcoinAddress::p2wpkh(
			&bdk::bitcoin::PublicKey::new(self.public_key_p2wpkh()),
			self.network(),
		)
		.unwrap()
	}

	/// Returns the Bitcoin P2TR address
	pub fn address_p2tr(&self) -> BitcoinAddress {
		BitcoinAddress::p2tr(
			&Secp256k1::new(),
			self.public_key_p2tr().x_only_public_key().0,
			None,
			self.network(),
		)
	}

	/// Returns the WIF for P2PKH
	pub fn wif_p2pkh(&self) -> WIF {
		WIF::new(self.network().into(), self.private_key_p2pkh())
	}

	/// Returns the WIF for P2WPKH
	pub fn wif_p2wpkh(&self) -> WIF {
		WIF::new(self.network().into(), self.private_key_p2wpkh())
	}

	/// Returns the WIF for P2TR
	pub fn wif_p2tr(&self) -> WIF {
		WIF::new(self.network().into(), self.private_key_p2tr())
	}
}
