use anchor_client::solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use anchor_lang::{
    solana_program::{instruction::Instruction, program_pack::Pack, system_program, sysvar},
    InstructionData, ToAccountMetas,
};
use anchor_spl::{
    associated_token::{self, get_associated_token_address},
    token::spl_token,
};
use mpl_token_metadata::pda::find_master_edition_account;
use nft_minter::{
    pda::find_metadata_account,
    utils::{token_metadata_program_id, Creator},
};
use solana_program_test::tokio;

mod utils;
use utils::*;

#[tokio::test]
async fn mint_token_success() {
    let mut context = nft_minter_test().start_with_context().await;

    let payer = context.payer;

    // CreateToken

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

    // MintToken

    let ata = get_associated_token_address(&payer.pubkey(), &mint.pubkey());

    let (master_edition, _) = find_master_edition_account(&mint.pubkey());

    let data = nft_minter::instruction::MintToken {
        max_supply: Some(1),
    };

    let accounts = nft_minter::accounts::MintToken {
        payer: payer.pubkey(),
        mint_account: mint.pubkey(),
        mint_authority: payer.pubkey(),
        update_authority: payer.pubkey(),
        associated_token_account: ata,
        metadata_account: metadata,
        edition_account: master_edition,
        token_metadata_program: token_metadata_program_id(),
        token_program: spl_token::id(),
        associated_token_program: associated_token::ID,
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

    let token_account = context
        .banks_client
        .get_account(ata)
        .await
        .expect("Account not found")
        .expect("Account is empty");

    let token_account_data =
        crate::spl_token::state::Account::unpack_from_slice(token_account.data.as_slice()).unwrap();

    assert_eq!(token_account_data.amount, 1);
    assert_eq!(token_account_data.owner, payer.pubkey());
}
