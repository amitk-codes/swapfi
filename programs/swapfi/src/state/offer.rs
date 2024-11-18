use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Offer{
  pub id: u64,
  pub creator: Pubkey,
  pub provided_token_mint: Pubkey,
  pub requested_token_mint: Pubkey,
  pub requested_amount: u64, 
  pub bump: u8,
} 