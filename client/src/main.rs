use anchor_client::{
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, signature::Keypair,
        signer::Signer, system_program,
    },
    Client, Cluster,
};
use anchor_lang::prelude::*;
use std::sync::Arc;
 
declare_program!(counter);
use counter::{accounts::Counter, client::accounts, client::args};
 
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let connection = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com", // Local validator URL
        CommitmentConfig::confirmed(),
    );
 
    // Generate Keypairs and request airdrop
    let payer = Keypair::new();
    let count = Arc::new(Keypair::new());
    println!("Generated Keypairs:");
    println!("   Payer: {}", payer.pubkey());
    println!("   Count: {}", count.pubkey());
 
    println!("\nRequesting 1 SOL airdrop to payer");
    let airdrop_signature = connection.request_airdrop(&payer.pubkey(), LAMPORTS_PER_SOL)?;
 
    // Wait for airdrop confirmation
    while !connection.confirm_transaction(&airdrop_signature)? {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!("   Airdrop confirmed!");
 
    // Create program client
    let provider = Client::new_with_options(
        Cluster::Devnet,
        Arc::new(payer),
        CommitmentConfig::confirmed(),
    );
    let program = provider.program(counter::ID)?;
 
    // Build and send instructions
    println!("\nSend transaction with initialize and increment instructions");
    let initialize_ix = program
        .request()
        .accounts(accounts::Initialize {
            counter: count.pubkey(),
            signer: program.payer(),
            system_program: system_program::ID,
        })
        .args(args::Initialize)
        .instructions()?
        .remove(0);
 
    let increment_ix = program
        .request()
        .accounts(accounts::Increment {
            counter: count.pubkey(),
        })
        .args(args::Increment)
        .instructions()?
        .remove(0);
 
    let signature = program
        .request()
        .instruction(initialize_ix)
        .instruction(increment_ix)
        .signer(count.clone())
        .send()
        .await?;
    println!("   Transaction confirmed: {}", signature);
 
    println!("\nFetch counter account data");
    let counter_account: Counter = program.account::<Counter>(count.pubkey()).await?;
    println!("   Counter value: {}", counter_account.count);
    Ok(())
}