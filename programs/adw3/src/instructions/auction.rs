use anchor_lang::prelude::*;
use crate::constants::{ANCHOR_DISCRIMINATOR, AUCTION_RECORD_PDA_SEED};
use crate::state::{
  AdRequest, AdResponse, ExchangeVault, ProtocolConfig, Publisher, RequestStatus, ResponseStatus, DSP, AuctionRecord, 
};
use crate::errors::AdW3Error;

use anchor_spl::{
  associated_token::AssociatedToken,
  token::{
      Token,
      TokenAccount,
      Transfer,
      transfer
  }
};

// the instructions module contains the logic for the auction program
//place bid instruction
#[derive(Accounts)]
//the value of the ad_request_id is the seed for the ad_request account so we have unique pda for each ad request
#[instruction(ad_request_id: [u8; 32])]
pub struct PlaceAsk<'info>{
    #[account(mut)]
    pub publisher: Signer<'info>,

    #[account(
        mut,
        seeds = [b"publisher", publisher.key().as_ref()],
        bump,
    )]
    pub publisher_state: Account<'info, Publisher>,

    #[account(
        seeds = [b"adw3_config"],
        bump,
    )]
    pub adw_config: Account<'info, ProtocolConfig>,

    #[account(
      init,
      payer = publisher,
      space = ANCHOR_DISCRIMINATOR + AuctionRecord::INIT_SPACE,
      seeds = [AUCTION_RECORD_PDA_SEED, publisher.key().as_ref(), &ad_request_id],
      bump,
      
    )]
    pub auction_record: Account<'info, AuctionRecord>,

    #[account(
        init,
        payer = publisher,
        space = ANCHOR_DISCRIMINATOR + AdRequest::INIT_SPACE,
        seeds = [b"ad_request", publisher.key().as_ref(), &ad_request_id],
        bump,
    )]
    pub ad_request: Account<'info, AdRequest>,
    
    pub system_program: Program<'info, System>,
}

impl <'info> PlaceAsk<'info> {
    pub fn place_ask(
        &mut self,
        ad_request_id: [u8; 32],
        ad_floor_price: u64,
        bumps: PlaceAskBumps
    ) -> Result<()> {
        // Check if protocol is paused
        require!(!self.adw_config.is_paused, AdW3Error::ProtocolPaused);

        // Create a unique auction ID
        let auction_id_str = format!("auction-{}", hex::encode(&ad_request_id[0..8]));
        let mut auction_id = [0u8; 32];
        let bytes = auction_id_str.as_bytes();
        let len = usize::min(bytes.len(), 32);
        auction_id[..len].copy_from_slice(&bytes[..len]);
        
        // Initialize ad request state
        self.ad_request.set_inner(AdRequest {
            publisher: self.publisher.key(),
            request_id: ad_request_id,
            floor_price: ad_floor_price,
            expiration: Clock::get()?.unix_timestamp + 60 * 60 * 12, // 12 hours expiration
            status: RequestStatus::Open,
            bump: bumps.ad_request,
        });

        self.auction_record.set_inner( AuctionRecord {
            id: auction_id,
            ad_request_id,
            publisher: self.publisher.key(),
            winning_dsp: None,
            bid_amount: 0,
            clearing_price : 0,
            publisher_payment : 0,
            platform_fee : 0,
            timestamp: 0,
            is_settled: false,
            bump: bumps.auction_record,
        }

        );

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(creative_id: [u8; 32])]
pub struct PlaceBid<'info> {
  #[account(mut)]
  pub dsp: Signer<'info>,
  #[account(
    mut,
    seeds = [b"dsp", dsp.key().as_ref()],
    bump,
  )]
  pub dsp_state: Account<'info, DSP>,

  #[account(
    init,
    payer = dsp,
    space = ANCHOR_DISCRIMINATOR + AdResponse::INIT_SPACE,
    seeds = [b"ad_response", dsp.key().as_ref(), &creative_id],
    bump,
  )]
  pub ad_dsp_response: Account<'info, AdResponse>,

  // config accounts
  #[account(
    seeds = [b"adw3_config"],
    bump,
)]
pub adw_config: Account<'info, ProtocolConfig>,
// need to load vault here.
#[account(
    mut,
    seeds = [b"adw3_vault"],
    bump,
)]
pub exchange_vault_state: Account<'info, ExchangeVault>,

#[account(
    mut,
    associated_token::mint = exchange_vault_state.token_mint,
    associated_token::authority = exchange_vault_state,
)]
pub exchange_vault: Account<'info, TokenAccount>,

  // DSP's token account
  #[account(
    mut,
    associated_token::mint = exchange_vault_state.token_mint,
    associated_token::authority = dsp,
  )]
  pub dsp_token_account: Account<'info, TokenAccount>,

pub token_program: Program<'info, Token>,
pub system_program: Program<'info, System>,
pub associated_token_program: Program<'info, AssociatedToken>,
}


impl <'info> PlaceBid<'info> {
  pub fn place_bid(
    &mut self,
    ad_request_id: [u8; 32],
    bid_amount: u64,
    creative_id: [u8; 32],
    bumps: PlaceBidBumps
  ) -> Result<()> {
    // Check string lengths individually
    require!(self.adw_config.is_paused == false, AdW3Error::ProtocolPaused);

    // start with cpi call to deposit the bid amount into the exchange vault

    let cpi_program = self.token_program.to_account_info();
    
    let cpi_accounts = Transfer{
      from: self.dsp_token_account.to_account_info(),
      to:  self.exchange_vault.to_account_info(),
      authority:  self.dsp.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    transfer(cpi_ctx, bid_amount)?;

    // Initialize ad request state
    self.ad_dsp_response.set_inner(AdResponse {
      dsp: self.dsp.key(),
      request_id: ad_request_id,
      creative_id,
      bid_amount,
      created_at: Clock::get()?.unix_timestamp,
      status: ResponseStatus::Submitted,
      bump: bumps.ad_dsp_response,
    });

    // self.dsp_state.locked_amount += bid_amount;
    
    self.exchange_vault_state.total_balance += bid_amount;
    Ok(())
  }
}
