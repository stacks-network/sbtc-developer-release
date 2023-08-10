/*!
Withdrawal fullfilment is a transaction with the output structure as below:

1. data output
2. Bitcoin address to send the BTC to

It also uses an the third input from the withdrawal request transaction to pay the fee.

The data output should contain data in the following format:

```text
0      2  3                     35                       80
|------|--|---------------------|------------------------|
 magic  op       Chain tip                  Memo
```
*/

use crate::{SBTCError, SBTCResult};

/// A stacks block ID
pub struct StacksBlockId(pub [u8; 32]);

impl StacksBlockId {
    /// Creates a new StacksBlockId from a slice of bytes
    pub fn new(bytes: &[u8]) -> Option<Self> {
        if bytes.len() == 32 {
            let mut buffer = [0; 32];
            buffer.copy_from_slice(bytes);

            Some(Self(buffer))
        } else {
            None
        }
    }
}

/// The parsed data output from a withdrawal fulfillment transaction
pub struct ParsedWithdrawalFulfillmentData {
    /// The chain tip block ID
    pub chain_tip: StacksBlockId,
    /// The memo
    pub memo: Vec<u8>,
}

/// Parses the subset of the data output from a deposit transaction. First 3 bytes need to be removed.
pub fn parse_data(data: &[u8]) -> SBTCResult<ParsedWithdrawalFulfillmentData> {
    if data.len() < 32 {
        return Err(SBTCError::MalformedData(
            "Withdrawal fulfillment data should be at least 32 bytes long",
        ));
    }

    let chain_tip = StacksBlockId::new(&data[..32])
        .expect("Withdrawalfulfillment chain tip data failed to convert to block ID");
    let memo = data.get(32..).unwrap_or(&[]).to_vec();

    Ok(ParsedWithdrawalFulfillmentData { chain_tip, memo })
}
