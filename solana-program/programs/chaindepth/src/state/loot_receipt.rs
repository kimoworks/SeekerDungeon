use anchor_lang::prelude::*;

/// Per-player loot receipt for a specific room.
/// Existence of this PDA proves the player has already looted the chest.
/// PDA seeds: ["loot_receipt", season_seed (8 bytes), room_x (1 byte), room_y (1 byte), player_pubkey]
#[account]
#[derive(InitSpace)]
pub struct LootReceipt {
    pub player: Pubkey,
    pub season_seed: u64,
    pub room_x: i8,
    pub room_y: i8,
    pub bump: u8,
}

impl LootReceipt {
    pub const SEED_PREFIX: &'static [u8] = b"loot_receipt";
}
