use anchor_lang::prelude::*;
use crate::state::{ProtocolConfig, ExchangeVault};
use anchor_spl::{
   associated_token::AssociatedToken,
   token::{
       Mint,
       Token,
       TokenAccount
   }
};
use crate::constants::ANCHOR_DISCRIMINATOR;
use crate::errors::AdW3Error;

// start with the init of the program
#[derive(Accounts)]
pub struct Initialize<'info> {
  
   #[account(mut)]
   pub authority: Signer<'info>,
   pub token_mint: Account<'info, Mint>,
   
   // the protocol config account
   #[account(
     init,
     payer = authority,
     space = ANCHOR_DISCRIMINATOR + ProtocolConfig::INIT_SPACE,
     seeds = [b"adw3_config"],
     bump,
   )]
   pub adw_config: Account<'info, ProtocolConfig>,
   
   //vault state init
   #[account(
     init,
     payer = authority,
     space = ANCHOR_DISCRIMINATOR + ExchangeVault::INIT_SPACE,
     seeds = [b"adw3_vault"],
     bump,
   )]
   pub exchange_vault_state: Account<'info, ExchangeVault>,
   
   // the vault itself
   #[account(
    init,
    payer = authority,
    associated_token::mint = token_mint,
    associated_token::authority = exchange_vault_state,
   )]
   pub exchange_vault: Account<'info, TokenAccount>,
   
   pub token_program: Program<'info, Token>,
   pub system_program: Program<'info, System>,
   pub associated_token_program: Program<'info, AssociatedToken>,
   pub rent: Sysvar<'info, Rent>,  
}

impl<'info> Initialize<'info> {
   pub fn init(
     &mut self,
     platform_fee_percentage: u8,
     publisher_rev_share: u8,
     bumps: InitializeBumps,
   ) -> Result<()> {
     
     // Validate fee percentages
    require!(platform_fee_percentage <= 100, AdW3Error::InvalidFeePercentage);
    require!(publisher_rev_share <= 100, AdW3Error::InvalidRevenueShare);
    require!(
        platform_fee_percentage + publisher_rev_share <= 100,
        AdW3Error::InvalidFeePercentage
    );
      
      // Initialize the protocol config account
      self.adw_config.set_inner(ProtocolConfig {
        authority: self.authority.key(),
        platform_fee_percentage,
        publisher_rev_share,
        is_paused: false,
        token_mint: self.token_mint.key(),
        bump: bumps.adw_config,
      });
      
      // Initialize the exchange vault state
      self.exchange_vault_state.set_inner(ExchangeVault {
        authority: self.authority.key(),
        total_balance: 0,
        pending_settlements: 0,
        fee_balance: 0,
        token_mint: self.token_mint.key(),
        token_account: self.exchange_vault.key(),
        bump: bumps.exchange_vault_state,
      });
      
      Ok(())
   }
}