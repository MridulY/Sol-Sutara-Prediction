use anchor_lang::prelude::*;
use crate::constants::MAX_MERKLE_DEPTH;

/// Settlement record — created when a keeper submits a Merkle proof
#[account]
pub struct Resolution {
    /// Parent market
    pub market: Pubkey,
    /// TxLINE match ID this proof covers
    pub match_id: [u8; 32],
    /// Merkle root published by TxLINE (must match protocol.txline_merkle_root)
    pub merkle_root: [u8; 32],
    /// Sibling hashes in the proof path (up to MAX_MERKLE_DEPTH levels)
    pub proof: [[u8; 32]; MAX_MERKLE_DEPTH],
    /// Number of valid proof elements
    pub proof_len: u8,
    /// Index of the leaf in the Merkle tree
    pub leaf_index: u64,
    /// Encoded leaf data: hash(match_id || outcome || score || timestamp)
    pub leaf: [u8; 32],
    /// Keeper that submitted this proof
    pub submitted_by: Pubkey,
    /// Unix timestamp of submission
    pub submitted_at: i64,
    /// Whether the CPI validation call has succeeded
    pub validated: bool,
    /// Winning outcome index (set after validation)
    pub winning_outcome: Option<u8>,
    pub bump: u8,
    pub _padding: [u8; 5],
}

impl Resolution {
    pub const LEN: usize = 8                    // discriminator
        + 32                                    // market
        + 32                                    // match_id
        + 32                                    // merkle_root
        + (MAX_MERKLE_DEPTH * 32)               // proof array
        + 1                                     // proof_len
        + 8                                     // leaf_index
        + 32                                    // leaf
        + 32                                    // submitted_by
        + 8                                     // submitted_at
        + 1                                     // validated
        + 2                                     // winning_outcome (Option<u8>)
        + 1                                     // bump
        + 5;                                    // padding
}
