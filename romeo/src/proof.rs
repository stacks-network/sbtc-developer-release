//! Proof
use bdk::bitcoin::{Block, BlockHash, Transaction, Txid as BitcoinTxId};

/// A proof for a bitcoin transaction used by clarity contracts
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Proof {
    /// The transaction id of the bitcoin transaction
    pub txid: BitcoinTxId,
    /// The block height of the bitcoin transaction
    pub block_height: u64,
    /// The block hash of the bitcoin transaction
    pub block_hash: BlockHash,
}

impl Proof {
    /// Create a new proof from a bitcoin transaction and a block
    pub fn from_block_and_index(block: &Block, index: usize) -> Self {
        let tx: &Transaction = block.txdata.get(index).unwrap();
        Self {
            txid: tx.txid(),
            block_height: block
                .bip34_block_height()
                .expect("Failed to get block height"),
            block_hash: block.block_hash(),
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
    fn should_create_correct_proof() {
        // testnet block 100,000
        let block_hex = "0200000035ab154183570282ce9afc0b494c9fc6a3cfea05aa8c1add2ecc56490000000038ba3d78e4500a5a7570dbe61960398add4410d278b21cd9708e6d9743f374d544fc055227f1001c29c1ea3b0101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff3703a08601000427f1001c046a510100522cfabe6d6d0000000000000000000068692066726f6d20706f6f6c7365727665726aac1eeeed88ffffffff0100f2052a010000001976a914912e2b234f941f30b18afbb4fa46171214bf66c888ac00000000";
        let block: Block = deserialize(&Vec::<u8>::from_hex(block_hex).unwrap()).unwrap();
        let block_height = 100000;
        let hash = "00000000009e2958c15ff9290d571bf9459e93b19765c6801ddeccadbb160a1e";
        let txindex: usize = 0;
        let txid = "d574f343976d8e70d91cb278d21044dd8a396019e6db70755a0a50e4783dba38";

        let proof = Proof::from_block_and_index(&block, txindex);

        assert_eq!(proof.block_height, block_height);
        assert_eq!(proof.txid.to_string(), txid);
        assert_eq!(proof.block_hash.to_string(), hash);
    }
}
