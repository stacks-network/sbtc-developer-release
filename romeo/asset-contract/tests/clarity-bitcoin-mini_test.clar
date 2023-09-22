(define-constant ERR-ERROR-EXPECTED (err u99001))
(define-constant ERR-OK-EXPECTED (err u99002))
(define-constant ERR-HEADER-HEIGHT-MISMATCH (err u6))
(define-constant ERR-INVALID-MERKLE-PROOF (err u7))
(define-constant ERR-PROOF-TOO-SHORT (err u8))

(define-public (add-burnchain-block-header-hash (burn-height uint) (header (buff 80)))
  (contract-call? .clarity-bitcoin-mini debug-insert-burn-header-hash (contract-call? .clarity-bitcoin-mini reverse-buff32 (sha256 (sha256 header))) burn-height)
)

(define-public (prepare)
	(begin
        (unwrap-panic (add-burnchain-block-header-hash u807525 0x00001733539613a8d931b08f0d3f746879572a6d3e12623b16d20000000000000000000034f521522fba52c2c5c75609d261b490ee620661319ab23c68f24d756ff4ced801230265ae32051718d9aadb))
        (unwrap-panic (add-burnchain-block-header-hash u2431087 0x0000a02065bc9201b5b5a1d695a18e4d5efe5d52d8ccc4129a2499141d000000000000009160ba7ae5f29f9632dc0cd89f466ee64e2dddfde737a40808ddc147cd82406f18b8486488a127199842cec7))
        (unwrap-panic (add-burnchain-block-header-hash u2501368 0x00600120df8b67bf9774d6a73cea577aa415b44154dd77c84472106546020000000000002d131ab39fcffcf300e7c9edc133d762c2d7854c753ab59afc5ca4961700559250a70465fcff031ac912deb6))
        (unwrap-panic (add-burnchain-block-header-hash u2501351 0x0060bf24490b72dd2e08d0f14f7ff058412f296c48d87f1717aae3ca8002000000000000854a0d71830a5c8f52f5cee4552f7a61d6996d2dd3d57a452301c41692c61d874da60465fcff031a0a885ddf))
        (ok true)))

;; @name check verify-block-header
(define-public (test-verify-block-header)
    (let (
        (burnchain-block-height u807525)
        ;; block id: 0000000000000000000104cf6179ece70fe28f1f6a24126e8d8c91d42d5eafb4
        (raw-block-header 0x00001733539613a8d931b08f0d3f746879572a6d3e12623b16d20000000000000000000034f521522fba52c2c5c75609d261b490ee620661319ab23c68f24d756ff4ced801230265ae32051718d9aadb))
            
            (asserts! (contract-call? .clarity-bitcoin-mini verify-block-header
                0x00001733539613a8d931b08f0d3f746879572a6d3e12623b16d20000000000000000000034f521522fba52c2c5c75609d261b490ee620661319ab23c68f24d756ff4ced801230265ae32051718d9aadb
                burnchain-block-height) ERR-HEADER-HEIGHT-MISMATCH)

            (ok true)))

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
            tree-depth: u2})
        (proof-response (unwrap! (contract-call? .clarity-bitcoin-mini verify-merkle-proof hash-wtx-le merkle-root proof) ERR-OK-EXPECTED)))

            (asserts! proof-response ERR-INVALID-MERKLE-PROOF)

            (ok true)))

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
            tree-depth: u2})
        (proof-response (unwrap! (contract-call? .clarity-bitcoin-mini verify-merkle-proof hash-wtx-le merkle-root proof) ERR-OK-EXPECTED)))

            (asserts! proof-response ERR-INVALID-MERKLE-PROOF)

            (ok true)))

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
            tree-depth: u2})
        (proof-response (unwrap! (contract-call? .clarity-bitcoin-mini verify-merkle-proof hash-wtx-le merkle-root proof) ERR-OK-EXPECTED)))
            
            (asserts! proof-response ERR-INVALID-MERKLE-PROOF)

            (ok true)))

;; @name check incorrect verify-merkle-proof (too short)
(define-public (test-incorrect-verify-merkle-proof-too-short)
    (let (
        (hash-wtx-le 0x04117dc370c45b8a44bf86a3ae4fa8d0b186b5b27d50939cda7501723fa12ec6)
        (hash-wtx-be 0xc62ea13f720175da9c93507db2b586b1d0a84faea386bf448a5bc470c37d1104)
        (merkle-root 0x15424423c2614c23aceec8d732b5330c21ff3a306f52243fbeef47a192c65c86)
        (proof {
            tx-index: u3,
            hashes: (list 0xb2d7ec769ce60ebc0c8fb9cc37f0ad7481690fc176b82c8d17d3c05da80fea6b),
            tree-depth: u2}    
        )
        (proof-result
            (contract-call? .clarity-bitcoin-mini verify-merkle-proof
            hash-wtx-le
            merkle-root
            proof)))
            (asserts! (is-err proof-result) ERR-ERROR-EXPECTED)
		    (asserts! (is-eq proof-result ERR-PROOF-TOO-SHORT) ERR-ERROR-EXPECTED)
            (ok true)))