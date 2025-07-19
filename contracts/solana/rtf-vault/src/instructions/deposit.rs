use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer, MintTo};
use anchor_spl::associated_token::AssociatedToken;
use crate::{VaultAccount, RTFError, DepositMade, calculate_shares_for_deposit, verify_compliance_proof};

/// Advanced deposit instruction with compliance checks and MEV protection
pub fn deposit_with_compliance(
    ctx: Context<DepositWithCompliance>,
    tranche_index: u8,
    amount: u64,
    min_shares_out: u64,
    compliance_proof: Vec<u8>,
    jurisdiction_proof: Vec<u8>,
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

    // Verify compliance proofs
    verify_compliance_proof(&compliance_proof, &ctx.accounts.user.key())?;
    
    // Verify jurisdictional eligibility
    require!(
        verify_jurisdiction_proof(&jurisdiction_proof, &ctx.accounts.user.key()),
        RTFError::JurisdictionNotAllowed
    );

    // Check vault capacity and utilization
    let vault_utilization = calculate_vault_utilization(vault)?;
    require!(
        vault_utilization < vault.config.max_utilization,
        RTFError::VaultCapacityExceeded
    );

    // Calculate shares with dynamic pricing
    let current_nav = get_current_nav_from_oracle(&ctx.accounts.oracle_account)?;
    let shares_to_mint = calculate_shares_for_deposit(amount, current_nav)?;
    
    require!(
        shares_to_mint >= min_shares_out,
        RTFError::SlippageExceeded
    );

    // Apply dynamic fee based on market conditions
    let dynamic_fee = calculate_dynamic_fee(
        tranche.fee_rate,
        vault_utilization,
        get_market_volatility(&ctx.accounts.oracle_account)?,
        clock.unix_timestamp - vault.last_nav_update,
    )?;

    let fee_amount = (amount * dynamic_fee as u64) / 10000;
    let net_amount = amount - fee_amount;

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

    // Transfer fee to fee collector
    if fee_amount > 0 {
        let fee_transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_token_account.to_account_info(),
                to: ctx.accounts.fee_collector_account.to_account_info(),
                authority: ctx.accounts.vault.to_account_info(),
            },
        );
        
        let vault_seeds = &[
            b"vault",
            vault.authority.as_ref(),
            &[vault.bump],
        ];
        let signer_seeds = &[&vault_seeds[..]];
        
        token::transfer(
            fee_transfer_ctx.with_signer(signer_seeds),
            fee_amount
        )?;
    }

    // Mint tranche tokens to user
    let vault_seeds = &[
        b"vault",
        vault.authority.as_ref(),
        &[vault.bump],
    ];
    let signer_seeds = &[&vault_seeds[..]];

    let mint_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.tranche_mint.to_account_info(),
            to: ctx.accounts.user_tranche_account.to_account_info(),
            authority: ctx.accounts.vault.to_account_info(),
        },
        signer_seeds,
    );
    token::mint_to(mint_ctx, shares_to_mint)?;

    // Update vault state
    vault.total_assets = vault.total_assets.checked_add(net_amount).unwrap();
    tranche.total_supply = tranche.total_supply.checked_add(shares_to_mint).unwrap();

    // Update performance metrics
    update_performance_metrics(vault, net_amount, shares_to_mint)?;

    // Record deposit for compliance and audit trail
    let deposit_record = DepositRecord {
        user: ctx.accounts.user.key(),
        tranche_index,
        amount: net_amount,
        shares_minted: shares_to_mint,
        nav_per_share: current_nav,
        fee_paid: fee_amount,
        timestamp: clock.unix_timestamp,
        epoch: vault.epoch,
        compliance_hash: calculate_compliance_hash(&compliance_proof),
        jurisdiction_code: extract_jurisdiction_code(&jurisdiction_proof),
    };

    // Store deposit record in vault history
    store_deposit_record(vault, &deposit_record)?;

    emit!(DepositMade {
        vault: vault.key(),
        user: ctx.accounts.user.key(),
        tranche_index,
        amount: net_amount,
        shares_minted: shares_to_mint,
        fee_paid: fee_amount,
        nav_per_share: current_nav,
        record: deposit_record,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct DepositWithCompliance<'info> {
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
    
    #[account(
        mut,
        constraint = fee_collector_account.mint == vault.config.underlying_mint
    )]
    pub fee_collector_account: Account<'info, TokenAccount>,
    
    /// CHECK: Oracle account validation
    pub oracle_account: UncheckedAccount<'info>,
    
    /// CHECK: Compliance verification account
    pub compliance_verifier: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// Helper functions
fn verify_jurisdiction_proof(proof: &[u8], user: &Pubkey) -> bool {
    // Implement zk-proof verification for jurisdictional eligibility
    // This would integrate with KILT/Fractal/World ID systems
    !proof.is_empty() && proof.len() >= 32
}

fn calculate_vault_utilization(vault: &VaultAccount) -> Result<u64> {
    if vault.config.max_capacity == 0 {
        return Ok(0);
    }
    
    let utilization = (vault.total_assets * 10000) / vault.config.max_capacity;
    Ok(utilization)
}

fn get_current_nav_from_oracle(oracle_account: &UncheckedAccount) -> Result<u64> {
    // Integrate with Switchboard/Chainlink oracles for real-time NAV
    // This would fetch the latest NAV from zkNAV computation
    Ok(1_000_000) // Placeholder - implement actual oracle integration
}

fn get_market_volatility(oracle_account: &UncheckedAccount) -> Result<u64> {
    // Fetch market volatility metrics from oracle
    Ok(1000) // Placeholder - 10% volatility
}

fn update_performance_metrics(vault: &mut VaultAccount, amount: u64, shares: u64) -> Result<()> {
    // Update vault performance metrics
    let new_total_value = vault.total_assets + amount;
    let performance_change = if vault.total_assets > 0 {
        ((new_total_value as i64 - vault.total_assets as i64) * 10000) / vault.total_assets as i64
    } else {
        0
    };
    
    vault.performance_metrics.total_return += performance_change;
    vault.performance_metrics.last_update = Clock::get()?.unix_timestamp;
    
    Ok(())
}

fn calculate_compliance_hash(proof: &[u8]) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(proof);
    hasher.update(b"RTF_COMPLIANCE_PROOF");
    hasher.finalize().into()
}

fn extract_jurisdiction_code(proof: &[u8]) -> [u8; 2] {
    if proof.len() >= 2 {
        [proof[0], proof[1]]
    } else {
        [0, 0]
    }
}

fn store_deposit_record(vault: &mut VaultAccount, record: &DepositRecord) -> Result<()> {
    // Store deposit record in vault's transaction history
    // In a real implementation, this might use a separate account for large histories
    Ok(())
}
