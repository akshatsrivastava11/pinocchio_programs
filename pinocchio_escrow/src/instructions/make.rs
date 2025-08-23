use core::{f32::consts::E, slice};

use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::find_program_address, sysvars::{rent::Rent, Sysvar}, ProgramResult};
use pinocchio_system::instructions::Transfer;

use crate::states::{escrow, Escrow};

pub struct Make<'a>{
    pub accounts:MakeAccount<'a>,
    pub instruction_data:MakeInstruction<'a>
}
pub struct MakeAccount<'a>{
       pub mint_a: &'a AccountInfo,
    pub mint_b: &'a AccountInfo,
    pub maker: &'a AccountInfo,
    pub escrow:&'a AccountInfo,
    pub vault:&'a AccountInfo,
    pub escrow_bump: &'a u8,
    pub seed: &'a u8,
}
pub struct MakeInstruction<'a>{
    pub amount:&'a u64,
    pub seed:&'a u8
}

impl <'a>TryFrom<(&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a u64,&'a u8,&'a u8)> for Make<'a>{
    type Error = ProgramError;
    fn try_from(data:(&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a u64,&'a u8,&'a u8)) -> Result<Self, Self::Error> {
              let (mint_a,mint_b,maker,escrow,vault,amount,escrow_bump,seed) = data else{
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        if !maker.is_signer(){
            return Err(ProgramError::MissingRequiredSignature);
        };
        if !maker.is_owned_by(&pinocchio_system::ID){
            return  Err(ProgramError::InvalidAccountOwner);
        };
        if !vault.is_owned_by(&pinocchio_system::ID){
            return Err(ProgramError::InvalidAccountOwner);
        };
        if !vault.lamports().ne(&0){
            return Err(ProgramError::InvalidAccountData);
        };
        let (vault_key,_)=find_program_address(
            &[b"vault",maker.key().as_ref(),seed.to_le_bytes().as_ref()]
            , &pinocchio_system::ID);
        if !vault.key().ne(&vault_key){
            return Err(ProgramError::InvalidSeeds)
        };
        let (escrow_key,_)=find_program_address(
            &[b"escrow",maker.key().as_ref(),seed.to_le_bytes().as_ref()], 
        &pinocchio_system::ID);
        
        if escrow.key().ne(&escrow_key){
         return Err(ProgramError::InvalidSeeds);   
        }
        Ok(Self { accounts: MakeAccount{seed:seed,escrow_bump:escrow_bump,escrow:escrow,vault:vault,maker:maker,mint_a:mint_a,mint_b:mint_b}, instruction_data: MakeInstruction { amount: &amount, seed: seed  }})
    }
}  
impl<'a> Make<'a>{
    pub fn process(&mut self)->ProgramResult{
        self.initialize();
        Transfer{
            from:self.accounts.maker,
            to:self.accounts.vault,
            lamports:*self.instruction_data.amount
        }.invoke()
    }
    fn initialize(&mut self)->ProgramResult{
        let (escrow_key,bump)=find_program_address(
            &[b"escrow",self.accounts.maker.key().as_ref(),self.instruction_data.seed.to_le_bytes().as_ref()], 
        &pinocchio_system::ID);

        let escrowAccounts=Escrow::from(Escrow { mint_a: self.accounts.mint_a, mint_b: self.accounts.mint_b, maker: self.accounts.maker, amount: self.instruction_data.amount, escrow_bump: Some(&bump), seed: Some(self.instruction_data.seed) });
        pinocchio_system::instructions::CreateAccount{
            from:self.accounts.maker,
            to:self.accounts.escrow,
            lamports:Rent::get()?.minimum_balance(Escrow::SIZE),
            owner:self.accounts.maker.key(),
            space:Escrow::SIZE as u64
        };
        Ok(())
    }
}