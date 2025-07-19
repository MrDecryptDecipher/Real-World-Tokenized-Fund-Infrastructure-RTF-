use anchor_lang::prelude::*;

declare_id!("RTFGovAdvancedDAOGovernanceProgram1111111");

#[program]
pub mod rtf_governance {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
