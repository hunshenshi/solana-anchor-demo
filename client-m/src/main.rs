use anchor_client::{
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        bs58, commitment_config::CommitmentConfig, instruction::Instruction, native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer, system_program
    },
    Client, Cluster,
};
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::get_associated_token_address, metadata::{self, mpl_token_metadata}};
use std::{ops::Deref, sync::Arc};
use clap::{Parser, Subcommand};

declare_program!(token);
declare_program!(nft);
use token::{client::accounts as token_accounts, client::args as token_args};
use nft::{client::accounts as nft_accounts, client::args as nft_args};

use crate::{nft::types::MintNFTParams, token::types::CreateTokenParams};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create and mint token
    Token {
        /// Amount to mint
        #[arg(short, long, default_value_t = 1000000000)]
        amount: u64,
    },
    /// Mint NFT
    Nft {
        /// NFT name
        #[arg(short, long)]
        name: String,
        /// NFT symbol
        #[arg(short, long)]
        symbol: String,
        /// NFT URI
        #[arg(short, long)]
        uri: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    let connection = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com",
        CommitmentConfig::confirmed(),
    );
    let payer = Arc::new(Keypair::new());
    println!("payer: {}", payer.pubkey());
    println!("payer private key: {:?}", payer.to_bytes());
    println!("payer base58: {}", bs58::encode(payer.to_bytes()).into_string());

    println!("\nRequesting 5 SOL airdrop to payer");
    let airdrop_signature = connection.request_airdrop(&payer.pubkey(), LAMPORTS_PER_SOL*5)?;
    while !connection.confirm_transaction(&airdrop_signature)? {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!("Airdrop confirmed!");

    let provider = Client::new_with_options(
        Cluster::Devnet,
        payer.clone(),
        CommitmentConfig::confirmed(),
    );

    match cli.command {
        Commands::Token { amount } => {
            let program = provider.program(token::ID)?;
            println!("\nExecuting token program...");
            
            let create_token_ix = create_token_instruction(&program).await?;
            let mint_token_ix = mint_token_instruction(&program, amount).await?;

            let signature = program
                .request()
                .instruction(create_token_ix)
                .instruction(mint_token_ix)
                .signer(payer.clone())
                .send()
                .await?;
            println!("Token transaction confirmed: {}", signature);

            let mint = Pubkey::find_program_address(&["mint".as_bytes(), "tick".as_bytes()], &program.id()).0;
            let destination = get_associated_token_address(&program.payer(), &mint);

            let token_account = connection.get_token_account(&destination)?.unwrap();
            println!("Token balance: {}", token_account.token_amount.amount);
        }
        Commands::Nft { name, symbol, uri } => {
            let program = provider.program(nft::ID)?;
            println!("\nExecuting NFT program...");
            
            // Create a new mint account for NFT
            let mint_account = Arc::new(Keypair::new());
            println!("mint_account: {}", mint_account.pubkey());
            println!("mint_account private key: {:?}", mint_account.to_bytes());
            println!("mint_account base58: {}", bs58::encode(mint_account.to_bytes()).into_string());

            let mint_nft_ix = mint_nft_instruction(&program, mint_account.pubkey(), name, symbol, uri).await?;

            let signature = program
                .request()
                .instruction(mint_nft_ix)
                .signer(payer.clone())
                .signer(mint_account.clone())
                .send()
                .await?;
            println!("NFT transaction confirmed: {}", signature);
        }
    }

    Ok(())
}

async fn mint_nft_instruction<C: Deref<Target = impl Signer> + Clone>(program: &anchor_client::Program<C>, mint_account: Pubkey, name: String, symbol: String, uri: String) -> anyhow::Result<Instruction> {
    let metadata_seeds = &["metadata".as_bytes(), mpl_token_metadata::ID.as_ref(), mint_account.as_ref()];
    let metadata = Pubkey::find_program_address(metadata_seeds, &mpl_token_metadata::ID).0;

    let destination = get_associated_token_address(&program.payer(),  &mint_account);

    let mint_nft_ix = program
        .request()
        .accounts(nft_accounts::MintNft{
            metadata: metadata,
            mint: mint_account,
            destination: destination,
            payer: program.payer(),
            rent: anchor_lang::solana_program::sysvar::rent::ID,
            system_program: system_program::ID,
            token_program: anchor_spl::token::ID,
            metadata_program: mpl_token_metadata::ID,
            associated_token_program: anchor_spl::associated_token::ID,
        })
        .args(nft_args::MintNft{
            nft_params: MintNFTParams {
                name: name,
                symbol: symbol,
                uri: uri,
            }

        })
        .instructions()?
        .remove(0);

    Ok(mint_nft_ix)
}

async fn create_token_instruction<C: Deref<Target = impl Signer> + Clone>(program: &anchor_client::Program<C>) -> anyhow::Result<Instruction> {
    let mint = Pubkey::find_program_address(&["mint".as_bytes(), "tick".as_bytes()], &program.id()).0;
    let metadata_seeds = &["metadata".as_bytes(), mpl_token_metadata::ID.as_ref(), mint.as_ref()];
    let metadata = Pubkey::find_program_address(metadata_seeds, &mpl_token_metadata::ID).0;
    
    let create_token_ix = program
        .request()
        .accounts(token_accounts::CreateToken {
            metadata: metadata,
            mint: mint,
            payer: program.payer(),
            rent: anchor_lang::solana_program::sysvar::rent::ID,
            system_program: system_program::ID,
            token_program: anchor_spl::token::ID,
            metadata_program: mpl_token_metadata::ID,
        })
        .args(token_args::CreateToken {
            matedata: CreateTokenParams {
                name: "My Token".to_string(),
                symbol: "MTK".to_string(),
                uri: "https://example.com/metadata".to_string(),
                decimals: 9,
            }
        })
        .instructions()?
        .remove(0);

    Ok(create_token_ix)
}

async fn mint_token_instruction<C: Deref<Target = impl Signer> + Clone>(program: &anchor_client::Program<C>, amount: u64) -> anyhow::Result<Instruction> {

    let mint = Pubkey::find_program_address(&["mint".as_bytes(), "tick".as_bytes()], &program.id()).0;
    let destination = get_associated_token_address(&program.payer(), &mint);

    let mint_token_ix = program
        .request()
        .accounts(token_accounts::MintToken {
            mint: mint,
            destination: destination,
            payer: program.payer(),
            system_program: system_program::ID,
            token_program: anchor_spl::token::ID,
            associated_token_program: anchor_spl::associated_token::ID,
        })
        .args(token_args::MintToken {
            amount: amount,
        })
        .instructions()?
        .remove(0);

    Ok(mint_token_ix)
}