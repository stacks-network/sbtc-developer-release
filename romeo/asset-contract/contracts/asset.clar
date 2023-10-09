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
(define-constant err-not-found (err u404))
(define-constant err-btc-tx-already-used (err u500))
(define-constant err-request-id-already-used (err u501))

;; data vars
;;
(define-data-var contract-owner principal tx-sender)
(define-data-var bitcoin-wallet-public-key (optional (buff 33)) none)
(define-data-var last-request-id uint u0)

;; stores all btc txids that have been used to mint or burn sBTC
(define-map amounts-by-btc-tx (buff 32) int)

;; stores all fulfilled withdrawal request that have been used to burn sBTC
(define-map amounts-by-request-id uint int)

;; stores withdrawal requests by tx-sender and request-id
(define-map requests uint {amount: uint, owner: principal, recipient: {hashbytes: (buff 32), version: (buff 1)}})

;; allowed contract-callers
(define-map allowance-contract-callers
    { sender: principal, contract-caller: principal }
    { until-burn-ht: (optional uint) })

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
(define-public (burn-by-request-id (request-id uint))
    (let ((details (unwrap! (map-get? requests request-id) err-not-found))
        (amount (get amount details))
        (owner (get owner details)))
        (try! (is-contract-owner))
        (asserts! (map-insert amounts-by-request-id request-id (* -1 (to-int amount))) err-request-id-already-used)
        (try! (ft-burn? sbtc amount owner))
        (print {notification: "burn", payload: request-id})
    	(ok true)
    )
)

;; #[allow(unchecked_data)]
(define-public (request-withdrawal (amount uint) (recipient {hashbytes: (buff 32), version: (buff 1)}))
    (let ((request-id (+ u1 (var-get last-request-id))))
        (asserts! (or (is-eq tx-sender contract-caller) (check-caller-allowed)) err-invalid-caller)
        (map-set requests request-id {amount: amount, owner: tx-sender, recipient: recipient})
        (var-set last-request-id request-id)
        (print {notification: "request-withdrawal", payload: {amount: amount, recipient: recipient}})
        (ok request-id)
    )
)

;; #[allow(unchecked_data)]
(define-public (cancle-request (request-id uint))
    (let ((details (unwrap! (map-get? requests request-id) err-not-found)))
        (asserts! (or (is-eq tx-sender contract-caller) (check-caller-allowed)) err-invalid-caller)
        (map-delete requests request-id)
        (print {notification: "cancle-request", payload: details})
        (ok request-id)
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

;;
;; contract caller allowance
;;

;; Revoke contract-caller authorization to call withdrawal request methods
;; #[allow(unchecked_data)]
(define-public (disallow-contract-caller (caller principal))
  (begin
    (asserts! (is-eq tx-sender contract-caller)
              err-invalid-caller)
    (ok (map-delete allowance-contract-callers { sender: tx-sender, contract-caller: caller }))))

;; Give a contract-caller authorization to call withdrawal request methods
;;  normally, withdrawal request methods may only be invoked by _direct_ transactions
;;   (i.e., the tx-sender issues a direct contract-call to the request methods)
;;  by issuing an allowance, the tx-sender may call through the allowed contract
;; #[allow(unchecked_data)]
(define-public (allow-contract-caller (caller principal) (until-burn-ht (optional uint)))
  (begin
    (asserts! (is-eq tx-sender contract-caller)
              err-invalid-caller)
    (ok (map-set allowance-contract-callers
               { sender: tx-sender, contract-caller: caller }
               { until-burn-ht: until-burn-ht }))))

;; Get the burn height at which a particular contract is allowed to make withdrawal requests for a particular principal.
;; Returns (some (some X)) if X is the burn height at which the allowance terminates
;; Returns (some none) if the caller is allowed indefinitely
;; Returns none if there is no allowance record
(define-read-only (get-allowance-contract-callers (sender principal) (calling-contract principal))
    (map-get? allowance-contract-callers { sender: sender, contract-caller: calling-contract })
)

(define-read-only (check-caller-allowed)
    (or (is-eq tx-sender contract-caller)
        (let ((caller-allowed
                 ;; if not in the caller map, return false
                 (unwrap! (map-get? allowance-contract-callers
                                    { sender: tx-sender, contract-caller: contract-caller })
                          false))
               (expires-at
                 ;; if until-burn-ht not set, then return true (because no expiry)
                 (unwrap! (get until-burn-ht caller-allowed) true)))
          ;; is the caller allowance expired?
          (if (>= burn-block-height expires-at)
              false
              true))))

;; private functions
;;
(define-private (is-contract-owner)
    (ok (asserts! (is-eq (var-get contract-owner) contract-caller) err-forbidden))
)

(define-read-only (verify-txid-exists-on-burn-chain (txid (buff 32)) (burn-chain-height uint) (merkle-proof (list 14 (buff 32))) (tx-index uint) (block-header (buff 80)))
    (contract-call? .clarity-bitcoin-mini was-txid-mined burn-chain-height txid block-header { tx-index: tx-index, hashes: merkle-proof})
)
