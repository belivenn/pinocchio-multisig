use pinocchio::{pubkey::Pubkey, ProgramResult};
use pinocchio::program_error::ProgramError;
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Permission {
    Readonly = 0,
    Vote = 1,
    VoteAndExecute = 2,
    Admin = 3,
}

impl Permission {
    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => Permission::Readonly,
            2 => Permission::Vote,
            3 => Permission::VoteAndExecute,
            4 => Permission::Admin,
            _ => Permission::Readonly,
        }
    }
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Member {
    pub key: Pubkey,
    pub permissions: Permission,
    pub is_active: u8,
}

impl Default for Member {
    fn default() -> Self {
        Member {
            key: Pubkey::default(),
            permissions: Permission::Readonly,
            is_active: 0,
        }
    }
}

pub fn update_member(update_type: u8, member_key: Pubkey, permission: u8, is_active: u8, member_index: u8, members: &mut [Member], num_members: &mut u8) -> ProgramResult {

    match update_type {
        0 => { // add member
            if *num_members >= 10 {
                return Err(ProgramError::InvalidAccountData);
            }
            if members[member_index as usize].key != Pubkey::default() {
                return Err(ProgramError::InvalidAccountData);
            }

            let new_member_index = *num_members as usize;
            let new_member = Member {
                key: member_key,
                permissions: Permission::from_u8(permission),
                is_active: is_active,
            };
            *num_members += 1;
            members[new_member_index] = new_member;


            Ok(())
        }
        1 => { // remove member
            let member_index = member_index as usize;
            let last_member_index  = *num_members as usize;

            if *num_members == 0 {
                return Err(ProgramError::InvalidAccountData);
            }
            if member_index > last_member_index  {
                return Err(ProgramError::InvalidAccountData);
            }

            for i in member_index..last_member_index {
                members[i] = members[i + 1];
            }
            members[last_member_index] = Member::default();
            *num_members -= 1;

            Ok(())
        }

        2 => { // update permission
            members[member_index as usize].permissions = Permission::from_u8(permission);
            members[member_index as usize].is_active = is_active;

            Ok(())
        }

        _ => Err(ProgramError::InvalidInstructionData),
    }
}
