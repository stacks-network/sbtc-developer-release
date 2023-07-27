/*!
Withdrawal request is a transaction with the output structure as below:

1. data output
2. Bitcoin address to send the BTC to
3. Bitcoin fee payment to the peg wallet (which the peg wallet will spend on fulfillment)

The data output should contain data in the following format:

```text
0      2  3         11                76   80
|------|--|---------|-----------------|----|
 magic  op   amount      signature     memo
```
*/

use crate::wireformat::ParseError;

pub struct MessageSignature(pub [u8; 65]);

impl MessageSignature {
    pub fn new(bytes: &[u8]) -> Option<Self> {
        if bytes.len() == 65 {
            let mut buffer = [0; 65];
            buffer.copy_from_slice(bytes);

            Some(Self(buffer))
        } else {
            None
        }
    }
}

pub struct ParsedWithdrawalRequestData {
    pub amount: u64,
    pub signature: MessageSignature,
    pub memo: Vec<u8>,
}

/// Parses the subset of the data output from a deposit transaction. First 3 bytes need to be removed.
pub fn parse(data: &[u8]) -> Result<ParsedWithdrawalRequestData, ParseError> {
    if data.len() < 73 {
        return Err(ParseError::MalformedData(
            "Withdrawal request data should contain at least 73 bytes",
        ));
    }

    let amount = u64::from_be_bytes(data[0..8].try_into().unwrap());
    let signature = MessageSignature::new(&data[8..73]).unwrap();
    let memo = data.get(73..).unwrap_or(&[]).to_vec();

    Ok(ParsedWithdrawalRequestData {
        amount,
        signature,
        memo,
    })
}
