/*!
Deposit is a transaction with the output structure as below:

1. data output
2. payment to peg wallet address

The data output should contain data in the following format:

0      2  3                  24                            65       80
|------|--|------------------|-----------------------------|--------|
 magic  op   Stacks address    Size prefixed contract name    memo
                                   (optional, see note)

Contract name is prefixed by a single byte size followed by the contract name
bytes.
```
*/
use std::io::Cursor;

use stacks_core::{
    address::{AddressVersion, StacksAddress},
    codec::Codec,
    utils::{ContractName, PrincipalData, StandardPrincipalData},
};

use crate::{SBTCError, SBTCResult};

/// Contains the parsed data from a deposit transaction
pub struct ParsedDepositData {
    /// The recipient of the deposit
    pub recipient: PrincipalData,
    /// The memo
    pub memo: Vec<u8>,
}

/// Parses the subset of the data output from a deposit transaction. First 3 bytes need to be removed.
pub fn parse(data: &[u8]) -> SBTCResult<ParsedDepositData> {
    if data.len() < 21 {
        return Err(SBTCError::MalformedData("Should contain at least 21 bytes"));
    }

    let standard_principal_data = {
        let version = AddressVersion::from_repr(data[0])
            .ok_or(SBTCError::MalformedData("Address version is invalid"))?;
        let addr = StacksAddress::deserialize(&mut Cursor::new(&data[0..21]))?;

        StandardPrincipalData::new(version, addr)
    };

    let contract_name_size = data[21] as usize;

    let recipient: PrincipalData = if contract_name_size == 0 {
        PrincipalData::Standard(standard_principal_data)
    } else {
        let contract_name_bytes = &data[22..(22 + contract_name_size)];

        let contract_name = std::str::from_utf8(contract_name_bytes)
            .map_err(|_| SBTCError::MalformedData("Contract name bytes are not valid UTF-8"))
            .and_then(|contract_name_string| {
                ContractName::new(contract_name_string).map_err(SBTCError::ContractNameError)
            })?;

        PrincipalData::Contract(standard_principal_data, contract_name)
    };

    let memo = data.get(62..).unwrap_or(&[]).to_vec();

    Ok(ParsedDepositData { recipient, memo })
}

#[cfg(test)]
mod tests {
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use stacks_core::{codec::Codec, crypto::generate_keypair};

    use super::*;

    fn generate_address() -> StacksAddress {
        let pk = generate_keypair(&mut thread_rng()).1;

        StacksAddress::p2pkh(AddressVersion::TestnetSingleSig, &pk)
    }

    fn generate_contract_name() -> Option<ContractName> {
        let should_have_contract_name: bool = thread_rng().gen();

        if should_have_contract_name {
            let contract_name_length: u8 = thread_rng().gen_range(1..40);
            let contract_name = {
                let mut contract_name_char_iter =
                    thread_rng().sample_iter(&Alphanumeric).map(char::from);

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

            Some(contract_name)
        } else {
            None
        }
    }

    fn build_deposit_data(
        addr: &StacksAddress,
        contract_name: Option<&ContractName>,
        memo: &[u8; 15],
    ) -> [u8; 77] {
        let mut data = [0u8; 77];

        data[0..21].copy_from_slice(&addr.serialize_to_vec());

        if let Some(contract_name) = contract_name {
            let contract_name_length = contract_name.len();

            data[21] = contract_name_length as u8;
            data[22..22 + contract_name_length].copy_from_slice(contract_name.as_bytes());
        }

        data[62..].copy_from_slice(memo);

        data
    }

    fn generate_deposit_data() -> (StacksAddress, Option<ContractName>, [u8; 15], [u8; 77]) {
        let addr = generate_address();
        let maybe_contract_name = generate_contract_name();
        let memo: [u8; 15] = thread_rng().gen();

        let data = build_deposit_data(&addr, maybe_contract_name.as_ref(), &memo);

        (addr, maybe_contract_name, memo, data)
    }

    #[test]
    fn should_parse_deposit_data() {
        for _ in 0..1000 {
            let (expected_addr, expected_contract_name, expected_memo, data) =
                generate_deposit_data();

            let parsed_data = parse(&data).unwrap();

            match parsed_data.recipient {
                PrincipalData::Standard(StandardPrincipalData(_, addr)) => {
                    assert!(expected_contract_name.is_none());
                    assert_eq!(addr, expected_addr);
                }
                PrincipalData::Contract(StandardPrincipalData(_, addr), contract_name) => {
                    assert!(expected_contract_name.is_some());
                    assert_eq!(addr, expected_addr);
                    assert_eq!(contract_name, expected_contract_name.unwrap());
                }
            }

            assert_eq!(parsed_data.memo, expected_memo);
        }
    }

    #[test]
    fn should_fail_on_missing_contract_name_bytes() {
        let addr = generate_address();
        let contract_name = loop {
            let contract_name = generate_contract_name();

            if contract_name.is_some() {
                break contract_name.unwrap();
            }
        };
        let memo: [u8; 15] = thread_rng().gen();

        let mut data = build_deposit_data(&addr, Some(&contract_name), &memo);
        data[22..62].iter_mut().for_each(|byte| *byte = 0);

        assert!(parse(&data).is_err());
    }

    #[test]
    fn should_fail_on_incomplete_contract_name_bytes() {
        let addr = generate_address();
        let contract_name = loop {
            let contract_name = generate_contract_name();

            if contract_name.is_some() {
                break contract_name.unwrap();
            }
        };
        let memo: [u8; 15] = thread_rng().gen();

        let mut data = build_deposit_data(&addr, Some(&contract_name), &memo);
        data[22 + contract_name.len() - 1..62]
            .iter_mut()
            .for_each(|byte| *byte = 0);

        assert!(parse(&data).is_err());
    }

    #[test]
    fn should_truncate_on_extra_contract_name_bytes() {
        let addr = generate_address();
        let expected_contract_name = loop {
            let maybe_contract_name = generate_contract_name();

            if let Some(contract_name) = maybe_contract_name {
                if contract_name.len() < 40 {
                    break contract_name;
                }
            }
        };
        let memo: [u8; 15] = thread_rng().gen();

        let mut data = build_deposit_data(&addr, Some(&expected_contract_name), &memo);
        data[22 + expected_contract_name.len() + 1] = b'X';

        let parsed_data = parse(&data).unwrap();

        match parsed_data.recipient {
            PrincipalData::Contract(_, contract_name) => {
                assert_eq!(contract_name, expected_contract_name)
            }
            PrincipalData::Standard(_) => panic!("Should be a contract principal"),
        }
    }

    #[test]
    fn should_ignore_contract_name_bytes_if_size_zero() {
        let expected_addr = generate_address();
        let expected_memo: [u8; 15] = thread_rng().gen();
        let mut data = build_deposit_data(&expected_addr, None, &expected_memo);

        data[22..62]
            .iter_mut()
            .for_each(|byte| *byte = thread_rng().gen());

        let parsed_data = parse(&data).unwrap();

        match parsed_data.recipient {
            PrincipalData::Standard(StandardPrincipalData(_, addr)) => {
                assert_eq!(addr, expected_addr);
            }
            PrincipalData::Contract(_, _) => panic!("Should be a standard principal"),
        }

        assert_eq!(parsed_data.memo, expected_memo);
    }
}
