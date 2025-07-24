use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_log::log;

use crate::state::{Multisig, MultisigConfig, Member, Permission, utils::{load_ix_data, DataLen}};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, shank::ShankType)]
pub struct UpdateMembersIxData {
    pub update_type: u8,
    pub member_key: Pubkey,
    pub permission: u8,
    pub is_active: u8,
    pub payer_index: u8,
    pub member_index: u8,
}

impl DataLen for UpdateMembersIxData {
    const LEN: usize = core::mem::size_of::<UpdateMembersIxData>();
}

pub fn process_update_members_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [payer, creator, multisig, multisig_config, treasury, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    // Multisig PDA
    let seed = [(b"multisig"), creator.key().as_slice()];
    let seeds = &seed[..];
    let (pda_multisig, multisig_bump) = pubkey::find_program_address(seeds, &crate::ID);
    assert_eq!(&pda_multisig, multisig.key());

    let update_members_ix_data = load_ix_data::<UpdateMembersIxData>(data)?;

    if payer.key() != &pda_multisig.members[update_members_ix_data.payer_index as usize].key {
        return Err(ProgramError::InvalidAccountData);
    }

    if update_members_ix_data.member_key != pda_multisig.members[update_members_ix_data.member_index as usize].key {
        return Err(ProgramError::InvalidAccountData);
    }

    if &pda_multisig.members[update_members_ix_data.payer_index as usize].permissions != Permission::Admin {
        return Err(ProgramError::InvalidAccountData);
    }


    update_member(update_members_ix_data.update_type,
        update_members_ix_data.member_key,
        update_members_ix_data.permission,
        update_members_ix_data.is_active,
        update_members_ix_data.member_index,
        &mut pda_multisig.members,
        &mut pda_multisig.num_members,
    )?;


    Ok(())
}
