;; sbtc-send-many
;; sends sbtc to many users

(define-constant err-not-found (err u404))
(define-constant err-invalid-amount (err u500))

(define-data-var last-request-id uint u0)
(define-map requests {owner:principal, request-id: uint}
	(list 200 {to: principal, sbtc-in-sats: uint, memo: (buff 34)}))

;; send xbtc in sats to single recipient
(define-public (transfer-sbtc-with-memo (sats uint) (to principal) (memo (buff 34)))
	(as-contract (contract-call?
		.asset
		transfer sats tx-sender to (some memo))))

;; send sbtc to single recipient
;; depending on their preference sbtc is swapped to stx via alex swap
(define-private (send-sbtc (recipient {to: principal, sbtc-in-sats: uint, memo: (buff 34)}))
	(transfer-sbtc-with-memo
		(get sbtc-in-sats recipient)
		(get to recipient)
		(get memo recipient)))

(define-read-only (sum (recipient {to: principal, sbtc-in-sats: uint, memo: (buff 34)}) (total uint))
	(+ total (get sbtc-in-sats recipient)))

;; request to send sbtc to many recipients
(define-public (request-send-sbtc-many (recipients (list 200 {to: principal, sbtc-in-sats: uint, memo: (buff 34)})))
	(let ((request-id (+ u1 (var-get last-request-id)))
		(total (fold sum recipients u0)))
		(asserts! (> total u0) err-invalid-amount)
		(var-set last-request-id request-id)
	 	(map-set requests {owner: tx-sender, request-id: request-id} recipients)
		(print {notification: "request", request-id: request-id, totoal: total})
		(ok {request-id: request-id, total: total})))

;; fullfill the request
(define-public (fulfill-send-request (request-id uint))
	(let ((request-key {owner: tx-sender, request-id: request-id})
		  (recipients (unwrap! (map-get? requests request-key) err-not-found))
		  (result (fold check-err
			(map send-sbtc recipients)
			(ok true))))
		(map-delete requests request-key)
		(print {notification: "fulfillment", request-id: request-id})
		result))

(define-private (check-err (result (response bool uint)) (prior (response bool uint)))
	(match prior
	  ok-value result
	  err-value (err err-value)))

(define-read-only (get-last-request-id)
	(var-get last-request-id))