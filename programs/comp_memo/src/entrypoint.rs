//! Program entrypoint

#![cfg(feature = "no-entrypoint")]

use anchor_lang::prelude::Pubkey as AnchorPubkey;
use anchor_lang::prelude::Result as AnchorResult;
use light_sdk::address::NewAddressParams;
use light_sdk::compressed_account::CompressedAccount;

use {
    light_sdk::merkle_context::AddressMerkleContext,
    solana_account_info::AccountInfo as SolanaAccountInfo,
    solana_program_entrypoint::ProgramResult, solana_pubkey::Pubkey,
};

//solana_program_entrypoint::entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[SolanaAccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    crate::processor::process_instruction(program_id, accounts, instruction_data)
}

fn create_compressed_account_with_memo(
    discriminator: &[u8; 8],
    //_account: &T,
    accounts: &[SolanaAccountInfo],
    address_seed: &[u8; 32],
    program_id: &AnchorPubkey,
    address_merkle_context: &AddressMerkleContext,
    address_merkle_tree_root_index: u16,
    input: &[u8],
) -> AnchorResult<(CompressedAccount, NewAddressParams)> {
    crate::processor::create_compressed_account_with_memo(
        discriminator,
        accounts,
        address_seed,
        program_id,
        address_merkle_context,
        address_merkle_tree_root_index,
        input,
    )
}
