;; title: asset
;; version:
;; summary: sBTC dev release asset contract
;; description:

;; traits
;;
(impl-trait 'ST1NXBK3K5YYMD6FD41MVNP3JS1GABZ8TRVX023PT.sip-010-trait-ft-standard.sip-010-trait)

;; token definitions
;;
(define-fungible-token sbtc u21000000000)

;; constants
;;
(define-constant err-invalid-caller u1)
(define-constant err-not-token-owner u2)

;; data vars
;;
(define-data-var contract-owner principal tx-sender)
(define-data-var bitcoin-wallet-public-key (optional (buff 33)) none)

;; public functions
;;
(define-public (set-bitcoin-wallet-public-key (public-key (buff 33)))
    (begin
        (asserts! (is-contract-owner) (err err-invalid-caller))
        (ok (var-set bitcoin-wallet-public-key (some public-key)))
    )
)

(define-public (mint! (amount uint) (dst principal) (deposit-txid (string-ascii 72)))
    (begin
        (asserts! (is-contract-owner) (err err-invalid-caller))
        ;; TODO #79: Assert deposit-txid exists on chain
        (print deposit-txid)
        (ft-mint? sbtc amount dst)
    )
)

(define-public (burn! (amount uint) (src principal) (withdraw-txid (string-ascii 72)))
    (begin
        (asserts! (is-contract-owner) (err err-invalid-caller))
        ;; TODO #79: Assert withdraw-txid exists on chain
        (print withdraw-txid)
        (ft-burn? sbtc amount src)
    )
)

(define-public (transfer (amount uint) (sender principal) (recipient principal) (memo (optional (buff 34))))
	(begin
		(asserts! (is-eq tx-sender sender) (err err-not-token-owner))
		(try! (ft-transfer? sbtc amount sender recipient))
		(match memo to-print (print to-print) 0x)
		(ok true)
	)
)

;; read only functions
;;
(define-read-only (get-bitcoin-wallet-public-key)
    (var-get bitcoin-wallet-public-key)
)

(define-read-only (get-name)
	(ok "sBTC")
)

(define-read-only (get-symbol)
	(ok "sBTC")
)

(define-read-only (get-decimals)
	(ok u8)
)

(define-read-only (get-balance (who principal))
	(ok (ft-get-balance sbtc who))
)

(define-read-only (get-total-supply)
	(ok (ft-get-supply sbtc))
)

(define-read-only (get-token-uri)
	(ok (some u"https://gateway.pinata.cloud/ipfs/QmZWaVN9582UaXvvSgYEZvyafSv4jNdn4a2qpxQCDho85W?_gl=1*7vcfsd*_ga*OTU1OTQzMjE2LjE2OTQwMzk2MjM.*_ga_5RMPXG14TE*MTY5NDAzOTYyNC4xLjEuMTY5NDAzOTczMi40OC4wLjA"))
)

;; private functions
;;
(define-private (is-contract-owner)
    (is-eq (var-get contract-owner) tx-sender)
)
