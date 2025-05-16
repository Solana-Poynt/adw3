use anchor_lang::prelude::*;

#[error_code]
pub enum AdW3Error {
    #[msg("Operation exceeds available funds")]
    InsufficientFunds,

    #[msg("The provided authority is not authorized to perform this action")]
    UnauthorizedAccess,

    #[msg("This auction has already been settled")]
    AuctionAlreadySettled,

    #[msg("Invalid fee percentage. Must be between 0-100")]
    InvalidFeePercentage,

    #[msg("Invalid revenue share. Must be between minimum revenue share and 100")]
    InvalidRevenueShare,

    #[msg("Publisher revenue share below program minimum")]
    RevShareTooLow,

    #[msg("The protocol is currently paused")]
    ProtocolPaused,

    #[msg("String exceeds maximum allowed length")]
    StringTooLong,

    #[msg("The rollup has already been finalized")]
    RollupAlreadyFinalized,

    #[msg("The rollup transaction limit has been exceeded")]
    RollupTransactionLimitExceeded,

    #[msg("Bid amount is below publisher floor price")]
    BidBelowFloorPrice,

    #[msg("The provided auction ID is invalid")]
    InvalidAuctionId,

    #[msg("Invalid merkle proof")]
    InvalidMerkleProof,

    #[msg("Merkle root mismatch")]
    MerkleRootMismatch,

    #[msg("This account has not been delegated to the ephemeral rollup")]
    AccountNotDelegated,

    #[msg("Failed to verify rollup commitment")]
    RollupVerificationFailed,


    #[msg("The amount to withdraw exceeds available balance")]
    ExcessiveWithdrawalAmount,

    #[msg("The requst has already been closed")]
    RequestClosed,

    #[msg("The request has expired")]
    RequestExpired,

    #[msg("This is an invalid Publisher")]
    InvalidPublisher,

    #[msg("This is an invalid DSP")]
    InvalidDSP,

    #[msg("There is an overflow somewhere")]
    Overflow,

}