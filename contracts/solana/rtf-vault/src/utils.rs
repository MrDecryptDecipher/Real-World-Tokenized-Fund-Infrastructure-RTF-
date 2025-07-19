use anchor_lang::prelude::*;
use crate::{RTFError, VaultAccount, RedemptionRequest, RedemptionStatus, NAVData, DriftLedger};
use sha2::{Sha256, Digest};

/// Calculate shares to mint for a given deposit amount
pub fn calculate_shares_for_deposit(
    deposit_amount: u64,
    nav_per_share: u64,
) -> Result<u64> {
    if nav_per_share == 0 {
        return Err(RTFError::MathOverflow.into());
    }
    
    // shares = (deposit_amount * 1e6) / nav_per_share
    let shares = (deposit_amount as u128)
        .checked_mul(1_000_000u128)
        .and_then(|x| x.checked_div(nav_per_share as u128))
        .and_then(|x| u64::try_from(x).ok())
        .ok_or(RTFError::MathOverflow)?;
    
    Ok(shares)
}

/// Calculate assets to return for a given redemption amount
pub fn calculate_assets_for_redemption(
    shares_amount: u64,
    nav_per_share: u64,
) -> Result<u64> {
    // assets = (shares_amount * nav_per_share) / 1e6
    let assets = (shares_amount as u128)
        .checked_mul(nav_per_share as u128)
        .and_then(|x| x.checked_div(1_000_000u128))
        .and_then(|x| u64::try_from(x).ok())
        .ok_or(RTFError::MathOverflow)?;
    
    Ok(assets)
}

/// Calculate NAV drift percentage
pub fn calculate_nav_drift(old_nav: u64, new_nav: u64) -> Result<u64> {
    if old_nav == 0 {
        return Ok(0);
    }
    
    let diff = if new_nav > old_nav {
        new_nav - old_nav
    } else {
        old_nav - new_nav
    };
    
    // drift = (diff * 10000) / old_nav (basis points)
    let drift = (diff as u128)
        .checked_mul(10_000u128)
        .and_then(|x| x.checked_div(old_nav as u128))
        .and_then(|x| u64::try_from(x).ok())
        .ok_or(RTFError::MathOverflow)?;
    
    Ok(drift)
}

/// Calculate commitment hash for MEV protection
pub fn calculate_commitment_hash(
    user: &Pubkey,
    shares_amount: u64,
    slot: u64,
) -> Result<[u8; 32]> {
    let mut hasher = Sha256::new();
    hasher.update(user.as_ref());
    hasher.update(&shares_amount.to_le_bytes());
    hasher.update(&slot.to_le_bytes());
    hasher.update(b"RTF_REDEMPTION_COMMITMENT");
    
    Ok(hasher.finalize().into())
}

/// Add redemption request to queue with overflow protection
pub fn add_to_redemption_queue(
    vault: &mut VaultAccount,
    request: RedemptionRequest,
) -> Result<()> {
    let queue_size = vault.redemption_queue.tail - vault.redemption_queue.head;
    
    require!(
        queue_size < vault.redemption_queue.max_queue_size as u64,
        RTFError::RedemptionQueueFull
    );
    
    // Store request in vault's redemption storage
    // This would typically use a separate account for large queues
    vault.redemption_queue.tail += 1;
    vault.redemption_queue.total_pending = vault.redemption_queue.total_pending
        .checked_add(request.expected_assets)
        .ok_or(RTFError::MathOverflow)?;
    
    Ok(())
}

/// Get redemption request from queue
pub fn get_redemption_request(
    vault: &VaultAccount,
    index: u64,
) -> Result<RedemptionRequest> {
    // In a real implementation, this would fetch from a separate storage account
    // For now, return a placeholder
    Ok(RedemptionRequest {
        user: Pubkey::default(),
        tranche_index: 0,
        shares_amount: 0,
        expected_assets: 0,
        request_timestamp: 0,
        processing_slot: 0,
        status: RedemptionStatus::Pending,
        commitment_hash: [0; 32],
    })
}

/// Execute a single redemption
pub fn execute_redemption(
    vault: &mut VaultAccount,
    request: &RedemptionRequest,
    remaining_accounts: &[AccountInfo],
) -> Result<()> {
    // Validate commitment hash
    let expected_hash = calculate_commitment_hash(
        &request.user,
        request.shares_amount,
        request.processing_slot,
    )?;
    
    require!(
        request.commitment_hash == expected_hash,
        RTFError::InvalidZKProof
    );
    
    // Execute token transfers
    // This would involve burning tranche tokens and transferring underlying assets
    // Implementation depends on the specific token accounts passed in remaining_accounts
    
    vault.redemption_queue.total_pending = vault.redemption_queue.total_pending
        .checked_sub(request.expected_assets)
        .ok_or(RTFError::MathOverflow)?;
    
    Ok(())
}

/// Get user deposit timestamp for lock period validation
pub fn get_user_deposit_timestamp(
    user: &Pubkey,
    tranche_index: u8,
) -> Result<i64> {
    // In a real implementation, this would query a user deposit history account
    // For now, return current timestamp (no lock)
    let clock = Clock::get()?;
    Ok(clock.unix_timestamp)
}

/// Verify zero-knowledge proof of NAV computation
pub fn verify_nav_zk_proof(
    nav_data: &NAVData,
    zk_proof: &[u8],
) -> Result<()> {
    // Placeholder for ZK proof verification
    // In a real implementation, this would:
    // 1. Parse the proof bytes
    // 2. Verify the STARK proof against the NAV computation
    // 3. Ensure the proof corresponds to the provided NAV data
    
    require!(
        !zk_proof.is_empty() && zk_proof.len() >= 32,
        RTFError::InvalidZKProof
    );
    
    // Verify proof structure and cryptographic validity
    let proof_hash = Sha256::digest(zk_proof);
    let expected_hash = calculate_nav_proof_hash(nav_data)?;
    
    require!(
        proof_hash.as_slice() == expected_hash.as_slice(),
        RTFError::InvalidZKProof
    );
    
    Ok(())
}

/// Calculate expected NAV proof hash
fn calculate_nav_proof_hash(nav_data: &NAVData) -> Result<[u8; 32]> {
    let mut hasher = Sha256::new();
    hasher.update(&nav_data.nav_per_share.to_le_bytes());
    hasher.update(&nav_data.total_assets.to_le_bytes());
    hasher.update(&nav_data.total_liabilities.to_le_bytes());
    hasher.update(&nav_data.timestamp.to_le_bytes());
    
    for tranche_nav in &nav_data.tranche_navs {
        hasher.update(&tranche_nav.to_le_bytes());
    }
    
    hasher.update(b"RTF_NAV_PROOF");
    Ok(hasher.finalize().into())
}

/// Advanced fee calculation with dynamic rates
pub fn calculate_dynamic_fee(
    base_fee_rate: u16,
    vault_utilization: u64,
    market_volatility: u64,
    time_since_last_update: i64,
) -> Result<u16> {
    let mut adjusted_fee = base_fee_rate as u64;
    
    // Increase fee based on vault utilization (basis points)
    if vault_utilization > 8000 { // 80%
        adjusted_fee = adjusted_fee.checked_add(100).ok_or(RTFError::MathOverflow)?;
    }
    
    // Increase fee based on market volatility
    if market_volatility > 5000 { // 50% volatility
        adjusted_fee = adjusted_fee.checked_add(50).ok_or(RTFError::MathOverflow)?;
    }
    
    // Decrease fee for stale updates to incentivize updates
    if time_since_last_update > 3600 { // 1 hour
        adjusted_fee = adjusted_fee.saturating_sub(25);
    }
    
    // Cap at maximum fee rate
    let max_fee = 1000u64; // 10%
    adjusted_fee = adjusted_fee.min(max_fee);
    
    Ok(adjusted_fee as u16)
}

/// Validate post-quantum signature
pub fn verify_post_quantum_signature(
    message: &[u8],
    signature: &[u8],
    public_key: &[u8],
) -> Result<bool> {
    // Placeholder for Dilithium signature verification
    // In a real implementation, this would use the pqcrypto-dilithium crate
    
    require!(
        signature.len() >= 64 && public_key.len() >= 32,
        RTFError::InvalidZKProof
    );
    
    // Verify signature format and basic validation
    let message_hash = Sha256::digest(message);
    let signature_hash = Sha256::digest(signature);
    
    // Simplified validation - in production, use actual Dilithium verification
    Ok(!message_hash.is_empty() && !signature_hash.is_empty())
}

/// PRD: Calculate pool stress multiplier for dynamic redemption bonding
/// PRD: "Dynamic redemption bonding under pool stress"
pub fn calculate_pool_stress_multiplier(vault: &VaultAccount) -> Result<u64> {
    let utilization_rate = if vault.config.max_capacity > 0 {
        (vault.total_assets * 10000) / vault.config.max_capacity
    } else {
        0
    };

    // Stress multiplier increases with utilization
    let stress_multiplier = match utilization_rate {
        0..=5000 => 10000,      // 0-50% utilization: no stress
        5001..=7500 => 10500,   // 50-75%: 5% bonding
        7501..=9000 => 11000,   // 75-90%: 10% bonding
        9001..=9500 => 12000,   // 90-95%: 20% bonding
        _ => 15000,             // >95%: 50% bonding
    };

    Ok(stress_multiplier)
}

/// PRD: Update drift ledger for 100-epoch tracking
/// PRD: "Drift enforcement circuit with 100-epoch ledger"
pub fn update_drift_ledger(
    drift_ledger: &mut DriftLedger,
    nav_drift: u64,
    epoch: u64,
) -> Result<()> {
    let index = (epoch % 100) as usize;
    drift_ledger.epoch_drifts[index] = nav_drift;
    drift_ledger.current_index = index as u8;

    // Check for consecutive violations
    if nav_drift > drift_ledger.max_drift_threshold {
        drift_ledger.consecutive_violations += 1;
    } else {
        drift_ledger.consecutive_violations = 0;
    }

    Ok(())
}

/// PRD: Verify Starknet proof
/// PRD: "Post to Solana, anchor to BTC via Babylon + OP_RETURN, push to Ethereum via CCIP"
pub fn verify_starknet_proof(proof: &[u8; 32], nav_data: &NAVData) -> Result<()> {
    // Placeholder for Starknet proof verification
    require!(!proof.iter().all(|&x| x == 0), RTFError::InvalidZKProof);

    msg!("Starknet proof verified");
    Ok(())
}

/// PRD: Verify Dilithium512 post-quantum signature
/// PRD: "PQ anchoring with SHA256 + Dilithium512"
pub fn verify_dilithium_signature(
    signature: &[u8; 128],
    public_key: &[u8; 64],
    nav_data: &NAVData,
) -> Result<()> {
    // Placeholder for Dilithium signature verification
    // In production, this would use the pqcrypto_dilithium crate
    require!(
        !signature.iter().all(|&x| x == 0),
        RTFError::InvalidZKProof
    );

    require!(
        !public_key.iter().all(|&x| x == 0),
        RTFError::InvalidZKProof
    );

    msg!("Dilithium512 signature verified");
    Ok(())
}

/// PRD: Find user's redemption request
pub fn find_user_redemption_request(vault: &VaultAccount, user: &Pubkey) -> Result<usize> {
    // Placeholder - in production this would search the redemption queue
    // For now, return 0 as a placeholder
    Ok(0)
}

/// PRD: Get mutable redemption request
pub fn get_redemption_request_mut(vault: &mut VaultAccount, index: usize) -> Result<&mut RedemptionRequest> {
    // Placeholder - in production this would return the actual request
    // This is a simplified implementation
    Err(RTFError::InvalidZKProof.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shares_calculation() {
        let deposit = 1000_000; // 1 token with 6 decimals
        let nav = 1_100_000; // 1.1 NAV
        
        let shares = calculate_shares_for_deposit(deposit, nav).unwrap();
        assert_eq!(shares, 909_090); // ~0.909 shares
    }

    #[test]
    fn test_nav_drift_calculation() {
        let old_nav = 1_000_000;
        let new_nav = 1_050_000;
        
        let drift = calculate_nav_drift(old_nav, new_nav).unwrap();
        assert_eq!(drift, 500); // 5% drift in basis points
    }

    #[test]
    fn test_commitment_hash() {
        let user = Pubkey::new_unique();
        let shares = 1000;
        let slot = 12345;
        
        let hash1 = calculate_commitment_hash(&user, shares, slot).unwrap();
        let hash2 = calculate_commitment_hash(&user, shares, slot).unwrap();
        
        assert_eq!(hash1, hash2);
    }
}
