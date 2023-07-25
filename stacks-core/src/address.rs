use std::fmt;

use crate::{
    c32::{decode_address, encode_address},
    crypto::{
        hash::{Hash160, SHA256Hash},
        PublicKey,
    },
    StacksError, StacksResult,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AddressHashMode {
    SerializeP2PKH,
    SerializeP2SH,
    SerializeP2WPKH,
    SerializeP2WSH,
}

#[derive(Debug, Clone)]
pub struct StacksAddress {
    version: u8,
    hash: Hash160,
}

impl StacksAddress {
    pub fn new(version: u8, hash: Hash160) -> Self {
        Self { version, hash }
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn hash(&self) -> &Hash160 {
        &self.hash
    }

    pub fn from_public_keys(
        version: u8,
        public_keys: &[PublicKey],
        signatures: usize,
        hash_mode: AddressHashMode,
    ) -> StacksResult<Self> {
        let public_key_count = public_keys.len();

        if public_key_count < signatures {
            return Err(StacksError::InvalidArguments(
                "Cannot require more signatures than public keys",
            ));
        }

        if matches!(
            hash_mode,
            AddressHashMode::SerializeP2PKH | AddressHashMode::SerializeP2WPKH
        ) {
            if public_key_count != 1 {
                return Err(StacksError::InvalidArguments(
                    "Cannot use more than one public key for this hash mode",
                ));
            }

            if signatures != 1 {
                return Err(StacksError::InvalidArguments(
                    "Cannot require more than one signature for this hash mode",
                ));
            }
        }

        let hash = match hash_mode {
            AddressHashMode::SerializeP2PKH => hash_p2pkh(&public_keys[0]),
            AddressHashMode::SerializeP2SH => hash_p2sh(signatures, public_keys),
            AddressHashMode::SerializeP2WPKH => hash_p2wpkh(&public_keys[0]),
            AddressHashMode::SerializeP2WSH => hash_p2wsh(signatures, public_keys),
        };

        Ok(Self::new(version, hash))
    }
}

impl TryFrom<&StacksAddress> for String {
    type Error = StacksError;

    fn try_from(address: &StacksAddress) -> Result<Self, Self::Error> {
        encode_address(address.version, address.hash.as_ref()).map_err(|err| err.into())
    }
}

impl TryFrom<&str> for StacksAddress {
    type Error = StacksError;

    fn try_from(address: &str) -> Result<Self, Self::Error> {
        let (version, hash) =
            decode_address(address).map_err::<StacksError, _>(|err| err.into())?;

        Ok(Self::new(version, Hash160::new(hash)))
    }
}

impl fmt::Display for StacksAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::try_from(self).unwrap())
    }
}

fn hash_p2pkh(key: &PublicKey) -> Hash160 {
    Hash160::new(key.serialize())
}

fn hash_p2sh(num_sigs: usize, pub_keys: &[PublicKey]) -> Hash160 {
    let mut buff = vec![];
    buff.push(num_sigs as u8 + 80);

    for pub_key in pub_keys {
        let bytes = pub_key.serialize();

        buff.push(bytes.len() as u8);
        buff.extend_from_slice(&bytes);
    }

    buff.push(pub_keys.len() as u8 + 80);
    buff.push(174);

    Hash160::new(&buff)
}

fn hash_p2wpkh(key: &PublicKey) -> Hash160 {
    let key_hash_hasher = Hash160::new(key.serialize());
    let key_hash = key_hash_hasher.as_ref();
    let key_hash_len = key_hash.len();

    let mut buff = Vec::with_capacity(key_hash_len + 2);
    buff.push(0);
    buff.push(key_hash_len as u8);
    buff.extend_from_slice(key_hash);

    Hash160::new(&buff)
}

fn hash_p2wsh(num_sigs: usize, pub_keys: &[PublicKey]) -> Hash160 {
    let mut script = vec![];
    script.push(num_sigs as u8 + 80);

    for pub_key in pub_keys {
        let bytes = pub_key.serialize();

        script.push(bytes.len() as u8);
        script.extend_from_slice(&bytes);
    }

    script.push(pub_keys.len() as u8 + 80);
    script.push(174);

    let digest = SHA256Hash::new(&script);
    let digest_bytes = digest.as_ref();

    let mut buff = vec![];
    buff.push(0);
    buff.push(digest_bytes.len() as u8);
    buff.extend_from_slice(digest_bytes);

    Hash160::new(&buff)
}

#[cfg(test)]
mod tests {
    use super::*;

    /**
    Sample data computed with these commands on MacOS:

    ```
    CREDENTIALS=$(stx make_keychain)
    PUBLIC_KEY=$(echo $CREDENTIALS | jq -r .key_info.publicKey)
    EXPECTED_HASH=$(echo $PUBLIC_KEY \
        | xxd -r -p \
        | openssl dgst -sha256 -binary \
        | openssl dgst -ripemd160 -binary \
        | xxd -p)
    ```
    */
    #[test]
    fn should_correctly_hash_p2pkh() {
        let public_key_hex = "03556902f83defc6c63a7eb56a2d8ee4baee109f2126aac41e4f9e3a0835f34bc5";
        let expected_hash_hex = "d24206d58967c61b6b302eb14cd254a8ae7e761a";

        let pk = PublicKey::from_slice(&hex::decode(public_key_hex).unwrap()).unwrap();
        let hash_hex = hex::encode(hash_p2pkh(&pk).as_ref());

        assert_eq!(hash_hex, expected_hash_hex);
    }

    #[test]
    fn should_correctly_hash_p2wpkh() {
        let input = PublicKey::from_slice(b"bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu").unwrap();
        let expected_str = "9ecb2946469c02135e5c9d85a58d18e33fb8b7fa";
        let expected_bytes: [u8; 20] = hex::encode(expected_str).into_bytes().try_into().unwrap();

        assert_eq!(hash_p2wpkh(&input).as_ref(), expected_bytes);
    }

    #[test]
    fn should_correctly_hash_p2sh() {
        let pk_hex = "03ef788b3830c00abe8f64f62dc32fc863bc0b2cafeb073b6c8e1c7657d9c2c3ab";
        let pk = PublicKey::from_slice(&hex::decode(pk_hex).unwrap()).unwrap();

        let expected_hash =
            Hash160::new(hex::decode("b10bb6d6ff7a8b4de86614fadcc58c35808f1176").unwrap());

        assert_eq!(hash_p2sh(2, &[pk, pk]).as_ref(), expected_hash.as_ref());
    }

    #[test]
    fn should_correctly_hash_p2wsh() {
        let pk_hex = "03ef788b3830c00abe8f64f62dc32fc863bc0b2cafeb073b6c8e1c7657d9c2c3ab";
        let pk = PublicKey::from_slice(&hex::decode(pk_hex).unwrap()).unwrap();

        let expected_hash =
            Hash160::new(hex::decode("99febcfc05cb5f5836d257f34c3acb4c3a221813").unwrap());

        assert_eq!(hash_p2wsh(2, &[pk, pk]).as_ref(), expected_hash.as_ref());
    }

    /// Data generated with `stx make_keychain`
    #[test]
    fn should_create_correct_address_from_public_key() {
        let public_key = "02e2ce887c1f1654936fbb7d4036749da5e7b9b64af406e1f3535c8f4336de1c6e";
        let expected_address = "SPR4FMGJCD78NF4FRGPM621CW1KHNFEG0HSRDSPK";

        let addr = StacksAddress::from_public_keys(
            22,
            &[PublicKey::from_slice(&hex::decode(public_key).unwrap()).unwrap()],
            1,
            AddressHashMode::SerializeP2PKH,
        )
        .unwrap();

        assert_eq!(String::try_from(&addr).unwrap(), expected_address);
    }

    /// Data generated with `stx make_keychain`
    #[test]
    fn should_create_correct_address_from_c32_encoded_string() {
        let addr = "SPR4FMGJCD78NF4FRGPM621CW1KHNFEG0HSRDSPK";
        let public_key_hex = "02e2ce887c1f1654936fbb7d4036749da5e7b9b64af406e1f3535c8f4336de1c6e";
        let expected_hash =
            hash_p2pkh(&PublicKey::from_slice(&hex::decode(public_key_hex).unwrap()).unwrap());

        let addr = StacksAddress::try_from(addr).unwrap();

        assert_eq!(addr.hash(), &expected_hash);
    }
}
