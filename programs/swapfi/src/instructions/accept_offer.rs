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
pub struct AcceptOffer<'info> {
    #[account(mut)]
    pub offer_acceptor: Signer<'info>,

    #[account(mut)]
    pub offer_creator: SystemAccount<'info>,

    pub provided_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub requested_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = requested_token_mint,
        associated_token::authority = offer_acceptor,
        associated_token::token_program = token_program,
    )]
    pub offer_acceptor_provided_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    // this is the token account (of offer_acceptor) for tokens that is being requested by the offer creator
    #[account(
        init_if_needed,
        payer = offer_acceptor,
        associated_token::mint = provided_token_mint,
        associated_token::authority = offer_acceptor,
        associated_token::token_program = token_program,
    )]
    pub offer_acceptor_requested_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = offer_acceptor,
        associated_token::mint = requested_token_mint,
        associated_token::authority = offer_creator,
        associated_token::token_program = token_program,
    )]
    pub offer_creator_requested_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        close = offer_creator,
        has_one = offer_creator,
        has_one = provided_token_mint,
        has_one = requested_token_mint,
        seeds = [b"offer", offer_creator.key().as_ref(), offer_account.id.to_le_bytes().as_ref()],
        bump = offer_account.bump,
    )]
    pub offer_account: Box<Account<'info, Offer>>,

    #[account(
        mut,
        associated_token::mint = provided_token_mint,
        associated_token::authority = offer_account,
        associated_token::token_program = token_program,
    )]
    pub vault_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,

    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn send_requested_tokens_to_offer_creator(ctx: &Context<AcceptOffer>) -> Result<()> {
    transfer_tokens(
        &ctx.accounts.offer_acceptor_provided_token_account,
        &ctx.accounts.offer_creator_requested_token_account,
        &ctx.accounts.offer_account.requested_amount,
        &ctx.accounts.requested_token_mint,
        &ctx.accounts.offer_acceptor,
        &ctx.accounts.token_program,
    )
}

pub fn send_tokens_from_vault_to_acceptor_and_close_vault(ctx: Context<AcceptOffer>) -> Result<()> {
    let seeds = &[
        b"offer",
        ctx.accounts.offer_creator.to_account_info().key.as_ref(),
        &ctx.accounts.offer_account.id.to_le_bytes()[..],
        &[ctx.accounts.offer_account.bump],
    ];

    let signer_seeds = [&seeds[..]];

    let accounts = TransferChecked {
        from: ctx.accounts.vault_account.to_account_info(),
        to: ctx
            .accounts
            .offer_acceptor_requested_token_account
            .to_account_info(),
        mint: ctx.accounts.provided_token_mint.to_account_info(),
        authority: ctx.accounts.offer_account.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );

    transfer_checked(
        cpi_context,
        ctx.accounts.vault_account.amount,
        ctx.accounts.provided_token_mint.decimals,
    )?;

    let accounts = CloseAccount {
        account: ctx.accounts.vault_account.to_account_info(),
        destination: ctx.accounts.offer_acceptor.to_account_info(),
        authority: ctx.accounts.offer_account.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );

    close_account(cpi_context)
}
