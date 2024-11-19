pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*; 

declare_id!("36Cj3imCYw1mKJV7FXwwohDSUWGHn2KL1MJXmSYGTqoz");

#[program]
pub mod swapfi {
    use super::*;

    pub fn create_swap_offer(ctx: Context<CreateOffer>, id: u64, provided_token_amount: u64, requested_token_amount: u64) -> Result<()>{
        instructions::lock_tokens_to_vault(&ctx, provided_token_amount)?;
        instructions::save_offer_on_chain(ctx, id, requested_token_amount)?;
        Ok(())
    }
}
