use anchor_lang::prelude::*;
use crate::constants::MAX_KEEPERS;

/// Registry of authorized keeper wallets
#[account]
pub struct KeeperRegistry {
    pub admin: Pubkey,
    /// Authorized keeper public keys (up to MAX_KEEPERS)
    pub keepers: [Pubkey; MAX_KEEPERS],
    /// Number of active keepers
    pub keeper_count: u8,
    pub bump: u8,
    pub _padding: [u8; 6],
}

impl KeeperRegistry {
    pub const LEN: usize = 8               // discriminator
        + 32                               // admin
        + (MAX_KEEPERS * 32)               // keepers
        + 1                                // keeper_count
        + 1                                // bump
        + 6;                               // padding

    pub fn is_authorized(&self, pubkey: &Pubkey) -> bool {
        self.keepers[..self.keeper_count as usize]
            .iter()
            .any(|k| k == pubkey)
    }

    pub fn add_keeper(&mut self, pubkey: Pubkey) -> Result<()> {
        if self.keeper_count as usize >= MAX_KEEPERS {
            return err!(crate::errors::SutaraError::Unauthorized);
        }
        self.keepers[self.keeper_count as usize] = pubkey;
        self.keeper_count = self.keeper_count.checked_add(1).unwrap();
        Ok(())
    }

    pub fn remove_keeper(&mut self, pubkey: &Pubkey) {
        if let Some(idx) = self.keepers[..self.keeper_count as usize]
            .iter()
            .position(|k| k == pubkey)
        {
            let last = (self.keeper_count as usize).saturating_sub(1);
            self.keepers[idx] = self.keepers[last];
            self.keepers[last] = Pubkey::default();
            self.keeper_count = self.keeper_count.saturating_sub(1);
        }
    }
}
