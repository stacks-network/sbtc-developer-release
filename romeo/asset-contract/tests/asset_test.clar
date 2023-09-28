(define-constant wallet-1 'ST1SJ3DTE5DN7X54YDH5D64R3BCB6A2AG2ZQ8YPD5)
(define-constant wallet-2 'ST2CY5V39NHDPWSXMW9QDT3HC3GD6Q6XX4CFRK9AG)
(define-constant test-mint-amount u10000000)
(define-constant expected-total-supply (* u3 test-mint-amount))
(define-constant expected-token-uri (some u"https://assets.stacks.co/sbtc.pdf"))
(define-constant expected-name "sBTC")
(define-constant expected-symbol "sBTC")
(define-constant expected-decimals u8)

(define-constant err-invalid-caller (err u4))
(define-constant err-forbidden (err u403))

(define-constant test-burn-height u1)
(define-constant test-block-header 0x02000000000000000000000000000000000000000000000000000000000000000000000075b8bf903d0153e1463862811283ffbec83f55411c9fa5bd24e4207dee0dc1f1000000000000000000000000)
(define-constant test-block-header-hash 0x346993fc64b2a124a681111bb1f381e24dbef3cd362f0a40019238846c7ebf93)
(define-constant test-txid 0x0168ee41db8a4766efe02bba1ebc0de320bc1b0abb7304f5f104818a9dd721cf)
(define-constant test-tx-index u1)
(define-constant test-tree-depth u1)
(define-constant test-merkle-proof (list 0x582b1900f55dad47d575138e91321c441d174e20a43336780c352a0b556ecc8b))


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
	(ok (asserts! (unwrap-panic (contract-call? .clarity-bitcoin-mini debug-insert-burn-header-hash test-block-header-hash test-burn-height)) (err "Block header hash insert failed")))
)

(define-public (prepare-revoke-contract-owner)
	(contract-call? .asset set-contract-owner 'SP000000000000000000002Q6VF78)
)

;; Prepare function called for all tests (unless overridden)
(define-public (prepare)
	(begin
		;; Mint some tokens to test principals.
		(try! (prepare-insert-header-hash))
		(unwrap! (contract-call? .asset mint u10000000 wallet-1 test-txid u1 test-merkle-proof test-tx-index test-tree-depth test-block-header) (err "Mint to wallet-1 failed"))
		(unwrap! (contract-call? .asset mint u10000000 wallet-2 test-txid u1 test-merkle-proof test-tx-index test-tree-depth test-block-header) (err "Mint to wallet-2 failed"))
		(unwrap! (contract-call? .asset mint u10000000 (as-contract tx-sender) test-txid u1 test-merkle-proof test-tx-index test-tree-depth test-block-header) (err "Mint to asset_test failed"))
		(ok true)
	)
)


;; --- Protocol tests

;; @name Protocol can mint tokens
;; @prepare prepare-insert-header-hash
;; @caller deployer
(define-public (test-protocol-mint)
	(contract-call? .asset mint u10000000 wallet-1 test-txid u1 test-merkle-proof test-tx-index test-tree-depth test-block-header)
)

;; @name Protocol can mint tokens several times with the same bitcoin transaction
;; @prepare prepare-insert-header-hash
;; @caller deployer
(define-public (test-protocol-mint-twice)
	(begin
		(try! (contract-call? .asset mint u10000000 wallet-1 test-txid u1 test-merkle-proof test-tx-index test-tree-depth test-block-header))
		(contract-call? .asset mint u10000000 wallet-1 test-txid u1 test-merkle-proof test-tx-index test-tree-depth test-block-header)
	)
)

;; @name Non-protocol contracts cannot mint tokens
;; @prepare prepare-revoke-contract-owner
;; @caller wallet_1
(define-public (test-protocol-mint-external)
	(assert-eq (contract-call? .asset mint u10000000 wallet-1 test-txid u1 test-merkle-proof test-tx-index test-tree-depth test-block-header) err-forbidden "Should have failed")
)

;; @name Protocol can burn tokens
;; @caller deployer
(define-public (test-protocol-burn)
	(contract-call? .asset burn u10000000 wallet-2 test-txid u1 test-merkle-proof test-tx-index test-tree-depth test-block-header)
)

;; @name Non-protocol contracts cannot burn tokens
;; @prepare prepare-revoke-contract-owner
;; @caller wallet_1
(define-public (test-protocol-burn-external)
	(assert-eq (contract-call? .asset burn u10000000 wallet-2 test-txid u1 test-merkle-proof test-tx-index test-tree-depth test-block-header) err-forbidden "Should have failed")
)

;; @name Protocol can set wallet address
;; @no-prepare
;; @caller deployer
(define-public (test-protocol-set-wallet-public-key)
	(contract-call? .asset set-bitcoin-wallet-public-key 0x1234)
)

;; @name Non-protocol contracts cannot set wallet public key
;; @prepare prepare-revoke-contract-owner
;; @caller wallet_1
(define-public (test-protocol-set-wallet-public-key-external)
	(assert-eq (contract-call? .asset set-bitcoin-wallet-public-key 0x1234) err-forbidden "Should have returned err forbidden")
)

;; --- SIP010 tests

;; @name Token owner can transfer their tokens
;; @caller wallet_1
(define-public (test-transfer)
	(contract-call? .asset transfer u100 contract-caller wallet-2 none)
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
