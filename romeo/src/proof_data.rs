//! Proof Data used in Clarity Contracts
use bdk::bitcoin::{Block, BlockHeader, Transaction, Txid as BitcoinTxId};
use blockstack_lib::util::hash::{DoubleSha256, MerklePath, MerkleTree};
use blockstack_lib::vm::types::{ListData, ListTypeData, SequenceData, Value, BUFF_32};

/// Data needed to prove that a bitcoin transaction was mined on the bitcoin
/// network. This data is used by clarity contracts.
#[derive(Debug, Clone)]
pub struct ProofData {
    /// The transaction id of the bitcoin transaction
    pub txid: BitcoinTxId,
    /// The index of the bitcoin transaction in the block
    pub tx_index: u32,
    /// The block height of the bitcoin transaction
    pub block_height: u64,
    /// The block hash of the bitcoin transaction
    pub block_header: BlockHeader,
    /// The path of the bitcoin transaction in the merkle tree
    pub merkle_path: MerklePath<DoubleSha256>,
    /// The depth of the merkle tree
    pub merkle_tree_depth: u32,
}

/// Clarity values for the proof data
pub struct ProofDataClarityValues {
    /// The transaction id of the bitcoin transaction
    pub txid: Value,
    /// The index of the bitcoin transaction in the block
    pub tx_index: Value,
    /// The block height of the bitcoin transaction
    pub block_height: Value,
    /// The block hash of the bitcoin transaction
    pub block_header: Value,
    /// The path of the bitcoin transaction in the merkle tree
    pub merkle_path: Value,
    /// The depth of the merkle tree
    pub merkle_tree_depth: Value,
}

impl ProofData {
    /// Create a new proof from a bitcoin transaction and a block
    pub fn from_block_and_index(block: &Block, index: usize) -> Self {
        let tx: &Transaction = block.txdata.get(index).expect("Invalid tx index");

        let txid_vecs = block.txdata.iter().map(|tx| tx.txid().to_vec()).collect();

        let merkle_tree = MerkleTree::<DoubleSha256>::new(&txid_vecs);

        let merkle_path = merkle_tree
            .path(&tx.txid())
            .expect("Failed to get merkle path");

        let merkle_tree_depth = merkle_path.len();

        Self {
            txid: tx.txid(),
            tx_index: index as u32,
            block_height: block
                .bip34_block_height()
                .expect("Failed to get block height"),
            block_header: block.header,
            merkle_path,
            merkle_tree_depth: merkle_tree_depth as u32,
        }
    }

    /// converts the proof data to a tuple of clarity values
    pub fn to_values(&self) -> ProofDataClarityValues {
        let mut header = self.block_header.version.to_le_bytes().to_vec();
        header.append(&mut self.block_header.prev_blockhash.to_vec());
        header.append(&mut self.block_header.merkle_root.to_vec());
        header.append(&mut self.block_header.time.to_le_bytes().to_vec());
        header.append(&mut self.block_header.bits.to_le_bytes().to_vec());
        header.append(&mut self.block_header.nonce.to_le_bytes().to_vec());

        ProofDataClarityValues {
            txid: Value::buff_from(self.txid.to_vec()).expect("Failed to convert txid to buffer"),
            tx_index: Value::UInt(self.tx_index as u128),
            block_height: Value::UInt(self.block_height as u128),
            block_header: Value::buff_from(header)
                .expect("Failed to convert block header to buffer"),
            merkle_path: Value::Sequence(SequenceData::List(ListData {
                data: self
                    .merkle_path
                    .iter()
                    .map(|v| Value::buff_from(v.hash.to_bytes().to_vec()).unwrap())
                    .collect(),
                type_signature: ListTypeData::new_list(BUFF_32.clone(), 14).unwrap(),
            })),
            merkle_tree_depth: Value::UInt(self.merkle_tree_depth as u128),
        }
    }
}

// test module
#[cfg(test)]
// test from_block returns correct Proof
mod tests {
    use super::*;
    use bdk::bitcoin::{consensus::deserialize, hashes::hex::FromHex, Block};

    #[test]
    fn should_create_correct_proof_data() {
        // testnet block 100,000
        let block_hex = "0200000035ab154183570282ce9afc0b494c9fc6a3cfea05aa8c1add2ecc56490000000038ba3d78e4500a5a7570dbe61960398add4410d278b21cd9708e6d9743f374d544fc055227f1001c29c1ea3b0101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff3703a08601000427f1001c046a510100522cfabe6d6d0000000000000000000068692066726f6d20706f6f6c7365727665726aac1eeeed88ffffffff0100f2052a010000001976a914912e2b234f941f30b18afbb4fa46171214bf66c888ac00000000";
        let block: Block = deserialize(&Vec::<u8>::from_hex(block_hex).unwrap()).unwrap();
        let block_height = 100000;
        let hash = "00000000009e2958c15ff9290d571bf9459e93b19765c6801ddeccadbb160a1e";
        let txindex: usize = 0;
        let txid = "d574f343976d8e70d91cb278d21044dd8a396019e6db70755a0a50e4783dba38";

        let proof_data = ProofData::from_block_and_index(&block, txindex);

        assert_eq!(proof_data.block_height, block_height);
        assert_eq!(proof_data.txid.to_string(), txid);
        assert_eq!(proof_data.block_header.block_hash().to_string(), hash);
    }

    #[test]
    #[should_panic(expected = "Invalid tx index")]
    fn should_throw_for_invalid_txindex() {
        // testnet block 100,000
        let block_hex = "0200000035ab154183570282ce9afc0b494c9fc6a3cfea05aa8c1add2ecc56490000000038ba3d78e4500a5a7570dbe61960398add4410d278b21cd9708e6d9743f374d544fc055227f1001c29c1ea3b0101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff3703a08601000427f1001c046a510100522cfabe6d6d0000000000000000000068692066726f6d20706f6f6c7365727665726aac1eeeed88ffffffff0100f2052a010000001976a914912e2b234f941f30b18afbb4fa46171214bf66c888ac00000000";
        let block: Block = deserialize(&Vec::<u8>::from_hex(block_hex).unwrap()).unwrap();
        let txindex: usize = 1;

        ProofData::from_block_and_index(&block, txindex);
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: Io(Error { kind: UnexpectedEof, message: \"failed to fill whole buffer\" })"
    )]
    fn should_throw_for_bad_tx() {
        let block_hex = "02000000010000000000ffffffff0100000000000000000000000000";
        let block: Block = deserialize(&Vec::<u8>::from_hex(block_hex).unwrap()).unwrap();
        let txindex: usize = 0;

        ProofData::from_block_and_index(&block, txindex);
    }

    #[test]
    fn should_convert_to_clarity_values() {
        let block_hex = "0200000035ab154183570282ce9afc0b494c9fc6a3cfea05aa8c1add2ecc56490000000038ba3d78e4500a5a7570dbe61960398add4410d278b21cd9708e6d9743f374d544fc055227f1001c29c1ea3b0101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff3703a08601000427f1001c046a510100522cfabe6d6d0000000000000000000068692066726f6d20706f6f6c7365727665726aac1eeeed88ffffffff0100f2052a010000001976a914912e2b234f941f30b18afbb4fa46171214bf66c888ac00000000";
        let block: Block = deserialize(&Vec::<u8>::from_hex(block_hex).unwrap()).unwrap();
        let txindex: usize = 0;
        let proof_data = ProofData::from_block_and_index(&block, txindex);
        let values = proof_data.to_values();
        assert_eq!(values.block_header.to_string(), "0x0200000035ab154183570282ce9afc0b494c9fc6a3cfea05aa8c1add2ecc56490000000038ba3d78e4500a5a7570dbe61960398add4410d278b21cd9708e6d9743f374d544fc055227f1001c29c1ea3b");
        assert_eq!(values.block_height.to_string(), "u100000");
        assert_eq!(
            values.merkle_path.to_string(),
            "(0xd6141b363505039cdb97b4552766872ad925b63be83cbb4bc286fe9970362242)"
        );
        assert_eq!(values.merkle_tree_depth.to_string(), "u1");
    }
}
