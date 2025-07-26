// file: programs/nft_staking/src/lib.rs

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    // CORRECTED: Import `TokenInterface` for the program type
    token_interface::{CloseAccount, Mint, TokenAccount, TokenInterface, Transfer},
};
use mpl_token_metadata::accounts::Metadata;

// After your first successful `anchor build`, paste your new Program ID here.
declare_id!("AEX1smJbH8pgMBL2Hpf6EJnuRaUwBt6NBYP7jVPixAeC");

#[program]
pub mod nft_staking {
    use super::*;

    // --- ADMIN INSTRUCTIONS ---
    // (No changes needed in these instructions)

    pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.admin = ctx.accounts.admin.key();
        pool.reward_mint = ctx.accounts.reward_mint.key();
        pool.reward_vault = ctx.accounts.reward_vault.key();
        pool.allowed_collections = Vec::new();
        pool.total_staked = 0;
        pool.current_epoch = 0;
        pool.last_update_time = Clock::get()?.unix_timestamp;
        pool.epoch_duration = 86400; // 24 hours
        pool.rewards_per_epoch = Vec::new();
        pool.reward_per_nft_stored = 0;
        pool.bump = ctx.bumps.pool;
        pool.start_staking_timestamp = Clock::get()?.unix_timestamp;
        pool.skipped_reward_per_nft = 0;
        pool.last_update_calc_reward_nft_index = 0; 
        pool.staked_counts = Vec::new(); // length = MAX_EPOCHS
        Ok(())
    }

    pub fn add_reward(
        ctx: Context<AddReward>,
        total_reward_amount: u64,
        num_epochs: u64,
    ) -> Result<()> {
        require_gt!(total_reward_amount, 0, ErrorCode::ZeroRewardAmount);
        require_gt!(num_epochs, 0, ErrorCode::ZeroEpochAmount);

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.admin_reward_token_account.to_account_info(),
            to: ctx.accounts.reward_vault.to_account_info(),
            authority: ctx.accounts.admin.to_account_info(),
        };
        anchor_spl::token_interface::transfer(
            CpiContext::new(cpi_program, cpi_accounts),
            total_reward_amount,
        )?;

        let pool = &mut ctx.accounts.pool;
        let reward_per_epoch = total_reward_amount.checked_div(num_epochs).unwrap();
        require!(
            pool.rewards_per_epoch.len() + (num_epochs as usize) <= Pool::MAX_EPOCHS,
            ErrorCode::MaxEpochsExceeded
        );
        for _ in 0..num_epochs {
            pool.rewards_per_epoch.push(reward_per_epoch);
        }

        emit!(RewardAdded {
            funder: ctx.accounts.admin.key(),
            total_amount: total_reward_amount,
            epochs_funded: num_epochs
        });
        Ok(())
    }

    pub fn add_collection(ctx: Context<ManageCollection>, collection_mint: Pubkey) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require!(
            !pool.allowed_collections.contains(&collection_mint),
            ErrorCode::CollectionAlreadyAllowed
        );
        pool.allowed_collections.push(collection_mint);
        Ok(())
    }

    pub fn remove_collection(
        ctx: Context<ManageCollection>,
        collection_mint: Pubkey,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.allowed_collections
            .retain(|&mint| mint != collection_mint);
        Ok(())
    }

    // --- USER INSTRUCTIONS ---

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        // CORRECTED: Manually deserialize the foreign Metaplex account from UncheckedAccount
        let nft_metadata_account_info = &ctx.accounts.nft_metadata_account.to_account_info();
        let nft_metadata =
            Metadata::safe_deserialize(&nft_metadata_account_info.try_borrow_data()?)?;

        let collection = nft_metadata
            .collection
            .ok_or(ErrorCode::NotPartOfCollection)?;
        require!(collection.verified, ErrorCode::CollectionNotVerified);

        let pool = &mut ctx.accounts.pool;
        require!(
            pool.allowed_collections.contains(&collection.key),
            ErrorCode::CollectionNotAllowed
        );

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_nft_token_account.to_account_info(),
            to: ctx.accounts.nft_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        anchor_spl::token_interface::transfer(CpiContext::new(cpi_program, cpi_accounts), 1)?;
        update_skipped_reward(pool);

        let stake_entry = &mut ctx.accounts.stake_entry;
        stake_entry.user = ctx.accounts.user.key();
        stake_entry.nft_mint = ctx.accounts.nft_mint.key();
        stake_entry.staked_at = Clock::get()?.unix_timestamp;
        stake_entry.last_claimed_epoch = pool.current_epoch;
        stake_entry.bump = ctx.bumps.stake_entry;
        stake_entry.skipped_reward = pool.skipped_reward_per_nft;

        pool.total_staked = pool.total_staked.checked_add(1).unwrap();
        let current_day = get_current_day(&pool)?;
        pool.staked_counts[current_day as usize] += 1;

       
        //update_skipped_reward(&pool);
        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        let user_key = ctx.accounts.user.key();
        let nft_mint_key = ctx.accounts.nft_mint.key();
        let seeds = &[
            b"stake_entry".as_ref(),
            user_key.as_ref(),
            nft_mint_key.as_ref(),
            &[ctx.accounts.stake_entry.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts_transfer = Transfer {
            from: ctx.accounts.nft_vault.to_account_info(),
            to: ctx.accounts.user_nft_token_account.to_account_info(),
            authority: ctx.accounts.stake_entry.to_account_info(),
        };
        anchor_spl::token_interface::transfer(
            CpiContext::new_with_signer(cpi_program.clone(), cpi_accounts_transfer, signer),
            1,
        )?;

        anchor_spl::token_interface::close_account(CpiContext::new_with_signer(
            cpi_program,
            CloseAccount {
                account: ctx.accounts.nft_vault.to_account_info(),
                destination: ctx.accounts.user.to_account_info(),
                authority: ctx.accounts.stake_entry.to_account_info(),
            },
            signer,
        ))?;

        pool.total_staked = pool.total_staked.checked_sub(1).unwrap();
        let current_day = get_current_day(&pool)?;
        pool.staked_counts[current_day as usize] -= 1;
        //update_skipped_reward(&pool);
        Ok(())
    }
}

// fn update_rewards_for_epoch(pool: &mut Account<Pool>) -> Result<()> {
//     if pool.total_staked > 0 {
//         if let Some(reward_for_current_epoch) = pool.rewards_per_epoch.get(pool.current_epoch as usize) {
//             let reward_addition = (*reward_for_current_epoch as u128)
//                 .checked_mul(PRECISION).unwrap()
//                 .checked_div(pool.total_staked as u128).unwrap();
            
//             pool.reward_per_nft_stored = pool.reward_per_nft_stored.checked_add(reward_addition).unwrap();
//         }
//     }
//     Ok(())
// }

fn get_current_day(pool: &Pool) -> Result<u64> {
    let now = Clock::get()?.unix_timestamp;

    if now < pool.start_staking_timestamp {
        return Ok(0); // Chưa bắt đầu
    }

    let elapsed_seconds = now - pool.start_staking_timestamp;
    let elapsed_days = elapsed_seconds / 86_400; // 1 ngày = 86400 giây

    Ok(elapsed_days as u64)
}

pub fn update_skipped_reward(pool: &mut Pool) -> Result<()> {
    let current_day = get_current_day(pool)?;

    // Bỏ qua nếu đã cập nhật đến ngày hiện tại
    if pool.last_update_calc_reward_nft_index >= current_day {
        return Ok(());
    }

    for day in pool.last_update_calc_reward_nft_index..current_day {
        let reward_today = *pool.rewards_per_epoch.get(day as usize).unwrap_or(&0);
        let staked_count = *pool.staked_counts.get(day as usize).unwrap_or(&0);

        if staked_count > 0 {
            let skipped_reward = reward_today / staked_count as u64;
            pool.skipped_reward_per_nft = pool
                .skipped_reward_per_nft
                .saturating_add(skipped_reward);
        }
    }

    pool.last_update_calc_reward_nft_index = current_day;

    Ok(())
}


// --- ACCOUNTS ---

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(init, payer = admin, space = 8 + Pool::ACCOUNT_SPACE, seeds = [b"pool"], bump)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub reward_mint: InterfaceAccount<'info, Mint>,
    #[account(init, payer = admin, token::mint = reward_mint, token::authority = pool, seeds = [b"reward_vault"], bump)]
    pub reward_vault: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct AddReward<'info> {
    #[account(mut, seeds = [b"pool"], bump = pool.bump, has_one = admin, has_one = reward_mint)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, address = pool.reward_vault)]
    pub reward_vault: InterfaceAccount<'info, TokenAccount>,
    pub reward_mint: InterfaceAccount<'info, Mint>,
    #[account(mut, associated_token::mint = reward_mint, associated_token::authority = admin)]
    pub admin_reward_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct ManageCollection<'info> {
    #[account(mut, seeds = [b"pool"], bump = pool.bump, has_one = admin)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"pool"], bump = pool.bump)]
    pub pool: Account<'info, Pool>,
    pub nft_mint: InterfaceAccount<'info, Mint>,
    // CORRECTED: Use `UncheckedAccount` for foreign accounts that will be manually deserialized.
    #[account(
        seeds = [b"metadata", mpl_token_metadata::ID.as_ref(), nft_mint.key().as_ref()],
        seeds::program = mpl_token_metadata::ID,
        bump
    )]
    /// CHECK: We deserialize this manually and verify its properties in the instruction.
    pub nft_metadata_account: UncheckedAccount<'info>,
    #[account(init, payer = user, space = 8 + NftStakeEntry::ACCOUNT_SPACE, seeds = [b"stake_entry", user.key().as_ref(), nft_mint.key().as_ref()], bump)]
    pub stake_entry: Account<'info, NftStakeEntry>,
    #[account(mut, associated_token::mint = nft_mint, associated_token::authority = user)]
    pub user_nft_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(init, payer = user, token::mint = nft_mint, token::authority = stake_entry, seeds = [b"nft_vault", user.key().as_ref(), nft_mint.key().as_ref()], bump)]
    pub nft_vault: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"pool"], bump = pool.bump)]
    pub pool: Account<'info, Pool>,
    pub nft_mint: InterfaceAccount<'info, Mint>,
    #[account(mut, close = user, has_one = user, has_one = nft_mint, seeds = [b"stake_entry", user.key().as_ref(), nft_mint.key().as_ref()], bump = stake_entry.bump)]
    pub stake_entry: Account<'info, NftStakeEntry>,
    #[account(mut, seeds = [b"nft_vault", user.key().as_ref(), nft_mint.key().as_ref()], bump)]
    pub nft_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(init_if_needed, payer = user, associated_token::mint = nft_mint, associated_token::authority = user)]
    pub user_nft_token_account: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

// --- DATA STRUCTS ---

#[account]
pub struct Pool {
    pub admin: Pubkey,
    pub reward_mint: Pubkey,
    pub reward_vault: Pubkey,
    pub allowed_collections: Vec<Pubkey>,
    pub total_staked: u64,
    pub current_epoch: u64,
    pub last_update_time: i64,
    pub epoch_duration: i64,
    pub rewards_per_epoch: Vec<u64>,
    pub reward_per_nft_stored: u128,
    pub bump: u8,
    pub start_staking_timestamp: i64, // ✅ Thời điểm bắt đầu staking chính thức
    pub skipped_reward_per_nft: u64, // ✅ Tổng phần thưởng bỏ lỡ mỗi NFT tính đến thời điểm cuối
    pub last_update_calc_reward_nft_index: u64, // ✅ Ngày cuối cùng đã update skipped reward
    pub staked_counts: Vec<u32> // nft staking/day
}
impl Pool {
    pub const MAX_EPOCHS: usize = 1200;
    pub const MAX_COLLECTIONS: usize = 2;
    pub const ACCOUNT_SPACE: usize = 8
        + 32
        + 32
        + 32
        + (4 + 32 * Self::MAX_COLLECTIONS)
        + 8
        + 8
        + 8
        + 8
        + (4 + 8 * Self::MAX_EPOCHS)
        + 16
        + 1
        + 8  // start_staking_timestamp ✅ mới
        + 8  // skipped_reward_per_nft ✅ mới
        + 8 // last_update_calc_reward_nft_index ✅ mới
        + (4 + 4 * Self::MAX_EPOCHS); // stake_counts
}
#[account]
pub struct NftStakeEntry {
    pub user: Pubkey,
    pub nft_mint: Pubkey,
    pub staked_at: i64,
    pub last_claimed_epoch: u64,
    pub skipped_reward: u64,
    pub bump: u8,
}
impl NftStakeEntry {
    pub const ACCOUNT_SPACE: usize = 32 + 32 + 8 + 8 + 8 + 1;
}
#[event]
pub struct RewardAdded {
    pub funder: Pubkey,
    pub total_amount: u64,
    pub epochs_funded: u64,
}
#[event]
pub struct RewardClaimed {
    pub user: Pubkey,
    pub nft_mint: Pubkey,
    pub amount: u64,
}
#[error_code]
pub enum ErrorCode {
    #[msg("This collection is already on the whitelist.")]
    CollectionAlreadyAllowed,
    #[msg("This collection is not on the whitelist.")]
    CollectionNotAllowed,
    #[msg("This NFT does not belong to a collection.")]
    NotPartOfCollection,
    #[msg("The NFT's collection is not verified by a creator.")]
    CollectionNotVerified,
    #[msg("Signer is not the pool admin.")]
    Unauthorized,
    #[msg("Cannot add zero rewards.")]
    ZeroRewardAmount,
    #[msg("Cannot fund zero epochs.")]
    ZeroEpochAmount,
    #[msg("No rewards available to claim at this time.")]
    NoRewardsToClaim,
    #[msg("Adding these epochs would exceed the maximum capacity.")]
    MaxEpochsExceeded,
}
