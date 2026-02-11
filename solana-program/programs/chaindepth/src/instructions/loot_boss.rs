use anchor_lang::prelude::*;

use crate::errors::ChainDepthError;
use crate::events::{item_types, BossLooted};
use crate::instructions::session_auth::authorize_player_action;
use crate::state::{
    item_ids, session_instruction_bits, BossFightAccount, GlobalAccount, InventoryAccount,
    PlayerAccount, RoomAccount, RoomPresence, SessionAuthority, MAX_LOOTERS, CENTER_BOSS,
};

#[derive(Accounts)]
pub struct LootBoss<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: wallet owner whose gameplay state is being modified
    pub player: UncheckedAccount<'info>,

    #[account(
        seeds = [GlobalAccount::SEED_PREFIX],
        bump = global.bump
    )]
    pub global: Account<'info, GlobalAccount>,

    #[account(
        mut,
        seeds = [PlayerAccount::SEED_PREFIX, player.key().as_ref()],
        bump = player_account.bump
    )]
    pub player_account: Account<'info, PlayerAccount>,

    #[account(
        mut,
        seeds = [
            RoomAccount::SEED_PREFIX,
            &global.season_seed.to_le_bytes(),
            &[player_account.current_room_x as u8],
            &[player_account.current_room_y as u8]
        ],
        bump
    )]
    pub room: Account<'info, RoomAccount>,

    #[account(
        mut,
        seeds = [
            RoomPresence::SEED_PREFIX,
            &global.season_seed.to_le_bytes(),
            &[player_account.current_room_x as u8],
            &[player_account.current_room_y as u8],
            player.key().as_ref()
        ],
        bump = room_presence.bump
    )]
    pub room_presence: Account<'info, RoomPresence>,

    #[account(
        seeds = [BossFightAccount::SEED_PREFIX, room.key().as_ref(), player.key().as_ref()],
        bump = boss_fight.bump
    )]
    pub boss_fight: Account<'info, BossFightAccount>,

    #[account(
        init_if_needed,
        payer = authority,
        space = InventoryAccount::DISCRIMINATOR.len() + InventoryAccount::INIT_SPACE,
        seeds = [InventoryAccount::SEED_PREFIX, player.key().as_ref()],
        bump
    )]
    pub inventory: Account<'info, InventoryAccount>,

    #[account(
        mut,
        seeds = [
            SessionAuthority::SEED_PREFIX,
            player.key().as_ref(),
            authority.key().as_ref()
        ],
        bump = session_authority.bump
    )]
    pub session_authority: Option<Account<'info, SessionAuthority>>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<LootBoss>) -> Result<()> {
    authorize_player_action(
        &ctx.accounts.authority,
        &ctx.accounts.player,
        ctx.accounts.session_authority.as_mut(),
        session_instruction_bits::LOOT_BOSS,
        0,
    )?;

    let room = &mut ctx.accounts.room;
    let player_account = &mut ctx.accounts.player_account;
    let inventory = &mut ctx.accounts.inventory;
    let player_key = ctx.accounts.player.key();
    let clock = Clock::get()?;

    require!(room.center_type == CENTER_BOSS, ChainDepthError::NoBoss);
    require!(room.boss_defeated, ChainDepthError::BossNotDefeated);
    require!(
        player_account.is_at_room(room.x, room.y),
        ChainDepthError::NotInRoom
    );
    require!(!room.has_looted(&player_key), ChainDepthError::AlreadyLooted);
    require!(
        room.looted_by.len() < MAX_LOOTERS,
        ChainDepthError::MaxLootersReached
    );

    room.looted_by.push(player_key);
    player_account.chests_looted += 1;
    ctx.accounts.room_presence.set_idle();

    let loot_hash = generate_loot_hash(clock.slot, &player_key, room.center_id);
    let (item_type, item_amount) = calculate_boss_loot(loot_hash);
    let item_id = map_item_type_to_item_id(item_type, loot_hash);
    let durability = item_durability(item_type, item_id);

    if inventory.owner == Pubkey::default() {
        inventory.owner = player_key;
        inventory.items = Vec::new();
        inventory.bump = ctx.bumps.inventory;
    }
    inventory.add_item(item_id, u32::from(item_amount), durability)?;

    emit!(BossLooted {
        room_x: room.x,
        room_y: room.y,
        player: player_key,
        item_type,
        item_amount,
    });

    Ok(())
}

fn generate_loot_hash(slot: u64, player: &Pubkey, boss_id: u16) -> u64 {
    let player_bytes = player.to_bytes();
    let mut hash = slot
        .wrapping_mul(31)
        .wrapping_add(u64::from(boss_id));
    for chunk in player_bytes.chunks(8) {
        let mut bytes = [0u8; 8];
        bytes[..chunk.len()].copy_from_slice(chunk);
        let value = u64::from_le_bytes(bytes);
        hash = hash.wrapping_mul(31).wrapping_add(value);
    }
    hash
}

fn calculate_boss_loot(hash: u64) -> (u8, u8) {
    let type_roll = hash % 100;
    let item_type = if type_roll < 35 {
        item_types::ORE
    } else if type_roll < 75 {
        item_types::TOOL
    } else {
        item_types::BUFF
    };

    let amount_hash = (hash >> 32) as u8;
    let item_amount = match item_type {
        item_types::ORE => (amount_hash % 8) + 3,
        item_types::TOOL => 1,
        item_types::BUFF => (amount_hash % 5) + 2,
        _ => 1,
    };

    (item_type, item_amount)
}

fn map_item_type_to_item_id(item_type: u8, hash: u64) -> u16 {
    let picker = ((hash >> 16) & 0xFFFF) as usize;
    match item_type {
        item_types::TOOL => {
            // Boss drops: includes rare weapons not found in chests
            const TOOLS: [u16; 9] = [
                item_ids::IRON_PICKAXE,
                item_ids::IRON_SWORD,
                item_ids::DIAMOND_SWORD,
                item_ids::NOKIA_3310,
                item_ids::IRON_SCIMITAR,
                item_ids::BRONZE_SWORD,
                item_ids::BRONZE_PICKAXE,
                item_ids::WOODEN_PIPE,
                item_ids::WOODEN_TANKARD,
            ];
            TOOLS[picker % TOOLS.len()]
        }
        item_types::ORE => {
            // Boss drops: includes rare valuables not found in chests
            const VALUABLES: [u16; 16] = [
                item_ids::GOLD_COIN,
                item_ids::GOLD_BAR,
                item_ids::GOLD_BAR,     // weighted: more common from bosses
                item_ids::DIAMOND,
                item_ids::RUBY,
                item_ids::SAPPHIRE,
                item_ids::EMERALD,
                item_ids::ANCIENT_CROWN,
                item_ids::DRAGON_SCALE,
                item_ids::CURSED_AMULET,
                item_ids::GOLDEN_CHALICE,
                item_ids::MYSTIC_ORB,
                item_ids::PHOENIX_FEATHER,
                item_ids::VOID_SHARD,
                item_ids::SKELETON_KEY,
                item_ids::ENCHANTED_SCROLL,
            ];
            VALUABLES[picker % VALUABLES.len()]
        }
        item_types::BUFF => {
            // Boss drops more major buffs
            const BUFFS: [u16; 3] = [
                item_ids::MINOR_BUFF,
                item_ids::MAJOR_BUFF,
                item_ids::MAJOR_BUFF,   // weighted: better from bosses
            ];
            BUFFS[picker % BUFFS.len()]
        }
        _ => item_ids::GOLD_COIN,
    }
}

fn item_durability(item_type: u8, item_id: u16) -> u16 {
    if item_type == item_types::TOOL {
        match item_id {
            // Bronze tier
            item_ids::BRONZE_PICKAXE | item_ids::BRONZE_SWORD => 80,
            // Iron tier
            item_ids::IRON_PICKAXE | item_ids::IRON_SWORD | item_ids::IRON_SCIMITAR => 120,
            // Diamond tier
            item_ids::DIAMOND_SWORD => 200,
            // Fun / novelty weapons
            item_ids::NOKIA_3310 => 9999,
            item_ids::WOODEN_PIPE | item_ids::WOODEN_TANKARD => 60,
            _ => 100,
        }
    } else {
        0
    }
}
