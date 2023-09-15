(define-constant test-contract-principal (as-contract tx-sender))
(define-constant zero-address 'SP000000000000000000002Q6VF78)

(define-public (add-burnchain-block-header-hash (burn-height uint) (header (buff 80)))
  (contract-call? .clarity-bitcoin-mini debug-insert-burn-header-hash (contract-call? .clarity-bitcoin-mini reverse-buff32 (sha256 (sha256 header))) burn-height)
)

(define-public (prepare)
	(begin
        ;; 1
        (unwrap-panic (add-burnchain-block-header-hash u807525 0x00001733539613a8d931b08f0d3f746879572a6d3e12623b16d20000000000000000000034f521522fba52c2c5c75609d261b490ee620661319ab23c68f24d756ff4ced801230265ae32051718d9aadb))
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

;; @name check valid verify-merkle-proof
(define-public (test-verify-merkle-proof-pass)
    (let (
        (raw-wtx 0x0200000000010218f905443202116524547142bd55b69335dfc4e4c66ff3afaaaab6267b557c4b030000000000000000e0dbdf1039321ab7a2626ca5458e766c6107690b1a1923e075c4f691cc4928ac0000000000000000000220a10700000000002200208730dbfaa29c49f00312812aa12a62335113909711deb8da5ecedd14688188363c5f26010000000022512036f4ff452cb82e505436e73d0a8b630041b71e037e5997290ba1fe0ae7f4d8d50140a50417be5a056f63e052294cb20643f83038d5cd90e2f90c1ad3f80180026cb99d78cd4480fadbbc5b9cad5fb2248828fb21549e7cb3f7dbd7aefd2d541bd34f0140acde555b7689eae41d5ccf872bb32a270893bdaa1defc828b76c282f6c87fc387d7d4343c5f7288cfd9aa5da0765c7740ca97e44a0205a1abafa279b530d5fe36d182500)
        (hash-wtx-le 0x04117dc370c45b8a44bf86a3ae4fa8d0b186b5b27d50939cda7501723fa12ec6)
        (merkle-root 0x15424423c2614c23aceec8d732b5330c21ff3a306f52243fbeef47a192c65c86)
        (proof {
            tx-index: u3,
            hashes: (list 0xb2d7ec769ce60ebc0c8fb9cc37f0ad7481690fc176b82c8d17d3c05da80fea6b 0x122f3217765b6e8f3163f6725d4aa3d303e4ffe4b99a5e85fb4ff91a026c17a8),
            tree-depth: u2})
        )

        (ok (contract-call? .clarity-bitcoin-mini verify-merkle-proof
            hash-wtx-le
            merkle-root
            proof)
        )
    )
)

;; @name check valid verify-merkle-proof-2
(define-public (test-verify-merkle-proof-pass-2)
    (let (
        (raw-wtx 0x02000000000101abb563d92ef886a675d41613c5cab151eac8aa7dc4168781daa8f0beb227b4a40100000000ffffffff02d2040000000000001600145c74e4e48fe9dd1ed747676cb9e43e40aeb398669210000000000000160014a43b692f36cb5ab3657580c96daf80f82fd5b1c70247304402204c9c4f0c4da81aea6dc9d4efa0f3c93881c0443b71362388586634edbfe7e26d02203b5c593a4e2d45f92bb8c5f469887b07cd4a06b2d2f0ee9c5587c2f71c446951012103eb16b2ab368f56f92ae70f24edc57c40519feade64d55029acecbba4ffca9b9800000000)
        (hash-wtx-le 0x3755aa59e30e045e8e9ccc31ff8c4cbc445f804a329287073d27eed965b9e4cb)
        (merkle-root 0x15424423c2614c23aceec8d732b5330c21ff3a306f52243fbeef47a192c65c86)
        (proof {
            tx-index: u3,
            hashes: (list 0xb2d7ec769ce60ebc0c8fb9cc37f0ad7481690fc176b82c8d17d3c05da80fea6b 0x122f3217765b6e8f3163f6725d4aa3d303e4ffe4b99a5e85fb4ff91a026c17a8),
            tree-depth: u2})
        )

        (ok (contract-call? .clarity-bitcoin-mini verify-merkle-proof
            hash-wtx-le
            merkle-root
            proof)
        )
    )
)

;; @name check incorrect valid verify-merkle-proof
(define-public (test-incorrect-verify-merkle-proof-pass)
    (let (
        (raw-wtx 0x0200000000010218f905443202116524547142bd55b69335dfc4e4c66ff3afaaaab6267b557c4b030000000000000000e0dbdf1039321ab7a2626ca5458e766c6107690b1a1923e075c4f691cc4928ac0000000000000000000220a10700000000002200208730dbfaa29c49f00312812aa12a62335113909711deb8da5ecedd14688188363c5f26010000000022512036f4ff452cb82e505436e73d0a8b630041b71e037e5997290ba1fe0ae7f4d8d50140a50417be5a056f63e052294cb20643f83038d5cd90e2f90c1ad3f80180026cb99d78cd4480fadbbc5b9cad5fb2248828fb21549e7cb3f7dbd7aefd2d541bd34f0140acde555b7689eae41d5ccf872bb32a270893bdaa1defc828b76c282f6c87fc387d7d4343c5f7288cfd9aa5da0765c7740ca97e44a0205a1abafa279b530d5fe36d182500)
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