#![allow(dead_code)]
use anchor_client::solana_sdk::transaction::TransactionError;
use anchor_lang::solana_program::instruction::InstructionError;
use nft_minter::utils::token_metadata_program_id;
use solana_program_test::{BanksClientError, ProgramTest};

// Error = Error code
pub const ERR_CREATORS_LIST_TOO_LONG: u32 = 36;

pub fn nft_minter_test() -> ProgramTest {
    let mut program = ProgramTest::new("nft_minter", nft_minter::id(), None);
    program.add_program("mpl_token_metadata", token_metadata_program_id(), None);
    program
}

pub fn assert_error(error: BanksClientError, expected_error: u32) {
    match error {
        BanksClientError::TransactionError(TransactionError::InstructionError(
            0,
            InstructionError::Custom(e),
        )) => assert_eq!(e, expected_error),
        _ => assert!(false),
    }
}
