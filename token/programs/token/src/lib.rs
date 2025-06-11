use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata,
    },
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

declare_id!("EBEURCVTVLbppfEqG6Fax6prSgGsfH5C9AcWKaGm2hvc");


#[program]
pub mod token {

    use super::*;

    pub fn create_token(ctx: Context<CreateToken>, matedata: CreateTokenParams) -> Result<()> {
        let seeds = &["mint".as_bytes(), "tick".as_bytes(), &[ctx.bumps.mint]];
        let signer_seeds = &[&seeds[..]];

        let token_data = DataV2 {
            name: matedata.name,
            symbol: matedata.symbol,
            uri: matedata.uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        let metadata_ctx = CpiContext::new_with_signer(
            ctx.accounts.metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                mint_authority: ctx.accounts.mint.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            signer_seeds,
        );

        create_metadata_accounts_v3(metadata_ctx, token_data, false, true, None)?;

        msg!("Token mint account created successfully");
        Ok(())
    }

    pub fn mint_token(ctx: Context<MintToken>, amount: u64) -> Result<()> {
        let seeds = &["mint".as_bytes(), "tick".as_bytes(), &[ctx.bumps.mint]];
        let signer_seeds = &[&seeds[..]];

        mint_to(CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.destination.to_account_info(),
                authority: ctx.accounts.mint.to_account_info(),
            },
            signer_seeds,
        ), amount)?;

        msg!("Token minted successfully, transferred to {} {}", ctx.accounts.destination.key(), amount);

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(matedata: CreateTokenParams)]
pub struct CreateToken<'info> {
    /// CHECK: New Metaplex Account being created
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    // #[account(init, seeds = [b"mint"], bump, payer = payer, space = 8 + std::mem::size_of::<Mint>() + std::mem::size_of::<Account>() + 8 + 8, owner = mint)]
    #[account(init, seeds = [b"mint", b"tick"], bump, payer = payer, mint::decimals = matedata.decimals, mint::authority = mint)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct CreateTokenParams {
    pub name: String,
    pub uri: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Accounts)]
pub struct MintToken<'info> {
    #[account(mut, seeds = [b"mint", b"tick"], bump, mint::authority = mint)]
    pub mint: Account<'info, Mint>,
    #[account(init_if_needed, payer = payer, associated_token::mint = mint, associated_token::authority = payer)]
    pub destination: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>, // 需要create account时，需要指定payer来付rent，如果只是tx，则可以不指定，由signer来支付gas
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
