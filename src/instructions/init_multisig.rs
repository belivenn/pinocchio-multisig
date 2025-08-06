use pinocchio::{
    account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::{self, Pubkey}, sysvars::{rent::{self, Rent}, Sysvar}, ProgramResult
};
use pinocchio_log::log;

use crate::state::Multisig;

pub fn process_init_multisig_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [creator, multisig, treasury, rent, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    // Multisig Config PDA
    let seed = [(b"multisig"), creator.key().as_slice()];
    let seeds = &seed[..];
    let (pda_multisig, multisig_bump) = pubkey::find_program_address(seeds, &crate::ID);
    assert_eq!(&pda_multisig, multisig.key());
   
    // Treasury PDA
    let treasury_seed = [(b"treasury"), multisig.key().as_slice()];
    let treasury_seeds = &treasury_seed[..];
    let (pda_treasury, treasury_bump) = pubkey::find_program_address(treasury_seeds, &crate::ID);
    assert_eq!(&pda_treasury, treasury.key());

    let rent = Rent::from_account_info(rent)?;


    if multisig.owner() != &crate::ID {
        log!("Creating Multisig Account");

        let bump = [multisig_bump.to_le()];
        let cpi_seed = [Seed::from(b"multisig"), Seed::from(creator.key().as_ref()), Seed::from(&bump)];
        let cpi_seeds = Signer::from(&cpi_seed[..]);

        // Create Multisig Account
        pinocchio_system::instructions::CreateAccount {
            from: creator,
            to: multisig,
            lamports: rent.minimum_balance(Multisig::LEN),
            space: Multisig::LEN as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&[cpi_seeds])?;

        let multisig_account = Multisig::from_account_info(&multisig)?;
        multisig_account.new(
            creator.key(),
            treasury.key(),
            treasury_bump,
            multisig_bump,
            data,
        );

    } else {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if treasury.owner() != &crate::ID {
        log!("Creating Treasury SystemAccount");

        let bump = [multisig_bump.to_le()];
        let cpi_seed = [Seed::from(b"multisig"), Seed::from(creator.key()), Seed::from(&bump)];
        let cpi_seeds = Signer::from(&cpi_seed[..]);
  
        // Create Treasury Account
        pinocchio_system::instructions::CreateAccount {
            from: creator,
            to: treasury,
            lamports: rent.minimum_balance(0),
            space: 0,
            owner: &pinocchio_system::ID,
        }
        .invoke_signed(&[cpi_seeds])?;
    } else {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    Ok(())
}