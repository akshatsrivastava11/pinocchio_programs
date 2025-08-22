use core::str;

use pinocchio::{account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::find_program_address, ProgramResult};
use pinocchio_system::instructions::Transfer;

pub struct Withdraw<'a>{
    pub accounts:WithdrawAccounts<'a>
}

impl<'a>TryFrom<(&'a [u8],&'a [AccountInfo])> for Withdraw<'a>{
    type Error = ProgramError;
    fn try_from((data,accounts): (&'a [u8],&'a [AccountInfo])) -> Result<Self, Self::Error> {
        let accounts=WithdrawAccounts::try_from(accounts)?;
        Ok(Self {
            accounts
        })
    }
}

impl<'a>Withdraw<'a>{
    pub const DISCRIMINATOR:&'a u8=&1;
    pub fn process(&mut self)->ProgramResult{
        let seeds=[
            Seed::from(b"vault"),
            Seed::from(self.accounts.owner.key().as_ref()),
            Seed::from(self.accounts.bump.as_ref())
        ];
        let signers=[Signer::from(&seeds)];
        Transfer{
            from:self.accounts.vault,
            lamports:self.accounts.vault.lamports(),
            to:self.accounts.owner,
        }.invoke_signed(&signers)
    }
}

pub struct WithdrawAccounts<'a>{
    pub vault:&'a AccountInfo,
    pub owner:&'a AccountInfo,
    pub bump:[u8;1]
}
impl<'a>TryFrom<&'a [AccountInfo]> for WithdrawAccounts<'a>{
    type Error = ProgramError;
    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let[vault,owner]=accounts else{
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        if !owner.is_signer(){
            return Err(ProgramError::MissingRequiredSignature);
        };
        if vault.is_owned_by(&pinocchio_system::ID){
            return Err(ProgramError::InvalidAccountOwner);
        };
        if vault.lamports().ne(&0){
           return Err(ProgramError::AccountAlreadyInitialized); 
        };
        let (vault_key,bump)=find_program_address(
            &[b"vault",owner.key().as_ref()]
            , &crate::ID);
        if vault.key().ne(&vault_key){
            return Err(ProgramError::InvalidAccountData);
        };
        Ok(Self {
            vault,
            owner,
            bump:[bump]
        })
            

    }
}
