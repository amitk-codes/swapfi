use anchor_lang::prelude::*;

use crate::state::Offer;
use crate::constants::ANCHOR_DISCRIMINATOR;

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct CreateOffer<'info>{
    #[account(mut)]
    pub offer_creator: Signer<'info>,

    #[account(
        init,
        payer = offer_creator,
        space = ANCHOR_DISCRIMINATOR + Offer::INIT_SPACE,
        seeds = [b"offer", offer_creator.key().as_ref(), id.to_le_bytes().as_ref()],
        bump,
    )]
    pub offer_account: Account<'info, Offer>,

    pub system_program: Program<'info, System>,
}