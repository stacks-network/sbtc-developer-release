(define-constant wallet-1 'ST1SJ3DTE5DN7X54YDH5D64R3BCB6A2AG2ZQ8YPD5)
(define-constant wallet-2 'ST2CY5V39NHDPWSXMW9QDT3HC3GD6Q6XX4CFRK9AG)
(define-constant wallet-3 'ST2JHG361ZXG51QTKY2NQCVBPPRRE2KZB1HR05NNC)

(define-constant test-mint-amount u10000000)

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
	(begin
		(asserts! (unwrap-panic (contract-call? .clarity-bitcoin-mini debug-insert-burn-header-hash test-block-header-hash test-burn-height)) (err u9999))
		(ok true)

	)
)

;; Prepare function called for all tests (unless overridden)
(define-public (prepare)
	(begin
		;; Mint some tokens to test principals.
		(try! (prepare-insert-header-hash))
		(unwrap! (contract-call? .asset mint u10000000 wallet-1 test-txid test-burn-height test-merkle-proof test-tx-index test-tree-depth test-block-header) (contract-call? .asset mint u10000000 wallet-1 test-txid test-burn-height test-merkle-proof test-tx-index test-tree-depth test-block-header))
		(ok true)
	)
)


;; @name User can request to send sbtc to many and fulfill the request
;; @caller wallet_1
(define-public (test-send-many)
    (begin
        (try! (contract-call? .sbtc-send-many request-send-sbtc-many 
            (list {to: wallet-2, sbtc-in-sats: u2222, memo: 0x}
             {to: wallet-3, sbtc-in-sats: u3333, memo: 0x})))
        (try! (contract-call? .asset transfer u5555 tx-sender .sbtc-send-many none))
        (try! (contract-call? .sbtc-send-many fulfill-send-request u1))
        (let (
            (balance-contract (contract-call? .asset get-balance .sbtc-send-many))
            (balance-wallet-2 (contract-call? .asset get-balance wallet-2))
            (balance-wallet-3 (contract-call? .asset get-balance wallet-3)))
            (asserts! (is-eq balance-contract (ok u0)) (err u991))
            (asserts! (is-eq balance-wallet-2 (ok u2222)) (err u992))
            (asserts! (is-eq balance-wallet-3 (ok u3333)) (err u993))
            (ok true)
        )
    )
)             