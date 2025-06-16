use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, metadata::{Metadata, create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3}, token::{mint_to, Mint, MintTo, Token, TokenAccount}};

declare_id!("8FYDpU57XSHSe8M7fygYeaMg7YbQ8j8CQ3PrSwb24Anm");

#[program]
pub mod nft {
    
    use super::*;

    pub fn mint_nft(ctx: Context<MintNFT>, nft_params: MintNFTParams) -> Result<()> {
        msg!("Minting NFT Token");
        mint_to(CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.destination.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ), 1)?;

        msg!("Create metadata account");
        let nft_data = DataV2 {
            name: nft_params.name,
            symbol: nft_params.symbol,
            uri: nft_params.uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };
        let metadata_ctx = CpiContext::new(
            ctx.accounts.metadata_program.to_account_info(), 
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                mint_authority: ctx.accounts.payer.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            }
        );

        create_metadata_accounts_v3(metadata_ctx, nft_data, false, true, None)?;
        msg!("Metadata account created successfully");

        msg!("Mint NFT Token successfully");
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct MintNFT<'info> {
    /// CHECK: New Metaplex Account being created
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    #[account(init, payer = payer, mint::decimals = 0, mint::authority = payer.key())]
    pub mint: Account<'info, Mint>,
    #[account(init_if_needed, payer = payer, associated_token::mint = mint, associated_token::authority = payer)]
    pub destination: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct MintNFTParams {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}