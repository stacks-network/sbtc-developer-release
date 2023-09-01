/*!
Tools for the construction and parsing of the sBTC OP_RETURN deposit transactions.

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
use std::{collections::HashMap, io};

use bdk::{
    bitcoin::{
        psbt::PartiallySignedTransaction, Address as BitcoinAddress, Network, PrivateKey,
        Transaction,
    },
    database::{BatchDatabase, MemoryDatabase},
    SignOptions, Wallet,
};
use stacks_core::{codec::Codec, utils::PrincipalData};

use crate::{
    operations::{
        magic_bytes,
        op_return::utils::{build_op_return_script, reorder_outputs},
        utils::setup_wallet,
        Opcode,
    },
    SBTCError, SBTCResult,
};

/// Builds a complete deposit transaction
pub fn build_deposit_transaction<T: BatchDatabase>(
    wallet: Wallet<T>,
    recipient: PrincipalData,
    dkg_address: BitcoinAddress,
    amount: u64,
    network: Network,
) -> SBTCResult<Transaction> {
    let mut tx_builder = wallet.build_tx();

    let deposit_data = DepositOutputData { network, recipient }.serialize_to_vec();
    let op_return_script = build_op_return_script(&deposit_data);

    let dkg_script = dkg_address.script_pubkey();
    let dust_amount = dkg_script.dust_value().to_sat();

    if amount < dust_amount {
        return Err(SBTCError::AmountInsufficient(amount, dust_amount));
    }

    let outputs = [(op_return_script, 0), (dkg_script, amount)];

    for (script, amount) in outputs.clone() {
        tx_builder.add_recipient(script, amount);
    }

    let (mut partial_tx, _) = tx_builder
        .finish()
        .map_err(|err| SBTCError::BDKError("Could not finish the transaction", err))?;

    partial_tx.unsigned_tx.output = reorder_outputs(partial_tx.unsigned_tx.output, outputs);

    wallet
        .sign(&mut partial_tx, SignOptions::default())
        .map_err(|err| SBTCError::BDKError("Could not sign the transaction", err))?;

    Ok(partial_tx.extract_tx())
}

#[derive(PartialEq, Eq, Debug)]
/// Data for the sBTC OP_RETURN deposit transaction output
pub struct DepositOutputData {
    /// Network to be used for the transaction
    network: Network,
    /// Recipient of the deposit
    recipient: PrincipalData,
}

impl Codec for DepositOutputData {
    fn codec_serialize<W: io::Write>(&self, dest: &mut W) -> io::Result<()> {
        dest.write_all(&magic_bytes(self.network))?;
        dest.write_all(&[Opcode::Deposit as u8])?;
        self.recipient.codec_serialize(dest)
    }

    fn codec_deserialize<R: io::Read>(data: &mut R) -> io::Result<Self>
    where
        Self: Sized,
    {
        let mut magic_bytes_buffer = [0; 2];
        data.read_exact(&mut magic_bytes_buffer)?;

        let network_magic_bytes = [
            Network::Bitcoin,
            Network::Testnet,
            Network::Signet,
            Network::Regtest,
        ]
        .into_iter()
        .map(|network| (magic_bytes(network), network))
        .collect::<HashMap<[u8; 2], Network>>();

        let network = network_magic_bytes
            .get(&magic_bytes_buffer)
            .cloned()
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown magic bytes: {:?}", magic_bytes_buffer),
            ))?;

        let opcode = Opcode::codec_deserialize(data)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        if !matches!(opcode, Opcode::Deposit) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid opcode, expected deposit: {:?}", opcode),
            ));
        }

        let recipient = PrincipalData::codec_deserialize(data)?;

        Ok(Self { network, recipient })
    }
}

fn create_partially_signed_deposit_transaction(
    wallet: &Wallet<MemoryDatabase>,
    recipient: PrincipalData,
    dkg_address: &BitcoinAddress,
    amount: u64,
    network: Network,
) -> SBTCResult<PartiallySignedTransaction> {
    let mut tx_builder = wallet.build_tx();

    let deposit_data = DepositOutputData { network, recipient }.serialize_to_vec();
    let op_return_script = build_op_return_script(&deposit_data);
    let dkg_script = dkg_address.script_pubkey();
    let dust_amount = dkg_script.dust_value().to_sat();

    if amount < dust_amount {
        return Err(SBTCError::AmountInsufficient(amount, dust_amount));
    }

    let outputs = [(op_return_script, 0), (dkg_script, amount)];

    for (script, amount) in outputs.clone() {
        tx_builder.add_recipient(script, amount);
    }

    let (mut partial_tx, _) = tx_builder.finish().map_err(|err| {
        SBTCError::BDKError("Could not finish the partially signed transaction", err)
    })?;

    partial_tx.unsigned_tx.output = reorder_outputs(partial_tx.unsigned_tx.output, outputs);

    Ok(partial_tx)
}

/// Construct a BTC transaction containing the provided sBTC deposit data
pub fn deposit(
    depositor_private_key: PrivateKey,
    recipient: PrincipalData,
    amount: u64,
    dkg_address: &BitcoinAddress,
) -> SBTCResult<Transaction> {
    let wallet = setup_wallet(depositor_private_key)?;

    let mut psbt = create_partially_signed_deposit_transaction(
        &wallet,
        recipient,
        dkg_address,
        amount,
        depositor_private_key.network,
    )?;

    wallet
        .sign(&mut psbt, SignOptions::default())
        .map_err(|err| SBTCError::BDKError("Could not sign transaction", err))?;

    Ok(psbt.extract_tx())
}

#[cfg(test)]
mod tests {
    use rand::{distributions::Alphanumeric, rngs::StdRng, Rng, SeedableRng};
    use stacks_core::{
        address::{AddressVersion, StacksAddress},
        contract_name::{ContractName, CONTRACT_MAX_NAME_LENGTH},
        crypto::generate_keypair,
        utils::{PrincipalData, StandardPrincipalData},
    };

    use super::*;

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

    fn generate_standard_principal_data(rng: &mut impl Rng) -> PrincipalData {
        PrincipalData::Standard(StandardPrincipalData::new(
            AddressVersion::TestnetSingleSig,
            generate_address(rng),
        ))
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
            generate_standard_principal_data(rng)
        } else {
            generate_contract_principal_data(rng)
        }
    }

    #[test]
    fn should_serialize_and_deserialize_deposit_output_data() {
        let mut rng = test_rng();

        for _ in 0..1000 {
            let recipient = generate_principal_data(&mut rng);
            let expected_data = DepositOutputData {
                network: Network::Testnet,
                recipient,
            };

            let serialized_data = expected_data.serialize_to_vec();
            let deserialized_data =
                DepositOutputData::deserialize(&mut serialized_data.as_slice()).unwrap();

            assert_eq!(deserialized_data, expected_data);
        }
    }
}
