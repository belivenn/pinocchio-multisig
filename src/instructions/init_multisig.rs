use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_log::log;

use crate::state::Multisig;

pub fn process_init_multisig_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [creator, multisig, treasury, _remaining @ ..] = accounts else {
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

    if multisig.owner() != &crate::ID {
        log!("Creating Multisig Account");

        // Create Multisig Account
        pinocchio_system::instructions::CreateAccount {
            from: creator,
            to: multisig,
            lamports: Rent::get()?.minimum_balance(Multisig::LEN),
            space: Multisig::LEN as u64,
            owner: &crate::ID,
        }
        .invoke()?;

        // Populate Multisig Account
        let multisig_account = Multisig::from_account_info(&multisig)?;
        multisig_account.admin = None; 
        multisig_account.admin_spending_limit = None; 
        multisig_account.creator = *creator.key();
        multisig_account.treasury = *treasury.key();
        multisig_account.treasury_bump = treasury_bump;
        multisig_account.bump = multisig_bump;
        multisig_account.min_threshold = unsafe { *(data.as_ptr() as *const u8) };
        multisig_account.max_expiry = unsafe { *(data.as_ptr().add(1) as *const u64) };
        multisig_account.transaction_index = 0;
        multisig_account.stale_transaction_index = 0;
        multisig_account.num_members = unsafe { *(data.as_ptr().add(9) as *const u8) };
        multisig_account.members = [Pubkey::default(); 10]; 

        match multisig_account.num_members {
            0..=10 => {
                for i in 0..multisig_account.num_members as usize {
                    let member_key = unsafe { *(data.as_ptr().add(10 + i * 32) as *const [u8; 32]) };
                    multisig_account.members[i] = member_key;
                }
            }
            _ => return Err(ProgramError::InvalidAccountData),
        }

    } else {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if treasury.owner() != &crate::ID {
        log!("Creating Treasury SystemAccount");

        // Create Treasury Account
        pinocchio_system::instructions::CreateAccount {
            from: creator,
            to: treasury,
            lamports: Rent::get()?.minimum_balance(0),
            space: 0,
            owner: &pinocchio_system::ID,
        }
        .invoke()?;
    } else {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    Ok(())
}
