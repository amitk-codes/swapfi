use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::ANCHOR_DISCRIMINATOR;
use crate::state::Offer;

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct CreateOffer<'info> {
    #[account(mut)]
    pub offer_creator: Signer<'info>,

    #[account(mint::token_program = token_program)]
    pub provided_token_mint: InterfaceAccount<'info, Mint>,

    #[account(mint::token_program = token_program)]
    pub requested_token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = provided_token_mint,
        associated_token::authority = offer_creator,
        associated_token::token_program = token_program,
    )]
    pub offer_creator_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = offer_creator,
        space = ANCHOR_DISCRIMINATOR + Offer::INIT_SPACE,
        seeds = [b"offer", offer_creator.key().as_ref(), id.to_le_bytes().as_ref()],
        bump,
    )]
    pub offer_account: Account<'info, Offer>,

    #[account(
        init,
        payer = offer_account,
        associated_token::mint = provided_token_mint,
        associated_token::authority = offer_account,
        associated_token::token_program = token_program,
    )]
    pub vault_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}
