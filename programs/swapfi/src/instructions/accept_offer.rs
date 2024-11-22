use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::Offer;

use super::transfer_tokens;

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct AcceptOffer<'info> {
    #[account(mut)]
    pub acceptor: Signer<'info>,

    #[account(mut)]
    pub creator: SystemAccount<'info>,

    #[account(mint::token_program = token_program)]
    pub provided_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mint::token_program = token_program)]
    pub requested_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = provided_token_mint,
        associated_token::authority = acceptor,
        associated_token::token_program = token_program,
    )]
    pub acceptor_token_account_for_sending_to_creator: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = acceptor,
        associated_token::mint = provided_token_mint,
        associated_token::authority = offer,
        associated_token::token_program = token_program,
    )]
    pub acceptor_token_granted_by_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = acceptor,
        associated_token::mint = requested_token_mint,
        associated_token::authority = acceptor,
        associated_token::token_program = token_program,
    )]
    pub creator_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        close = creator,
        has_one = creator,
        has_one = provided_token_mint,
        has_one = requested_token_mint,
        seeds = [b"offer", creator.key().as_ref(), id.to_le_bytes().as_ref()],
        bump = offer.bump,
    )]
    pub offer: Box<Account<'info, Offer>>,

    #[account(
        mut,
        associated_token::mint = provided_token_mint,
        associated_token::authority = offer,
        associated_token::token_program = token_program,
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,

    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}




pub fn send_requested_tokens_to_offer_creator(ctx: &Context<AcceptOffer>) -> Result<()> {
    transfer_tokens(
        &ctx.accounts.acceptor_token_account_for_sending_to_creator,
        &ctx.accounts.creator_token_account,
        &ctx.accounts.offer.requested_amount,
        &ctx.accounts.requested_token_mint,
        &ctx.accounts.acceptor,
        &ctx.accounts.token_program,
    )
}

pub fn send_tokens_from_vault_to_acceptor_and_close_vault(ctx: Context<AcceptOffer>) -> Result<()> {
    let seeds = &[
        b"offer",
        ctx.accounts.creator.to_account_info().key.as_ref(),
        &ctx.accounts.offer.id.to_le_bytes()[..],
        &[ctx.accounts.offer.bump],
    ];

    let signer_seeds = [&seeds[..]];

    let accounts = TransferChecked {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx
            .accounts
            .acceptor_token_granted_by_vault
            .to_account_info(),
        mint: ctx.accounts.provided_token_mint.to_account_info(),
        authority: ctx.accounts.offer.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );

    transfer_checked(
        cpi_context,
        ctx.accounts.vault.amount,
        ctx.accounts.provided_token_mint.decimals,
    )?;

    let accounts = CloseAccount {
        account: ctx.accounts.vault.to_account_info(),
        destination: ctx.accounts.acceptor.to_account_info(),
        authority: ctx.accounts.offer.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );

    close_account(cpi_context)
}
