(define-constant test-contract-principal (as-contract tx-sender))
(define-constant zero-address 'SP000000000000000000002Q6VF78)

(define-public (add-burnchain-block-header-hash (burn-height uint) (header (buff 80)))
  (contract-call? .clarity-bitcoin-mini debug-insert-burn-header-hash (contract-call? .clarity-bitcoin-mini reverse-buff32 (sha256 (sha256 header))) burn-height)
)

(define-public (prepare)
	(begin
    ;; 1
    (unwrap-panic (add-burnchain-block-header-hash u807525 0x00001733539613a8d931b08f0d3f746879572a6d3e12623b16d20000000000000000000034f521522fba52c2c5c75609d261b490ee620661319ab23c68f24d756ff4ced801230265ae32051718d9aadb))
    (ok true)
    )
)

;; @name verify set block header
(define-public (test-verify-block-header)
    (let (
    (burnchain-block-height u807525)
    ;; block id: 0000000000000000000104cf6179ece70fe28f1f6a24126e8d8c91d42d5eafb4
    (raw-block-header 0x00001733539613a8d931b08f0d3f746879572a6d3e12623b16d20000000000000000000034f521522fba52c2c5c75609d261b490ee620661319ab23c68f24d756ff4ced801230265ae32051718d9aadb)
  )
    ;; prepare
    (ok (contract-call? .clarity-bitcoin-mini verify-block-header
      0x00001733539613a8d931b08f0d3f746879572a6d3e12623b16d20000000000000000000034f521522fba52c2c5c75609d261b490ee620661319ab23c68f24d756ff4ced801230265ae32051718d9aadb
      burnchain-block-height
    ))
  )
)