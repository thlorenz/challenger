#![cfg(feature = "test-sbf")]

use assert_matches::assert_matches;
use borsh::BorshSerialize;
use challenge::{
    challenge_id,
    ixs::{self, ChallengeInstruction},
    state::Challenge,
    utils::hash_solutions,
};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};
use solana_program_test::*;

use solana_sdk::{
    account::{AccountSharedData, ReadableAccount},
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};
use utils::add_challenge_account;

use crate::utils::{get_deserialized, hash_solution, program_test};

mod utils;

fn add_challenge_with_solutions(
    context: &mut ProgramTestContext,
    solutions: Vec<&str>,
    authority: Option<Pubkey>,
) -> AccountSharedData {
    let solutions = hash_solutions(&solutions);
    add_challenge_account(
        context,
        Challenge {
            authority: authority.unwrap_or_else(|| context.payer.pubkey()),
            admit_cost: 200,
            tries_per_admit: 1,
            redeem: Pubkey::new_unique(),
            solving: 0,
            solutions,
        },
    )
}

#[tokio::test]
async fn add_solutions_creator_pays_to_empty_solutions() {
    let mut context = program_test().start_with_context().await;
    let creator = context.payer.pubkey();
    let added_acc = add_challenge_with_solutions(&mut context, vec![], None);

    let (challenge_pda, _) =
        Challenge::shank_pda(&challenge_id(), &context.payer.pubkey());

    let solutions = vec!["hello", "world"];
    let ix = ixs::add_solutions(context.payer.pubkey(), creator, solutions)
        .expect("failed to create instruction");

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("Failed add solutions");

    let (acc, value) =
        get_deserialized::<Challenge>(&mut context, &challenge_pda).await;

    assert_matches!(
        value,
        Challenge {
            authority,
            admit_cost: 200,
            tries_per_admit: 1,
            redeem: _,
            solving: 0,
            solutions,
        } => {
            assert_eq!(&authority, &creator);
            assert_eq!(solutions.len(), 2);
            assert_eq!(solutions[0], hash_solution("hello"));
            assert_eq!(solutions[1], hash_solution("world"));
            assert_eq!(acc.data.len(), Challenge::needed_size(&solutions));
            assert!(acc.lamports > added_acc.lamports(), "does transfer extra lamports");
        }
    );
}

#[tokio::test]
async fn add_solutions_creator_not_payer_to_empty_solutions() {
    let mut context = program_test().start_with_context().await;
    let creator = Keypair::new();

    let added_acc = add_challenge_with_solutions(
        &mut context,
        vec![],
        Some(creator.pubkey()),
    );

    let solutions = vec!["hello", "world"];

    let ix =
        ixs::add_solutions(context.payer.pubkey(), creator.pubkey(), solutions)
            .expect("failed to create instruction");

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &creator],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("Failed add solutions");

    let (challenge_pda, _) =
        Challenge::shank_pda(&challenge_id(), &creator.pubkey());

    let (acc, value) =
        get_deserialized::<Challenge>(&mut context, &challenge_pda).await;

    assert_matches!(
        value,
        Challenge {
            authority,
            admit_cost: 200,
            tries_per_admit: 1,
            redeem: _,
            solving: 0,
            solutions,
        } => {
            assert_eq!(&authority, &creator.pubkey());
            assert_eq!(solutions.len(), 2);
            assert_eq!(solutions[0], hash_solution("hello"));
            assert_eq!(solutions[1], hash_solution("world"));
            assert_eq!(acc.data.len(), Challenge::needed_size(&solutions));
            assert!(acc.lamports > added_acc.lamports(), "does transfer extra lamports");
        }
    );
}

#[tokio::test]
async fn add_solutions_creator_pays_to_two_solutions() {
    let mut context = program_test().start_with_context().await;
    let creator = context.payer.pubkey();
    let added_acc =
        add_challenge_with_solutions(&mut context, vec!["hola", "mundo"], None);

    let (challenge_pda, _) =
        Challenge::shank_pda(&challenge_id(), &context.payer.pubkey());

    let solutions = vec!["hello", "world"];
    let ix = ixs::add_solutions(context.payer.pubkey(), creator, solutions)
        .expect("failed to create instruction");

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("Failed add solutions");

    let (acc, value) =
        get_deserialized::<Challenge>(&mut context, &challenge_pda).await;

    assert_matches!(
        value,
        Challenge {
            authority,
            admit_cost: 200,
            tries_per_admit: 1,
            redeem: _,
            solving: 0,
            solutions,
        } => {
            assert_eq!(&authority, &creator);
            assert_eq!(solutions.len(), 4);
            assert_eq!(solutions[0], hash_solution("hola"));
            assert_eq!(solutions[1], hash_solution("mundo"));
            assert_eq!(solutions[2], hash_solution("hello"));
            assert_eq!(solutions[3], hash_solution("world"));
            assert_eq!(acc.data.len(), Challenge::needed_size(&solutions));
            assert!(acc.lamports > added_acc.lamports(), "does transfer extra lamports");
        }
    );
}

// -----------------
// Error Cases
// -----------------
#[tokio::test]
#[should_panic]
async fn add_solutions_with_empty_solutions() {
    let mut context = program_test().start_with_context().await;
    let creator = context.payer.pubkey();
    add_challenge_with_solutions(&mut context, vec!["hola", "mundo"], None);

    let solutions = vec![];
    let ix = ixs::add_solutions(context.payer.pubkey(), creator, solutions)
        .expect("failed to create instruction");

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("Failed add solutions");
}

#[tokio::test]
#[should_panic]
async fn add_solutions_without_creating_account() {
    let mut context = program_test().start_with_context().await;
    let creator = context.payer.pubkey();

    let solutions = vec!["hola", "mundo"];
    let ix = ixs::add_solutions(context.payer.pubkey(), creator, solutions)
        .expect("failed to create instruction");

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("Failed add solutions");
}

#[tokio::test]
#[should_panic]
async fn add_solutions_with_invalid_creator() {
    let mut context = program_test().start_with_context().await;
    let creator = context.payer.pubkey();
    add_challenge_with_solutions(&mut context, vec![], Some(creator));

    let solutions = vec!["hola", "mundo"];
    let other_creator = Keypair::new();

    let ix = {
        let (challenge_pda, _) =
            Challenge::shank_pda(&challenge_id(), &creator);
        let solutions = hash_solutions(&solutions);
        Instruction {
            program_id: challenge_id(),
            accounts: vec![
                AccountMeta::new(context.payer.pubkey(), true),
                AccountMeta::new_readonly(other_creator.pubkey(), true),
                AccountMeta::new(challenge_pda, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: ChallengeInstruction::AddSolutions { solutions }
                .try_to_vec()
                .expect("failed to create custom instruction"),
        }
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &other_creator],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("Failed add solutions");
}

#[tokio::test]
#[should_panic]
async fn add_solutions_with_creator_not_signer() {
    let mut context = program_test().start_with_context().await;
    let creator_pair = Keypair::new();
    let creator = creator_pair.pubkey();
    add_challenge_with_solutions(&mut context, vec![], Some(creator));

    let solutions = vec!["hola", "mundo"];

    let ix = {
        let (challenge_pda, _) =
            Challenge::shank_pda(&challenge_id(), &creator);
        let solutions = hash_solutions(&solutions);
        Instruction {
            program_id: challenge_id(),
            accounts: vec![
                AccountMeta::new(context.payer.pubkey(), true),
                AccountMeta::new_readonly(creator, false),
                AccountMeta::new(challenge_pda, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: ChallengeInstruction::AddSolutions { solutions }
                .try_to_vec()
                .expect("failed to create custom instruction"),
        }
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("Failed add solutions");
}
