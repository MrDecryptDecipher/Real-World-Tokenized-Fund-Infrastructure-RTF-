use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use std::collections::VecDeque;

declare_id!("RTFRedemptionEngine11111111111111111111111111");

/// PRD Section 3.3: Redemption Engine
/// PRD: "First-in, time-bound, tranche-weighted redemption queue"
/// PRD: "Flashloan-resistance via proof-of-holding duration"
/// PRD: "Redemption auction: Epoch-batched, MEV-protected batch submission"
/// PRD: "Instant-exit quoting with LLM forecasts"
/// PRD: "zkProof-of-chain-origin validation"

#[program]
pub mod rtf_redemption {
    use super::*;

    /// Initialize redemption engine with MEV protection
    pub fn initialize(
        ctx: Context<Initialize>,
        max_queue_size: u64,
        min_holding_duration: i64,
        epoch_duration: i64,
        mev_protection_delay: i64,
    ) -> Result<()> {
        let redemption_engine = &mut ctx.accounts.redemption_engine;
        redemption_engine.authority = ctx.accounts.authority.key();
        redemption_engine.vault = ctx.accounts.vault.key();
        redemption_engine.max_queue_size = max_queue_size;
        redemption_engine.min_holding_duration = min_holding_duration;
        redemption_engine.epoch_duration = epoch_duration;
        redemption_engine.mev_protection_delay = mev_protection_delay;
        redemption_engine.current_epoch = 0;
        redemption_engine.total_pending_redemptions = 0;
        redemption_engine.bump = ctx.bumps.redemption_engine;

        msg!("RTF Redemption Engine initialized with MEV protection");
        Ok(())
    }

    /// PRD: Submit redemption request with MEV protection (commit phase)
    pub fn submit_redemption_commitment(
        ctx: Context<SubmitRedemptionCommitment>,
        commitment_hash: [u8; 32],
        tranche_index: u8,
    ) -> Result<()> {
        let redemption_engine = &mut ctx.accounts.redemption_engine;
        let user_position = &ctx.accounts.user_position;
        let clock = Clock::get()?;

        // PRD: Verify proof-of-holding duration
        require!(
            clock.unix_timestamp - user_position.last_deposit_time >= redemption_engine.min_holding_duration,
            RedemptionError::InsufficientHoldingDuration
        );

        // PRD: Check queue capacity
        require!(
            redemption_engine.total_pending_redemptions < redemption_engine.max_queue_size,
            RedemptionError::QueueFull
        );

        let commitment = RedemptionCommitment {
            user: ctx.accounts.user.key(),
            commitment_hash,
            tranche_index,
            timestamp: clock.unix_timestamp,
            revealed: false,
            executed: false,
        };

        redemption_engine.commitments.push(commitment);
        redemption_engine.total_pending_redemptions += 1;

        emit!(RedemptionCommitted {
            user: ctx.accounts.user.key(),
            commitment_hash,
            tranche_index,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// PRD: Reveal redemption details (reveal phase for MEV protection)
    pub fn reveal_redemption_request(
        ctx: Context<RevealRedemptionRequest>,
        amount: u64,
        min_assets_out: u64,
        nonce: u64,
        commitment_index: u64,
    ) -> Result<()> {
        let redemption_engine = &mut ctx.accounts.redemption_engine;
        let clock = Clock::get()?;

        // Verify commitment exists and timing
        require!(
            (commitment_index as usize) < redemption_engine.commitments.len(),
            RedemptionError::InvalidCommitmentIndex
        );

        let commitment = &mut redemption_engine.commitments[commitment_index as usize];
        require!(
            commitment.user == ctx.accounts.user.key(),
            RedemptionError::UnauthorizedReveal
        );

        require!(
            !commitment.revealed,
            RedemptionError::AlreadyRevealed
        );

        // PRD: Verify MEV protection delay has passed
        require!(
            clock.unix_timestamp - commitment.timestamp >= redemption_engine.mev_protection_delay,
            RedemptionError::MEVProtectionActive
        );

        // PRD: Advanced commit-reveal scheme verification
        let revealed_hash = self.compute_commitment_hash(
            amount,
            min_assets_out,
            nonce,
            ctx.accounts.user.key(),
            commitment.timestamp,
        )?;

        require!(
            revealed_hash == commitment.commitment_hash,
            RedemptionError::InvalidCommitmentReveal
        );

        // Additional MEV protection: verify timing constraints
        require!(
            clock.unix_timestamp - commitment.timestamp <= 3600, // 1 hour max reveal window
            RedemptionError::RevealWindowExpired
        );

        // Create redemption request
        let redemption_request = RedemptionRequest {
            user: ctx.accounts.user.key(),
            amount,
            min_assets_out,
            tranche_index: commitment.tranche_index,
            timestamp: commitment.timestamp,
            priority_score: calculate_priority_score(amount, commitment.timestamp, commitment.tranche_index),
            status: RedemptionStatus::Pending,
        };

        redemption_engine.pending_requests.push(redemption_request);
        commitment.revealed = true;

        emit!(RedemptionRevealed {
            user: ctx.accounts.user.key(),
            amount,
            min_assets_out,
            tranche_index: commitment.tranche_index,
            priority_score: redemption_request.priority_score,
        });

        Ok(())
    }

    /// PRD: Execute epoch-batched redemption auction
    pub fn execute_redemption_batch(
        ctx: Context<ExecuteRedemptionBatch>,
        max_batch_size: u32,
    ) -> Result<()> {
        let redemption_engine = &mut ctx.accounts.redemption_engine;
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        // PRD: Check if epoch has ended
        let epoch_start = redemption_engine.current_epoch * redemption_engine.epoch_duration;
        require!(
            clock.unix_timestamp >= epoch_start + redemption_engine.epoch_duration,
            RedemptionError::EpochNotEnded
        );

        // PRD: Sort requests by priority (first-in, time-bound, tranche-weighted)
        redemption_engine.pending_requests.sort_by(|a, b| {
            b.priority_score.cmp(&a.priority_score)
        });

        let mut executed_count = 0;
        let mut total_assets_out = 0u64;
        let batch_size = std::cmp::min(max_batch_size as usize, redemption_engine.pending_requests.len());

        // Execute redemptions in priority order
        for i in 0..batch_size {
            if executed_count >= max_batch_size {
                break;
            }

            let request = &mut redemption_engine.pending_requests[i];
            if request.status != RedemptionStatus::Pending {
                continue;
            }

            // Calculate assets out based on current NAV
            let nav_per_share = vault.nav_per_share;
            let assets_out = (request.amount * nav_per_share) / 1_000_000; // Assuming 6 decimals

            // Check minimum assets out requirement
            if assets_out < request.min_assets_out {
                request.status = RedemptionStatus::Failed;
                emit!(RedemptionFailed {
                    user: request.user,
                    amount: request.amount,
                    reason: "Insufficient assets out".to_string(),
                });
                continue;
            }

            // Check vault liquidity
            if vault.available_liquidity < assets_out {
                request.status = RedemptionStatus::Deferred;
                continue;
            }

            // Execute redemption
            vault.available_liquidity -= assets_out;
            vault.total_shares -= request.amount;
            total_assets_out += assets_out;

            request.status = RedemptionStatus::Executed;
            executed_count += 1;

            emit!(RedemptionExecuted {
                user: request.user,
                shares_redeemed: request.amount,
                assets_out,
                nav_per_share,
                tranche_index: request.tranche_index,
            });
        }

        // Remove executed requests
        redemption_engine.pending_requests.retain(|req| req.status != RedemptionStatus::Executed);
        redemption_engine.total_pending_redemptions = redemption_engine.pending_requests.len() as u64;
        redemption_engine.current_epoch += 1;

        emit!(RedemptionBatchExecuted {
            epoch: redemption_engine.current_epoch - 1,
            executed_count,
            total_assets_out,
            remaining_requests: redemption_engine.total_pending_redemptions,
        });

        Ok(())
    }

    /// PRD: Get instant-exit quote with LLM forecasts
    pub fn get_instant_exit_quote(
        ctx: Context<GetInstantExitQuote>,
        amount: u64,
        tranche_index: u8,
    ) -> Result<()> {
        let vault = &ctx.accounts.vault;
        let llm_oracle = &ctx.accounts.llm_oracle;

        // Get current NAV
        let base_nav_per_share = vault.nav_per_share;

        // PRD: Apply LLM forecast adjustment
        let forecast_adjustment = llm_oracle.get_nav_forecast_adjustment(tranche_index)?;
        let adjusted_nav = apply_forecast_adjustment(base_nav_per_share, forecast_adjustment);

        // Calculate instant exit penalty (for immediate liquidity)
        let instant_exit_penalty = calculate_instant_exit_penalty(amount, vault.available_liquidity);
        let final_nav = adjusted_nav * (10000 - instant_exit_penalty) / 10000;

        let assets_out = (amount * final_nav) / 1_000_000;

        emit!(InstantExitQuote {
            user: ctx.accounts.user.key(),
            amount,
            assets_out,
            base_nav: base_nav_per_share,
            adjusted_nav,
            penalty_bps: instant_exit_penalty,
            forecast_confidence: llm_oracle.confidence_score,
            valid_until: Clock::get()?.unix_timestamp + 300, // 5 minutes
        });

        Ok(())
    }

    /// PRD: Verify zkProof-of-chain-origin for cross-chain redemptions
    pub fn verify_chain_origin_proof(
        ctx: Context<VerifyChainOriginProof>,
        proof: Vec<u8>,
        origin_chain_id: u64,
        origin_tx_hash: [u8; 32],
    ) -> Result<()> {
        let redemption_engine = &ctx.accounts.redemption_engine;

        // PRD: Verify zkProof of chain origin
        let proof_valid = verify_zk_proof_of_origin(
            &proof,
            origin_chain_id,
            origin_tx_hash,
            ctx.accounts.user.key(),
        )?;

        require!(proof_valid, RedemptionError::InvalidChainOriginProof);

        emit!(ChainOriginVerified {
            user: ctx.accounts.user.key(),
            origin_chain_id,
            origin_tx_hash,
            proof_hash: solana_program::keccak::hash(&proof).to_bytes(),
        });

        Ok(())
    }
}

// Helper functions
fn calculate_priority_score(amount: u64, timestamp: i64, tranche_index: u8) -> u64 {
    // PRD: First-in, time-bound, tranche-weighted priority
    let time_weight = (Clock::get().unwrap().unix_timestamp - timestamp) as u64;
    let tranche_weight = match tranche_index {
        0 => 1000, // Senior tranche gets highest priority
        1 => 500,  // Junior tranche
        2 => 100,  // LP tranche gets lowest priority
        _ => 0,
    };
    let amount_weight = amount / 1000; // Normalize amount

    time_weight + tranche_weight + amount_weight
}

/// PRD: Advanced commit-reveal scheme with enhanced MEV protection
fn compute_commitment_hash(
    amount: u64,
    min_assets_out: u64,
    nonce: u64,
    user: Pubkey,
    timestamp: i64,
) -> Result<[u8; 32]> {
    // Enhanced commitment hash with timestamp and additional entropy
    let commitment_data = [
        &amount.to_le_bytes(),
        &min_assets_out.to_le_bytes(),
        &nonce.to_le_bytes(),
        user.as_ref(),
        &timestamp.to_le_bytes(),
        b"RTF_REDEMPTION_COMMITMENT_V2", // Version identifier
    ].concat();

    let hash = solana_program::keccak::hash(&commitment_data);
    Ok(hash.to_bytes())
}

fn apply_forecast_adjustment(base_nav: u64, adjustment_bps: i16) -> u64 {
    if adjustment_bps >= 0 {
        base_nav + (base_nav * adjustment_bps as u64) / 10000
    } else {
        base_nav - (base_nav * (-adjustment_bps) as u64) / 10000
    }
}

fn calculate_instant_exit_penalty(amount: u64, available_liquidity: u64) -> u16 {
    // Higher penalty for larger redemptions relative to available liquidity
    let utilization_bps = (amount * 10000) / available_liquidity.max(1);

    match utilization_bps {
        0..=1000 => 50,    // 0.5% penalty for <10% utilization
        1001..=2500 => 100, // 1% penalty for 10-25% utilization
        2501..=5000 => 200, // 2% penalty for 25-50% utilization
        _ => 500,          // 5% penalty for >50% utilization
    }
}

fn verify_zk_proof_of_origin(
    proof: &[u8],
    chain_id: u64,
    tx_hash: [u8; 32],
    user: Pubkey,
) -> Result<bool> {
    // Simplified zkProof verification
    // In production, this would use actual zk-SNARK verification
    let proof_hash = solana_program::keccak::hash(&[
        proof,
        &chain_id.to_le_bytes(),
        &tx_hash,
        user.as_ref(),
    ]);

    // Verify proof is not empty and has valid structure
    Ok(proof.len() >= 32 && proof_hash.to_bytes()[0] != 0)
}

// Account structures
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + RedemptionEngine::INIT_SPACE,
        seeds = [b"redemption_engine", vault.key().as_ref()],
        bump
    )]
    pub redemption_engine: Account<'info, RedemptionEngine>,

    /// CHECK: Vault account
    pub vault: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SubmitRedemptionCommitment<'info> {
    #[account(mut)]
    pub redemption_engine: Account<'info, RedemptionEngine>,

    /// CHECK: User position account
    pub user_position: UncheckedAccount<'info>,

    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct RevealRedemptionRequest<'info> {
    #[account(mut)]
    pub redemption_engine: Account<'info, RedemptionEngine>,

    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteRedemptionBatch<'info> {
    #[account(mut)]
    pub redemption_engine: Account<'info, RedemptionEngine>,

    #[account(mut)]
    pub vault: Account<'info, VaultAccount>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct GetInstantExitQuote<'info> {
    pub redemption_engine: Account<'info, RedemptionEngine>,
    pub vault: Account<'info, VaultAccount>,

    /// CHECK: LLM Oracle account
    pub llm_oracle: UncheckedAccount<'info>,

    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct VerifyChainOriginProof<'info> {
    pub redemption_engine: Account<'info, RedemptionEngine>,
    pub user: Signer<'info>,
}

// Data structures
#[account]
pub struct RedemptionEngine {
    pub authority: Pubkey,
    pub vault: Pubkey,
    pub max_queue_size: u64,
    pub min_holding_duration: i64,
    pub epoch_duration: i64,
    pub mev_protection_delay: i64,
    pub current_epoch: u64,
    pub total_pending_redemptions: u64,
    pub commitments: Vec<RedemptionCommitment>,
    pub pending_requests: Vec<RedemptionRequest>,
    pub bump: u8,
}

impl RedemptionEngine {
    pub const INIT_SPACE: usize = 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 4 + 4 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RedemptionCommitment {
    pub user: Pubkey,
    pub commitment_hash: [u8; 32],
    pub tranche_index: u8,
    pub timestamp: i64,
    pub revealed: bool,
    pub executed: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub struct RedemptionRequest {
    pub user: Pubkey,
    pub amount: u64,
    pub min_assets_out: u64,
    pub tranche_index: u8,
    pub timestamp: i64,
    pub priority_score: u64,
    pub status: RedemptionStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum RedemptionStatus {
    Pending,
    Executed,
    Failed,
    Deferred,
}

#[account]
pub struct VaultAccount {
    pub nav_per_share: u64,
    pub total_shares: u64,
    pub available_liquidity: u64,
}

// Events
#[event]
pub struct RedemptionCommitted {
    pub user: Pubkey,
    pub commitment_hash: [u8; 32],
    pub tranche_index: u8,
    pub timestamp: i64,
}

#[event]
pub struct RedemptionRevealed {
    pub user: Pubkey,
    pub amount: u64,
    pub min_assets_out: u64,
    pub tranche_index: u8,
    pub priority_score: u64,
}

#[event]
pub struct RedemptionExecuted {
    pub user: Pubkey,
    pub shares_redeemed: u64,
    pub assets_out: u64,
    pub nav_per_share: u64,
    pub tranche_index: u8,
}

#[event]
pub struct RedemptionFailed {
    pub user: Pubkey,
    pub amount: u64,
    pub reason: String,
}

#[event]
pub struct RedemptionBatchExecuted {
    pub epoch: u64,
    pub executed_count: u32,
    pub total_assets_out: u64,
    pub remaining_requests: u64,
}

#[event]
pub struct InstantExitQuote {
    pub user: Pubkey,
    pub amount: u64,
    pub assets_out: u64,
    pub base_nav: u64,
    pub adjusted_nav: u64,
    pub penalty_bps: u16,
    pub forecast_confidence: u8,
    pub valid_until: i64,
}

#[event]
pub struct ChainOriginVerified {
    pub user: Pubkey,
    pub origin_chain_id: u64,
    pub origin_tx_hash: [u8; 32],
    pub proof_hash: [u8; 32],
}

// Errors
#[error_code]
pub enum RedemptionError {
    #[msg("Insufficient holding duration for redemption")]
    InsufficientHoldingDuration,

    #[msg("Redemption queue is full")]
    QueueFull,

    #[msg("Invalid commitment index")]
    InvalidCommitmentIndex,

    #[msg("Unauthorized to reveal this commitment")]
    UnauthorizedReveal,

    #[msg("Commitment already revealed")]
    AlreadyRevealed,

    #[msg("MEV protection delay still active")]
    MEVProtectionActive,

    #[msg("Invalid commitment reveal")]
    InvalidCommitmentReveal,

    #[msg("Epoch has not ended yet")]
    EpochNotEnded,

    #[msg("Invalid chain origin proof")]
    InvalidChainOriginProof,

    #[msg("Reveal window has expired")]
    RevealWindowExpired,
}
