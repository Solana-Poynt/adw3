use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral;
use ephemeral_rollups_sdk::cpi::DelegateConfig;
use ephemeral_rollups_sdk::ephem::{commit_accounts, commit_and_undelegate_accounts};

// Module declarations
mod state;
mod constants;
mod errors;
mod instructions;

// Import from modules
use instructions::*;
use constants::{AD_REQUEST_PDA_SEED, AD_RESPONSE_PDA_SEED, AUCTION_RECORD_PDA_SEED};
use state::{
    AdResponse, RequestStatus, ResponseStatus,
    RequestDelegated, AuctionCompleted, 
};

declare_id!("FdurepYmbwe1Wv2uqn91E45U5DS28Ai2uaFiwpgWaBri");

#[ephemeral]
#[program]
pub mod poynt_adw3 {
    use super::*;

    // Initialize instruction handler
    pub fn initialize(
        ctx: Context<Initialize>,
        platform_fee_percentage: u8,
        publisher_rev_share: u8,
    ) -> Result<()> {
       ctx.accounts.init(
            platform_fee_percentage, 
            publisher_rev_share,
            ctx.bumps
        )
    }

    pub fn register_publisher(
        ctx: Context<RegisterPublisher>,
        name: String,
        domain: String,
        payment_address: Option<Pubkey>,
    ) -> Result<()> {
        ctx.accounts.register(
            name,
            domain, 
            payment_address, 
            ctx.bumps
        )
    }

    pub fn register_dsp(
        ctx: Context<RegisterDSP>,
        name: String,
        domain: String,
    ) -> Result<()> {
        ctx.accounts.register(
            name,
            domain, 
            ctx.bumps
        )
    }

    pub fn place_ad_ask(
        ctx: Context<PlaceAsk>,
        ad_request_id: [u8; 32],
        ad_floor_price: u64,
    ) -> Result<()> {
        ctx.accounts.place_ask(
            ad_request_id,
            ad_floor_price,
            ctx.bumps
        )
    }

    pub fn place_ad_bid(
        ctx: Context<PlaceBid>,
        ad_request_id: [u8; 32],
        bid_amount: u64,
        creative_id: [u8; 32],
    ) -> Result<()> {
        ctx.accounts.place_bid(
            ad_request_id,
            bid_amount,
            creative_id,
            ctx.bumps
        )
    }

    // ===== EPHEMERAL ROLLUPS FUNCTIONALITY =====

    // Delegate ad request to ER
    pub fn delegate_ad_request(
        ctx: Context<DelegateAdRequest>,
        ad_request_id: [u8; 32],
    ) -> Result<()> {
        ctx.accounts.delegate_ad_request(
            &ctx.accounts.authority,
            &[AD_REQUEST_PDA_SEED, ctx.accounts.publisher.key().as_ref(), &ad_request_id],
            DelegateConfig::default()
        )?;

        // Emit delegation event
        emit!(RequestDelegated {
            request_id: ad_request_id,
            publisher: ctx.accounts.publisher.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }
    
    // Delegate ad response to ER
    pub fn delegate_ad_response(
        ctx: Context<DelegateAdResponse>,
        creative_id: [u8; 32],
    ) -> Result<()> {
        ctx.accounts.delegate_ad_response(
            &ctx.accounts.authority,
            &[AD_RESPONSE_PDA_SEED, ctx.accounts.dsp.key().as_ref(), &creative_id],
            DelegateConfig::default()
        )?;
        
        Ok(())
    }
    
   
    pub fn delegate_auction_record(
        ctx: Context<DelegateAuctionRecord>,
        ad_request_id: [u8; 32],
    ) -> Result<()> {
        ctx.accounts.delegate_auction_record(
            &ctx.accounts.authority,
            &[AUCTION_RECORD_PDA_SEED, ctx.accounts.publisher.key().as_ref(), &ad_request_id],
            DelegateConfig::default()
        )?;
        
        Ok(())
    }

    // MINIMAL auction processing in ephemeral rollup
    // Only determine winner and clearing price
    pub fn process_auction<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, ProcessAuction<'info>>,
        ad_request_id: [u8; 32],
    ) -> Result<()> 
    where 'c: 'info
    {
        // Mark the request as in progress
        ctx.accounts.ad_request.status = RequestStatus::AuctionInProgress;
        
        // Process all bid responses (from remaining_accounts)
        let mut valid_bids: Vec<(u64, Pubkey, Pubkey)> = Vec::new(); // (bid_amount, dsp, response_key)
        let mut response_map: std::collections::HashMap<Pubkey, &AccountInfo> = std::collections::HashMap::new();
    
        // Process bids from remaining_accounts
        for response_info in ctx.remaining_accounts.iter() {
            // Skip if not a valid AdResponse account
            if response_info.owner != &crate::ID {
                continue;
            }
    
            // Deserialize the response account
            let response = match Account::<AdResponse>::try_from(response_info) {
                Ok(response) => response,
                Err(_) => continue, // Skip if deserialization fails
            };
    
            // Skip if not for our particular request
            if response.request_id != ad_request_id {
                continue;
            }
    
            // Skip if bid is less than floor price
            if response.bid_amount < ctx.accounts.ad_request.floor_price {
                continue;
            }
    
            // Add valid bid
            let response_key = response_info.key();
            valid_bids.push((response.bid_amount, response.dsp, response_key));
            response_map.insert(response_key, response_info);
        }
    
        // If no valid bids found
        if valid_bids.is_empty() {
            // Mark request as completed with no winner
            ctx.accounts.ad_request.status = RequestStatus::Completed;
            
            // Commit the updated ad request
            commit_accounts(
                &ctx.accounts.authority, 
                vec![&ctx.accounts.ad_request.to_account_info()], 
                &ctx.accounts.magic_context,
                &ctx.accounts.magic_program,
            )?;
            return Ok(());
        }
        
        // Sort bids in descending order (highest first)
        valid_bids.sort_by(|a, b| b.0.cmp(&a.0));
        
        // Get winning bid (highest amount)
        let winning_bid = &valid_bids[0].clone();
        
        // Determine clearing price (second-price auction logic)
        let clearing_price = if valid_bids.len() > 1 {
            // Use second highest bid price
            valid_bids[1].0
        } else {
            // If only one bid, use floor price
            ctx.accounts.ad_request.floor_price
        };
        
        // Ensure clearing price is at least the floor price
        let clearing_price = clearing_price.max(ctx.accounts.ad_request.floor_price);
        
        // Update auction record with minimal info needed
        ctx.accounts.auction_record.winning_dsp = Some(winning_bid.1);
        ctx.accounts.auction_record.bid_amount = winning_bid.0;
        ctx.accounts.auction_record.clearing_price = clearing_price;
        ctx.accounts.auction_record.timestamp = Clock::get()?.unix_timestamp;


        // Mark the request as completed
        ctx.accounts.ad_request.status = RequestStatus::Completed;
        
        // Get winning key for comparison
        let winning_key = winning_bid.2;

        // Update and commit response accounts
        for (_bid_amount, _dsp, response_key) in valid_bids {
            if let Some(response_info) = response_map.get(&response_key) {
                let is_winner = response_key == winning_key;
                
                // Get mutable copy to update
                let mut response_data = Account::<AdResponse>::try_from(response_info)?;
                
                // Update status
                response_data.status = if is_winner { 
                    ResponseStatus::Win 
                } else { 
                    ResponseStatus::Loss 
                };
                
                // Access the account info
                let mut data = response_info.try_borrow_mut_data()?;
                response_data.try_serialize(&mut *data)?;
                
                // Commit the updated account
                commit_accounts(
                    &ctx.accounts.authority,
                    vec![*response_info],
                    &ctx.accounts.magic_context,
                    &ctx.accounts.magic_program,
                )?;
            }
        }
        
        // Commit the updated ad request and auction record
        commit_accounts(
            &ctx.accounts.authority,
            vec![
                &ctx.accounts.ad_request.to_account_info(),
                &ctx.accounts.auction_record.to_account_info(),
            ],
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
        )?;
        
        // Emit auction completion event
        emit!(AuctionCompleted {
            request_id: ad_request_id,
            publisher: ctx.accounts.publisher.key(),
            winning_dsp: winning_bid.1,
            clearing_price,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }

    // Process auction results on the base chain - calculates fees and updates state
    pub fn process_auction_results(
        ctx: Context<ProcessAuctionResults>,
        ad_request_id: [u8; 32],
    ) -> Result<()> {
        // Process auction results
        ctx.accounts.process_results(ad_request_id)
    }

    // Undelegate accounts after auction
    pub fn undelegate_request_after_auction(
        ctx: Context<UndelegateRequestAfterAuction>,
        _ad_request_id: [u8; 32],
    ) -> Result<()> {
        // Undelegate the ad request account
        commit_and_undelegate_accounts(
            &ctx.accounts.authority,
            vec![&ctx.accounts.ad_request.to_account_info()],
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
        )?;
        
        Ok(())
    }

    // Undelegate response account after auction
    pub fn undelegate_response_after_auction(
        ctx: Context<UndelegateResponseAfterAuction>,
        _creative_id: [u8; 32],
    ) -> Result<()> {
        // Undelegate the ad response account
        commit_and_undelegate_accounts(
            &ctx.accounts.authority,
            vec![&ctx.accounts.ad_response.to_account_info()],
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
        )?;
        
        Ok(())
    }
    
    // Undelegate auction record after auction
    pub fn undelegate_auction_record_after_auction(
        ctx: Context<UndelegateAuctionRecordAfterAuction>,
        _ad_request_id: [u8; 32],
    ) -> Result<()> {
        // Undelegate the auction record account
        commit_and_undelegate_accounts(
            &ctx.accounts.authority,
            vec![&ctx.accounts.auction_record.to_account_info()],
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
        )?;
        
        Ok(())
    }
    
    // Settle auction by transferring funds
    pub fn settle_auction(
        ctx: Context<SettleAuction>,
    ) -> Result<()> {
        ctx.accounts.settle()
    }
}