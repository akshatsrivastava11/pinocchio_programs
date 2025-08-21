use pinocchio::{account_info::AccountInfo, instruction::Account, program_error::ProgramError, pubkey::find_program_address, ProgramResult};
use pinocchio_system::instructions::Transfer;

pub struct Deposit<'a> {
    pub accounts: DepositAccounts<'a>,
    pub instruction_data: DepositInstructions,
}
impl<'a>TryFrom<(&'a [u8],&'a [AccountInfo])> for Deposit<'a>{
    type Error = ProgramError;
    fn try_from((data,accounts): (&'a [u8],&'a [AccountInfo])) -> Result<Self, Self::Error> {
        let accounts=DepositAccounts::try_from(accounts)?;
        let data=DepositInstructions::try_from(data)?;
     Ok(Self {
            accounts,
            instruction_data:data
        })    }
} 

impl<'a> Deposit<'a>{
    pub const DISCRIMINATOR:&'a u8=&0;
    pub fn process0(&mut self)->ProgramResult{
        Transfer{
            from:self.accounts.owner,
            to:self.accounts.vault,
            lamports:self.instruction_data.amount
        }.invoke()
    }
}

pub struct DepositAccounts<'a>{
    pub owner:&'a AccountInfo,
    pub vault:&'a AccountInfo
}


impl<'a>TryFrom<&'a [AccountInfo]> for DepositAccounts<'a>  {
    type Error = ProgramError;
    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner,vault]=accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        if !owner.is_signer(){
            return Err(ProgramError::MissingRequiredSignature);
        };
        if !vault.is_owned_by(&pinocchio_system::ID){
            return Err(ProgramError::InvalidAccountOwner);
        };
        if vault.lamports().ne(&0){
            return Err(ProgramError::InvalidAccountData);
        };

        let (vault_key,_)=find_program_address(
            &[b"vault",owner.key().as_ref()]
            , &crate::ID);
        if vault.key().ne(&vault_key){
            return Err(ProgramError::InvalidAccountData);
        };
        Ok(Self { owner: owner, vault: vault })
    }

}   

pub struct DepositInstructions{
    pub amount:u64
}
impl<'a>TryFrom<&'a [u8]> for DepositInstructions{
    type Error = ProgramError;
    fn try_from(data:&'a [u8]) -> Result<Self, Self::Error> {
        if (data.len()!=size_of::<u8>()){
            return Err(ProgramError::InvalidAccountData);
        };
        let amounts=u64::from_le_bytes(data.try_into().unwrap());
        if amounts.eq(&0){
            return Err(ProgramError::InvalidAccountData);
        };
        Ok(Self { amount: amounts })
    }
}