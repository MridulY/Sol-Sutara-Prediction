/// Merkle proof verification for TxLINE sports data attestations.
///
/// TxLINE publishes a Merkle tree after every completed match.
/// Each leaf encodes: hash(match_id ‖ outcome_idx ‖ score_home ‖ score_away ‖ timestamp)
/// The root is stored on-chain in ProtocolConfig.txline_merkle_root.
///
/// Security properties:
///  - Sorted-pair hashing prevents second-preimage attacks
///  - proof_len is bounded by MAX_MERKLE_DEPTH (20) limiting compute usage
///  - root is verified against the protocol-stored root, not a user-supplied value

use anchor_lang::prelude::*;
use solana_program::hash::{hash, hashv};
use crate::errors::SutaraError;
use crate::constants::MAX_MERKLE_DEPTH;

/// Verify a Merkle proof.
///
/// proof:      sibling hashes from leaf to root
/// root:       expected Merkle root (from ProtocolConfig)
/// leaf:       hash of the leaf node (pre-computed by keeper)
/// leaf_index: position of the leaf in the tree (determines left/right at each level)
pub fn verify_merkle_proof(
    proof: &[[u8; 32]],
    root: &[u8; 32],
    leaf: &[u8; 32],
    leaf_index: u64,
    proof_len: u8,
) -> Result<bool> {
    require!(proof_len as usize <= MAX_MERKLE_DEPTH, SutaraError::InvalidMerkleProof);
    require!((proof_len as usize) <= proof.len(), SutaraError::InvalidMerkleProof);

    let mut current = *leaf;
    let mut index = leaf_index;

    for i in 0..proof_len as usize {
        let sibling = &proof[i];
        current = hash_pair(&current, sibling, index % 2 == 0);
        index >>= 1;
    }

    Ok(&current == root)
}

/// Build a leaf hash from match result data.
///
/// leaf = SHA256(match_id ‖ outcome_idx ‖ score_home ‖ score_away ‖ finalized_ts)
pub fn build_leaf(
    match_id: &[u8; 32],
    outcome_idx: u8,
    score_home: u8,
    score_away: u8,
    finalized_ts: i64,
) -> [u8; 32] {
    let ts_bytes = finalized_ts.to_le_bytes();
    hashv(&[
        match_id.as_ref(),
        &[outcome_idx],
        &[score_home],
        &[score_away],
        &ts_bytes,
    ]).to_bytes()
}

/// Sorted-pair hashing: hash(min(a,b) ‖ max(a,b)).
/// Prevents second-preimage attacks on Merkle trees.
fn hash_pair(a: &[u8; 32], b: &[u8; 32], a_is_left: bool) -> [u8; 32] {
    if a_is_left {
        hashv(&[a.as_ref(), b.as_ref()]).to_bytes()
    } else {
        hashv(&[b.as_ref(), a.as_ref()]).to_bytes()
    }
}
