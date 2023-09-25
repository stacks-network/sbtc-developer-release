//! Proof Data used in Clarity Contracts
use bdk::bitcoin::{Block, BlockHeader, Transaction, Txid as BitcoinTxId};
use blockstack_lib::vm::types::{
	ListData, ListTypeData, SequenceData, Value, BUFF_32,
};
use rs_merkle::{Hasher, MerkleTree};
use stacks_core::crypto::{sha256::DoubleSha256Hasher, Hashing};
/// The double sha256 algorithm used for bitcoin
#[derive(Clone)]
pub struct DoubleSha256Algorithm {}

impl Hasher for DoubleSha256Algorithm {
	type Hash = [u8; 32];

	fn hash(data: &[u8]) -> [u8; 32] {
		DoubleSha256Hasher::hash(data)
			.as_bytes()
			.try_into()
			.unwrap()
	}
}

/// Data needed to prove that a bitcoin transaction was mined on the bitcoin
/// network. This data is used by clarity contracts.
#[derive(Debug, Clone)]
pub struct ProofData {
	/// The reversed transaction id of the bitcoin transaction
	/// in little endian format
	/// as it is produced by bdk crate.
	/// It is the reversed txid of the one seen in explorers.
	pub reversed_txid: BitcoinTxId,
	/// The index of the bitcoin transaction in the block
	pub tx_index: u32,
	/// The block height of the bitcoin transaction
	pub block_height: u64,
	/// The block hash of the bitcoin transaction
	pub block_header: BlockHeader,
	/// The path of the bitcoin transaction in the merkle tree
	pub merkle_path: Vec<Vec<u8>>,
	/// The depth of the merkle tree
	pub merkle_tree_depth: u32,
	/// merkle root
	pub merkle_root: String,
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
		let tx: &Transaction =
			block.txdata.get(index).expect("Invalid tx index");
		let mut merkle_tree = MerkleTree::<DoubleSha256Algorithm>::new();
		for tx in &block.txdata {
			merkle_tree.insert(tx.txid().to_vec().try_into().unwrap());
		}
		// append last tx id if number of leaves is odd
		if block.txdata.len() % 2 == 1 {
			merkle_tree.insert(
				block
					.txdata
					.last()
					.unwrap()
					.txid()
					.to_vec()
					.try_into()
					.unwrap(),
			);
		}
		merkle_tree.commit();
		let merkle_path = merkle_tree.proof(&[index]);

		// rs_merkle tree depth counts leaves as well
		// we only care about the layers above
		// therefore minus 1.
		let merkle_tree_depth = merkle_tree.depth() - 1;

		Self {
			reversed_txid: tx.txid(),
			tx_index: index as u32,
			block_height: block
				.bip34_block_height()
				.expect("Failed to get block height"),
			block_header: block.header,
			merkle_path: merkle_path
				.proof_hashes()
				.iter()
				.map(|h| h.to_vec())
				.collect(),
			merkle_tree_depth: merkle_tree_depth as u32,
			merkle_root: hex::encode(merkle_tree.root().unwrap()),
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

		// use txid in big endian for clarity call
		let mut txid = self.reversed_txid.to_vec();
		txid.reverse();
		ProofDataClarityValues {
			txid: Value::buff_from(txid)
				.expect("Failed to convert txid to buffer"),
			tx_index: Value::UInt(self.tx_index as u128),
			block_height: Value::UInt(self.block_height as u128),
			block_header: Value::buff_from(header)
				.expect("Failed to convert block header to buffer"),
			merkle_path: Value::Sequence(SequenceData::List(ListData {
				data: self
					.merkle_path
					.iter()
					.map(|v| Value::buff_from(v.clone()).unwrap())
					.collect(),
				type_signature: ListTypeData::new_list(BUFF_32.clone(), 14)
					.unwrap(),
			})),
			merkle_tree_depth: Value::UInt(self.merkle_tree_depth as u128),
		}
	}
}

// test module
#[cfg(test)]
// test from_block returns correct Proof
mod tests {
	use bdk::bitcoin::{consensus::deserialize, hashes::hex::FromHex, Block};

	use super::*;

	#[test]
	fn should_create_correct_proof_data() {
		// testnet block 100,000
		let block_hex = "0200000035ab154183570282ce9afc0b494c9fc6a3cfea05aa8c1add2ecc56490000000038ba3d78e4500a5a7570dbe61960398add4410d278b21cd9708e6d9743f374d544fc055227f1001c29c1ea3b0101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff3703a08601000427f1001c046a510100522cfabe6d6d0000000000000000000068692066726f6d20706f6f6c7365727665726aac1eeeed88ffffffff0100f2052a010000001976a914912e2b234f941f30b18afbb4fa46171214bf66c888ac00000000";
		let block: Block =
			deserialize(&Vec::<u8>::from_hex(block_hex).unwrap()).unwrap();
		let block_height = 100000;
		let hash =
			"00000000009e2958c15ff9290d571bf9459e93b19765c6801ddeccadbb160a1e";
		let txindex: usize = 0;
		let txid =
			"d574f343976d8e70d91cb278d21044dd8a396019e6db70755a0a50e4783dba38";

		let proof_data = ProofData::from_block_and_index(&block, txindex);

		assert_eq!(proof_data.block_height, block_height);
		assert_eq!(proof_data.reversed_txid.to_string(), txid);
		assert_eq!(proof_data.block_header.block_hash().to_string(), hash);
	}

	#[test]
	#[should_panic(expected = "Invalid tx index")]
	fn should_throw_for_invalid_txindex() {
		// testnet block 100,000
		let block_hex = "0200000035ab154183570282ce9afc0b494c9fc6a3cfea05aa8c1add2ecc56490000000038ba3d78e4500a5a7570dbe61960398add4410d278b21cd9708e6d9743f374d544fc055227f1001c29c1ea3b0101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff3703a08601000427f1001c046a510100522cfabe6d6d0000000000000000000068692066726f6d20706f6f6c7365727665726aac1eeeed88ffffffff0100f2052a010000001976a914912e2b234f941f30b18afbb4fa46171214bf66c888ac00000000";
		let block: Block =
			deserialize(&Vec::<u8>::from_hex(block_hex).unwrap()).unwrap();
		let txindex: usize = 1;

		ProofData::from_block_and_index(&block, txindex);
	}

	#[test]
	#[should_panic(
		expected = "called `Result::unwrap()` on an `Err` value: Io(Error { kind: UnexpectedEof, message: \"failed to fill whole buffer\" })"
	)]
	fn should_throw_for_bad_tx() {
		let block_hex =
			"02000000010000000000ffffffff0100000000000000000000000000";
		let block: Block =
			deserialize(&Vec::<u8>::from_hex(block_hex).unwrap()).unwrap();
		let txindex: usize = 0;

		ProofData::from_block_and_index(&block, txindex);
	}

	#[test]
	fn should_convert_to_clarity_values() {
		// testnet block 100,000
		let block_hex = "0200000035ab154183570282ce9afc0b494c9fc6a3cfea05aa8c1add2ecc56490000000038ba3d78e4500a5a7570dbe61960398add4410d278b21cd9708e6d9743f374d544fc055227f1001c29c1ea3b0101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff3703a08601000427f1001c046a510100522cfabe6d6d0000000000000000000068692066726f6d20706f6f6c7365727665726aac1eeeed88ffffffff0100f2052a010000001976a914912e2b234f941f30b18afbb4fa46171214bf66c888ac00000000";
		let block: Block =
			deserialize(&Vec::<u8>::from_hex(block_hex).unwrap()).unwrap();
		let txindex: usize = 0;
		let proof_data = ProofData::from_block_and_index(&block, txindex);
		let values = proof_data.to_values();
		assert_eq!(values.block_header.to_string(), "0x0200000035ab154183570282ce9afc0b494c9fc6a3cfea05aa8c1add2ecc56490000000038ba3d78e4500a5a7570dbe61960398add4410d278b21cd9708e6d9743f374d544fc055227f1001c29c1ea3b");
		assert_eq!(values.block_height.to_string(), "u100000");
		assert_eq!(values.merkle_tree_depth.to_string(), "u1");
		assert_eq!(
            values.merkle_path.to_string(),
            "(0x38ba3d78e4500a5a7570dbe61960398add4410d278b21cd9708e6d9743f374d5)"
        );
	}

	// test from_block_and_index returns correct proof
	// block taken from local regtest node
	#[test]
	fn should_create_correct_merkle_root() {
		let block_hex = "000000205214e3b1be1007826f4537f7d86d8f890104587beae37af2fb17e31195a62325bb8940196d4479391e3460fcc904963da6726ecbb99cb9dfc3705ad9ba748f2182270865ffff7f200000000003020000000001010000000000000000000000000000000000000000000000000000000000000000ffffffff0402d20d00ffffffff029f470000000000001976a914ee9369fb719c0ba43ddf4d94638a970b84775f4788ac0000000000000000266a24aa21a9ed2bec0280b5488f0dd3cca56932fdc3eb7b7fba766f6819a0de1cfeaf74c61ecb0120000000000000000000000000000000000000000000000000000000000000000000000000010000000131bde99ad6d1edb8a0f25b7f8458e605e6c1725217346757e7c9a4c4365ef634030000006b483045022100af3c5c67e972b3c744309e79476d71b8c408ce1e45891b3f4d0a9b8bb76c3a8f0220132e8850d7573747a83ccd0b969f12056b8e960a7e6556f389a87b9be218fc2301210239810ebf35e6f6c26062c99f3e183708d377720617c90a986859ec9c95d00be9fdffffff040000000000000000536a4c5069645b76f4413c41080e57ba4b01a485dc7d2465051bfbd2c97f419ddace3e993f88be7b621278694299a79abc623dd56d071f01245e8648e141bfec88d9ba3b1deef100000dd1000100000ab900014a10270000000000001976a914000000000000000000000000000000000000000088ac10270000000000001976a914000000000000000000000000000000000000000088ac82b0c524010000001976a914ee9369fb719c0ba43ddf4d94638a970b84775f4788ac0000000001000000000101010da73321be48f30562e44ff379ea981e204a4fa4bc859c6cd99418e705c7390000000000feffffff0300000000000000001b6a1969643c051a6d78de7b0625dfbfc16c3a8a5735f6dc3dc3f2cee8030000000000002251205e682db7c014ab76f2b4fdcbbdb76f9b8111468174cdb159df6e88fe9d078ce6ab040000000000001600148ae4a48cb0c3b7874460a6f5287d9dd512a182460247304402206387c555478eb821311ef4d3b125a8b4beb698be624e186ff6234f6cd1deb75702207cf063c9cd57dcd7c34b9477129a3a70403856a46be7b9e8942d79482b246379012103ab37f5b606931d7828855affe75199d952bc6174b4a23861b7ac94132210508cc10d0000";
		let block: Block =
			deserialize(&Vec::<u8>::from_hex(block_hex).unwrap()).unwrap();
		let txindex: usize = 0;
		let proof_data = ProofData::from_block_and_index(&block, txindex);
		let values = proof_data.to_values();
		assert_eq!(values.block_header.to_string(), "0x000000205214e3b1be1007826f4537f7d86d8f890104587beae37af2fb17e31195a62325bb8940196d4479391e3460fcc904963da6726ecbb99cb9dfc3705ad9ba748f2182270865ffff7f2000000000");
		assert_eq!(values.block_height.to_string(), "u3538");
		assert_eq!(values.merkle_tree_depth.to_string(), "u2");
		assert_eq!(values.merkle_path.to_string(), "(0x30955a1f27461b4ca06d68147a377a585d05499d186853a2e05e21cf4f9bf55f 0xb4a7cc817198247161027ab3584b0c6a1bd2f7319d6468d2c6e128ec3acb2a47)");
		assert_eq!(
            values.txid.to_string(),
            "0xd564f1a4e53e7bad92f67c9a05b748e504ac1b8155db4c2d9b4ed12afd32139f"
        )
	}
}
