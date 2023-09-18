(define-constant test-contract-principal (as-contract tx-sender))
(define-constant zero-address 'SP000000000000000000002Q6VF78)

(define-public (add-burnchain-block-header-hash (burn-height uint) (header (buff 80)))
  (contract-call? .clarity-bitcoin-mini debug-insert-burn-header-hash (contract-call? .clarity-bitcoin-mini reverse-buff32 (sha256 (sha256 header))) burn-height)
)

(define-public (prepare)
	(begin
        ;; 1
        (unwrap-panic (add-burnchain-block-header-hash u807525 0x00001733539613a8d931b08f0d3f746879572a6d3e12623b16d20000000000000000000034f521522fba52c2c5c75609d261b490ee620661319ab23c68f24d756ff4ced801230265ae32051718d9aadb))
        (unwrap-panic (add-burnchain-block-header-hash u2431087 0x0000a02065bc9201b5b5a1d695a18e4d5efe5d52d8ccc4129a2499141d000000000000009160ba7ae5f29f9632dc0cd89f466ee64e2dddfde737a40808ddc147cd82406f18b8486488a127199842cec7))
        (ok true)))

;; @name check verify-block-header
(define-public (test-verify-block-header)
    (let (
        (burnchain-block-height u807525)
        ;; block id: 0000000000000000000104cf6179ece70fe28f1f6a24126e8d8c91d42d5eafb4
        (raw-block-header 0x00001733539613a8d931b08f0d3f746879572a6d3e12623b16d20000000000000000000034f521522fba52c2c5c75609d261b490ee620661319ab23c68f24d756ff4ced801230265ae32051718d9aadb))
        ;; prepare
            (ok (contract-call? .clarity-bitcoin-mini verify-block-header
                0x00001733539613a8d931b08f0d3f746879572a6d3e12623b16d20000000000000000000034f521522fba52c2c5c75609d261b490ee620661319ab23c68f24d756ff4ced801230265ae32051718d9aadb
                burnchain-block-height))))

;; @name check valid verify-merkle-proof-1
(define-public (test-verify-merkle-proof-pass)
    (let (
        ;; hash256 wtx raw hex of tx 3b3a7a31c949048fabf759e670a55ffd5b9472a12e748b684db5d264b6852084
        (hash-wtx-le 0x04117dc370c45b8a44bf86a3ae4fa8d0b186b5b27d50939cda7501723fa12ec6)
        ;; witness merkle root found in coinbase op_return (below concatenated with 32-bytes of 0x00 then hash256)
        (merkle-root 0x15424423c2614c23aceec8d732b5330c21ff3a306f52243fbeef47a192c65c86)
        ;; witness proof
        (proof {
            tx-index: u3,
            hashes: (list 0xb2d7ec769ce60ebc0c8fb9cc37f0ad7481690fc176b82c8d17d3c05da80fea6b 0x122f3217765b6e8f3163f6725d4aa3d303e4ffe4b99a5e85fb4ff91a026c17a8),
            tree-depth: u2}))

            (ok (asserts! (is-ok (contract-call? .clarity-bitcoin-mini verify-merkle-proof
                hash-wtx-le
                merkle-root
                proof)) (err u5)))
    )
)

;; @name check valid verify-merkle-proof-2
(define-public (test-verify-merkle-proof-pass-2)
    (let (
        ;; hash256 wtx raw hex of tx 0ba1d831915fb5d9bbd4c19665ebf12157ee6aaf1d81e054b667bb5539e77bd8
        (hash-wtx-le 0x073782910dca06f17d09828a116b1b991e1255d1da2f541a465e681e11ebf32b)
        ;; witness merkle root found in coinbase op_return (below concatenated with 32-bytes of 0x00 then hash256)
        (merkle-root 0x73cbac43da0bffeb8161d9da16f9106ea0d13f6c29b4fb031a61282d1b7f4255)
        (proof {
            tx-index: u2,
            hashes: (list 0x1bd75b05fec22e8436fa721115f5a8843ba3d2d17640df9653303b186c6a3701 0x073782910dca06f17d09828a116b1b991e1255d1da2f541a465e681e11ebf32b),
            tree-depth: u2}))

            (ok (contract-call? .clarity-bitcoin-mini verify-merkle-proof
                hash-wtx-le
                merkle-root
                proof)
            )
    )
)

;; @name check valid verify-merkle-proof-3
(define-public (test-verify-merkle-proof-pass-3)
    (let (
        ;; hash256 wtx raw hex of tx c519ca51369814a3277c4dd381757b0e49e52d5250ffb374d6283b99ec5ae875
        (hash-wtx-le 0x29c233c7a7b9237cc201b570af1013dd051f79f3fb39e73731afe58c3bcce09c)
        ;; witness merkle root found in coinbase op_return (below concatenated with 32-bytes of 0x00 then hash256)
        (merkle-root 0xdf3e48b1d8b0c86358205f8b40e149ba040ef0d6bf556e29169e369887686b02)
        (proof {
            tx-index: u2,
            hashes: (list 0x29c233c7a7b9237cc201b570af1013dd051f79f3fb39e73731afe58c3bcce09c 0xf18cd848313c7f988409dc15555ab4301aa9237ddbac3552ac01700c9c5fd454),
            tree-depth: u2}))

            (contract-call? .clarity-bitcoin-mini verify-merkle-proof
                hash-wtx-le
                merkle-root
                proof)
    )
)

;; @name check incorrect valid verify-merkle-proof
(define-public (test-incorrect-verify-merkle-proof-pass)
    (let (
        (hash-wtx-le 0x04117dc370c45b8a44bf86a3ae4fa8d0b186b5b27d50939cda7501723fa12ec6)
        (hash-wtx-be 0xc62ea13f720175da9c93507db2b586b1d0a84faea386bf448a5bc470c37d1104)
        (merkle-root 0x15424423c2614c23aceec8d732b5330c21ff3a306f52243fbeef47a192c65c86)
        (proof {
            tx-index: u3,
            hashes: (list 0xb2d7ec769ce60ebc0c8fb9cc37f0ad7481690fc176b82c8d17d3c05da80fea6b 0x122f3217765b6e8f3163f6725d4aa3d303e4ffe4b99a5e85fb4ff91a026c17a8),
            tree-depth: u2})
        )

        (ok (is-err (contract-call? .clarity-bitcoin-mini verify-merkle-proof
            hash-wtx-be
            merkle-root
            proof))
        )
    )
)