;; @contract stateless contract to verify bitcoin transaction, mini edition

;; it can only check if a txid is part of a burn chain block

(define-constant DEBUG-MODE true)

;; Error codes
(define-constant ERR-HEADER-HEIGHT-MISMATCH (err u6))
(define-constant ERR-INVALID-MERKLE-PROOF (err u7))
(define-constant ERR-PROOF-TOO-SHORT (err u8))
(define-constant ERR-INVALID-BLOCK-HEADER-LENGTH (err u9))


(define-constant block-header-merkle-root-start u36)
(define-constant block-header-merkle-root-end u68)

(define-map debug-burn-header-hashes uint (buff 32))

;; #[allow(unchecked_data)]
(define-public (debug-insert-burn-header-hash (header-hash (buff 32)) (burn-height uint))
	(ok (and DEBUG-MODE (map-set debug-burn-header-hashes burn-height header-hash))))

(define-private (reverse-buff16 (input (buff 16)))
	(unwrap-panic (slice? (unwrap-panic (to-consensus-buff? (buff-to-uint-le input))) u1 u17)))

(define-read-only (reverse-buff32 (input (buff 32)))
	(unwrap-panic (as-max-len? (concat
		(reverse-buff16 (unwrap-panic (as-max-len? (unwrap-panic (slice? input u16 u32)) u16)))
		(reverse-buff16 (unwrap-panic (as-max-len? (unwrap-panic (slice? input u0 u16)) u16)))) u32)))

(define-read-only (get-burn-block-header-hash (burn-height uint))
	(if DEBUG-MODE (map-get? debug-burn-header-hashes burn-height) (get-burn-block-info? header-hash burn-height)))


;; Verify that a block header hashes to a burnchain header hash at a given height.
;; Returns true if so; false if not.
(define-read-only (verify-block-header (header (buff 80)) (expected-block-height uint))
	(is-eq (get-burn-block-header-hash expected-block-height) (some (reverse-buff32 (sha256 (sha256 header))))))

;; Determine if the ith bit in a uint is set to 1
(define-read-only (is-bit-set (val uint) (bit uint))
	(> (bit-and val (bit-shift-left u1 bit)) u0))

;; Verify the next step of a Merkle proof.
;; This hashes cur-hash against the ctr-th hash in proof-hashes, and uses that as the next cur-hash.
;; The path is a bitfield describing the walk from the txid up to the merkle root:
;; * if the ith bit is 0, then cur-hash is hashed before the next proof-hash (cur-hash is "left").
;; * if the ith bit is 1, then the next proof-hash is hashed before cur-hash (cur-hash is "right").
;; The proof verifies if cur-hash is equal to root-hash, and we're out of proof-hashes to check.
(define-read-only (inner-merkle-proof-verify (ctr uint) (state { path: uint, root-hash: (buff 32), proof-hashes: (list 14 (buff 32)), cur-hash: (buff 32), verified: bool}))
		(if (get verified state)
				state
				(if (>= ctr (len (get proof-hashes state)))
						(merge state { verified: false})
						(let ((path (get path state))
									(is-left (is-bit-set path ctr))
									(proof-hashes (get proof-hashes state))
									(cur-hash (get cur-hash state))
									(root-hash (get root-hash state))

									(h1 (if is-left (unwrap-panic (element-at proof-hashes ctr)) cur-hash))
									(h2 (if is-left cur-hash (unwrap-panic (element-at proof-hashes ctr))))
									(next-hash (sha256 (sha256 (concat h1 h2))))
									(is-verified (and (is-eq (+ u1 ctr) (len proof-hashes)) (is-eq next-hash root-hash))))
						 (merge state { cur-hash: next-hash, verified: is-verified})))))

;; Verify a Merkle proof, given the _reversed_ txid of a transaction, the merkle root of its block, and a proof consisting of:
;; * The index in the block where the transaction can be found (starting from 0),
;; * The list of hashes that link the txid to the merkle root,
;; * The depth of the block's merkle tree (required because Bitcoin does not identify merkle tree nodes as being leaves or intermediates).
;; The _reversed_ txid is required because that's the order (little-endian) processes them in.
;; The tx-index is required because it tells us the left/right traversals we'd make if we were walking down the tree from root to transaction,
;; and is thus used to deduce the order in which to hash the intermediate hashes with one another to link the txid to the merkle root.
;; Returns (ok true) if the proof is valid.
;; Returns (ok false) if the proof is invalid.
;; Returns (err ERR-PROOF-TOO-SHORT) if the proof's hashes aren't long enough to link the txid to the merkle root.
(define-read-only (verify-merkle-proof (reversed-txid (buff 32)) (merkle-root (buff 32)) (proof { tx-index: uint, hashes: (list 14 (buff 32))}))
	(let ((proof-hashes (get hashes proof))
		  (proof-length (len proof-hashes)))
		(get verified
			(fold inner-merkle-proof-verify
					(unwrap-panic (slice? (list u0 u1 u2 u3 u4 u5 u6 u7 u8 u9 u10 u11 u12 u13) u0 proof-length))
					{ path: (+ (pow u2 proof-length) (get tx-index proof)), root-hash: merkle-root, proof-hashes: proof-hashes, cur-hash: reversed-txid, verified: false}))))


(define-read-only (was-txid-mined
	(height uint)
	(txid (buff 32))
	(header (buff 80))
	(proof { tx-index: uint, hashes: (list 14 (buff 32))}))
	(let ((merkle-root-reversed (unwrap-panic (as-max-len? (unwrap! (slice? header block-header-merkle-root-start block-header-merkle-root-end) ERR-INVALID-BLOCK-HEADER-LENGTH) u32))))
		(asserts! (verify-block-header header height) ERR-HEADER-HEIGHT-MISMATCH)
		(ok (asserts! (verify-merkle-proof (reverse-buff32 txid) merkle-root-reversed proof) ERR-INVALID-MERKLE-PROOF))))
