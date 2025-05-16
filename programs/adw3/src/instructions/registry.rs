use anchor_lang::prelude::*;
use crate::constants::ANCHOR_DISCRIMINATOR;
use crate::errors::AdW3Error;
use crate::state::{Publisher, ProtocolConfig,DSP};

#[derive(Accounts)]
pub struct RegisterPublisher<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,
    
    #[account(
        init,
        payer = publisher,
        space = ANCHOR_DISCRIMINATOR + Publisher::INIT_SPACE,
        seeds = [b"publisher", publisher.key().as_ref()],
        bump,
    )]
    pub publisher_state: Account<'info, Publisher>,

    #[account(
        seeds = [b"adw3_config"],
        bump,
    )]
    pub adw_config: Account<'info, ProtocolConfig>,
    
    pub system_program: Program<'info, System>,
}

impl<'info> RegisterPublisher<'info> {
    pub fn register(
        &mut self,
        name: String,
        domain: String,
        payment_address: Option<Pubkey>,
        bumps: RegisterPublisherBumps,
    ) -> Result<()> {
        // Check string lengths individually
        require!(name.len() <= 50, AdW3Error::StringTooLong);
        require!(domain.len() <= 50, AdW3Error::StringTooLong);

        require!(self.adw_config.is_paused == false, AdW3Error::ProtocolPaused);
        
        // Use provided payment address or default to the publisher's address
        let payment = payment_address.unwrap_or(self.publisher.key());
        
        // Initialize publisher state
        self.publisher_state.set_inner(Publisher {
            authority: self.publisher.key(),
            payment_address: payment,
            name,
            domain,
            total_revenue: 0,
            created_at: Clock::get()?.unix_timestamp,
            bump: bumps.publisher_state,
        });

        Ok(())
    }
}


#[derive(Accounts)]
pub struct RegisterDSP <'info> {

  #[account(mut)]
  pub dsp: Signer<'info>,

  #[account(
    init,
    payer = dsp,
    space = ANCHOR_DISCRIMINATOR + DSP::INIT_SPACE,
    seeds = [b"dsp", dsp.key().as_ref()],
    bump,
  )]
  pub dsp_state: Account<'info, DSP>,

  #[account(
    seeds = [b"adw3_config"],
    bump,
  )]
  pub adw_config: Account<'info, ProtocolConfig>,
  pub system_program: Program<'info, System>,

}

impl <'info> RegisterDSP <'info> {
  pub fn register(
    &mut self,
    name: String,
    domain: String,
    bumps: RegisterDSPBumps,
  ) -> Result<()> {
    // Check string lengths individually
    require!(name.len() <= 50, AdW3Error::StringTooLong);
    require!(domain.len() <= 50, AdW3Error::StringTooLong);

    require!(self.adw_config.is_paused == false, AdW3Error::ProtocolPaused);
    // Initialize DSP state
    self.dsp_state.set_inner(DSP {
        authority: self.dsp.key(),
        name,
        domain,
        balance: 0,
        // locked_amount: 0,
        created_at: Clock::get()?.unix_timestamp,
        bump: bumps.dsp_state,
    });

    Ok(())
  }
}