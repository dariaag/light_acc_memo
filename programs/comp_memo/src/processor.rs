//! Program state processor

use anchor_lang::prelude::*;
use borsh::BorshSerialize;
use light_hasher::{Hasher, Poseidon};
use light_sdk::{
    address::{derive_address, NewAddressParams},
    compressed_account::{
        CompressedAccount, CompressedAccountData, OutputCompressedAccountWithPackedContext,
    },
    merkle_context::{AddressMerkleContext, MerkleOutputContext},
    program_merkle_context::unpack_address_merkle_context,
};

use {
    solana_account_info::AccountInfo as SolanaAccountInfo, solana_msg::msg,
    solana_program_entrypoint::ProgramResult,
    solana_program_error::ProgramError as SolanaProgramError,
    solana_pubkey::Pubkey as SolanaPubkey, std::str::from_utf8,
};

#[allow(missing_docs)]
#[error_code]
pub enum MemoProgramError {
    #[msg("Memo exceeds maximum allowed size")]
    MemoTooLarge,
    #[msg("At least one signer is required")]
    MissingSigner,
    #[msg("The first account must be a signer")]
    FirstAccountNotSigner,
    #[msg("Invalid system program account")]
    InvalidSystemProgram,
    #[msg("Derived address does not match PDA")]
    InvalidDerivedAddress,
    #[msg("Failed to hash input data")]
    HashingFailed,
    #[msg("Failed to serialize account data")]
    SerializationFailed,
    #[msg("Invalid input data")]
    InvalidInputData,
}

#[allow(missing_docs)]
pub fn create_compressed_account_with_memo(
    discriminator: &[u8; 8],
    //_account: &T,
    accounts: &[SolanaAccountInfo],
    address_seed: &[u8; 32],
    program_id: &Pubkey,
    address_merkle_context: &AddressMerkleContext,
    address_merkle_tree_root_index: u16,
    input: &[u8],
) -> Result<(CompressedAccount, NewAddressParams)> {
    let account_info_iter = &mut accounts.iter();
    let mut missing_required_signature = false;
    for account_info in account_info_iter {
        if let Some(address) = account_info.signer_key() {
            msg!("Signed by {:?}", address);
        } else {
            missing_required_signature = true;
        }
    }
    if missing_required_signature {
        return Err(MemoProgramError::MissingSigner.into());
    }

    let memo = from_utf8(input);

    if memo.is_err() {
        return Err(MemoProgramError::MemoTooLarge.into());
    }

    let data = input.try_to_vec()?;
    //has data input
    let data_hash = Poseidon::hash(input).unwrap();

    let compressed_account_data = CompressedAccountData {
        discriminator: *discriminator,
        data,
        data_hash,
    };

    let address = derive_address(address_seed, &address_merkle_context);

    let compressed_account = CompressedAccount {
        owner: *program_id,
        lamports: 0,
        address: Some(address),
        data: Some(compressed_account_data),
    };
    let address_merkle_tree_pubkey = address_merkle_context.address_merkle_tree_pubkey;

    let new_address_params = NewAddressParams {
        seed: *address_seed,
        address_merkle_tree_pubkey: address_merkle_tree_pubkey,
        address_queue_pubkey: address_merkle_context.address_queue_pubkey,
        address_merkle_tree_root_index,
    };

    Ok((compressed_account, new_address_params))
}
/// Instruction processor
pub fn process_instruction(
    _program_id: &SolanaPubkey,
    accounts: &[SolanaAccountInfo],
    input: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let mut missing_required_signature = false;
    for account_info in account_info_iter {
        if let Some(address) = account_info.signer_key() {
            msg!("Signed by {:?}", address);
        } else {
            missing_required_signature = true;
        }
    }
    if missing_required_signature {
        return Err(SolanaProgramError::MissingRequiredSignature);
    }

    let memo = from_utf8(input).map_err(|err| {
        msg!("Invalid UTF-8, from byte {}", err.valid_up_to());
        SolanaProgramError::InvalidInstructionData
    })?;
    msg!("Memo (len {}): {:?}", memo.len(), memo);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh::BorshSerialize;
    use light_hasher::{Hasher, Poseidon};
    use light_sdk::merkle_context::{AddressMerkleContext, MerkleOutputContext};
    use {
        super::*, solana_account_info::AccountInfo, solana_account_info::IntoAccountInfo,
        solana_program_error::ProgramError, solana_pubkey::Pubkey as SolanaPubkey,
        solana_sdk::account::Account as SolanaAccount,
    };
    #[test]
    fn test_create_compressed_account_with_memo() {
        //define dummy account
        let pubkey0: SolanaPubkey = SolanaPubkey::new_unique();
        let pubkey1 = SolanaPubkey::new_unique();
        let pubkey2 = SolanaPubkey::new_unique();
        let mut account0 = SolanaAccount::default();
        let mut account1 = SolanaAccount::default();
        let mut account2 = SolanaAccount::default();
        let account_info0 = AccountInfo::new(
            &pubkey0,
            true,
            false,
            &mut account0.lamports,
            &mut account0.data,
            &pubkey0,
            false,
            0,
        );
        let account_info1 = AccountInfo::new(
            &pubkey1,
            true,
            false,
            &mut account1.lamports,
            &mut account1.data,
            &pubkey1,
            false,
            0,
        );
        let account_info2 = AccountInfo::new(
            &pubkey2,
            true,
            false,
            &mut account2.lamports,
            &mut account2.data,
            &pubkey2,
            false,
            0,
        );

        let signed_account_infos = vec![account_info0, account_info1, account_info2];

        // Define test inputs
        let discriminator: [u8; 8] = [1; 8];
        let address_seed: [u8; 32] = [2; 32];
        let program_id = Pubkey::new_unique();

        let address_merkle_context = AddressMerkleContext {
            address_merkle_tree_pubkey: pubkey!("11111111111111111111111111111111"),
            address_queue_pubkey: pubkey!("22222222222222222222222222222222222222222222"),
        };

        let address_merkle_tree_root_index = 0;

        let input_memo = b"Hello:)";

        let (compressed_account_output, new_address_params) = create_compressed_account_with_memo(
            &discriminator,
            &signed_account_infos,
            // &mock_account,
            &address_seed,
            &program_id,
            &address_merkle_context,
            address_merkle_tree_root_index,
            input_memo,
        )
        .unwrap();

        assert_eq!(
            compressed_account_output
                .data
                .as_ref()
                .unwrap()
                .discriminator,
            discriminator
        );

        // veridy input data
        let data_in_account = &compressed_account_output.data.as_ref().unwrap().data;
        let input_data_serialized = input_memo.try_to_vec().unwrap();

        //assert_eq!(data_in_account, &input_data_serialized);
        println!("DATA: {:?}", data_in_account);
        println!("INPUT DATA: {:?}", input_data_serialized);

        let expected_data_hash = Poseidon::hash(input_memo).unwrap();
        let data_hash_in_account = &compressed_account_output.data.as_ref().unwrap().data_hash;
        assert_eq!(data_hash_in_account, &expected_data_hash);
        println!("DATA HASH: {:?}", data_hash_in_account);
        println!("EXP DATA HASH: {:?}", expected_data_hash);

        /*   let expected_address = derive_address(
            &address_seed,
            &unpack_address_merkle_context(address_merkle_context, remaining_accounts),
        );
        assert_eq!(
            compressed_account_output
                .compressed_account
                .address
                .unwrap(),
            expected_address
        ); */

        assert_eq!(compressed_account_output.owner, program_id);

        assert_eq!(new_address_params.seed, address_seed);
        assert_eq!(
            new_address_params.address_merkle_tree_pubkey,
            address_merkle_context.address_merkle_tree_pubkey
        );

        assert_eq!(
            new_address_params.address_merkle_tree_root_index,
            address_merkle_tree_root_index
        );
    }
}
