use anchor_lang::prelude::*;

//this is the accounts struct/state for AD-W3
//. publisher, dsp, exchange vault, auctionrecord, rollupinstance, protocolconfig

#[account]
#[derive(InitSpace)]
pub struct Publisher {
    pub authority: Pubkey,
    pub payment_address: Pubkey,
    #[max_len(50)] 
    pub name: String,
    #[max_len(50)]
    pub domain: String,
    pub total_revenue: u64,
    pub created_at: i64,
    pub bump: u8,
    // we need to ad publisher's total ad recieved
}

#[account]
#[derive(InitSpace)]
pub struct DSP {
    pub authority: Pubkey,
    #[max_len(50)] 
    pub name: String,
    #[max_len(50)] 
    pub domain: String,
    pub balance: u64,
    // pub locked_amount: u64,
    pub created_at: i64,
    pub bump: u8,
    // we need to ad DSP's total ad spent
}

#[account]
#[derive(InitSpace)]
pub struct ExchangeVault {
    pub authority: Pubkey,
    pub total_balance: u64,
    pub pending_settlements: u64, // Funds reserved for in-progress settlements
    pub fee_balance: u64,         // Platform fees collected
    pub token_mint: Pubkey,       // USDC token mint
    pub token_account: Pubkey,    // Token account holding USDC
    pub bump: u8,
}

// Account of each successful auction
#[account]
#[derive(InitSpace)]
pub struct AuctionRecord {
    #[max_len(32)]
    pub id: [u8; 32],
    pub ad_request_id: [u8; 32],
    pub publisher: Pubkey,
    pub winning_dsp: Option<Pubkey>,
    pub bid_amount: u64,
    pub clearing_price: u64,
    pub publisher_payment: u64,
    pub platform_fee: u64,
    pub timestamp: i64,
    pub is_settled: bool,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum RequestStatus {
    Open,
    AuctionInProgress,
    Completed,
}

impl Space for RequestStatus{
  const INIT_SPACE: usize = 1;
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum ResponseStatus {
    Submitted,
    AuctionInProgress,
    Win,
    Loss,
}

impl Space for ResponseStatus{
  const INIT_SPACE: usize = 1;
}

#[account]
#[derive(InitSpace)]
pub struct AdRequest {
    pub publisher: Pubkey,         // Publisher account
    pub request_id: [u8; 32], // Reference ID for off-chain details
    pub floor_price: u64, // Minimum bid 
    pub expiration: i64,
    pub status: RequestStatus, // Current status
    pub bump: u8,                  
}

#[account]
#[derive(InitSpace)]
pub struct AdResponse {
    pub dsp: Pubkey,               // DSP making the bid
    pub request_id: [u8; 32], // Reference ID matching the AdRequest
    pub bid_amount: u64,           // Bid amount
    pub creative_id: [u8; 32],     // Hash of creative content
    pub created_at: i64,           // When bid was submitted
    pub status: ResponseStatus,    // Current status
    pub bump: u8,
}

// #[account]
// #[derive(InitSpace)]
// pub struct RollupInstance {
//     #[max_len(32)]
//     pub id: String,
//     pub creator: Pubkey,
//     #[max_len(20)]
//     pub region: String,
//     pub max_transactions: u64,
//     pub transaction_count: u64,
//     pub state_root: [u8; 32],      // Merkle root of rollup state
//     pub is_finalized: bool,
//     pub created_at: i64,
//     pub bump: u8,
// }

#[account]
#[derive(InitSpace)]
pub struct ProtocolConfig {
    pub authority: Pubkey,
    pub platform_fee_percentage: u8,
    pub publisher_rev_share: u8,
    pub is_paused: bool,
    pub token_mint: Pubkey,
    pub bump: u8,
}



//EVENTS 

#[event]
pub struct RequestDelegated {
    pub request_id: [u8; 32],
    pub publisher: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct AuctionCompleted {
    pub request_id: [u8; 32],
    pub publisher: Pubkey,
    pub winning_dsp: Pubkey,
    pub clearing_price: u64,
    pub timestamp: i64,
}