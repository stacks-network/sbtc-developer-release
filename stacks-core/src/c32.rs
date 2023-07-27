use crate::{address::AddressVersion, crypto::hash::SHA256Hash};

const C32_ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
#[rustfmt::skip]
const C32_BYTE_MAP: [Option<u8>; 128] = [
    None,       None,       None,       None,       None,       None,       None,       None,
    None,       None,       None,       None,       None,       None,       None,       None,
    None,       None,       None,       None,       None,       None,       None,       None,
    None,       None,       None,       None,       None,       None,       None,       None,
    None,       None,       None,       None,       None,       None,       None,       None,
    None,       None,       None,       None,       None,       None,       None,       None,
    Some(0),    Some(1),    Some(2),    Some(3),    Some(4),    Some(5),    Some(6),    Some(7),
    Some(8),    Some(9),    None,       None,       None,       None,       None,       None,
    None,       Some(10),   Some(11),   Some(12),   Some(13),   Some(14),   Some(15),   Some(16),
    Some(17),   Some(1),    Some(18),   Some(19),   Some(1),    Some(20),   Some(21),   Some(0),
    Some(22),   Some(23),   Some(24),   Some(25),   Some(26),   None,       Some(27),   Some(28),
    Some(29),   Some(30),   Some(31),   None,       None,       None,       None,       None,
    None,       Some(10),   Some(11),   Some(12),   Some(13),   Some(14),   Some(15),   Some(16),
    Some(17),   Some(1),    Some(18),   Some(19),   Some(1),    Some(20),   Some(21),   Some(0),
    Some(22),   Some(23),   Some(24),   Some(25),   Some(26),   None,       Some(27),   Some(28),
    Some(29),   Some(30),   Some(31),   None,       None,       None,       None,       None,
];

fn encode_overhead(len: usize) -> usize {
    (len * 8 + 4) / 5
}

fn decode_underhead(len: usize) -> usize {
    len / (8f64 / 5f64).ceil() as usize
}

#[derive(thiserror::Error, Clone, Debug, Eq, PartialEq)]
pub enum C32Error {
    /// Invalid C32 string.
    #[error("Invalid C32 string")]
    InvalidC32,
    /// Invalid character.
    #[error("Invalid C32 character: {0}")]
    InvalidChar(char),
    /// Invalid checksum.
    #[error("Invalid C32 checksum - expected {0:?}, got {1:?}")]
    InvalidChecksum([u8; 4], Vec<u8>),
    /// Invalid C32 address.
    #[error("Invalid C32 address: {0}")]
    InvalidAddress(String),
    /// Invalid C32 address.
    #[error("Invalid C32 address version: {0}")]
    InvalidVersion(u8),
    /// Conversion error, from utf8.
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    /// Integer conversion error.
    #[error(transparent)]
    IntConversionError(#[from] std::num::TryFromIntError),
}

pub fn encode(data: impl AsRef<[u8]>) -> Vec<u8> {
    let data = data.as_ref();

    let mut encoded = Vec::with_capacity(encode_overhead(data.len()));
    let mut buffer = 0u32;
    let mut bits = 0;

    for byte in data.iter().rev() {
        buffer |= (*byte as u32) << bits;
        bits += 8;

        while bits >= 5 {
            encoded.push(C32_ALPHABET[(buffer & 0x1F) as usize]);
            buffer >>= 5;
            bits -= 5;
        }
    }

    if bits > 0 {
        encoded.push(C32_ALPHABET[(buffer & 0x1F) as usize]);
    }

    while let Some(i) = encoded.pop() {
        if i != C32_ALPHABET[0] {
            encoded.push(i);
            break;
        }
    }

    for i in data {
        if *i == 0 {
            encoded.push(C32_ALPHABET[0]);
        } else {
            break;
        }
    }

    encoded.reverse();

    encoded
}

pub fn decode(input: impl AsRef<[u8]>) -> Result<Vec<u8>, C32Error> {
    let input = input.as_ref();

    if !input.is_ascii() {
        return Err(C32Error::InvalidC32);
    }

    let mut decoded = Vec::with_capacity(decode_underhead(input.len()));
    let mut carry = 0u16;
    let mut carry_bits = 0;

    for byte in input.iter().rev() {
        let Some(bits) = C32_BYTE_MAP.get(*byte as usize).unwrap() else {
            return Err(C32Error::InvalidChar(*byte as char));
        };

        carry |= (u16::from(*bits)) << carry_bits;
        carry_bits += 5;

        if carry_bits >= 8 {
            decoded.push((carry & 0xFF) as u8);
            carry >>= 8;
            carry_bits -= 8;
        }
    }

    if carry_bits > 0 {
        decoded.push(u8::try_from(carry)?);
    }

    while let Some(i) = decoded.pop() {
        if i != 0 {
            decoded.push(i);
            break;
        }
    }

    for byte in input.iter() {
        if *byte == b'0' {
            decoded.push(0);
        } else {
            break;
        }
    }

    decoded.reverse();

    Ok(decoded)
}

pub fn version_check_encode(version: AddressVersion, data: impl AsRef<[u8]>) -> Vec<u8> {
    let data = data.as_ref();

    let mut buffer = vec![version as u8];
    buffer.extend_from_slice(data);

    let checksum = SHA256Hash::double(&buffer).checksum();
    buffer.extend_from_slice(&checksum);

    let mut encoded = encode(&buffer[1..]);
    encoded.insert(0, C32_ALPHABET[version as usize]);

    encoded
}

pub fn version_check_decode(
    input: impl AsRef<[u8]>,
) -> Result<(AddressVersion, Vec<u8>), C32Error> {
    let input = input.as_ref();

    if !input.is_ascii() {
        return Err(C32Error::InvalidC32);
    }

    let (encoded_version_bytes, encoded_data_bytes) = input.split_at(1);

    let decoded_version_bytes = decode(encoded_version_bytes)?;
    let decoded_version_byte = *decoded_version_bytes.first().unwrap();
    let decoded_data_bytes = decode(encoded_data_bytes)?;

    if decoded_data_bytes.len() < 4 {
        return Err(C32Error::InvalidC32);
    }

    let (data_bytes, expected_checksum) = decoded_data_bytes.split_at(decoded_data_bytes.len() - 4);

    let mut buffer_to_check = vec![decoded_version_byte];
    buffer_to_check.extend_from_slice(data_bytes);

    let computed_checksum = SHA256Hash::double(&buffer_to_check).checksum();

    if computed_checksum != expected_checksum {
        return Err(C32Error::InvalidChecksum(
            computed_checksum,
            expected_checksum.to_vec(),
        ));
    }

    Ok((
        decoded_version_byte
            .try_into()
            .map_err(|_| C32Error::InvalidVersion(decoded_version_byte))?,
        data_bytes.to_vec(),
    ))
}

pub fn encode_address(version: AddressVersion, data: &[u8]) -> String {
    let encoded = String::from_utf8(version_check_encode(version, data)).unwrap();
    let address = format!("S{}", encoded);

    address
}

pub fn decode_address(address: impl AsRef<str>) -> Result<(AddressVersion, Vec<u8>), C32Error> {
    let address = address.as_ref();

    if !address.starts_with('S') || address.len() <= 5 {
        return Err(C32Error::InvalidAddress(address.to_string()));
    }

    version_check_decode(&address[1..])
}

#[cfg(test)]
mod tests {
    use rand::thread_rng;
    use rand::Rng;
    use rand::RngCore;
    use strum::IntoEnumIterator;

    use crate::address::AddressVersion;

    use super::encode;
    use super::encode_address;

    #[test]
    fn test_c32_encode() {
        let input = vec![1, 2, 3, 4, 6, 1, 2, 6, 2, 3, 6, 9, 4, 0, 0];
        let encoded = String::from_utf8(encode(&input)).unwrap();
        assert_eq!(encoded, "41061060410C0G30R4G8000");
    }

    #[test]
    fn test_c32_decode() {
        let input = vec![1, 2, 3, 4, 6, 1, 2, 6, 2, 3, 6, 9, 4, 0, 0];
        let encoded = String::from_utf8(encode(&input)).unwrap();
        let decoded = super::decode(encoded).unwrap();
        assert_eq!(input, decoded);
    }

    #[test]
    fn test_c32_check() {
        let version = AddressVersion::MainnetSingleSig;
        let data = hex::encode("8a4d3f2e55c87f964bae8b2963b3a824a2e9c9ab").into_bytes();

        let encoded = encode_address(version, &data);
        let (decoded_version, decoded) = super::decode_address(encoded).unwrap();

        assert_eq!(decoded, data);
        assert_eq!(decoded_version, version);
    }

    #[test]
    fn test_c32_randomized_input() {
        let mut rng = thread_rng();

        for _ in 0..10_000 {
            let len = rng.gen_range(10..=10);
            let mut input = vec![0u8; len];
            rng.fill_bytes(&mut input);

            let encoded_bytes = encode(&input);
            let encoded = String::from_utf8(encoded_bytes.clone()).unwrap();
            let decoded = super::decode(encoded.clone()).unwrap();

            assert_eq!(decoded, input);
        }
    }

    #[test]
    fn test_c32_check_randomized_input() {
        let mut rng = thread_rng();

        for _ in 0..10_000 {
            let bytes = rng.gen::<[u8; 20]>();

            for version in AddressVersion::iter() {
                let encoded = encode_address(version, &bytes);
                let (decoded_version, decoded) = super::decode_address(encoded).unwrap();

                assert_eq!(decoded, bytes);
                assert_eq!(decoded_version, version);
            }
        }
    }
}
