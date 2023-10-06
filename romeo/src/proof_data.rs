//! Proof Data used in Clarity Contracts
use bdk::bitcoin::{Block, BlockHeader, Transaction, Txid as BitcoinTxId};
use blockstack_lib::vm::types::{
	ListData, ListTypeData, SequenceData, Value, BUFF_32,
};
use rs_merkle::Hasher;
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
}

/// Merkle tree for Bitcoin block transactions
pub struct BitcoinMerkleTree {
	data: Vec<Vec<[u8; 32]>>,
}

impl BitcoinMerkleTree {
	/// Make a new Merkle tree out of the given Bitcoin txids
	pub fn new(txs: &[BitcoinTxId]) -> Self {
		if txs.is_empty() {
			return Self { data: vec![] };
		}

		let mut tree = vec![];

		// fill in leaf hashes
		let mut leaf_hashes = vec![];
		for tx in txs {
			let mut hash_slice = [0u8; 32];
			hash_slice.copy_from_slice(tx);
			leaf_hashes.push(hash_slice);
		}
		// must have an even number of hashes
		if txs.len() % 2 == 1 {
			let last_hash_slice = leaf_hashes
				.last()
				.expect(
					"FATAL: unreachable: non-empty vec does not have `last()`",
				)
				.to_owned();
			leaf_hashes.push(last_hash_slice);
		}

		tree.push(leaf_hashes);

		// calculate parent hashes until we reach the root
		let mut last_row_len =
			tree.last().expect("FATAL: unreachable: empty tree").len();
		loop {
			let mut next_row = vec![];
			let last_row = tree.last().expect("FATAL: unreachable: empty tree");
			for i in 0..(last_row_len / 2) {
				let mut intermediate_preimage = [0u8; 64];
				intermediate_preimage[0..32].copy_from_slice(&last_row[2 * i]);
				intermediate_preimage[32..64]
					.copy_from_slice(&last_row[2 * i + 1]);

				let intermediate_hash =
					DoubleSha256Algorithm::hash(&intermediate_preimage);
				next_row.push(intermediate_hash);
			}

			// reached the root
			if next_row.len() == 1 {
				tree.push(next_row);
				break;
			}

			// have more to go -- this row must have an even number of nodes
			if next_row.len() % 2 == 1 {
				let last_hash_slice = next_row
					.last()
					.expect("FATAL: unreachable: next_row is empty")
					.to_owned();
				next_row.push(last_hash_slice);
			}

			last_row_len = next_row.len();
			tree.push(next_row);
		}

		Self { data: tree }
	}

	/// Get the Merkle root.
	/// It will be None if the tree is empty
	pub fn root(&self) -> Option<[u8; 32]> {
		self.data.last().map(|root_row| root_row[0])
	}

	/// Calculate a merkle proof for a transaction, given its index.
	/// This algorithm uses the following insight: the ith bit in the index
	/// tells us which sibling to use (left or right) at the ith level of the
	/// Merkle tree.
	/// * if ith bit of `index` is 0, then use the _right_ sibling at height i
	/// * if ith bit of `index` is 1, then use the _left_ sibling at height i
	pub fn proof(&self, mut index: usize) -> Option<Vec<[u8; 32]>> {
		if self.data.is_empty() {
			// empty tree
			return None;
		}
		if index >= self.data[0].len() {
			// off the end of the leaf row
			return None;
		}

		let mut proof = vec![];
		for i in 0..(self.data.len() - 1) {
			let sibling = if index % 2 == 0 {
				assert!(
					index + 1 < self.data[i].len(),
					"BUG: {} + 1 >= data[{}].len() ({})",
					index,
					i,
					self.data[i].len()
				);
				self.data[i][index + 1]
			} else {
				assert!(index > 0, "BUG: index == 0");
				self.data[i][index - 1]
			};
			proof.push(sibling);
			index >>= 1;
		}

		Some(proof)
	}

	/// Calculate the tree depth, including leaves.
	/// This value is always greater than 0, unless the tree is empty.
	pub fn depth(&self) -> usize {
		self.data.len()
	}
}

impl ProofData {
	/// Create a new proof from a bitcoin transaction and a block
	pub fn from_block_and_index(block: &Block, index: usize) -> Self {
		let tx: &Transaction =
			block.txdata.get(index).expect("Invalid tx index");

		let txids: Vec<BitcoinTxId> =
			block.txdata.iter().map(|tx| tx.txid()).collect();
		let merkle_tree = BitcoinMerkleTree::new(&txids);
		let merkle_path = merkle_tree
			.proof(index)
			.expect("FATAL: index is out-of-bounds");

		Self {
			reversed_txid: tx.txid(),
			tx_index: index as u32,
			block_height: block
				.bip34_block_height()
				.expect("Failed to get block height"),
			block_header: block.header,
			merkle_path: merkle_path.into_iter().map(|h| h.to_vec()).collect(),
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
		// converted to big endian through DISPLAY_BACKWARDS
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
		assert_eq!(values.txid.to_string(), "0xd574f343976d8e70d91cb278d21044dd8a396019e6db70755a0a50e4783dba38");
		assert_eq!(values.block_header.to_string(), "0x0200000035ab154183570282ce9afc0b494c9fc6a3cfea05aa8c1add2ecc56490000000038ba3d78e4500a5a7570dbe61960398add4410d278b21cd9708e6d9743f374d544fc055227f1001c29c1ea3b");
		assert_eq!(values.block_height.to_string(), "u100000");
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
		assert_eq!(values.merkle_path.to_string(), "(0x30955a1f27461b4ca06d68147a377a585d05499d186853a2e05e21cf4f9bf55f 0xb4a7cc817198247161027ab3584b0c6a1bd2f7319d6468d2c6e128ec3acb2a47)");
		assert_eq!(
            values.txid.to_string(),
            "0xd564f1a4e53e7bad92f67c9a05b748e504ac1b8155db4c2d9b4ed12afd32139f"
        )
	}

	// test from_block_and_index returns correct proof
	// block taken from testnet, failed burn tx
	#[test]
	fn should_create_correct_merkle_proof() {
		let block_hex = "00002020b8a796757a3e087dfdbb0d68d7b74a632579561d5be646f015010000000000003b576e83c8e964e5a56fb443e5b8b10a001e9641328144a28f223ac45acee665802e1d6530b2031a4ddc3ff009020000000001010000000000000000000000000000000000000000000000000000000000000000ffffffff230366982604802e1d654d65726d61696465722046545721010002a5bd9a080000000000ffffffff02dc6e130000000000160014c035e789d9efffa10aa92e93f48f29b8cfb224c20000000000000000266a24aa21a9ed2e42ffd390d39224c48c334444e06a7a83ae954b699bc36ce21b1103ec4959f901200000000000000000000000000000000000000000000000000000000000000000000000000200000001e790e0351515d924bacf2baafec27e6ae51622e3d423be3dfb3df00a3f1f43a4010000006a473044022036de7ebb625c475e1320f44c940e7c25e18abffe18d7b92593505fac3154b7cf02202a165962f6235d45de9f5c1d3a02dd31d2f17919ffd9c4feaac8f4612aad9f0d0121022c7f4e04dea8be8ffc76587c34676b0fa0d3f266dde875d0431c7996e3462695fdffffff027dbc1300000000001976a914546582e3af948c9065d39f00d2bf56ff998b91e288ac1b826e1d010000001976a9145b3c1c6518afdac084750c98b9ccda8520e2c4f088ac65982600010000000182e15c6b31e4871d530ed58c2ed8ac24c2ed9280bca800100106a95bcaee1ada020000006a47304402204e8ae4d5c246e37c95c1806419a9fb3260eaf49790378c4df7f16c55aacef336022059733743e9ac9bc78919bd5459b93528cc3feefebdee4c57c784b13d641ba9690121032f20eae43e911857fdb914fd40806a783a19b05607107c2e514e0b72b24477e2ffffffff01582b0200000000008c21032f20eae43e911857fdb914fd40806a783a19b05607107c2e514e0b72b24477e2ad512102f8dc94efa5016af7cde4f5433d9e46f9ebfc1cfafae2cc949bd2a369b8993da22102605350338e279a0e163b9581c43cccf822dbf45e5affe16ff81cb660a5b1f9372102d53f9790b9d03e7fd65507447db5c0f81b796b58763cb0febf91eed1e4b25f7253ae0000000001000000017ced464f994e79fe75ac19e50980e1f8335fb5a286cd624b0cfb43ba9acacf87030000006a47304402204dbe45d743d027f5362e3d7d53178d70aa9c24594241c407d7067ac7b6f37949022058576523f36b895186dfa971848d2af05110b2923824f4f2be3f4d48f49a69e60121037435c194e9b01b3d7f7a2802d6684a3af68d05bbf4ec8f17021980d777691f1dfdffffff040000000000000000536a4c5054325b76eaa00b1829bcf11d22b8b08747960f8c892c75b76641dc81fb74e7f0f42e0215a88d449445b54513aee65fbc3e71262534434e8853687e665bc5ca1e1356e4002698630002002694df00024910270000000000001976a914000000000000000000000000000000000000000088ac10270000000000001976a914000000000000000000000000000000000000000088ac2dc75801000000001976a914ba27f99e007c7f605a8305e318c1abde3cd220ac88ac000000000200000000010199c60618b12177ef73f14ee1a1d6531884344e7b18bacf3dc2fe8456a26367d90100000017160014ab85a42e84f1734dfcc50321decb751009e3ea3afdffffff0226200000000000002251206b0a1b2a5a618a9abdbd2f2f454a4b412d705290bd950e0fd4d23a523b1c4545df101b000000000017a914be42fa1629963ecab0e2ff1d8bb94273544632ef8702483045022100d25f5e6c4d410166ba170c08ec875448dea19576c8c45f0fcda49bd23683b6e10220202262b92543f73f6d7440a85e6f2e2b7a91077233ea599d21483bb6817b8cea012103c453710ff8121a8e01be0096404077ffab916d545f69adc196e9a8fa723312010000000002000000000101ab72c53b49545d8ada45ea1544e00bc161297bd9cf348546e828368b2505bc5d0200000000ffffffff0300000000000000004f6a4c4c54323e00000000000003e800a5075604a3d6efa3d15ddd1a3ab6db8b57ac037fc1a2207fe5fd6d1e29c772047b9318b30a3f6f4b208bbd84a9521316c8eaf72c0ee91d6f3495e0bb98ba4ecff401000000000000160014764ad6983a6455cca54cd6a4f7b0da71ba6a0baba5caf50500000000160014764ad6983a6455cca54cd6a4f7b0da71ba6a0bab02483045022100966e347c5673df63f78fd316aac2ed0a7e4b8f77e226b55bc5422a955abb65da02207a6509b852079cb4b2ae623d8ae7f0e5b20526c136f5b090fdb1ab522778f9d7012103968e761cb836bfc6711748cf05d093c80621144b1482fea29553492538887e6a0000000001000000000101e57e57dea1958ded04ca010d566ef2bdd791360320914dbb2ee640c2bac975a70100000000ffffffff02e7230000000000001976a9149c4b12bb5a2e7e4b2721a25d8abebd6a8144d41288acd4a1e81100000000160014c783068b2593c7138d8744956f9d048032c580800247304402204d68dfed915eed93158f0221b6bf8ec7778bf93286d6709f74ca9eb718c016aa022026780192ae7bdeee8053cc84d1124aa0a4049972c223ba34eee127b43593e770012103f500418025ba3babca935e9f7617c438210ab72ae3ece0b25e5dff579c31ddd10000000002000000000101006280955059670da318c1811713b9c1398687d15227ee91c0210279c0d8b2ec0100000000fdffffff02401f000000000000160014c67d2be99415528a01d7c8c13000d4ca0eb963fedc0d0e0000000000160014be9257af0584f100e7f16c8a1cf55f32a5aae47602473044022035147e241be86217240618be72b982f31e6873c8f4c8c1824bcea78b1c91238a02204f48e4a7b8022009726f4af42a3ce7ec7c83f0bb7609f5ffa2781defcd7ee2ab012102fc3bd735a715499b5ffa7d96d08f42f5eea78aed455de5bd095606cebdd4594e6598260001000000000101550dae167d4568d1d53e201eb9481348e90fa3086867aaa9a9f293af48d0df9d0100000000ffffffff02e80300000000000016001463c7dec8d97feed8f9e003eca65c8ca26152bea874661100000000001600142481f3daab15b06eeb768af20eb9b64c275dc65c02483045022100937cdd969a1b000a8bacf6549382b7ab8fb7c59dd23332139a03e1d2cfe446af02200569a1a3885058a358ba2f69df31951a1db5000e8f8c3ec407caf165f74da36e0121039a66476dd5fa7a668dc8f540a8fdfa63405baf2491ce907f055137460d0cc2ae65982600";
		let block: Block =
			deserialize(&Vec::<u8>::from_hex(block_hex).unwrap()).unwrap();
		let txindex: usize = 4;
		let proof_data = ProofData::from_block_and_index(&block, txindex);
		let values = proof_data.to_values();
		assert_eq!(
            values.txid.to_string(),
            "0x07268a427a3e0a0618fe94dcf434cd976c0cd29f2b0d645315ec56c4b04393a4"
        );
		assert_eq!(values.block_header.to_string(), "0x00002020b8a796757a3e087dfdbb0d68d7b74a632579561d5be646f015010000000000003b576e83c8e964e5a56fb443e5b8b10a001e9641328144a28f223ac45acee665802e1d6530b2031a4ddc3ff0");
		assert_eq!(values.block_height.to_string(), "u2529382");
		assert_eq!(values.merkle_path.to_string(), "(0xa9db8b2c0b4de3ee6945db550541adcc18852acef9148dc59747a31c9fbf8327 0xde7c38d3e809bcb86fa94695de178e1b27d8d9b6d25a5683b598c36deca50580 0x02f0523e28df15bf268ab52b9a3826d7f933467ea2708c0d7e7d7cd5b2e44892 0x7f37d80a06a9c7d9db4cf14d63e826ecf136b59df3583cb2b94e0a438d3ae506)");
	}

	// test empty merkle tree
	#[test]
	fn should_create_merkle_trees_correctly() {
		let txids0 = vec![];
		let merkle_tree = BitcoinMerkleTree::new(&txids0);
		assert_eq!(merkle_tree.root(), None);
		assert_eq!(merkle_tree.proof(0), None);
	}
}
