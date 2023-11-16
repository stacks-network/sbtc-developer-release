pub mod bitcoin_client;
pub mod deposit;
pub mod stacks_client;
pub mod withdrawal;

use std::ops::Index;

struct KeyRing {
	address: &'static str,
	#[allow(unused)]
	private_key: &'static str,
	#[allow(unused)]
	public_key: &'static str,
	wif: &'static str,
}

#[repr(usize)]
pub(crate) enum KeyType {
	#[allow(unused)]
	P2pkh,
	P2tr,
	P2wpkh,
	Stacks,
}

impl Index<KeyType> for [KeyRing] {
	type Output = KeyRing;

	fn index(&self, index: KeyType) -> &Self::Output {
		&self[index as usize]
	}
}

type FullRing = [KeyRing; 4];

const WALLETS: [FullRing; 3] = [[
	KeyRing {
		address: "n4dN5bVeriVW9gKZMfNqHn21aJkwTM8QPH",
		private_key: "a38cbb2ca77786b9d37fd0feb34df2e423130ec74d0189736bf52561562c9565",
		public_key: "03bcb048737cc2f239db2b3db6eae00263861bfbe5b2577e573e3c32f61a46ac8c",
		wif: "cT4cy7eAKPjhaZ3h72GdpbrtrFzsUQShZEcM5eNCiHrfuzmoXvBt",
	},
	KeyRing {
		address: "bcrt1pte5zmd7qzj4hdu45lh9mmdm0nwq3z35pwnxmzkwld6y0a8g83nnqhj6vc0",
		private_key: "6596d84eef5b73430712dde88fbf6a1d96f97f5f241ab1bf247d04bc241dd28d",
		public_key: "034a45bd09cc815da165b8987a7263a2c4111b79951562fc5c0989e9cdf5ceded2",
		wif: "cQzBGXC4YACb61oCwxDK9F1a8nxCjUiBZ5rBUaUJAeQvTytUBBFi",
	},
	KeyRing {
		address: "bcrt1q3tj2fr9scwmcw3rq5m6jslva65f2rqjxfrjz47",
		private_key: "bea4ecfec5cfa1e965ee1b3465ca4deff4f04b36a1fb5286a07660d5158789fb",
		public_key: "03ab37f5b606931d7828855affe75199d952bc6174b4a23861b7ac94132210508c",
		wif: "cTyHitzs3VRnNxrpwxo3fXTTe569wHNUs57tQM7Z1FrzUDNB5mqm",
	},
	KeyRing {
		address: "ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM",
		private_key: "753b7cc01a1a2e86221266a154af739463fce51219d97e4f856cd7200c3bd2a6",
		public_key: "0390a5cac7c33fda49f70bc1b0866fa0ba7a9440d9de647fecb8132ceb76a94dfa",
		wif: "cRWawjcDj2J28jczAtjJGKs1pzFxM6V6tJHNZp3WrYoLhr2PLMVB",
	},
],[
    KeyRing {
        address: "mvTQYAcGa17CTxWcJhRPXn6qecBQLSWuaJ",
        private_key: "a1fc751f0bb64c01adb7c60dbe966e2bb9e262aa23bf41158e64c2142fc4fa78",
        public_key: "030ede1203e7873388f81a7801df5714152c72273507a0fe0609e3f223fb6f56ae",
        wif: "cT1agfN8XrWY1bSW5DGWpXUWYFjrmKWeJHyU79Kk66dPnMC6fw3L"
    },
    KeyRing {
        address: "bcrt1prm3lfhsgnnxe0def39n7xpa9etqrjfdxeqnar9xegqu4c044td0sfktyxq",
        private_key: "73f560df660fb11c9aff8178971acd67e75ecb4e5683f5e9bdc52fb3c967c7a3",
        public_key: "02aaed53527e3771645a050568a3cc9820361899c36f689cac15b57cc7885f3ca1",
        wif: "cRU7KinnTuwaJ9N6WFDg2YdhtKcU5vxTSKFtgmT2gTtFvNfhnXx9"
    },
    KeyRing {
        address: "bcrt1q3zl64vadtuh3vnsuhdgv6pm93n82ye8q6cr4ch",
        private_key: "1ec64b686cf94a4d8c741ed34db074b86d91c0971a38fe6e161b402489d7a74e",
        public_key: "03969ff3e2bf7f2f73dc903cd11442032c8c7811d57d96ce327ee89c9edea63fa8",
        wif: "cNcXK2r8bNdWJQymtAW8tGS7QHNtFFvG5CdXqhhT752u29WspXRM"
    },
    KeyRing {
        address: "ST2ST2H80NP5C9SPR4ENJ1Z9CDM9PKAJVPYWPQZ50",
        private_key: "6a7c24ee77649c0cc314864596a6bd1addf3efb93bd63bcdb99be08437420847",
        public_key: "038386f533650ff82714eeac9438faaa8a20ada5dd68a7eb8e00cf46cab5325a68",
        wif: "cR9hENRFiuHzKpj9B3QCTBrt19c5ZCJKHJwYcqj5dfB6aKyf6ndm"
    }

],[
    KeyRing {
        address: "n3cR74zFVWNnEusWTWKvDyuCemjT6zVv2y",
        private_key: "9f1e7c24320af2b6c26b977e0eac0d19b69444b5a00b6a4ceca9849dcfa0e1d8",
        public_key: "02d1b831b466e71161bfb91c7933483e9414f14435fddfdf37c7e41b78f657a880",
        wif: "cSv1SKPZijZm3Kjip78csAiRq5JpQCCJLTumi9CCMqJzEDe2DNji"
        },
        KeyRing{
          address: "bcrt1p3cq25gmqaltumf3l9d9e6qz836s3nu7vvjsnjvvkudaucly3h4fqq63tug",
          private_key: "39657a54a708a1f2df728c40612aac7605c093daefb4552dceebfebe06aef1c8",
          public_key: "02d3f0669085642a8cb94d574fd1cdc74ae0bc01b07c9572d4cd5f32b1e622d378",
          wif: "cPWGmk3RD9FWqt3MJN78zGKf3AzpUXmbmvof4x9Gggh54DnFUByL"
        },
        KeyRing{
          address: "bcrt1q266gk7s8efpwl0nasamcmc627tm37wnzxmgugt",
          private_key: "66f8f1682915abb46f6a669ada600ade92e04739ce6e005fdde34d57ce64d40d",
          public_key: "02a6510b8cf31689d9fd51c3237f0e81bf53201072bb9b16e34f86108566465aa6",
          wif: "cR2sDUprEWkAQra7hY832dSjPhuc5eTbpJr8KnvBKBNtAJH1pvMJ"
      },
      KeyRing {
        address: "ST2Y2SFNVZBT8SSZ00XXKH930MCN0RFREB2GQG7CJ",
        private_key: "6703304161a59dc3369c650ae97cca299df8bebb5638f12d4ff69778cba6ce3a",
        public_key: "0388a0608e9268022ab38bedc8db10e562b6b6672e64dc30191a64f22b9a4a8d4d",
        wif: "cR2wjBsDB1HVZghipz8Lxox2XgK47RURJHF694g8WqDzVd9oRv8u"
    }
]];
