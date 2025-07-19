use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer, MintTo, Burn};
use anchor_spl::token_2022::{self as token_2022, Token2022};
use anchor_spl::associated_token::AssociatedToken;
use switchboard_v2::AggregatorAccountData;
// use chainlink_ccip::{CCIPMessage, CCIPRouter};  // Will implement interface

pub mod utils;
pub mod state;
pub mod instructions;
pub mod events;
pub mod errors;
pub mod zk_proofs;
pub mod post_quantum;
pub mod cross_chain;
pub mod compliance;
pub mod emergency;

pub use state::*;
pub use utils::*;
pub use instructions::*;
pub use events::*;
pub use errors::*;
pub use zk_proofs::*;
pub use post_quantum::*;
pub use cross_chain::*;
pub use compliance::*;
pub use emergency::*;

declare_id!("RTFVau1tAdvancedSPLTokenVau1tProgram11111111");

/// RTF Vault Program - Advanced Multi-Tranche Token Vault
/// 
/// Features:
/// - SPL Token-2022 compatibility with transfer hooks
/// - Multi-tranche architecture (Senior, Junior, LP)
/// - MEV-protected redemption queues
/// - Dynamic fee structures
/// - Post-quantum signature verification
/// - Real-time NAV updates via oracles
#[program]
pub mod rtf_vault {
    use super::*;

    /// Initialize RTF vault following PRD Section 3.1: SPL-compatible vaults per RTF instance
    /// PRD: "Modular tranching (senior, junior, LP)"
    /// PRD: "Fund-Origin Proof with comprehensive ancestry tracking"
    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        vault_id: String,
        fund_origin_hash: [u8; 32], // PRD: Fund-Origin Proof
        legal_doc_hash: [u8; 32],   // PRD: Legal doc anchoring
        vault_config: VaultConfig,
        tranche_configs: Vec<TrancheConfig>,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        // Validate configuration
        require!(
            tranche_configs.len() >= 2 && tranche_configs.len() <= 5,
            RTFError::InvalidTrancheCount
        );

        // Validate oracle authorities
        require!(
            vault_config.oracle_authority != Pubkey::default(),
            RTFError::InvalidOracleAuthority
        );

        // Initialize vault state following PRD specifications
        vault.authority = ctx.accounts.authority.key();
        vault.config = vault_config;
        vault.total_assets = 0;
        vault.total_liabilities = 0;
        vault.nav_per_share = 1_000_000; // 1.0 with 6 decimals
        vault.last_nav_update = clock.unix_timestamp;
        vault.epoch = 0;
        vault.status = VaultStatus::Active;
        vault.bump = ctx.bumps.vault;

        // PRD: Fund-Origin Proof with comprehensive ancestry tracking
        vault.fund_origin_hash = fund_origin_hash;

        // PRD: Legal doc anchoring (OpenLaw/Accord JSON → machine-verifiable term tree)
        vault.legal_doc_hash = legal_doc_hash;

        // Initialize advanced metrics
        vault.performance_metrics = PerformanceMetrics {
            total_return: 0,
            annualized_return: 0,
            monthly_returns: [0; 12],
            benchmark_return: 0,
            tracking_error: 0,
            information_ratio: 0,
            last_update: clock.unix_timestamp,
        };

        vault.risk_metrics = RiskMetrics {
            var_95: 0,
            var_99: 0,
            volatility: 0,
            sharpe_ratio: 0,
            max_drawdown: 0,
            beta: 0,
            last_update: clock.unix_timestamp,
        };

        // Initialize tranches with enhanced features
        for (i, tranche_config) in tranche_configs.iter().enumerate() {
            vault.tranches[i] = Tranche {
                tranche_type: tranche_config.tranche_type,
                mint: tranche_config.mint,
                total_supply: 0,
                nav_per_share: 1_000_000,
                fee_rate: tranche_config.fee_rate,
                min_deposit: tranche_config.min_deposit,
                max_deposit: tranche_config.max_deposit,
                lock_period: tranche_config.lock_period,
                yield_rate: 0,
                last_yield_update: clock.unix_timestamp,
                waterfall_priority: i as u8,
                protection_level: tranche_config.protection_level,
            };
        }

        // Initialize advanced redemption queue with MEV protection
        vault.redemption_queue = RedemptionQueue {
            head: 0,
            tail: 0,
            total_pending: 0,
            max_queue_size: vault_config.max_redemption_queue_size,
            processing_window: vault_config.redemption_processing_window,
            mev_protection_delay: vault_config.mev_protection_delay,
            batch_size: vault_config.batch_size,
        };

        // Initialize cross-chain state tracking
        vault.cross_chain_state = CrossChainState {
            ethereum_root: [0; 32],
            bitcoin_anchor: [0; 32],
            starknet_proof: [0; 32],
            last_sync_timestamp: clock.unix_timestamp,
            sync_status: SyncStatus::Pending,
        };

        emit!(VaultInitialized {
            vault: vault.key(),
            authority: vault.authority,
            config: vault_config,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Deposit assets into a specific tranche with advanced validation
    pub fn deposit(
        ctx: Context<Deposit>,
        tranche_index: u8,
        amount: u64,
        min_shares_out: u64,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        // Validate tranche
        require!(
            (tranche_index as usize) < vault.tranches.len(),
            RTFError::InvalidTrancheIndex
        );

        let tranche = &mut vault.tranches[tranche_index as usize];

        // Validate deposit amount
        require!(amount >= tranche.min_deposit, RTFError::DepositTooSmall);
        require!(
            tranche.max_deposit == 0 || amount <= tranche.max_deposit,
            RTFError::DepositTooLarge
        );

        // Calculate shares to mint based on current NAV
        let shares_to_mint = calculate_shares_for_deposit(amount, tranche.nav_per_share)?;
        
        require!(
            shares_to_mint >= min_shares_out,
            RTFError::SlippageExceeded
        );

        // Transfer tokens to vault
        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.vault_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::transfer(transfer_ctx, amount)?;

        // Mint tranche tokens to user
        let vault_seeds = &[
            b"vault",
            vault.authority.as_ref(),
            &[vault.bump],
        ];
        let signer_seeds = &[&vault_seeds[..]];

        let mint_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.tranche_mint.to_account_info(),
                to: ctx.accounts.user_tranche_account.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
            },
            signer_seeds,
        );
        token::mint_to(mint_ctx, shares_to_mint)?;

        // Update vault state
        vault.total_assets = vault.total_assets.checked_add(amount).unwrap();
        tranche.total_supply = tranche.total_supply.checked_add(shares_to_mint).unwrap();

        // Record deposit for compliance
        let deposit_record = DepositRecord {
            user: ctx.accounts.user.key(),
            tranche_index,
            amount,
            shares_minted: shares_to_mint,
            nav_per_share: tranche.nav_per_share,
            timestamp: clock.unix_timestamp,
            epoch: vault.epoch,
        };

        emit!(DepositMade {
            vault: vault.key(),
            user: ctx.accounts.user.key(),
            tranche_index,
            amount,
            shares_minted: shares_to_mint,
            record: deposit_record,
        });

        Ok(())
    }

    /// PRD Section 3.5: Redemption Engine with MEV Protection
    /// PRD: "Timestamped redemption queue with MEV protection"
    /// PRD: "Flashloan-resistance via proof-of-holding (duration > M blocks)"
    /// PRD: "Dynamic redemption bonding under pool stress"
    pub fn request_redemption(
        ctx: Context<RequestRedemption>,
        tranche_index: u8,
        shares_amount: u64,
        min_assets_out: u64,
        commitment_hash: [u8; 32], // PRD: Commit-reveal scheme for MEV protection
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

        // PRD: Flashloan-resistance via proof-of-holding (duration > M blocks)
        let deposit_timestamp = get_user_deposit_timestamp(
            &ctx.accounts.user.key(),
            tranche_index,
        )?;

        // Check minimum holding duration for flashloan resistance
        let holding_duration = clock.unix_timestamp - deposit_timestamp;
        require!(
            holding_duration >= 3600, // 1 hour minimum holding
            RTFError::InsufficientHoldingDuration
        );

        // Check tranche lock period
        require!(
            clock.unix_timestamp >= deposit_timestamp + tranche.lock_period as i64,
            RTFError::SharesStillLocked
        );

        // PRD: Dynamic redemption bonding under pool stress
        let pool_stress_multiplier = calculate_pool_stress_multiplier(vault)?;
        let bonding_amount = (shares_amount * pool_stress_multiplier) / 10000; // Basis points

        // Calculate assets to return with stress adjustment
        let base_assets = calculate_assets_for_redemption(
            shares_amount,
            tranche.nav_per_share,
        )?;

        let assets_to_return = if pool_stress_multiplier > 10000 {
            // Under stress, apply bonding discount
            base_assets - ((base_assets * (pool_stress_multiplier - 10000)) / 10000)
        } else {
            base_assets
        };

        require!(
            assets_to_return >= min_assets_out,
            RTFError::SlippageExceeded
        );

        // PRD: MEV-protected batch submission with commit-reveal scheme
        let redemption_request = RedemptionRequest {
            user: ctx.accounts.user.key(),
            tranche_index,
            shares_amount,
            expected_assets: assets_to_return,
            request_timestamp: clock.unix_timestamp,
            processing_slot: clock.slot + vault.config.mev_protection_delay,
            status: RedemptionStatus::Pending,
            commitment_hash, // User-provided commitment hash
            bonding_amount,  // Dynamic bonding based on pool stress
            reveal_deadline: clock.unix_timestamp + 300, // 5 minutes to reveal
        };

        // Add to queue
        add_to_redemption_queue(vault, redemption_request)?;

        emit!(RedemptionRequested {
            vault: vault.key(),
            user: ctx.accounts.user.key(),
            tranche_index,
            shares_amount,
            expected_assets: assets_to_return,
            queue_position: vault.redemption_queue.tail,
            processing_slot: redemption_request.processing_slot,
        });

        Ok(())
    }

    /// PRD: Reveal phase of commit-reveal scheme for MEV protection
    /// PRD: "MEV-protected batch submission"
    pub fn reveal_redemption(
        ctx: Context<RevealRedemption>,
        nonce: u64,
        actual_shares_amount: u64,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        // Find user's pending commitment
        let request_index = find_user_redemption_request(vault, &ctx.accounts.user.key())?;
        let request = get_redemption_request_mut(vault, request_index)?;

        // Verify reveal is within window
        require!(
            clock.unix_timestamp <= request.reveal_deadline,
            RTFError::RevealWindowExpired
        );

        // Verify commitment hash
        let computed_hash = calculate_commitment_hash(
            &ctx.accounts.user.key(),
            actual_shares_amount,
            nonce,
        )?;

        require!(
            computed_hash == request.commitment_hash,
            RTFError::InvalidCommitmentReveal
        );

        // Update request with revealed amount
        request.shares_amount = actual_shares_amount;
        request.status = RedemptionStatus::Revealed;

        emit!(RedemptionRevealed {
            vault: vault.key(),
            user: ctx.accounts.user.key(),
            actual_shares_amount,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Process redemption queue with batch execution
    pub fn process_redemptions(
        ctx: Context<ProcessRedemptions>,
        max_redemptions: u8,
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

        // Process redemptions in FIFO order
        while processed_count < max_redemptions && 
              vault.redemption_queue.head < vault.redemption_queue.tail {
            
            let request = get_redemption_request(vault, vault.redemption_queue.head)?;
            
            // Check if ready for processing (MEV protection)
            if clock.slot < request.processing_slot {
                break;
            }

            // Execute redemption
            execute_redemption(vault, &request, &ctx.remaining_accounts)?;
            
            vault.redemption_queue.head += 1;
            processed_count += 1;
            total_assets_redeemed += request.expected_assets;
        }

        emit!(RedemptionsProcessed {
            vault: vault.key(),
            processed_count,
            total_assets_redeemed,
            remaining_queue_size: vault.redemption_queue.tail - vault.redemption_queue.head,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// PRD Section 3.2: zkNAV Layer with Starknet + ICP
    /// PRD: "NAV is computed daily using a verifiable zk circuit"
    /// PRD: "Drift enforcement circuit with 100-epoch ledger"
    /// PRD: "PQ anchoring with SHA256 + Dilithium512"
    pub fn update_nav_with_zk_proof(
        ctx: Context<UpdateNAV>,
        new_nav_data: NAVData,
        zk_proof: Vec<u8>,
        starknet_proof: [u8; 32],
        dilithium_signature: [u8; 128], // Post-quantum signature
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        // Verify oracle authority
        require!(
            ctx.accounts.oracle_authority.key() == vault.config.oracle_authority,
            RTFError::UnauthorizedOracle
        );

        // PRD: Verify zkProof of NAV computation from Starknet
        verify_nav_zk_proof(&new_nav_data, &zk_proof)?;

        // PRD: Verify Starknet proof
        verify_starknet_proof(&starknet_proof, &new_nav_data)?;

        // PRD: PQ anchoring with SHA256 + Dilithium512
        verify_dilithium_signature(
            &dilithium_signature,
            &vault.config.dilithium_public_key,
            &new_nav_data,
        )?;

        // Validate NAV data freshness
        require!(
            new_nav_data.timestamp >= vault.last_nav_update,
            RTFError::StaleNAVData
        );

        // PRD: Drift enforcement circuit with 100-epoch ledger
        let nav_drift = calculate_nav_drift(vault.nav_per_share, new_nav_data.nav_per_share)?;

        // Update drift ledger for 100-epoch tracking
        update_drift_ledger(&mut vault.drift_ledger, nav_drift, vault.epoch)?;

        // Check for excessive drift
        require!(
            nav_drift <= vault.config.max_nav_drift,
            RTFError::ExcessiveNAVDrift
        );

        // Check for consecutive drift violations
        require!(
            vault.drift_ledger.consecutive_violations <= 3,
            RTFError::ConsecutiveDriftViolations
        );

        // Update vault NAV
        vault.nav_per_share = new_nav_data.nav_per_share;
        vault.last_nav_update = new_nav_data.timestamp;
        vault.total_assets = new_nav_data.total_assets;
        vault.total_liabilities = new_nav_data.total_liabilities;

        // Update tranche NAVs
        for (i, tranche_nav) in new_nav_data.tranche_navs.iter().enumerate() {
            if i < vault.tranches.len() {
                vault.tranches[i].nav_per_share = *tranche_nav;
                vault.tranches[i].last_yield_update = clock.unix_timestamp;
            }
        }

        // PRD: Update cross-chain state for anchoring
        vault.cross_chain_state.starknet_proof = starknet_proof;
        vault.cross_chain_state.last_sync_timestamp = clock.unix_timestamp;
        vault.cross_chain_state.sync_status = SyncStatus::Synced;

        // PRD: Update zkNAV state
        vault.zk_nav_state.current_proof = zk_proof.try_into().unwrap_or([0; 32]);
        vault.zk_nav_state.last_computation = clock.unix_timestamp;
        vault.zk_nav_state.proof_verification_count += 1;

        emit!(NAVUpdated {
            vault: vault.key(),
            old_nav: vault.nav_per_share,
            new_nav: new_nav_data.nav_per_share,
            total_assets: new_nav_data.total_assets,
            total_liabilities: new_nav_data.total_liabilities,
            timestamp: new_nav_data.timestamp,
            oracle: ctx.accounts.oracle_authority.key(),
        });

        // PRD: Emit cross-chain anchoring event
        emit!(CrossChainAnchor {
            vault: vault.key(),
            starknet_proof,
            bitcoin_anchor: vault.cross_chain_state.bitcoin_anchor,
            ethereum_root: vault.cross_chain_state.ethereum_root,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// PRD: "Modular tranching with sophisticated risk management"
    /// Advanced tranche creation with dynamic risk assessment and allocation optimization
    pub fn create_advanced_tranche(
        ctx: Context<CreateAdvancedTranche>,
        tranche_config: AdvancedTrancheConfig,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        // Advanced tranche configuration validation
        require!(
            tranche_config.risk_level <= 100 && tranche_config.risk_level >= 1,
            VaultError::InvalidRiskLevel
        );

        require!(
            tranche_config.target_allocation <= 10000, // Max 100% in basis points
            VaultError::InvalidAllocation
        );

        // Validate risk-return profile consistency
        Self::validate_risk_return_profile(&tranche_config)?;

        // Calculate dynamic risk metrics
        let risk_metrics = Self::calculate_advanced_risk_metrics(&tranche_config, &vault)?;

        // Find available tranche slot
        let tranche_index = vault.tranches.iter().position(|t| t.tranche_type == TrancheType::Inactive)
            .ok_or(VaultError::MaxTranchesReached)?;

        // Initialize advanced tranche with sophisticated parameters
        vault.tranches[tranche_index] = Tranche {
            tranche_type: tranche_config.tranche_type.clone(),
            mint: tranche_config.mint,
            total_supply: 0,
            nav_per_share: 1_000_000, // 1.0 with 6 decimals
            fee_rate: tranche_config.performance_fees.performance_fee_bps,
            min_deposit: tranche_config.liquidity_parameters.min_deposit,
            max_deposit: tranche_config.liquidity_parameters.max_deposit,
            lock_period: tranche_config.redemption_restrictions.lock_period,
            risk_level: tranche_config.risk_level,
            target_allocation: tranche_config.target_allocation,
            current_allocation: 0,
            yield_strategy: tranche_config.yield_strategy,
            correlation_limits: tranche_config.correlation_limits,
            stress_test_scenarios: tranche_config.stress_test_scenarios,
            risk_metrics,
            liquidity_parameters: tranche_config.liquidity_parameters,
            performance_fees: tranche_config.performance_fees,
            redemption_restrictions: tranche_config.redemption_restrictions,
            created_at: clock.unix_timestamp,
            last_rebalance: clock.unix_timestamp,
        };

        // Dynamic allocation optimization
        Self::optimize_tranche_allocation(vault, tranche_index)?;

        // Update vault-level metrics
        vault.total_tranches += 1;
        vault.risk_weighted_allocation += tranche_config.target_allocation * tranche_config.risk_level as u64 / 100;
        Self::update_vault_risk_metrics(vault)?;

        emit!(AdvancedTrancheCreated {
            vault: vault.key(),
            tranche_index: tranche_index as u8,
            tranche_type: tranche_config.tranche_type,
            risk_level: tranche_config.risk_level,
            target_allocation: tranche_config.target_allocation,
            risk_metrics,
            liquidity_tier: tranche_config.liquidity_parameters.liquidity_tier,
            performance_fee_bps: tranche_config.performance_fees.performance_fee_bps,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Advanced risk-return profile validation
    fn validate_risk_return_profile(config: &AdvancedTrancheConfig) -> Result<()> {
        // Validate risk-return consistency
        let expected_return = config.yield_strategy.target_apy;
        let risk_level = config.risk_level as f64;

        // Risk-return correlation check (higher risk should target higher returns)
        let min_expected_return = risk_level * 0.1; // 0.1% per risk point
        require!(
            expected_return >= min_expected_return,
            VaultError::InconsistentRiskReturn
        );

        // Liquidity-risk consistency check
        match config.liquidity_parameters.liquidity_tier {
            LiquidityTier::Instant => {
                require!(risk_level <= 30, VaultError::HighRiskInstantLiquidity);
            },
            LiquidityTier::Daily => {
                require!(risk_level <= 60, VaultError::HighRiskDailyLiquidity);
            },
            LiquidityTier::Weekly => {
                require!(risk_level <= 80, VaultError::HighRiskWeeklyLiquidity);
            },
            LiquidityTier::Monthly => {
                // No restriction for monthly liquidity
            },
        }

        Ok(())
    }

    /// Calculate advanced risk metrics for tranche
    fn calculate_advanced_risk_metrics(
        config: &AdvancedTrancheConfig,
        vault: &Vault,
    ) -> Result<AdvancedRiskMetrics> {
        let base_volatility = config.risk_level as f64 * 0.02; // 2% volatility per risk point

        // Calculate correlation-adjusted risk
        let correlation_adjustment = Self::calculate_correlation_adjustment(config, vault)?;
        let adjusted_volatility = base_volatility * correlation_adjustment;

        // Calculate Value at Risk (VaR)
        let var_95 = adjusted_volatility * 1.645; // 95% VaR
        let var_99 = adjusted_volatility * 2.326; // 99% VaR

        // Calculate Expected Shortfall (Conditional VaR)
        let expected_shortfall = var_95 * 1.2;

        // Calculate Sharpe ratio estimate
        let risk_free_rate = 0.02; // 2% risk-free rate
        let expected_return = config.yield_strategy.target_apy / 100.0;
        let sharpe_ratio = (expected_return - risk_free_rate) / adjusted_volatility;

        // Calculate maximum drawdown estimate
        let max_drawdown = adjusted_volatility * 2.5;

        Ok(AdvancedRiskMetrics {
            volatility: adjusted_volatility,
            var_95,
            var_99,
            expected_shortfall,
            sharpe_ratio,
            max_drawdown,
            correlation_score: correlation_adjustment,
            liquidity_risk_score: Self::calculate_liquidity_risk_score(&config.liquidity_parameters),
            concentration_risk_score: Self::calculate_concentration_risk_score(config),
            tail_risk_score: var_99 / expected_return,
        })
    }

    /// Optimize tranche allocation using advanced algorithms
    fn optimize_tranche_allocation(vault: &mut Vault, tranche_index: usize) -> Result<()> {
        // Modern Portfolio Theory optimization
        let optimal_allocation = Self::calculate_optimal_allocation(vault, tranche_index)?;

        // Risk parity adjustment
        let risk_parity_allocation = Self::calculate_risk_parity_allocation(vault, tranche_index)?;

        // Black-Litterman model adjustment
        let bl_adjustment = Self::calculate_black_litterman_adjustment(vault, tranche_index)?;

        // Final allocation combining all methods
        let final_allocation = (optimal_allocation * 0.4) +
                              (risk_parity_allocation * 0.4) +
                              (bl_adjustment * 0.2);

        vault.tranches[tranche_index].optimized_allocation = final_allocation as u64;

        Ok(())
    }
}

}

// Account validation contexts
#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + VaultAccount::INIT_SPACE,
        seeds = [b"vault", authority.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, VaultAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub vault: Account<'info, VaultAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key(),
        constraint = user_token_account.mint == vault.config.underlying_mint
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_token_account.owner == vault.key(),
        constraint = vault_token_account.mint == vault.config.underlying_mint
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub tranche_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = tranche_mint,
        associated_token::authority = user
    )]
    pub user_tranche_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RequestRedemption<'info> {
    #[account(mut)]
    pub vault: Account<'info, VaultAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = user_tranche_account.owner == user.key()
    )]
    pub user_tranche_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ProcessRedemptions<'info> {
    #[account(mut)]
    pub vault: Account<'info, VaultAccount>,

    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RevealRedemption<'info> {
    #[account(mut)]
    pub vault: Account<'info, VaultAccount>,

    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateNAV<'info> {
    #[account(mut)]
    pub vault: Account<'info, VaultAccount>,

    pub oracle_authority: Signer<'info>,

    /// CHECK: Oracle account validation
    pub oracle_account: UncheckedAccount<'info>,
}

// Data structures
#[account]
#[derive(InitSpace)]
pub struct VaultAccount {
    pub authority: Pubkey,
    pub config: VaultConfig,
    pub total_assets: u64,
    pub total_liabilities: u64,
    pub nav_per_share: u64,
    pub last_nav_update: i64,
    pub epoch: u64,
    pub status: VaultStatus,
    pub bump: u8,
    #[max_len(5)]
    pub tranches: Vec<Tranche>,
    pub redemption_queue: RedemptionQueue,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct VaultConfig {
    pub underlying_mint: Pubkey,
    pub oracle_authority: Pubkey,
    pub operator: Pubkey,
    pub max_redemption_queue_size: u32,
    pub redemption_processing_window: u32,
    pub mev_protection_delay: u64,
    pub max_nav_drift: u64,
    pub fee_collector: Pubkey,
    pub emergency_pause_authority: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct Tranche {
    pub tranche_type: TrancheType,
    pub mint: Pubkey,
    pub total_supply: u64,
    pub nav_per_share: u64,
    pub fee_rate: u16,
    pub min_deposit: u64,
    pub max_deposit: u64,
    pub lock_period: u32,
    pub yield_rate: u64,
    pub last_yield_update: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub enum TrancheType {
    Senior,
    Junior,
    LP,
    Mezzanine,
    Equity,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub enum VaultStatus {
    Active,
    Paused,
    Emergency,
    Liquidating,
}

// Error definitions
#[error_code]
pub enum RTFError {
    #[msg("Invalid tranche count")]
    InvalidTrancheCount,
    #[msg("Invalid tranche index")]
    InvalidTrancheIndex,
    #[msg("Deposit amount too small")]
    DepositTooSmall,
    #[msg("Deposit amount too large")]
    DepositTooLarge,
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    #[msg("Insufficient shares for redemption")]
    InsufficientShares,
    #[msg("Shares still locked")]
    SharesStillLocked,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Unauthorized oracle")]
    UnauthorizedOracle,
    #[msg("Stale NAV data")]
    StaleNAVData,
    #[msg("Excessive NAV drift")]
    ExcessiveNAVDrift,
    #[msg("Redemption queue full")]
    RedemptionQueueFull,
    #[msg("Invalid zkProof")]
    InvalidZKProof,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Insufficient holding duration for flashloan resistance")]
    InsufficientHoldingDuration,
    #[msg("Reveal window expired")]
    RevealWindowExpired,
    #[msg("Invalid commitment reveal")]
    InvalidCommitmentReveal,
    #[msg("Consecutive drift violations")]
    ConsecutiveDriftViolations,
    #[msg("Invalid Starknet proof")]
    InvalidStarknetProof,
    #[msg("Invalid Dilithium signature")]
    InvalidDilithiumSignature,
    #[msg("Redemption request not found")]
    RedemptionRequestNotFound,
}

/// PRD: Advanced Yield Strategy for sophisticated return optimization
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct YieldStrategy {
    pub strategy_type: YieldStrategyType,
    pub target_apy: f64,
    pub risk_tolerance: f64,
    pub rebalancing_frequency: RebalancingFrequency,
    pub underlying_protocols: Vec<ProtocolAllocation>,
    pub hedging_strategies: Vec<HedgingStrategy>,
    pub leverage_parameters: LeverageParameters,
    pub performance_benchmarks: Vec<PerformanceBenchmark>,
    pub optimization_algorithm: OptimizationAlgorithm,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum YieldStrategyType {
    Conservative,
    Moderate,
    Aggressive,
    DynamicRiskParity,
    VolatilityTargeting,
    MomentumBased,
    MeanReversion,
    ArbitrageFocused,
    Custom,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum RebalancingFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    EventDriven,
    VolatilityTriggered,
    PerformanceTriggered,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ProtocolAllocation {
    pub protocol_name: String,
    pub allocation_percentage: f64,
    pub risk_weight: f64,
    pub expected_yield: f64,
    pub liquidity_score: f64,
    pub security_rating: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct HedgingStrategy {
    pub hedge_type: HedgeType,
    pub hedge_ratio: f64,
    pub cost_basis_points: u16,
    pub effectiveness_score: f64,
    pub rebalancing_threshold: f64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum HedgeType {
    DeltaNeutral,
    VolatilityHedge,
    CurrencyHedge,
    InterestRateHedge,
    CreditHedge,
    LiquidityHedge,
    TailRiskHedge,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LeverageParameters {
    pub max_leverage: f64,
    pub target_leverage: f64,
    pub leverage_cost: f64,
    pub margin_requirements: f64,
    pub liquidation_threshold: f64,
    pub deleveraging_triggers: Vec<DeleveragingTrigger>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DeleveragingTrigger {
    pub trigger_type: TriggerType,
    pub threshold_value: f64,
    pub action_severity: ActionSeverity,
    pub cooldown_period: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum TriggerType {
    VolatilitySpike,
    DrawdownLimit,
    LiquidityDrop,
    CorrelationBreakdown,
    RiskMetricBreach,
    ExternalShock,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum ActionSeverity {
    Mild,
    Moderate,
    Aggressive,
    Emergency,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PerformanceBenchmark {
    pub benchmark_name: String,
    pub benchmark_type: BenchmarkType,
    pub target_outperformance: f64,
    pub tracking_error_limit: f64,
    pub information_ratio_target: f64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum BenchmarkType {
    MarketIndex,
    PeerGroup,
    RiskFreeRate,
    CustomComposite,
    AbsoluteReturn,
    VolatilityAdjusted,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum OptimizationAlgorithm {
    MeanVarianceOptimization,
    BlackLitterman,
    RiskParity,
    MinimumVariance,
    MaximumDiversification,
    HierarchicalRiskParity,
    MachineLearningBased,
    QuantumOptimization,
}

// Helper functions and additional structures will be in separate modules
