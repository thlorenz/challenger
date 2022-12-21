use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};

use crate::{error::ChallengeError, Solution};

pub fn assert_keys_equal(
    provided_key: &Pubkey,
    expected_key: &Pubkey,
    msg: &str,
) -> ProgramResult {
    if provided_key.ne(expected_key) {
        msg!("Err: {}", msg);
        msg!("Err: provided {} expected {}", provided_key, expected_key);
        Err(ChallengeError::ProvidedAtaIsIncorrect.into())
    } else {
        Ok(())
    }
}

pub fn assert_max_supported_solutions(solutions: &[Solution]) -> ProgramResult {
    let len = solutions.len();
    if len > u8::MAX as usize {
        msg!(
            "Err: solutions len ({}) is greater than maximum supported solutions ({})",
            len,
            u8::MAX
        );
        Err(ChallengeError::ExceedingMaxSupportedSolutions.into())
    } else {
        Ok(())
    }
}

pub fn assert_can_add_solutions(
    solutions: &[Solution],
    extra_solutions: &[Solution],
) -> ProgramResult {
    let solutions_len = solutions.len();
    let extra_solutions_len = extra_solutions.len();

    let final_len = solutions_len.saturating_add(extra_solutions_len);
    if final_len > u8::MAX as usize {
        msg!(
            "Err: adding {} solutions would result in {} total solutions which exceeds max supported {}",
            extra_solutions_len,
            final_len,
            u8::MAX
        );
        Err(ChallengeError::ExceedingMaxSupportedSolutions.into())
    } else {
        Ok(())
    }
}

pub fn assert_adding_non_empty(extra_solutions: &[Solution]) -> ProgramResult {
    if extra_solutions.is_empty() {
        msg!("Err: no solutions to add cannot be empty");
        Err(ChallengeError::NoSolutionsToAddProvided.into())
    } else {
        Ok(())
    }
}

pub fn assert_account_is_funded_and_has_data(
    account: &AccountInfo,
) -> ProgramResult {
    if account.try_data_len()?.eq(&0) {
        msg!(
            "Err: account data is empty, did you intialize it via create_challenge()?",
        );
        Err(ChallengeError::AccountHasNoData.into())
    } else if account.try_lamports()? < 1 {
        msg!("Err: account is not funded, did you intialize it via create_challenge()?");
        Err(ChallengeError::AccountNotFunded.into())
    } else {
        Ok(())
    }
}

pub fn assert_account_has_no_data(account: &AccountInfo) -> ProgramResult {
    if account.try_data_len()?.ne(&0) {
        msg!(
            "Err: account data is not empty, did you already intialize it via create_challenge()?",
        );
        Err(ChallengeError::AccountAlreadyHasData.into())
    } else {
        Ok(())
    }
}

pub fn assert_is_signer(
    account: &AccountInfo,
    account_label: &str,
) -> ProgramResult {
    if !account.is_signer {
        msg!(
            "Err: account '{}' ({}) should be signer",
            account_label,
            account.key
        );
        Err(ChallengeError::AccountShouldBeSigner.into())
    } else {
        Ok(())
    }
}
