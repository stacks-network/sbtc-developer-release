/*!
Deposit is a Bitcoin transaction with the output structure as below:

1. data output
2. payment to peg wallet address

The data output should contain data in the following byte format:

```text
0     2  3                                                                    80
|-----|--|---------------------------------------------------------------------|
 magic op                           deposit data
```

Where deposit data should be in the following format:

```text
3                                                      25 >= N <= 66          80
|------------------------------------------------------------------|-----------|
                           principal data                              extra
                                                                       bytes
```

There are two types of principal data:

- standard (includes only principal type, address version and address hash)
- contract (includes everything from the last diagram)

If the principal data is of the contract type, then the contract name cannot be
longer than 40 characters.

Principal data should be in the following format:

```text
3         4         5                25       26                         N <= 66
|---------|---------|-----------------|--------|-------------------------------|
 principal  address       address      contract            contract
   type     version        hash          name                name
                                      length (N)
```
*/
use std::io::{Cursor, Read};

use stacks_core::{codec::Codec, utils::PrincipalData, StacksError};

use crate::{SBTCError, SBTCResult};

const WIRE_DATA_MIN_LENGTH: usize = 25;
const WIRE_PRINCIPAL_DATA_INDEX: usize = 3;

/// Contains the parsed data from a deposit transaction
pub struct ParsedDepositData {
    /// The recipient of the deposit
    pub recipient: PrincipalData,
    /// The memo
    pub memo: Vec<u8>,
}

/// Parses the data output of the deposit transaction
pub fn parse(data: &[u8]) -> SBTCResult<ParsedDepositData> {
    if data.len() < WIRE_DATA_MIN_LENGTH {
        return Err(SBTCError::MalformedData("Should contain at least 21 bytes"));
    }

    let mut data = Cursor::new(&data[WIRE_PRINCIPAL_DATA_INDEX..]);
    let recipient = PrincipalData::deserialize(&mut data)?;

    let mut memo = vec![];
    data.read_to_end(&mut memo)
        .map_err(|_| StacksError::InvalidData("Could not read memo bytes"))?;

    Ok(ParsedDepositData { recipient, memo })
}

#[cfg(test)]
mod tests {
    use std::io::{Seek, SeekFrom, Write};

    use rand::{distributions::Alphanumeric, rngs::StdRng, Rng, SeedableRng};
    use stacks_core::{
        address::{AddressVersion, StacksAddress},
        codec::Codec,
        contract_name::{ContractName, CONTRACT_MAX_NAME_LENGTH},
        crypto::generate_keypair,
        utils::StandardPrincipalData,
    };

    use super::*;

    const WIRE_CONTRACT_PRINCIPAL_DATA_NAME_INDEX: usize = 26;
    const WIRE_CONTRACT_PRINCIPAL_DATA_NAME_MAX_INDEX: usize = 66;
    const WIRE_DATA_MAX_LENGTH: usize = 80;

    fn test_rng() -> StdRng {
        StdRng::seed_from_u64(0)
    }

    fn generate_address(rng: &mut impl Rng) -> StacksAddress {
        let pk = generate_keypair(rng).1;

        StacksAddress::p2pkh(AddressVersion::TestnetSingleSig, &pk)
    }

    fn generate_contract_name(rng: &mut impl Rng) -> ContractName {
        let contract_name_length: u8 = rng.gen_range(1..CONTRACT_MAX_NAME_LENGTH as u8);

        let contract_name = {
            let mut contract_name_char_iter = rng.sample_iter(&Alphanumeric).map(char::from);

            let first_letter = loop {
                let letter = contract_name_char_iter.next().unwrap();

                if letter.is_digit(10) {
                    continue;
                } else {
                    break letter;
                };
            };

            let other_letters = contract_name_char_iter.take(contract_name_length as usize - 1);

            let contract_name_string = [first_letter]
                .into_iter()
                .chain(other_letters)
                .collect::<String>();

            ContractName::new(&contract_name_string).unwrap()
        };

        contract_name
    }

    fn generate_contract_principal_data(rng: &mut impl Rng) -> PrincipalData {
        PrincipalData::Contract(
            StandardPrincipalData::new(AddressVersion::TestnetSingleSig, generate_address(rng)),
            generate_contract_name(rng),
        )
    }

    fn generate_principal_data(rng: &mut impl Rng) -> PrincipalData {
        let should_be_standard_principal: bool = rng.gen();

        if should_be_standard_principal {
            PrincipalData::Standard(StandardPrincipalData::new(
                AddressVersion::TestnetSingleSig,
                generate_address(rng),
            ))
        } else {
            generate_contract_principal_data(rng)
        }
    }

    fn build_deposit_data(
        rng: &mut impl Rng,
        principal_data: &PrincipalData,
    ) -> ([u8; WIRE_DATA_MAX_LENGTH], Vec<u8>) {
        let mut data = [0u8; WIRE_DATA_MAX_LENGTH];

        let memo = {
            let mut data_cursor = Cursor::new(&mut data[..]);
            data_cursor
                .seek(SeekFrom::Start(WIRE_PRINCIPAL_DATA_INDEX as u64))
                .unwrap();

            principal_data.serialize(&mut data_cursor).unwrap();

            let position = data_cursor.position() as usize;
            let memo: Vec<u8> = std::iter::from_fn(|| Some(rng.gen::<u8>()))
                .take(WIRE_DATA_MAX_LENGTH - position)
                .collect();

            data_cursor.write(&memo).unwrap();

            memo
        };

        (data, memo)
    }

    fn generate_deposit_data(
        rng: &mut impl Rng,
    ) -> (PrincipalData, Vec<u8>, [u8; WIRE_DATA_MAX_LENGTH]) {
        let principal_data = generate_principal_data(rng);
        let (data, memo) = build_deposit_data(rng, &principal_data);

        (principal_data, memo, data)
    }

    #[test]
    fn should_parse_deposit_data() {
        let mut rng = test_rng();

        for _ in 0..1000 {
            let (expected_principal_data, expected_memo, data) = generate_deposit_data(&mut rng);
            let parsed_data = parse(&data).unwrap();

            assert_eq!(parsed_data.recipient, expected_principal_data);
            assert_eq!(parsed_data.memo, expected_memo);
        }
    }

    #[test]
    fn should_fail_on_missing_contract_name_bytes() {
        let mut rng = test_rng();

        let principal_data = generate_contract_principal_data(&mut rng);
        let (mut data, _) = build_deposit_data(&mut rng, &principal_data);

        data[WIRE_CONTRACT_PRINCIPAL_DATA_NAME_INDEX..WIRE_CONTRACT_PRINCIPAL_DATA_NAME_MAX_INDEX]
            .iter_mut()
            .for_each(|byte| *byte = 0);

        assert!(parse(&data).is_err());
    }

    #[test]
    fn should_fail_on_incomplete_contract_name_bytes() {
        let mut rng = test_rng();

        let principal_data = generate_contract_principal_data(&mut rng);
        let principal_data_contract_name_length = match &principal_data {
            PrincipalData::Contract(_, contract_name) => contract_name.len(),
            PrincipalData::Standard(_) => panic!("Should be contract principal data"),
        };

        let (mut data, _) = build_deposit_data(&mut rng, &principal_data);

        data[WIRE_CONTRACT_PRINCIPAL_DATA_NAME_INDEX + principal_data_contract_name_length - 1
            ..WIRE_CONTRACT_PRINCIPAL_DATA_NAME_MAX_INDEX]
            .iter_mut()
            .for_each(|byte| *byte = 0);

        assert!(parse(&data).is_err());
    }

    #[test]
    fn should_truncate_on_extra_contract_name_bytes() {
        let mut rng = test_rng();

        let (expected_principal_data, principal_data_contract_name_length) = loop {
            let principal_data = generate_contract_principal_data(&mut rng);

            let principal_data_contract_name_length = match &principal_data {
                PrincipalData::Contract(_, contract_name) => contract_name.len(),
                PrincipalData::Standard(_) => panic!("Should be contract principal data"),
            };

            if principal_data_contract_name_length < CONTRACT_MAX_NAME_LENGTH {
                break (principal_data, principal_data_contract_name_length);
            }
        };

        let (mut data, _) = build_deposit_data(&mut rng, &expected_principal_data);
        data[WIRE_CONTRACT_PRINCIPAL_DATA_NAME_INDEX + principal_data_contract_name_length + 1] =
            b'X';

        let parsed_data = parse(&data).unwrap();

        assert_eq!(parsed_data.recipient, expected_principal_data);
    }
}
