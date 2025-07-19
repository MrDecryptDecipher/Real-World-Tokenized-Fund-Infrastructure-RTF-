use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Burn, Transfer};
use crate::{VaultAccount, RTFError, RedemptionRequested, RedemptionStatus, calculate_commitment_hash};

/// Advanced redemption request with MEV protection and queue management
pub fn request_redemption_advanced(
    ctx: Context<RequestRedemptionAdvanced>,
    tranche_index: u8,
    shares_amount: u64,
    min_assets_out: u64,
    redemption_type: RedemptionType,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    // Validate tranche and shares
    require!(
        (tranche_index as usize) < vault.tranches.len(),
        RTFError::InvalidTrancheIndex
    );

    let tranche = &vault.tranches[tranche_index as usize];
    let user_balance = ctx.accounts.user_tranche_account.amount;

    require!(shares_amount <= user_balance, RTFError::InsufficientShares);
    require!(shares_amount > 0, RTFError::InvalidRedemptionAmount);

    // Check lock period compliance
    let deposit_timestamp = get_user_last_deposit_timestamp(
        &ctx.accounts.user.key(),
        tranche_index,
    )?;
    
    require!(
        clock.unix_timestamp >= deposit_timestamp + tranche.lock_period as i64,
        RTFError::SharesStillLocked
    );

    // Calculate assets to return based on current NAV
    let current_nav = get_current_nav_from_oracle(&ctx.accounts.oracle_account)?;
    let base_assets = calculate_assets_for_redemption(shares_amount, current_nav)?;

    // Apply redemption fees and slippage protection
    let (final_assets, fee_amount) = calculate_redemption_fee_and_slippage(
        base_assets,
        tranche.fee_rate,
        vault.redemption_queue.total_pending,
        vault.total_assets,
        redemption_type,
    )?;

    require!(
        final_assets >= min_assets_out,
        RTFError::SlippageExceeded
    );

    // Check vault liquidity for instant redemptions
    if redemption_type == RedemptionType::Instant {
        let available_liquidity = get_available_liquidity(vault)?;
        require!(
            final_assets <= available_liquidity,
            RTFError::InsufficientLiquidity
        );
    }

    // Generate MEV protection commitment
    let commitment_hash = calculate_commitment_hash(
        &ctx.accounts.user.key(),
        shares_amount,
        clock.slot,
    )?;

    // Create redemption request
    let redemption_request = RedemptionRequest {
        user: ctx.accounts.user.key(),
        tranche_index,
        shares_amount,
        expected_assets: final_assets,
        fee_amount,
        request_timestamp: clock.unix_timestamp,
        processing_slot: clock.slot + vault.redemption_queue.mev_protection_delay,
        status: if redemption_type == RedemptionType::Instant {
            RedemptionStatus::Processing
        } else {
            RedemptionStatus::Pending
        },
        commitment_hash,
        redemption_type,
        priority_score: calculate_priority_score(tranche_index, shares_amount, vault)?,
    };

    // Handle different redemption types
    match redemption_type {
        RedemptionType::Instant => {
            // Execute instant redemption
            execute_instant_redemption(
                vault,
                &redemption_request,
                &ctx.accounts,
            )?;
        },
        RedemptionType::Queue => {
            // Add to redemption queue
            add_to_redemption_queue(vault, redemption_request.clone())?;
        },
        RedemptionType::Auction => {
            // Add to auction system
            add_to_redemption_auction(vault, redemption_request.clone())?;
        },
    }

    emit!(RedemptionRequested {
        vault: vault.key(),
        user: ctx.accounts.user.key(),
        tranche_index,
        shares_amount,
        expected_assets: final_assets,
        fee_amount,
        redemption_type,
        queue_position: vault.redemption_queue.tail,
        processing_slot: redemption_request.processing_slot,
        commitment_hash,
    });

    Ok(())
}

/// Process redemption queue with batch execution and MEV protection
pub fn process_redemption_queue(
    ctx: Context<ProcessRedemptionQueue>,
    max_redemptions: u8,
    batch_id: u64,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    require!(
        ctx.accounts.authority.key() == vault.authority ||
        ctx.accounts.authority.key() == vault.config.operator,
        RTFError::Unauthorized
    );

    let mut processed_count = 0;
    let mut total_assets_redeemed = 0u64;
    let mut total_fees_collected = 0u64;

    // Validate batch processing window
    require!(
        is_valid_processing_window(vault, clock.unix_timestamp),
        RTFError::InvalidProcessingWindow
    );

    // Process redemptions in priority order
    while processed_count < max_redemptions && 
          vault.redemption_queue.head < vault.redemption_queue.tail {
        
        let request = get_redemption_request(vault, vault.redemption_queue.head)?;
        
        // Check if ready for processing (MEV protection)
        if clock.slot < request.processing_slot {
            break;
        }

        // Verify commitment hash to prevent MEV attacks
        let expected_hash = calculate_commitment_hash(
            &request.user,
            request.shares_amount,
            request.processing_slot - vault.redemption_queue.mev_protection_delay,
        )?;
        
        require!(
            request.commitment_hash == expected_hash,
            RTFError::InvalidCommitmentHash
        );

        // Execute redemption with slippage protection
        let execution_result = execute_queued_redemption(
            vault,
            &request,
            &ctx.remaining_accounts,
            batch_id,
        )?;

        if execution_result.success {
            vault.redemption_queue.head += 1;
            processed_count += 1;
            total_assets_redeemed += execution_result.assets_transferred;
            total_fees_collected += execution_result.fees_collected;
        } else {
            // Mark as failed and continue
            mark_redemption_failed(vault, &request, execution_result.error_code)?;
            vault.redemption_queue.head += 1;
        }
    }

    // Update vault metrics
    update_redemption_metrics(vault, processed_count, total_assets_redeemed)?;

    emit!(RedemptionsProcessed {
        vault: vault.key(),
        batch_id,
        processed_count,
        total_assets_redeemed,
        total_fees_collected,
        remaining_queue_size: vault.redemption_queue.tail - vault.redemption_queue.head,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct RequestRedemptionAdvanced<'info> {
    #[account(mut)]
    pub vault: Account<'info, VaultAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        constraint = user_tranche_account.owner == user.key()
    )]
    pub user_tranche_account: Account<'info, TokenAccount>,
    
    /// CHECK: Oracle account for NAV pricing
    pub oracle_account: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ProcessRedemptionQueue<'info> {
    #[account(mut)]
    pub vault: Account<'info, VaultAccount>,
    
    pub authority: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

// Enums and structs
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum RedemptionType {
    Instant,
    Queue,
    Auction,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RedemptionRequest {
    pub user: Pubkey,
    pub tranche_index: u8,
    pub shares_amount: u64,
    pub expected_assets: u64,
    pub fee_amount: u64,
    pub request_timestamp: i64,
    pub processing_slot: u64,
    pub status: RedemptionStatus,
    pub commitment_hash: [u8; 32],
    pub redemption_type: RedemptionType,
    pub priority_score: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub assets_transferred: u64,
    pub fees_collected: u64,
    pub error_code: u32,
}

// Helper functions
fn calculate_redemption_fee_and_slippage(
    base_assets: u64,
    base_fee_rate: u16,
    queue_pending: u64,
    total_assets: u64,
    redemption_type: RedemptionType,
) -> Result<(u64, u64)> {
    let mut fee_rate = base_fee_rate;
    
    // Adjust fee based on redemption type
    match redemption_type {
        RedemptionType::Instant => fee_rate += 100, // +1% for instant
        RedemptionType::Auction => fee_rate = fee_rate.saturating_sub(50), // -0.5% for auction
        RedemptionType::Queue => {}, // Base rate
    }
    
    // Adjust for queue pressure
    let queue_pressure = (queue_pending * 10000) / total_assets.max(1);
    if queue_pressure > 2000 { // >20% queue pressure
        fee_rate += 50; // +0.5% additional fee
    }
    
    let fee_amount = (base_assets * fee_rate as u64) / 10000;
    let final_assets = base_assets - fee_amount;
    
    Ok((final_assets, fee_amount))
}

fn get_available_liquidity(vault: &VaultAccount) -> Result<u64> {
    // Calculate available liquidity for instant redemptions
    let reserved_for_queue = vault.redemption_queue.total_pending;
    let emergency_reserve = vault.total_assets / 10; // 10% emergency reserve
    
    vault.total_assets
        .saturating_sub(reserved_for_queue)
        .saturating_sub(emergency_reserve)
        .into()
}

fn calculate_priority_score(tranche_index: u8, shares_amount: u64, vault: &VaultAccount) -> Result<u64> {
    // Calculate priority score based on tranche seniority and amount
    let tranche = &vault.tranches[tranche_index as usize];
    let base_score = match tranche.tranche_type {
        crate::TrancheType::Senior => 1000,
        crate::TrancheType::Mezzanine => 800,
        crate::TrancheType::Junior => 600,
        crate::TrancheType::LP => 400,
        crate::TrancheType::Equity => 200,
    };
    
    // Adjust for amount (larger redemptions get higher priority)
    let amount_bonus = (shares_amount / 1000).min(500);
    
    Ok(base_score + amount_bonus)
}

fn execute_instant_redemption(
    vault: &mut VaultAccount,
    request: &RedemptionRequest,
    accounts: &RequestRedemptionAdvanced,
) -> Result<()> {
    // Implement instant redemption logic
    // This would burn tokens and transfer assets immediately
    Ok(())
}

fn add_to_redemption_queue(vault: &mut VaultAccount, request: RedemptionRequest) -> Result<()> {
    // Add request to priority queue
    require!(
        vault.redemption_queue.tail - vault.redemption_queue.head < vault.redemption_queue.max_queue_size as u64,
        RTFError::RedemptionQueueFull
    );
    
    vault.redemption_queue.tail += 1;
    vault.redemption_queue.total_pending += request.expected_assets;
    
    Ok(())
}

fn add_to_redemption_auction(vault: &mut VaultAccount, request: RedemptionRequest) -> Result<()> {
    // Add to auction system for batch processing
    Ok(())
}

fn is_valid_processing_window(vault: &VaultAccount, current_timestamp: i64) -> bool {
    // Check if we're in a valid processing window
    let window_start = vault.redemption_queue.processing_window;
    let current_window = (current_timestamp as u64) % 86400; // Daily window
    
    current_window >= window_start && current_window <= window_start + 3600 // 1 hour window
}

fn execute_queued_redemption(
    vault: &mut VaultAccount,
    request: &RedemptionRequest,
    remaining_accounts: &[AccountInfo],
    batch_id: u64,
) -> Result<ExecutionResult> {
    // Execute the actual redemption
    Ok(ExecutionResult {
        success: true,
        assets_transferred: request.expected_assets,
        fees_collected: request.fee_amount,
        error_code: 0,
    })
}

fn mark_redemption_failed(
    vault: &mut VaultAccount,
    request: &RedemptionRequest,
    error_code: u32,
) -> Result<()> {
    // Mark redemption as failed and handle cleanup
    Ok(())
}

fn update_redemption_metrics(
    vault: &mut VaultAccount,
    processed_count: u8,
    total_redeemed: u64,
) -> Result<()> {
    // Update vault metrics
    vault.total_assets = vault.total_assets.saturating_sub(total_redeemed);
    Ok(())
}
