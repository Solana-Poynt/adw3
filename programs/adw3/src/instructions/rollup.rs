use anchor_lang::prelude::*;
use anchor_spl::{
  associated_token::AssociatedToken,
  token::{
      Token,
      TokenAccount,
      Transfer,
      transfer
  }
};

use ephemeral_rollups_sdk::anchor::{commit, delegate};

use crate::constants::{AD_REQUEST_PDA_SEED, AD_RESPONSE_PDA_SEED, AUCTION_RECORD_PDA_SEED};
use crate::errors::AdW3Error;
use crate::state::{AdRequest, AdResponse, AuctionRecord, ExchangeVault, ProtocolConfig, Publisher, RequestStatus,  DSP};

// Step 1: Delegate the ad request to the ER
#[delegate]
#[derive(Accounts)]
#[instruction(ad_request_id: [u8; 32])]
pub struct DelegateAdRequest<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,
  /// CHECK : publisher account for pda derivation
  pub publisher : AccountInfo<'info>,
  /// CHECK : This is the ad request PDA we are delegating
  #[account(
    mut,
    del,
    seeds = [AD_REQUEST_PDA_SEED, publisher.key().as_ref(), &ad_request_id],
    bump,
  )]
  pub ad_request: AccountInfo<'info>, 
}

// Delegate response accounts
#[delegate]
#[derive(Accounts)]
#[instruction(creative_id: [u8; 32])]
pub struct DelegateAdResponse<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,
  ///CHECK: DSP account for PDA derivation
  pub dsp: AccountInfo<'info>,
  ///CHECK: This is the response pda we're delegating
  #[account(
      mut,
      del,
      seeds = [AD_RESPONSE_PDA_SEED, dsp.key().as_ref(), &creative_id],
      bump,
  )]
  pub ad_response: AccountInfo<'info>,
}

// Delegate the auction record account (already initialized during PlaceAsk)
#[delegate]
#[derive(Accounts)]
#[instruction(ad_request_id: [u8; 32])]
pub struct DelegateAuctionRecord<'info> {
  #[account(mut)]
  pub authority: Signer<'info>,
  /// CHECK: publisher account for pda derivation
  pub publisher: AccountInfo<'info>,
  /// CHECK: This is the auction record PDA we are delegating
  #[account(
    mut,
    del,
    seeds = [AUCTION_RECORD_PDA_SEED, publisher.key().as_ref(), &ad_request_id],
    bump,
  )]
  pub auction_record: AccountInfo<'info>,
}

// Step 2: Minimal auction processing in the ER context
#[commit]
#[derive(Accounts)]
#[instruction(ad_request_id: [u8; 32])]
pub struct ProcessAuction<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [AD_REQUEST_PDA_SEED, publisher.key().as_ref(), &ad_request_id],
        bump,
        constraint = ad_request.status == RequestStatus::Open @ AdW3Error::RequestClosed,
        constraint = Clock::get()?.unix_timestamp < ad_request.expiration @ AdW3Error::RequestExpired,
    )]
    pub ad_request: Account<'info, AdRequest>,
    
    // Keep read-only for minimal state updates in rollup
    #[account(
        seeds = [b"publisher", publisher.key().as_ref()],
        bump,
    )]
    pub publisher: Account<'info, Publisher>,
    
    // Keep read-only for minimal state updates in rollup
    #[account(
        seeds = [b"adw3_config"],
        bump,
    )]
    pub adw_config: Account<'info, ProtocolConfig>,
    
    // Auction record is already initialized during PlaceAsk
    #[account(
        mut,
        seeds = [AUCTION_RECORD_PDA_SEED, publisher.key().as_ref(), &ad_request_id],
        bump,
    )]
    pub auction_record: Account<'info, AuctionRecord>,
    
    pub system_program: Program<'info, System>,
  
}

// Process auction results on the base chain after rollup processing
#[derive(Accounts)]
#[instruction(ad_request_id: [u8; 32])]
pub struct ProcessAuctionResults<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [AUCTION_RECORD_PDA_SEED, publisher.key().as_ref(), &ad_request_id],
        bump,
        constraint = !auction_record.is_settled @ AdW3Error::AuctionAlreadySettled,
    )]
    pub auction_record: Account<'info, AuctionRecord>,
    
    #[account(
        mut,
        seeds = [b"publisher", publisher.key().as_ref()],
        bump,
        constraint = publisher.key() == auction_record.publisher @ AdW3Error::InvalidPublisher,
    )]
    pub publisher: Account<'info, Publisher>,
    
    #[account(
        seeds = [b"adw3_config"],
        bump,
    )]
    pub adw_config: Account<'info, ProtocolConfig>,
    
    #[account(
        mut,
        seeds = [b"adw3_vault"],
        bump,
    )]
    pub exchange_vault_state: Account<'info, ExchangeVault>,
    pub system_program: Program<'info, System>,
}

// For undelegating request after auction
#[commit]
#[derive(Accounts)]
#[instruction(ad_request_id: [u8; 32])]
pub struct UndelegateRequestAfterAuction<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    ///CHECK: Publisher account for PDA derivation
    pub publisher: AccountInfo<'info>,
    
    #[account(
        mut,
        seeds = [AD_REQUEST_PDA_SEED, publisher.key().as_ref(), &ad_request_id],
        bump,
    )]
    pub ad_request: Account<'info, AdRequest>,
}

// For undelegating response after auction
#[commit]
#[derive(Accounts)]
#[instruction(creative_id: [u8; 32])]
pub struct UndelegateResponseAfterAuction<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    ///CHECK: DSP account for PDA derivation
    pub dsp: AccountInfo<'info>,
    
    #[account(
        mut,
        seeds = [AD_RESPONSE_PDA_SEED, dsp.key().as_ref(), &creative_id],
        bump,
    )]
    pub ad_response: Account<'info, AdResponse>,
}

#[commit]
#[derive(Accounts)]
#[instruction(ad_request_id: [u8; 32])]
pub struct UndelegateAuctionRecordAfterAuction<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    ///CHECK: Publisher account for PDA derivation
    pub publisher: AccountInfo<'info>,
    
    #[account(
        mut,
        seeds = [AUCTION_RECORD_PDA_SEED, publisher.key().as_ref(), &ad_request_id], 
        bump,
    )]
    pub auction_record: Account<'info, AuctionRecord>,
}

// Final settlement instruction
#[derive(Accounts)]
#[instruction(ad_request_id: [u8; 32])]
pub struct SettleAuction<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [AUCTION_RECORD_PDA_SEED, publisher.key().as_ref(), &ad_request_id],
        bump,
        constraint = !auction_record.is_settled @ AdW3Error::AuctionAlreadySettled,
    )]
    pub auction_record: Account<'info, AuctionRecord>,
    
    #[account(
        mut,
        seeds = [b"publisher", publisher.key().as_ref()],
        bump,
        constraint = publisher.key() == auction_record.publisher @ AdW3Error::InvalidPublisher,
    )]
    pub publisher: Account<'info, Publisher>,

    #[account(
        mut,
        seeds = [b"dsp", dsp.key().as_ref()],
        bump,
    )]
    pub dsp: Account<'info, DSP>,
    
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

    #[account(
        mut,
        associated_token::mint = exchange_vault_state.token_mint,
        associated_token::authority = publisher.payment_address,
    )]
    pub publisher_token_account: Account<'info, TokenAccount>,
  
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}


impl<'info> ProcessAuctionResults<'info> {
    pub fn process_results(
      &mut self,
      ad_request_id: [u8; 32],
     ) -> Result<()> {
        // Ensure auction is not already settled
        require!(!self.auction_record.is_settled, AdW3Error::AuctionAlreadySettled);
        require!(self.auction_record.ad_request_id == ad_request_id, AdW3Error::InvalidAuctionId);
        
        
        // Calculate fees based on clearing price
        let clearing_price = self.auction_record.clearing_price;
        let platform_fee = (clearing_price * self.adw_config.platform_fee_percentage as u64) / 100;
        let publisher_payment = (clearing_price * self.adw_config.publisher_rev_share as u64) / 100;
        
        // Update auction record with calculated fees
        self.auction_record.platform_fee = platform_fee;
        self.auction_record.publisher_payment = publisher_payment;
        
        // Update publisher stats
        self.publisher.total_revenue = self.publisher.total_revenue
            .checked_add(publisher_payment)
            .ok_or(AdW3Error::Overflow)?;
        
        // Update exchange vault
        self.exchange_vault_state.pending_settlements = self.exchange_vault_state.pending_settlements
            .checked_add(publisher_payment + platform_fee)
            .ok_or(AdW3Error::Overflow)?;
  
        Ok(())
    }
}

impl<'info> SettleAuction<'info> {
    pub fn settle(&mut self) -> Result<()> {
        // // Update DSP stats - reduce locked amount
        // self.dsp.locked_amount = self.dsp.locked_amount
        //     .checked_sub(self.auction_record.clearing_price)
        //     .ok_or(AdW3Error::Overflow)?;

        // Begin transfer to publisher 
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.exchange_vault.to_account_info(),
            to: self.publisher_token_account.to_account_info(),
            authority: self.exchange_vault_state.to_account_info(),
        };

        // Need vault state seeds for signing
        let vault_seeds = &[
            b"adw3_vault".as_ref(),
            &[self.exchange_vault_state.bump]
        ];

        let vault_signer = &[&vault_seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, vault_signer);

        transfer(cpi_ctx, self.auction_record.publisher_payment)?;
        
        // Update exchange vault stats
        self.exchange_vault_state.pending_settlements = self.exchange_vault_state.pending_settlements
            .checked_sub(self.auction_record.publisher_payment + self.auction_record.platform_fee)
            .ok_or(AdW3Error::Overflow)?;
        
        self.exchange_vault_state.fee_balance = self.exchange_vault_state.fee_balance
            .checked_add(self.auction_record.platform_fee)
            .ok_or(AdW3Error::Overflow)?;
        
        // Mark auction as settled
        self.auction_record.is_settled = true;
        
        Ok(())
    }
}