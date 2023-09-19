/*!
Withdrawal fulfillment is a Bitcoin transaction with the output structure as
below:

1. data output
2. Bitcoin address to send the BTC to

The data output should contain data in the following byte format:

```text
0     2  3                                                                    80
|-----|--|---------------------------------------------------------------------|
 magic op                      withdrawal fulfillment data
```

Where withdrawal fulfillment data should be in the following format:

```text
0                                                                             32
|------------------------------------------------------------------------------|
                                  chain tip
*/

use std::{collections::HashMap, io};

use bdk::{
    bitcoin::{
        psbt::PartiallySignedTransaction, Address as BitcoinAddress, Network as BitcoinNetwork,
        Script, Transaction,
    },
    database::BatchDatabase,
    SignOptions, Wallet,
};
use stacks_core::{codec::Codec, BlockId};

use crate::{
    operations::{magic_bytes, op_return::utils::build_op_return_script, Opcode},
    SBTCError, SBTCResult,
};

use super::utils::reorder_outputs;

/// Construct a withdrawal fulfillment transaction
pub fn build_withdrawal_fulfillment_tx(
    wallet: &Wallet<impl BatchDatabase>,
    stacks_chain_tip: BlockId,
    bitcoin_network: BitcoinNetwork,
    recipient_bitcoin_address: &BitcoinAddress,
    amount: u64,
) -> SBTCResult<Transaction> {
    let mut psbt = create_psbt(
        wallet,
        stacks_chain_tip,
        bitcoin_network,
        recipient_bitcoin_address,
        amount,
    )?;

    wallet
        .sign(&mut psbt, SignOptions::default())
        .map_err(|err| SBTCError::BDKError("Could not sign withdrawal transaction", err))?;

    Ok(psbt.extract_tx())
}

/// Construct a withdrawal fulfillment partially signed transaction
pub fn create_psbt<D: BatchDatabase>(
    wallet: &Wallet<D>,
    stacks_chain_tip: BlockId,
    bitcoin_network: BitcoinNetwork,
    recipient_bitcoin_address: &BitcoinAddress,
    amount: u64,
) -> SBTCResult<PartiallySignedTransaction> {
    let outputs = create_outputs(
        stacks_chain_tip,
        bitcoin_network,
        recipient_bitcoin_address,
        amount,
    )?;

    let mut tx_builder = wallet.build_tx();

    for (script, amount) in outputs.clone() {
        tx_builder.add_recipient(script, amount);
    }

    let (mut partial_tx, _) = tx_builder.finish().map_err(|err| {
        SBTCError::BDKError(
            "Could not build partially signed withdrawal fulfillment transaction",
            err,
        )
    })?;

    partial_tx.unsigned_tx.output = reorder_outputs(partial_tx.unsigned_tx.output, outputs);

    Ok(partial_tx)
}

/// Create the outputs for a withdrawal fulfillment transaction
pub fn create_outputs(
    stacks_chain_tip: BlockId,
    bitcoin_network: BitcoinNetwork,
    recipient_bitcoin_address: &BitcoinAddress,
    amount: u64,
) -> SBTCResult<[(Script, u64); 2]> {
    let data = ParsedWithdrawalFulfillmentData {
        network: bitcoin_network,
        chain_tip: stacks_chain_tip,
    };

    let data_script = build_op_return_script(&data.serialize_to_vec());
    let recipient_script = recipient_bitcoin_address.script_pubkey();

    Ok([(data_script, 0), (recipient_script, amount)])
}

/// Data output for a withdrawal fulfillment transaction
pub struct ParsedWithdrawalFulfillmentData {
    /// The Bitcoin network
    pub network: BitcoinNetwork,

    /// The chain tip block ID
    pub chain_tip: BlockId,
}

impl Codec for ParsedWithdrawalFulfillmentData {
    fn codec_serialize<W: io::Write>(&self, dest: &mut W) -> io::Result<()> {
        dest.write_all(&magic_bytes(self.network))?;
        dest.write_all(&[Opcode::WithdrawalFulfillment as u8])?;
        self.chain_tip.codec_serialize(dest)
    }

    fn codec_deserialize<R: io::Read>(data: &mut R) -> io::Result<Self>
    where
        Self: Sized,
    {
        let mut magic_bytes_buffer = [0; 2];
        data.read_exact(&mut magic_bytes_buffer)?;

        let network_magic_bytes = [
            BitcoinNetwork::Bitcoin,
            BitcoinNetwork::Testnet,
            BitcoinNetwork::Signet,
            BitcoinNetwork::Regtest,
        ]
        .into_iter()
        .map(|network| (magic_bytes(network), network))
        .collect::<HashMap<[u8; 2], BitcoinNetwork>>();

        let network = network_magic_bytes
            .get(&magic_bytes_buffer)
            .cloned()
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown magic bytes: {:?}", magic_bytes_buffer),
            ))?;

        let opcode = Opcode::codec_deserialize(data)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        if !matches!(opcode, Opcode::WithdrawalFulfillment) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid opcode, expected withdrawal fulfillment: {:?}",
                    opcode
                ),
            ));
        }

        let chain_tip = BlockId::codec_deserialize(data)?;

        Ok(Self { network, chain_tip })
    }
}
