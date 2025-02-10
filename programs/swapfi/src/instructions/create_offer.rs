use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::ANCHOR_DISCRIMINATOR;
use crate::state::Offer;

use super::transfer_tokens;

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
    pub offer_creator_provided_token_account: InterfaceAccount<'info, TokenAccount>,

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
        payer = offer_creator,
        associated_token::mint = provided_token_mint,
        associated_token::authority = offer_account,
        associated_token::token_program = token_program,
    )]
    pub vault_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn lock_tokens_to_vault(ctx: &Context<CreateOffer>, provided_token_amount: u64) -> Result<()> {
    transfer_tokens(
        &ctx.accounts.offer_creator_provided_token_account,
        &ctx.accounts.vault_account,
        &provided_token_amount,
        &ctx.accounts.provided_token_mint,
        &ctx.accounts.offer_creator,
        &ctx.accounts.token_program,
    )
}

pub fn save_offer_on_chain(
    ctx: Context<CreateOffer>,
    id: u64,
    requested_amount: u64,
) -> Result<()> {
    ctx.accounts.offer_account.set_inner(Offer {
        id,
        creator: ctx.accounts.offer_creator.key(),
        provided_token_mint: ctx.accounts.provided_token_mint.key(),
        requested_token_mint: ctx.accounts.requested_token_mint.key(),
        requested_amount,
        bump: ctx.bumps.offer_account,
    });
    Ok(())
}
