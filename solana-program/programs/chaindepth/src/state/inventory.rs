use anchor_lang::prelude::*;

use crate::errors::ChainDepthError;

pub const MAX_INVENTORY_SLOTS: usize = 64;

pub mod item_ids {
    // Legacy IDs (kept for backward compat with existing inventories)
    pub const LEGACY_ORE: u16 = 1;
    pub const LEGACY_TOOL: u16 = 2;
    pub const LEGACY_BUFF: u16 = 3;

    // ── Wearable Weapons (100-199) ──
    pub const BRONZE_PICKAXE: u16 = 100;
    pub const IRON_PICKAXE: u16 = 101;
    pub const BRONZE_SWORD: u16 = 102;
    pub const IRON_SWORD: u16 = 103;
    pub const DIAMOND_SWORD: u16 = 104;
    pub const NOKIA_3310: u16 = 105;
    pub const WOODEN_PIPE: u16 = 106;
    pub const IRON_SCIMITAR: u16 = 107;
    pub const WOODEN_TANKARD: u16 = 108;

    // ── Lootable Valuables (200-299) ──
    pub const SILVER_COIN: u16 = 200;
    pub const GOLD_COIN: u16 = 201;
    pub const GOLD_BAR: u16 = 202;
    pub const DIAMOND: u16 = 203;
    pub const RUBY: u16 = 204;
    pub const SAPPHIRE: u16 = 205;
    pub const EMERALD: u16 = 206;
    pub const ANCIENT_CROWN: u16 = 207;
    pub const GOBLIN_TOOTH: u16 = 208;
    pub const DRAGON_SCALE: u16 = 209;
    pub const CURSED_AMULET: u16 = 210;
    pub const DUSTY_TOME: u16 = 211;
    pub const ENCHANTED_SCROLL: u16 = 212;
    pub const GOLDEN_CHALICE: u16 = 213;
    pub const SKELETON_KEY: u16 = 214;
    pub const MYSTIC_ORB: u16 = 215;
    pub const RUSTED_COMPASS: u16 = 216;
    pub const DWARF_BEARD_RING: u16 = 217;
    pub const PHOENIX_FEATHER: u16 = 218;
    pub const VOID_SHARD: u16 = 219;

    // ── Consumable Buffs (300-399) ──
    pub const MINOR_BUFF: u16 = 300;
    pub const MAJOR_BUFF: u16 = 301;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct InventoryItem {
    pub item_id: u16,
    pub amount: u32,
    pub durability: u16,
}

#[account]
#[derive(InitSpace)]
pub struct InventoryAccount {
    pub owner: Pubkey,
    #[max_len(MAX_INVENTORY_SLOTS)]
    pub items: Vec<InventoryItem>,
    pub bump: u8,
}

impl InventoryAccount {
    pub const SEED_PREFIX: &'static [u8] = b"inventory";

    pub fn add_item(&mut self, item_id: u16, amount: u32, durability: u16) -> Result<()> {
        require!(item_id > 0, ChainDepthError::InvalidItemId);
        require!(amount > 0, ChainDepthError::InvalidItemAmount);

        if let Some(existing) = self
            .items
            .iter_mut()
            .find(|item| item.item_id == item_id && item.durability == durability)
        {
            existing.amount = existing
                .amount
                .checked_add(amount)
                .ok_or(ChainDepthError::Overflow)?;
            return Ok(());
        }

        require!(
            self.items.len() < MAX_INVENTORY_SLOTS,
            ChainDepthError::InventoryFull
        );

        self.items.push(InventoryItem {
            item_id,
            amount,
            durability,
        });

        Ok(())
    }

    pub fn remove_item(&mut self, item_id: u16, amount: u32) -> Result<()> {
        require!(item_id > 0, ChainDepthError::InvalidItemId);
        require!(amount > 0, ChainDepthError::InvalidItemAmount);

        let mut remaining = amount;
        for item in self.items.iter_mut().filter(|item| item.item_id == item_id) {
            if remaining == 0 {
                break;
            }
            let remove_here = remaining.min(item.amount);
            item.amount = item
                .amount
                .checked_sub(remove_here)
                .ok_or(ChainDepthError::Overflow)?;
            remaining = remaining
                .checked_sub(remove_here)
                .ok_or(ChainDepthError::Overflow)?;
        }

        require!(remaining == 0, ChainDepthError::InsufficientItemAmount);

        self.items.retain(|item| item.amount > 0);
        Ok(())
    }
}

