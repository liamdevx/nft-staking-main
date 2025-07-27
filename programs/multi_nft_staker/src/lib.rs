// file: programs/nft_staking/src/lib.rs

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{CloseAccount, Mint, TokenAccount, TokenInterface, Transfer},
};
use mpl_token_metadata::accounts::Metadata;

// After your first successful `anchor build`, paste your new Program ID here.
declare_id!("AEX1smJbH8pgMBL2Hpf6EJnuRaUwBt6NBYP7jVPixAeC");

#[program]
pub mod nft_staking {
    use super::*;

    // ... (previous admin instructions: initialize_pool, add_reward, add_collection, remove_collection) ...

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
        pool.bump = ctx.bumps.pool;
        pool.start_staking_timestamp = Clock::get()?.unix_timestamp;
        pool.cumulative_reward_per_nft = 0; // Renamed
        pool.last_update_calc_reward_nft_index = 0; 
        pool.staked_counts = Vec::new(); // Initialize as empty
        pool.staked_counts_start_day = 0; // New field initialization
        pool.total_staked_at_window_start = 0; // New field initialization
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
        let reward_per_epoch = total_reward_amount
            .checked_div(num_epochs)
            .ok_or(ErrorCode::ZeroEpochAmount)?; // Use ok_or for better error
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

    // add reward for today
    pub fn add_reward_for_today(
        ctx: Context<AddReward>,
        amount: u64,
    ) -> Result<()> {
        require_gt!(amount, 0, ErrorCode::ZeroRewardAmount);
    
        // Transfer reward token vào vault như bình thường
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.admin_reward_token_account.to_account_info(),
            to: ctx.accounts.reward_vault.to_account_info(),
            authority: ctx.accounts.admin.to_account_info(),
        };
        anchor_spl::token_interface::transfer(
            CpiContext::new(cpi_program, cpi_accounts),
            amount,
        )?;
    
        let pool = &mut ctx.accounts.pool;
        let current_day = get_current_day(pool)?;
    
        // Nếu mảng chưa đủ dài tới current_day, mở rộng bằng 0
        if pool.rewards_per_epoch.len() <= current_day as usize {
            pool.rewards_per_epoch
                .resize(current_day as usize + 1, 0);
        }
    
        // Cộng dồn reward cho ngày hiện tại
        pool.rewards_per_epoch[current_day as usize] = pool.rewards_per_epoch[current_day as usize]
            .checked_add(amount)
            .ok_or(ErrorCode::RewardCalculationError)?;
    
        emit!(RewardAdded {
            funder: ctx.accounts.admin.key(),
            total_amount: amount,
            epochs_funded: 1,
        });
    
        Ok(())
    }
    

    pub fn add_collection(ctx: Context<ManageCollection>, collection_mint: Pubkey) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        // only admin
        require_keys_eq!(ctx.accounts.admin.key(), pool.admin, ErrorCode::Unauthorized);
        require!(
            !pool.allowed_collections.contains(&collection_mint),
            ErrorCode::CollectionAlreadyAllowed
        );
        require!(
            pool.allowed_collections.len() < Pool::MAX_COLLECTIONS,
            ErrorCode::MaxCollectionsExceeded
        ); // Added max collection check
        pool.allowed_collections.push(collection_mint);
        Ok(())
    }

    pub fn remove_collection(
        ctx: Context<ManageCollection>,
        collection_mint: Pubkey,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require_keys_eq!(ctx.accounts.admin.key(), pool.admin, ErrorCode::Unauthorized);
        let initial_len = pool.allowed_collections.len();
        pool.allowed_collections
            .retain(|&mint| mint != collection_mint);
        require!(
            pool.allowed_collections.len() < initial_len,
            ErrorCode::CollectionNotAllowed // Return error if collection wasn't found
        );
        Ok(())
    }

    // --- USER INSTRUCTIONS ---

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
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
        
        // Ensure cumulative_reward_per_nft is updated before recording it for the stake entry
        update_skipped_reward(pool)?; 

        let stake_entry = &mut ctx.accounts.stake_entry;
        stake_entry.user = ctx.accounts.user.key();
        stake_entry.nft_mint = ctx.accounts.nft_mint.key();
        stake_entry.staked_at = Clock::get()?.unix_timestamp;
        stake_entry.last_claimed_epoch = pool.current_epoch; // This field might be redundant with cumulative_reward
        stake_entry.bump = ctx.bumps.stake_entry;
        stake_entry.skipped_reward = pool.cumulative_reward_per_nft; // Record current global cumulative reward

        pool.total_staked = pool.total_staked.checked_add(1).unwrap();
        
        let current_day = get_current_day(&pool)?;
        let index_for_current_day = current_day.checked_sub(pool.staked_counts_start_day)
                                            .ok_or(ErrorCode::RewardCalculationError)?;

        require!(
            (index_for_current_day as usize) < Pool::MAX_STAKED_COUNTS_WINDOW_DAYS,
            ErrorCode::MaxStakedCountsExceeded
        );

        // Ensure staked_counts has enough capacity for the current day's index.
        // New elements will be 0, which is fine as update_skipped_reward handles the propagation.
        if (index_for_current_day as usize) >= pool.staked_counts.len() {
            pool.staked_counts.resize((index_for_current_day as usize) + 1, 0);
        }
        // Set the current day's entry to the *actual new total_staked* after this transaction.
        pool.staked_counts[index_for_current_day as usize] = pool.total_staked as u32; // Store the snapshot

        // Emit StakeEvent
        emit!(StakeEvent {
            user: ctx.accounts.user.key(),
            nft_mint: ctx.accounts.nft_mint.key(),
            staked_at: stake_entry.staked_at,
        });

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let stake_entry = &mut ctx.accounts.stake_entry;

        // Ensure the pool's cumulative_reward_per_nft is up-to-date before calculating rewards
        update_skipped_reward(pool)?;

        // Calculate the reward amount: current global cumulative reward - cumulative reward at stake time
        let reward_amount = pool.cumulative_reward_per_nft.checked_sub(stake_entry.skipped_reward)
            .ok_or(ErrorCode::RewardCalculationError)?;
        
        // Only transfer rewards if there are any
        if reward_amount > 0 {
            let pool_seeds = &[
                b"pool".as_ref(),
                &[pool.bump],
            ];
            let pool_signer = &[&pool_seeds[..]];

            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_accounts = Transfer {
                from: ctx.accounts.reward_vault.to_account_info(),
                to: ctx.accounts.user_reward_token_account.to_account_info(),
                authority: pool.to_account_info(),
            };
            anchor_spl::token_interface::transfer(
                CpiContext::new_with_signer(cpi_program, cpi_accounts, pool_signer),
                reward_amount,
            )?;

            // Update the stake entry's skipped_reward to the current pool's cumulative_reward_per_nft.
            stake_entry.skipped_reward = pool.cumulative_reward_per_nft;

            emit!(RewardClaimed {
                user: ctx.accounts.user.key(),
                nft_mint: stake_entry.nft_mint,
                amount: reward_amount,
            });
        }


        let user_key = ctx.accounts.user.key();
        let nft_mint_key = ctx.accounts.nft_mint.key();
        let stake_entry_seeds = &[
            b"stake_entry".as_ref(),
            user_key.as_ref(),
            nft_mint_key.as_ref(),
            &[ctx.accounts.stake_entry.bump],
        ];
        let stake_entry_signer = &[&stake_entry_seeds[..]];

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts_transfer = Transfer {
            from: ctx.accounts.nft_vault.to_account_info(),
            to: ctx.accounts.user_nft_token_account.to_account_info(),
            authority: ctx.accounts.stake_entry.to_account_info(),
        };
        anchor_spl::token_interface::transfer(
            CpiContext::new_with_signer(cpi_program.clone(), cpi_accounts_transfer, stake_entry_signer),
            1,
        )?;

        anchor_spl::token_interface::close_account(CpiContext::new_with_signer(
            cpi_program,
            CloseAccount {
                account: ctx.accounts.nft_vault.to_account_info(),
                destination: ctx.accounts.user.to_account_info(),
                authority: ctx.accounts.stake_entry.to_account_info(),
            },
            stake_entry_signer,
        ))?;

        pool.total_staked = pool.total_staked.checked_sub(1).unwrap();
        
        let current_day = get_current_day(&pool)?;
        let index_for_current_day = current_day.checked_sub(pool.staked_counts_start_day)
                                            .ok_or(ErrorCode::RewardCalculationError)?;
        require!(
            (index_for_current_day as usize) < Pool::MAX_STAKED_COUNTS_WINDOW_DAYS,
            ErrorCode::MaxStakedCountsExceeded
        );
        
        // Ensure staked_counts has enough capacity for the current day's index.
        // New elements will be 0, which is fine as update_skipped_reward handles the propagation.
        if (index_for_current_day as usize) >= pool.staked_counts.len() {
            pool.staked_counts.resize((index_for_current_day as usize) + 1, 0);
        }
        // Set the current day's entry to the *actual new total_staked* after this transaction.
        pool.staked_counts[index_for_current_day as usize] = pool.total_staked as u32; // Store the snapshot
        
        // Emit UnstakeEvent
        emit!(UnstakeEvent {
            user: ctx.accounts.user.key(),
            nft_mint: ctx.accounts.nft_mint.key(),
            unstaked_at: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let stake_entry = &mut ctx.accounts.stake_entry;

        // Ensure the pool's cumulative_reward_per_nft is up-to-date before calculating rewards
        update_skipped_reward(pool)?;

        // Calculate the reward amount: current global cumulative reward - cumulative reward at stake time
        let reward_amount = pool.cumulative_reward_per_nft.checked_sub(stake_entry.skipped_reward)
            .ok_or(ErrorCode::RewardCalculationError)?; // Use a specific error for calculation issues
        require_gt!(reward_amount, 0, ErrorCode::NoRewardsToClaim);

        // Transfer rewards from the pool's vault to the user
        //let pool_key = pool.key();
        let seeds = &[
            b"pool".as_ref(),
            &[pool.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.reward_vault.to_account_info(),
            to: ctx.accounts.user_reward_token_account.to_account_info(),
            authority: pool.to_account_info(),
        };
        anchor_spl::token_interface::transfer(
            CpiContext::new_with_signer(cpi_program, cpi_accounts, signer),
            reward_amount,
        )?;

        // Update the stake entry's skipped_reward to the current pool's cumulative_reward_per_nft.
        // This ensures that future claims only account for new rewards accrued since this claim.
        stake_entry.skipped_reward = pool.cumulative_reward_per_nft;

        emit!(RewardClaimed {
            user: ctx.accounts.user.key(),
            nft_mint: stake_entry.nft_mint,
            amount: reward_amount,
        });

        Ok(())
    }

    // --- ADMIN INSTRUCTIONS ---

    /// Cho phép admin rút một lượng token cụ thể từ reward_vault.
    pub fn admin_claim(ctx: Context<AdminClaim>, amount: u64) -> Result<()> {
        let pool = &ctx.accounts.pool;
        
        // Chỉ admin mới có thể thực hiện giao dịch này
        require_keys_eq!(ctx.accounts.admin.key(), pool.admin, ErrorCode::Unauthorized);
        // Đảm bảo số lượng rút lớn hơn 0
        require_gt!(amount, 0, ErrorCode::ZeroRewardAmount);

        // Kiểm tra số dư trong vault
        require_gte!(ctx.accounts.reward_vault.amount, amount, ErrorCode::InsufficientVaultBalance);

        let seeds = &[
            b"pool".as_ref(),
            &[pool.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.reward_vault.to_account_info(),
            to: ctx.accounts.admin_reward_token_account.to_account_info(),
            authority: pool.to_account_info(),
        };
        anchor_spl::token_interface::transfer(
            CpiContext::new_with_signer(cpi_program, cpi_accounts, signer),
            amount,
        )?;

        Ok(())
    }
}

fn get_current_day(pool: &Pool) -> Result<u64> {
    let now = Clock::get()?.unix_timestamp;

    if now < pool.start_staking_timestamp {
        return Ok(0); // Before staking started, consider day 0
    }

    let elapsed_seconds = now - pool.start_staking_timestamp;
    let elapsed_days = elapsed_seconds / 86400; // 1 day = 86400 seconds

    Ok(elapsed_days as u64)
}

pub fn update_skipped_reward(pool: &mut Pool) -> Result<()> {
    let current_day = get_current_day(pool)?;

    // Không cần cập nhật nếu không có ngày mới
    if pool.last_update_calc_reward_nft_index >= current_day {
        return Ok(());
    }

    // Khởi tạo với tổng số stake tại thời điểm bắt đầu cửa sổ hiện tại
    let mut last_valid_staked_count = pool.total_staked_at_window_start; 

    for day in pool.last_update_calc_reward_nft_index..current_day {
        let index_in_staked_counts = day
            .checked_sub(pool.staked_counts_start_day)
            .ok_or(ErrorCode::RewardCalculationError)?;

        // Lấy phần thưởng trong ngày đó
        let reward_today = *pool.rewards_per_epoch.get(day as usize).unwrap_or(&0);

        // Lấy số lượng stake trong ngày từ snapshot.
        // Nếu index hợp lệ và có giá trị trong staked_counts, sử dụng nó.
        // Ngược lại, tiếp tục sử dụng last_valid_staked_count từ ngày trước đó.
        let staked_count_snapshot = if (index_in_staked_counts as usize) < pool.staked_counts.len() {
            *pool.staked_counts.get(index_in_staked_counts as usize).unwrap() as u64
        } else {
            // Nếu chỉ mục nằm ngoài phạm vi của staked_counts hiện tại,
            // điều đó có nghĩa là không có giao dịch stake/unstake rõ ràng
            // vào ngày này trong cửa sổ hiện tại.
            // Trong trường hợp này, chúng ta sử dụng giá trị `last_valid_staked_count`
            // từ ngày gần nhất có giá trị hợp lệ (hoặc từ `total_staked_at_window_start`).
            last_valid_staked_count 
        };

        // Nếu snapshot cho ngày này khác 0, cập nhật last_valid_staked_count.
        // Điều này đảm bảo rằng chúng ta luôn sử dụng giá trị stake gần nhất có hoạt động.
        if staked_count_snapshot > 0 {
            last_valid_staked_count = staked_count_snapshot;
        }

        // Chỉ cộng reward nếu có last_valid_staked_count (tránh chia 0)
        if last_valid_staked_count > 0 {
            let cumulative_reward_for_day = reward_today
                .checked_div(last_valid_staked_count)
                .ok_or(ErrorCode::RewardCalculationError)?;

            pool.cumulative_reward_per_nft = pool
                .cumulative_reward_per_nft
                .checked_add(cumulative_reward_for_day)
                .ok_or(ErrorCode::RewardCalculationError)?;
        }
    }

    // Sau khi tính xong, clear dữ liệu staked cũ
    pool.staked_counts.clear();
    pool.staked_counts_start_day = current_day;
    pool.last_update_calc_reward_nft_index = current_day;
    // Cập nhật total_staked_at_window_start cho cửa sổ tiếp theo
    pool.total_staked_at_window_start = pool.total_staked; 
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
    pub user_nft_token_account: InterfaceAccount<'info, TokenAccount>, // Changed from Interface to InterfaceAccount
    // Accounts for claiming rewards
    #[account(address = pool.reward_mint)] // Add constraint to ensure it's the correct reward mint
    pub reward_mint: InterfaceAccount<'info, Mint>, 
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = reward_mint, // Reference the reward_mint account
        associated_token::authority = user
    )]
    pub user_reward_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, address = pool.reward_vault)] // Add reward_vault to unstake context
    pub reward_vault: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"pool"], bump = pool.bump)]
    pub pool: Account<'info, Pool>,
    // The Mint account for the reward token, required for init_if_needed on user_reward_token_account
    #[account(address = pool.reward_mint)] // Add constraint to ensure it's the correct reward mint
    pub reward_mint: InterfaceAccount<'info, Mint>, 
    // The NFT mint associated with the stake entry being claimed
    pub nft_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        has_one = user,
        has_one = nft_mint, // Ensure this stake_entry belongs to this user and NFT
        seeds = [b"stake_entry", user.key().as_ref(), nft_mint.key().as_ref()],
        bump = stake_entry.bump
    )]
    pub stake_entry: Account<'info, NftStakeEntry>,
    #[account(mut, address = pool.reward_vault)]
    pub reward_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = reward_mint, // Reference the reward_mint account
        associated_token::authority = user
    )]
    pub user_reward_token_account: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

/// Cấu trúc tài khoản cho lệnh `admin_claim`.
#[derive(Accounts)]
pub struct AdminClaim<'info> {
    /// Tài khoản Pool, chứa thông tin quản trị viên và vault.
    #[account(mut, seeds = [b"pool"], bump = pool.bump, has_one = admin, has_one = reward_mint)]
    pub pool: Account<'info, Pool>,
    /// Tài khoản quản trị viên, phải là người ký.
    #[account(mut)]
    pub admin: Signer<'info>,
    /// Vault chứa token phần thưởng của pool.
    #[account(mut, address = pool.reward_vault)]
    pub reward_vault: InterfaceAccount<'info, TokenAccount>,
    /// Mint của token phần thưởng, để kiểm tra tính hợp lệ.
    pub reward_mint: InterfaceAccount<'info, Mint>,
    /// Tài khoản token của quản trị viên nơi tiền sẽ được chuyển đến.
    #[account(mut, associated_token::mint = reward_mint, associated_token::authority = admin)]
    pub admin_reward_token_account: InterfaceAccount<'info, TokenAccount>,
    /// Chương trình token để thực hiện chuyển khoản.
    pub token_program: Interface<'info, TokenInterface>,
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
    pub bump: u8,
    pub start_staking_timestamp: i64, // ✅ Thời điểm bắt đầu staking chính thức
    pub cumulative_reward_per_nft: u64, // ✅ Tổng phần thưởng bỏ lỡ mỗi NFT tính đến thời điểm cuối - Renamed
    pub last_update_calc_reward_nft_index: u64, // ✅ Ngày cuối cùng đã update cumulative reward
    pub staked_counts: Vec<u32>, // Store snapshot of total staked for each day in the window
    pub staked_counts_start_day: u64, // The day corresponding to staked_counts[0]
    pub total_staked_at_window_start: u64, // The total_staked value at the day staked_counts_start_day represents
}
impl Pool {
    pub const MAX_EPOCHS: usize = 1200;
    pub const MAX_COLLECTIONS: usize = 2; // Increased to 2 for example
    pub const MAX_STAKED_COUNTS_WINDOW_DAYS: usize = 365; // Max days to store in staked_counts vector
    pub const ACCOUNT_SPACE: usize = 8
        + 32 // admin
        + 32 // reward_mint
        + 32 // reward_vault
        + (4 + 32 * Self::MAX_COLLECTIONS) // allowed_collections
        + 8  // total_staked
        + 8  // current_epoch
        + 8  // last_update_time
        + 8  // epoch_duration
        + (4 + 8 * Self::MAX_EPOCHS) // rewards_per_epoch
        + 1  // bump
        + 8  // start_staking_timestamp
        + 8  // cumulative_reward_per_nft
        + 8  // last_update_calc_reward_nft_index
        + 8  // staked_counts_start_day
        + 8  // total_staked_at_window_start (new field)
        + (4 + 4 * Self::MAX_STAKED_COUNTS_WINDOW_DAYS); // staked_counts - 4 bytes per u32, using new max window
}
#[account]
pub struct NftStakeEntry {
    pub user: Pubkey,
    pub nft_mint: Pubkey,
    pub staked_at: i64,
    pub last_claimed_epoch: u64, // This field might become less relevant with cumulative_reward
    pub skipped_reward: u64, // The cumulative_reward_per_nft value when this NFT was staked/last claimed
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
#[event]
pub struct StakeEvent {
    pub user: Pubkey,
    pub nft_mint: Pubkey,
    pub staked_at: i64,
}
#[event]
pub struct UnstakeEvent {
    pub user: Pubkey,
    pub nft_mint: Pubkey,
    pub unstaked_at: i64,
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
    #[msg("Adding this collection would exceed the maximum capacity.")]
    MaxCollectionsExceeded, // New error code
    #[msg("An error occurred during reward calculation (e.g., overflow, underflow, division by zero).")]
    RewardCalculationError, // New error code for math operations
    #[msg("Insufficient balance in the vault to perform this operation.")]
    InsufficientVaultBalance, // New error code for insufficient funds
    #[msg("Staked counts window exceeded maximum capacity. Please update rewards more frequently.")]
    MaxStakedCountsExceeded, // New error code for staked_counts window
}
