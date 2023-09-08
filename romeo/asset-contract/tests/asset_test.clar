(define-constant wallet-1 'ST1SJ3DTE5DN7X54YDH5D64R3BCB6A2AG2ZQ8YPD5)
(define-constant wallet-2 'ST2CY5V39NHDPWSXMW9QDT3HC3GD6Q6XX4CFRK9AG)
(define-constant test-mint-amount u10000000)
(define-constant expected-total-supply (* u2 test-mint-amount))
(define-constant expected-token-uri (some u"https://gateway.pinata.cloud/ipfs/Qma5P7LFGQAXt7gzkNZGxet5qJcVxgeXsenDXwu9y45hpr?_gl=1*1mxodt*_ga*OTU1OTQzMjE2LjE2OTQwMzk2MjM.*_ga_5RMPXG14TE*MTY5NDA4MzA3OC40LjEuMTY5NDA4MzQzOC42MC4wLjA"))
(define-constant expected-name "sBTC")
(define-constant expected-symbol "sBTC")
(define-constant expected-decimals u8)

(define-constant err-forbidden (err u403))

(define-private (assert-eq (result (response bool uint)) (compare (response bool uint)) (message (string-ascii 100)))
	(ok (asserts! (is-eq result compare) (err message)))
)

(define-private (assert-eq-string (result (response (string-ascii 32) uint)) (compare (response (string-ascii 32) uint)) (message (string-ascii 100)))
	(ok (asserts! (is-eq result compare) (err message)))
)

(define-private (assert-eq-uint (result (response uint uint)) (compare (response uint uint)) (message (string-ascii 100)))
	(ok (asserts! (is-eq result compare) (err message)))
)

;; Prepare function called for all tests (unless overridden)
(define-public (prepare)
	(begin
		;; Mint some tokens to test principals.
		(try! (contract-call? .asset mint test-mint-amount wallet-1 "a txid 1"))
		(try! (contract-call? .asset mint test-mint-amount wallet-2 "a txid_2"))
		(ok true)
	)
)


;; --- Protocol tests

;; @name Protocol can mint tokens
;; @no-prepare
;; @caller deployer
(define-public (test-protocol-mint)
	(contract-call? .asset mint u10000000 wallet-1 "a txid")
)

;; @name Non-protocol contracts cannot mint tokens
;; @no-prepare
;; @caller wallet_1
(define-public (test-protocol-mint-external)
	(assert-eq (contract-call? .asset mint u10000000 wallet-1 "a txid") err-forbidden "Should have failed")
)

;; @name Protocol can burn tokens
;; @caller deployer
(define-public (test-protocol-burn)
	(contract-call? .asset burn u10000000 wallet-1 "a txid")
)

;; @name Non-protocol contracts cannot burn tokens
;; @caller wallet_1
(define-public (test-protocol-burn-external)
	(assert-eq (contract-call? .asset burn u10000000 wallet-1 "a txid") err-forbidden "Should have failed")
)

;; @name Protocol can set wallet address
;; @no-prepare
;; @caller deployer
(define-public (test-protocol-set-wallet-public-key)
	(contract-call? .asset set-bitcoin-wallet-public-key 0x1234)
)

;; @name Non-protocol contracts cannot mint tokens
;; @no-prepare
;; @caller wallet_1
(define-public (test-protocol-set-wallet-public-key-external)
	(assert-eq (contract-call? .asset set-bitcoin-wallet-public-key 0x1234) err-forbidden "Should have returned err forbidden")
)

;; --- SIP010 tests

;; @name Token owner can transfer their tokens
;; @caller wallet_1
(define-public (test-transfer)
	(contract-call? .asset transfer u100 tx-sender wallet-2 none)
)

;; @name Cannot transfer someone else's tokens
;; @caller wallet_1
(define-public (test-transfer-external)
	(assert-eq (contract-call? .asset transfer u100 wallet-2 tx-sender none) err-forbidden "Should have failed")
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
