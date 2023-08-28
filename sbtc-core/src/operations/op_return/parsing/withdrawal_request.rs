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

use stacks_core::StacksError;

use crate::SBTCResult;

/// A message signature
pub struct MessageSignature(pub [u8; 65]);

impl MessageSignature {
    /// Creates a new message signature from a slice of bytes
    pub fn new(bytes: [u8; 65]) -> Self {
        Self(bytes)
    }
}

impl TryFrom<&[u8]> for MessageSignature {
    type Error = StacksError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}

/// The parsed data from a withdrawal request transaction
pub struct ParsedWithdrawalRequestData {
    /// The amount to withdraw
    pub amount: u64,
    /// The signature
    pub signature: MessageSignature,
    /// The memo
    pub memo: Vec<u8>,
}

/// Parses the subset of the data output from a deposit transaction. First 3 bytes need to be removed.
pub fn parse(data: &[u8]) -> SBTCResult<ParsedWithdrawalRequestData> {
    if data.len() < 73 {
        return Err(crate::SBTCError::MalformedData(
            "Withdrawal request data should contain at least 73 bytes",
        ));
    }

    let amount = u64::from_be_bytes(data[0..8].try_into().unwrap());
    let signature: MessageSignature = data[8..73].try_into().unwrap();
    let memo = data.get(73..).unwrap_or(&[]).to_vec();

    Ok(ParsedWithdrawalRequestData {
        amount,
        signature,
        memo,
    })
}
