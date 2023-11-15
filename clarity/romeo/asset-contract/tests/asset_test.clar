(define-constant wallet-1 'ST1SJ3DTE5DN7X54YDH5D64R3BCB6A2AG2ZQ8YPD5)
(define-constant wallet-2 'ST2CY5V39NHDPWSXMW9QDT3HC3GD6Q6XX4CFRK9AG)
(define-constant test-mint-amount u10000000)
(define-constant expected-total-supply (* u3 test-mint-amount))
(define-constant expected-token-uri (some u"https://gateway.pinata.cloud/ipfs/Qma5P7LFGQAXt7gzkNZGxet5qJcVxgeXsenDXwu9y45hpr?_gl=1*1mxodt*_ga*OTU1OTQzMjE2LjE2OTQwMzk2MjM.*_ga_5RMPXG14TE*MTY5NDA4MzA3OC40LjEuMTY5NDA4MzQzOC42MC4wLjA"))
(define-constant expected-name "sBTC")
(define-constant expected-symbol "sBTC")
(define-constant expected-decimals u8)

(define-constant err-invalid-caller (err u4))
(define-constant err-forbidden (err u403))
(define-constant err-btc-tx-already-used (err u500))

(define-constant test-burn-height u1)
(define-constant test-block-header 0x02000000000000000000000000000000000000000000000000000000000000000000000075b8bf903d0153e1463862811283ffbec83f55411c9fa5bd24e4207dee0dc1f1000000000000000000000000)
(define-constant test-block-header-hash 0x346993fc64b2a124a681111bb1f381e24dbef3cd362f0a40019238846c7ebf93)
(define-constant test-txid 0x0168ee41db8a4766efe02bba1ebc0de320bc1b0abb7304f5f104818a9dd721cf)
(define-constant test-tx-index u1)
(define-constant test-merkle-proof (list 0x582b1900f55dad47d575138e91321c441d174e20a43336780c352a0b556ecc8b))

;; testnet block 2
(define-constant test-burn-height-2 u2)
(define-constant test-block-header-2 0x020000002840bc6c31378c0a314609fb50f21811c5370f7df387b30d109d620000000000a9858cc9be942ea7459f026b09e3c25287706bc3d0d9ba2d59d8ea39168c6ce72400065227f1001c4a0c9887)
(define-constant test-block-header-hash-2 0x0000000000977ac3d9f5261efc88a3c2d25af92a91350750d00ad67744fa8d03)
(define-constant test-txid-2 0x3fe5373efdada483b5fa7bdf2249d8274f1b8c04ab5a98bce3edfb732d8e2f86)
(define-constant test-tx-index-2 u1)
(define-constant test-merkle-proof-2 (list 0xf823d9b527ee0427d95d355d03e95bd30639cda85bcf65050ca32bd7d11ee1f7))

;; used to mint for contract
(define-constant test-burn-height-3 u3)
(define-constant test-block-header-3 0x00000000000000000000000000000000000000000000000000000000000000000000000019f249885791b56b8b24ddb8d625522a6fb42629a56cad300da6213a808005b2000000000000000000000000)
(define-constant test-block-header-hash-3 0x2ee611e1b02c558e838f531b6fa3e33dd66747ca57532bee2be4efd9f3d85292)
(define-constant test-txid-3 0x6801cb417573220564c3cec34dd39a0879e24ea75a7ca1ba6a3b8c11c1c6c6b3)
(define-constant test-tx-index-3 u1)
(define-constant test-merkle-proof-3 (list 0xcfd12454d95e1f7af0f7e183862b5adf8bf7be7414c7afb0c549f360b960471d))

;; used to burn
(define-constant test-burn-height-4 u4)
(define-constant test-block-header-4 0x00000000000000000000000000000000000000000000000000000000000000000000000041918a58bf1189a6eaeb80766ddea620363cf3f4e4a93ab101b0bb3b08e5eed1000000000000000000000000)
(define-constant test-block-header-hash-4 0xf83d0cb1e608b3402de733bd655a644714309801edb151ae274e0ccb03cfc981)
(define-constant test-txid-4 0x2d1f657e970f724c4cd690494152a83bd297cd10e86ed930daa2dd76576d974c)
(define-constant test-tx-index-4 u1)
(define-constant test-merkle-proof-4 (list 0xe79e0239e1f9d0f23613387a4831495fd5179943419bcad3a2c20c6931ab1abc))

(define-private (assert-eq (result (response bool uint)) (compare (response bool uint)) (message (string-ascii 100)))
	(ok (asserts! (is-eq result compare) (err message)))
)

(define-private (assert-eq-string (result (response (string-ascii 32) uint)) (compare (response (string-ascii 32) uint)) (message (string-ascii 100)))
	(ok (asserts! (is-eq result compare) (err message)))
)

(define-private (assert-eq-uint (result (response uint uint)) (compare (response uint uint)) (message (string-ascii 100)))
	(ok (asserts! (is-eq result compare) (err message)))
)

(define-public (prepare-insert-header-hash)
	(begin
		(asserts! (unwrap-panic (contract-call? .clarity-bitcoin-mini debug-insert-burn-header-hash test-block-header-hash test-burn-height)) (err "Block header hash insert failed"))
		(asserts! (unwrap-panic (contract-call? .clarity-bitcoin-mini debug-insert-burn-header-hash test-block-header-hash-2 test-burn-height-2)) (err "Block header hash-2 insert failed"))
		(asserts! (unwrap-panic (contract-call? .clarity-bitcoin-mini debug-insert-burn-header-hash test-block-header-hash-3 test-burn-height-3)) (err "Block header hash-3 insert failed"))
		(asserts! (unwrap-panic (contract-call? .clarity-bitcoin-mini debug-insert-burn-header-hash test-block-header-hash-4 test-burn-height-4)) (err "Block header hash-4 insert failed"))
		(ok true)

	)
)

(define-public (prepare-revoke-contract-owner)
	(contract-call? .asset set-contract-owner 'SP000000000000000000002Q6VF78)
)

;; Prepare function called for all tests (unless overridden)
(define-public (prepare)
	(begin
		;; Mint some tokens to test principals.
		(try! (prepare-insert-header-hash))
		(unwrap! (contract-call? .asset mint u10000000 wallet-1 test-txid test-burn-height test-merkle-proof test-tx-index test-block-header) (err "Mint to wallet-1 failed"))
		(unwrap! (contract-call? .asset mint u10000000 wallet-2 test-txid-2 test-burn-height-2 test-merkle-proof-2 test-tx-index-2 test-block-header-2) (err "Mint to wallet-2 failed"))
		(unwrap! (contract-call? .asset mint u10000000 (as-contract tx-sender) test-txid-3 test-burn-height-3 test-merkle-proof-3 test-tx-index-3 test-block-header-3) (err "Mint to asset_test failed"))
		(ok true)
	)
)


;; --- Protocol tests

;; @name Protocol can mint tokens
;; @prepare prepare-insert-header-hash
;; @caller deployer
(define-public (test-protocol-mint)
	(contract-call? .asset mint u10000000 wallet-1 test-txid u1 test-merkle-proof test-tx-index test-block-header)
)

;; @name Protocol can mint tokens several times with the same bitcoin transaction
;; @prepare prepare-insert-header-hash
;; @caller deployer
(define-public (test-protocol-mint-twice)
	(begin
		(unwrap! (contract-call? .asset mint u10000000 wallet-1 test-txid u1 test-merkle-proof test-tx-index test-block-header) (err "Should succeed"))
		(assert-eq (contract-call? .asset mint u10000000 wallet-1 test-txid u1 test-merkle-proof test-tx-index test-block-header) err-btc-tx-already-used "Should have failed with err-btc-tx-already-used")
	)
)

;; @name Non-protocol contracts cannot mint tokens
;; @prepare prepare-revoke-contract-owner
;; @caller wallet_1
(define-public (test-protocol-mint-external)
	(assert-eq (contract-call? .asset mint u10000000 wallet-1 test-txid u1 test-merkle-proof test-tx-index test-block-header) err-forbidden "Should have failed")
)

;; @name Protocol can burn tokens
;; @caller deployer
(define-public (test-protocol-burn)
	(contract-call? .asset burn u10000000 wallet-2 test-txid-4 test-burn-height-4 test-merkle-proof-4 test-tx-index-4 test-block-header-4)
)

;; @name Non-protocol contracts cannot burn tokens
;; @prepare prepare-revoke-contract-owner
;; @caller wallet_1
(define-public (test-protocol-burn-external)
	(assert-eq (contract-call? .asset burn u10000000 wallet-2 test-txid test-burn-height test-merkle-proof test-tx-index test-block-header) err-forbidden "Should have failed")
)

;; @name Protocol can burn tokens with the same bitcoin transaction only once
;; @caller deployer
(define-public (test-protocol-burn-twice)
	(begin
		(unwrap! (contract-call? .asset burn u10000000 wallet-2 test-txid-4 test-burn-height-4 test-merkle-proof-4 test-tx-index-4 test-block-header-4) (err "Should succeed"))
		(assert-eq (contract-call? .asset burn u10000000 wallet-2 test-txid-4 test-burn-height-4 test-merkle-proof-4 test-tx-index-4 test-block-header-4) err-btc-tx-already-used "Should have failed with err-btc-tx-already-used")
	)
)

;; @name Protocol cannot burn tokens than owned by wallet
;; @caller deployer
(define-public (test-protocol-burn-max-amount)
	(assert-eq (contract-call? .asset burn u10000001 wallet-2 test-txid-4 test-burn-height-4 test-merkle-proof-4 test-tx-index-4 test-block-header-4) (err u1) "Should have failed with err-btc-tx-already-used")
)

;; @name Protocol can set wallet address
;; @no-prepare
;; @caller deployer
(define-public (test-protocol-set-wallet-public-key)
	(begin 
		(asserts! (is-eq (contract-call? .asset get-bitcoin-wallet-public-key) none) (err "Public key should be none"))
		(try! (assert-eq (contract-call? .asset set-bitcoin-wallet-public-key 0x1234) (ok true) "Should have succeeded"))
		(asserts! (is-eq (contract-call? .asset get-bitcoin-wallet-public-key) (some 0x1234)) (err "Public key should be 0x1234"))
		(ok true)
	)
)

;; @name Non-protocol contracts cannot set wallet public key
;; @prepare prepare-revoke-contract-owner
;; @caller wallet_1
(define-public (test-protocol-set-wallet-public-key-external)
	(assert-eq (contract-call? .asset set-bitcoin-wallet-public-key 0x1234) err-forbidden "Should have returned err forbidden")
)

;; @name Amounts can be retrieved by btc transaction id
(define-public (test-get-amounts-by-txid)
	(begin
	 	(asserts! (is-eq (contract-call? .asset get-amount-by-btc-txid test-txid) (some 10000000)) (err "Amounts do not match"))
		(asserts! (is-eq (contract-call? .asset get-amount-by-btc-txid test-txid-2) (some 10000000)) (err "Amounts do not match"))
		(asserts! (is-eq (contract-call? .asset get-amount-by-btc-txid 0x1234) none) (err "Amount should be none"))
		(ok true)
	)
)

;; --- SIP010 tests

;; @name Token owner can transfer their tokens
;; @caller wallet_1
(define-public (test-transfer)
	(contract-call? .asset transfer u100 contract-caller wallet-2 none)
)

;; @name Token owner can transfer their tokens with memo
;; @caller wallet_1
(define-public (test-transfer-with-memo)
	(contract-call? .asset transfer u200 contract-caller wallet-2 (some 0x1357))
)

;; @name User can transfer tokens owned by contract
;; @caller wallet_1
(define-public (test-transfer-contract)
	(as-contract (distribute-tokens u100 tx-sender wallet-2))
)

(define-public (distribute-tokens (amount uint) (from principal) (to principal))
	(contract-call? .asset transfer u100 from to none)
)

;; @name Cannot transfer someone else's tokens
;; @caller deployer
(define-public (test-transfer-external)
	(assert-eq (contract-call? .asset transfer u100 wallet-2 wallet-1 none) err-invalid-caller "Should have failed")
)

;; @name Can get name
(define-public (test-get-name)
	(assert-eq-string (contract-call? .asset get-name) (ok expected-name) "Name does not match")
)

;; @name Can get symbol
(define-public (test-get-symbol)
	(assert-eq-string (contract-call? .asset get-symbol) (ok expected-symbol) "Symbol does not match")
)

;; @name Can get decimals
(define-public (test-get-decimals)
	(assert-eq-uint (contract-call? .asset get-decimals) (ok expected-decimals) "Decimals do not match")
)

;; @name Can user balance
(define-public (test-get-balance)
	(assert-eq-uint (contract-call? .asset get-balance wallet-1) (ok test-mint-amount) "Balance does not match")
)

;; @name Can get total supply
(define-public (test-get-total-supply)
	(assert-eq-uint (contract-call? .asset get-total-supply) (ok expected-total-supply) "Total supply does not match")
)

;; @name Can get token URI
(define-public (test-get-token-uri)
	(ok (asserts! (is-eq (contract-call? .asset get-token-uri) (ok expected-token-uri)) (err "Token uri does not match")))
)

;; @name Set valid new owner
;; @caller deployer
(define-public (test-set-valid-owner)
	(begin
		(try! (contract-call? .asset set-contract-owner 'ST11NJTTKGVT6D1HY4NJRVQWMQM7TVAR091EJ8P2Y))
		;; Check new owner set
		(asserts! (is-eq (contract-call? .asset get-contract-owner) 'ST11NJTTKGVT6D1HY4NJRVQWMQM7TVAR091EJ8P2Y) (err u100))
		(ok true)
	)
)

;; @name Try to set owner without being current owner
;; @prepare prepare-revoke-contract-owner
;; @caller wallet_1
(define-public (test-set-invalid-owner)
	(assert-eq (contract-call? .asset set-contract-owner wallet-2) err-forbidden "Should have failed")
)
