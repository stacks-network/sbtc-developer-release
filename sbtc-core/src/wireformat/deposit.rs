/*!
Deposit is a transaction with the output structure as below:

- output 0, data output
- output 1, payment to peg wallet address

The data output should contain data in the following format:

```
0      2  3                  24                            64       80
|------|--|------------------|-----------------------------|--------|
 magic  op   Stacks address      Contract name (optional)     memo
```
*/
use std::str::from_utf8;

use stacks_core::contract_name::ContractName;
use stacks_core::{PrincipalData, StandardPrincipalData};
use stacks_rs::crypto::Hash160;
use stacks_rs::StacksAddress;

use crate::wireformat::ParseError;

fn find_leading_non_zero_bytes(data: &[u8]) -> Option<&[u8]> {
    match data.iter().rev().position(|&b| b != 0) {
        Some(end) if end != 0 => Some(&data[0..=end]),
        Some(_) | None => None,
    }
}

pub struct ParsedDeposit {
    pub recipient: PrincipalData,
    pub memo: Vec<u8>,
}

/**
Parses the subset of the data output from a deposit transaction. First 3 bytes need to be removed.
*/
pub fn parse(data: &[u8]) -> Result<ParsedDeposit, ParseError> {
    if data.len() < 21 {
        return Err(ParseError::MalformedData(
            "Should contain at least 21 bytes",
        ));
    }

    let standard_principal_data = {
        let version = *data.get(0).expect("No version byte in data");
        let address_data: [u8; 20] = data
            .get(1..21)
            .ok_or(ParseError::MalformedData("Could not get address data"))?
            .try_into()?;

        StandardPrincipalData::new(
            version,
            StacksAddress::new(Hash160::from_slice(&address_data)),
        )
    };

    let recipient = find_leading_non_zero_bytes(&data[21..=61])
        .map(|contract_bytes| {
            let contract_name_string: String = from_utf8(contract_bytes)
                .map_err(|_| ParseError::MalformedData("Could not parse contract name bytes"))?
                .to_owned();
            let contract_name = ContractName::new(&contract_name_string)
                .map_err(|_| ParseError::MalformedData("Could not parse contract name"))?;

            Result::<_, ParseError>::Ok(PrincipalData::Contract(
                standard_principal_data.clone(),
                contract_name,
            ))
        })
        .unwrap_or(Ok(PrincipalData::Standard(standard_principal_data)))?;

    let memo = data.get(61..).unwrap_or(&[]).to_vec();

    Ok(ParsedDeposit { recipient, memo })
}
