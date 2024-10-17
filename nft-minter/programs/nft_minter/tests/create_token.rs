use anchor_client::solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use anchor_lang::{
    solana_program::{instruction::Instruction, system_program, sysvar},
    InstructionData, ToAccountMetas,
};
use anchor_spl::token::spl_token;
use nft_minter::{
    pda::find_metadata_account,
    utils::{token_metadata_program_id, Creator},
};
use solana_program_test::tokio;

mod utils;
use utils::*;

#[tokio::test]
async fn create_token_success() {
    let mut context = nft_minter_test().start_with_context().await;

    let payer = context.payer;

    let mint = Keypair::new();

    let (metadata, _) = find_metadata_account(&mint.pubkey());

    let metadata_creators = vec![Creator {
        address: payer.pubkey(),
        share: 100,
        verified: false,
    }];

    let data = nft_minter::instruction::CreateToken {
        name: String::from("Solana Course NFT"),
        symbol: String::from("SOLC"),
        uri: String::from(
            "https://raw.githubusercontent.com/arsenijkovalov/nft-assets/main/assets/nft.json",
        ),
        creators: Some(metadata_creators),
        seller_fee_basis_points: 10,
        is_mutable: false,
    };

    let accounts = nft_minter::accounts::CreateToken {
        payer: payer.pubkey(),
        mint_account: mint.pubkey(),
        mint_authority: payer.pubkey(),
        update_authority: payer.pubkey(),
        metadata_account: metadata,
        token_metadata_program: token_metadata_program_id(),
        token_program: spl_token::id(),
        system_program: system_program::id(),
        rent: sysvar::rent::id(),
    };

    let ix = Instruction {
        program_id: nft_minter::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer, &mint],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();
}

#[tokio::test]
async fn failure_create_token_creators_list_too_long() {
    let mut context = nft_minter_test().start_with_context().await;

    let payer = context.payer;

    let mint = Keypair::new();

    let (metadata, _) = find_metadata_account(&mint.pubkey());

    // Provided 6 creators out of 5
    let metadata_creators = vec![
        Creator {
            address: payer.pubkey(),
            share: 10,
            verified: false,
        },
        Creator {
            address: payer.pubkey(),
            share: 10,
            verified: false,
        },
        Creator {
            address: payer.pubkey(),
            share: 10,
            verified: false,
        },
        Creator {
            address: payer.pubkey(),
            share: 10,
            verified: false,
        },
        Creator {
            address: payer.pubkey(),
            share: 10,
            verified: false,
        },
        Creator {
            address: payer.pubkey(),
            share: 10,
            verified: false,
        },
    ];

    let data = nft_minter::instruction::CreateToken {
        name: String::from("Solana Course NFT"),
        symbol: String::from("SOLC"),
        uri: String::from(
            "https://raw.githubusercontent.com/arsenijkovalov/nft-assets/main/assets/nft.json",
        ),
        creators: Some(metadata_creators),
        seller_fee_basis_points: 10,
        is_mutable: false,
    };

    let accounts = nft_minter::accounts::CreateToken {
        payer: payer.pubkey(),
        mint_account: mint.pubkey(),
        mint_authority: payer.pubkey(),
        update_authority: payer.pubkey(),
        metadata_account: metadata,
        token_metadata_program: token_metadata_program_id(),
        token_program: spl_token::id(),
        system_program: system_program::id(),
        rent: sysvar::rent::id(),
    };

    let ix = Instruction {
        program_id: nft_minter::id(),
        data: data.data(),
        accounts: accounts.to_account_metas(None),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer, &mint],
        context.last_blockhash,
    );

    let tx_error = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();

    assert_error(tx_error, ERR_CREATORS_LIST_TOO_LONG);
}
