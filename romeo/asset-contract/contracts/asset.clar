;; title: wrapped BTC on Stacks
;; version: 0.1.0
;; summary: sBTC dev release asset contract
;; description: sBTC is a wrapped BTC asset on Stacks.
;; It is a fungible token (SIP-10) that is backed 1:1 by BTC
;; For this version the wallet is controlled by a centralized entity.
;; sBTC is minted when BTC is deposited into the wallet and
;; burned when BTC is withdrawn from the wallet.
;; Requests for minting and burning are made by the contract owner.

;; token definitions
;; 100 M sats = 1 sBTC
;; 21 M sBTC supply = 2.1 Q sats total
(define-fungible-token sbtc u2100000000000000)

;; constants
;;
(define-constant err-invalid-caller (err u4))
(define-constant err-forbidden (err u403))
(define-constant err-btc-tx-already-used (err u500))

;; data vars
;;
(define-data-var contract-owner principal tx-sender)
(define-data-var bitcoin-wallet-public-key (optional (buff 33)) none)

;; stores all btc txids that have been used to mint or burn sBTC
(define-map amounts-by-btc-tx (buff 32) int)

;; public functions
;;

;; #[allow(unchecked_data)]
(define-public (set-contract-owner (new-owner principal))
  (begin
    (try! (is-contract-owner))
    (ok (var-set contract-owner new-owner))
  )
)

;; #[allow(unchecked_data)]
(define-public (set-bitcoin-wallet-public-key (public-key (buff 33)))
    (begin
        (try! (is-contract-owner))
        (ok (var-set bitcoin-wallet-public-key (some public-key)))
    )
)

;; #[allow(unchecked_data)]
(define-public (mint (amount uint)
    (destination principal)
    (deposit-txid (buff 32))
    (burn-chain-height uint)
    (merkle-proof (list 14 (buff 32)))
    (tx-index uint)
    (block-header (buff 80)))
    (begin
        (try! (is-contract-owner))
        (try! (verify-txid-exists-on-burn-chain deposit-txid burn-chain-height merkle-proof tx-index block-header))
        (asserts! (map-insert amounts-by-btc-tx deposit-txid (to-int amount)) err-btc-tx-already-used)
        (try! (ft-mint? sbtc amount destination))
        (print {notification: "mint", payload: deposit-txid})
        (ok true)
    )
)

;; #[allow(unchecked_data)]
(define-public (burn (amount uint)
    (owner principal)
    (withdraw-txid (buff 32))
    (burn-chain-height uint)
    (merkle-proof (list 14 (buff 32)))
    (tx-index uint)
    (block-header (buff 80)))
    (begin
        (try! (is-contract-owner))
        (try! (verify-txid-exists-on-burn-chain withdraw-txid burn-chain-height merkle-proof tx-index block-header))
        (asserts! (map-insert amounts-by-btc-tx withdraw-txid (* -1 (to-int amount))) err-btc-tx-already-used)
        (try! (ft-burn? sbtc amount owner))
        (print {notification: "burn", payload: withdraw-txid})
    	(ok true)
    )
)

;; #[allow(unchecked_data)]
(define-public (transfer (amount uint) (sender principal) (recipient principal) (memo (optional (buff 34))))
	(begin
        (asserts! (or (is-eq tx-sender sender) (is-eq contract-caller sender)) err-invalid-caller)
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

(define-read-only (get-contract-owner)
    (var-get contract-owner)
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
	(ok (some u"https://gateway.pinata.cloud/ipfs/Qma5P7LFGQAXt7gzkNZGxet5qJcVxgeXsenDXwu9y45hpr?_gl=1*1mxodt*_ga*OTU1OTQzMjE2LjE2OTQwMzk2MjM.*_ga_5RMPXG14TE*MTY5NDA4MzA3OC40LjEuMTY5NDA4MzQzOC42MC4wLjA"))
)

(define-read-only (get-amount-by-btc-txid (btc-txid (buff 32)))
    (map-get? amounts-by-btc-tx btc-txid)
)

;; private functions
;;
(define-private (is-contract-owner)
    (ok (asserts! (is-eq (var-get contract-owner) contract-caller) err-forbidden))
)

(define-read-only (verify-txid-exists-on-burn-chain (txid (buff 32)) (burn-chain-height uint) (merkle-proof (list 14 (buff 32))) (tx-index uint) (block-header (buff 80)))
    (contract-call? .clarity-bitcoin-mini was-txid-mined burn-chain-height txid block-header { tx-index: tx-index, hashes: merkle-proof})
)
