use anchor_client::{
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        bs58, commitment_config::CommitmentConfig, instruction::Instruction, native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer, system_program
    },
    Client, Cluster,
};
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::get_associated_token_address, metadata::mpl_token_metadata};
use std::{ops::Deref, sync::Arc};

declare_program!(token);
use token::{client::accounts, client::args};

use crate::token::types::CreateTokenParams;

#[tokio::main]
async fn main() -> anyhow::Result<()>{
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
    let program = provider.program(token::ID)?;
    
    println!("\nSend transaction with create_token and mint_token instructions");

    let create_token_ix = create_token_instruction(&program).await?;
    let mint_token_ix = mint_token_instruction(&program, 10000000000000).await?;

    let signature = program
        .request()
        // .instruction(create_token_ix)
        .instruction(mint_token_ix)
        .signer(payer.clone())
        .send()
        .await?;
    println!("Transaction confirmed: {}", signature);

    let mint = Pubkey::find_program_address(&["mint".as_bytes(), "tick".as_bytes()], &program.id()).0;
    let destination = get_associated_token_address(&program.payer(), &mint);

    // Get token account balance
    let token_account = connection.get_token_account(&destination)?.unwrap();
    println!("Token balance: {}", token_account.token_amount.amount);
    println!("Token account owner: {}", token_account.owner);
    println!("Token mint: {}", token_account.mint);

    Ok(())
}

async fn create_token_instruction<C: Deref<Target = impl Signer> + Clone>(program: &anchor_client::Program<C>) -> anyhow::Result<Instruction> {
    let mint = Pubkey::find_program_address(&["mint".as_bytes(), "tick".as_bytes()], &program.id()).0;
    let metadata_seeds = &["metadata".as_bytes(), mpl_token_metadata::ID.as_ref(), mint.as_ref()];
    let metadata = Pubkey::find_program_address(metadata_seeds, &mpl_token_metadata::ID).0;
    
    let create_token_ix = program
        .request()
        .accounts(accounts::CreateToken {
            metadata: metadata,
            mint: mint,
            payer: program.payer(),
            rent: anchor_lang::solana_program::sysvar::rent::ID,
            system_program: system_program::ID,
            token_program: anchor_spl::token::ID,
            metadata_program: mpl_token_metadata::ID,
        })
        .args(args::CreateToken {
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
        .accounts(accounts::MintToken {
            mint: mint,
            destination: destination,
            payer: program.payer(),
            system_program: system_program::ID,
            token_program: anchor_spl::token::ID,
            associated_token_program: anchor_spl::associated_token::ID,
        })
        .args(args::MintToken {
            amount: amount,
        })
        .instructions()?
        .remove(0);

    Ok(mint_token_ix)
}