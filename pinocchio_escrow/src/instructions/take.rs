use core::{pin, slice};

use pinocchio::{account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::find_program_address, ProgramResult};
use pinocchio_system::instructions::Transfer;

pub struct Take<'a>{
    pub accounts:TakeAccounts<'a>,
    pub instruction_data:TakeInstruction<'a>
}
pub struct TakeAccounts<'a>{
       pub mint_a: &'a AccountInfo,
    pub mint_b: &'a AccountInfo,
    pub maker: &'a AccountInfo,
    pub taker_mint_b: &'a AccountInfo,
    pub taker_mint_a: &'a AccountInfo,
    pub maker_mint_b: &'a AccountInfo,
    pub escrow:&'a AccountInfo,
    pub vault:&'a AccountInfo,
    pub escrow_bump: &'a u8,
    pub seed: &'a u8,

}
pub struct TakeInstruction<'a>{
    pub amount: &'a u64,
    pub seed: &'a u8
} 
impl <'a>TryFrom<(&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a u64,&'a u8,&'a u8)> for Take<'a>{
    type Error = ProgramError;
    fn try_from(data: (&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a u64,&'a u8,&'a u8)) -> Result<Self, Self::Error> {
              let (mint_a,mint_b,maker,taker_mint_b,taker_mint_a,maker_mint_b,escrow,vault,amount,escrow_bump,seed) = data else{
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
        Ok(Self { accounts: TakeAccounts{seed:seed,taker_mint_b,maker_mint_b,taker_mint_a,escrow_bump:escrow_bump,escrow:escrow,vault:vault,maker:maker,mint_a:mint_a,mint_b:mint_b}, instruction_data: TakeInstruction { amount: &amount, seed: seed  }})
    }
}


impl<'a> Take<'a>{
    pub fn process(&mut self)->ProgramResult{

        let key=self.instruction_data.seed.to_le_bytes();
        let seeds=[Seed::from(b"vault"),Seed::from(self.accounts.maker.key().as_ref()),Seed::from(key.as_ref())];
        let signers=[Signer::from(&seeds)];
        //taker_mint_b to maker_mint_b
        pinocchio_token::instructions::Transfer{
            from:self.accounts.taker_mint_b,
            to:self.accounts.maker_mint_b,
            amount:*self.instruction_data.amount,
            authority:self.accounts.escrow
        }.invoke_signed(&signers)?;
        //vault to taker_mint_a
        pinocchio_token::instructions::Transfer{
            from:self.accounts.vault,
            to:self.accounts.taker_mint_a,
            amount:*self.instruction_data.amount,
            authority:self.accounts.escrow
        }.invoke_signed(&signers)
           
    }
}