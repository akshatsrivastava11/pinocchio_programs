use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

pub struct Escrow<'a>{
    pub mint_a:&'a AccountInfo,
    pub mint_b:&'a AccountInfo,
    pub maker:&'a AccountInfo,
    pub amount:&'a u64,
    pub escrow_bump: Option<&'a u8>,
    pub seed: Option<&'a u8>
}
impl <'a>TryFrom<(&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a u64,&'a u8,&'a u64)> for Escrow<'a>{
    type Error = ProgramError;
    fn try_from(data: (&'a AccountInfo,&'a AccountInfo,&'a AccountInfo,&'a u64,&'a u8,&'a u64)) -> Result<Self, Self::Error> {
        let (mint_a,mint_b,maker,amount,escrow_bump,seed) = data else{
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        if !maker.is_signer(){
            return Err(ProgramError::MissingRequiredSignature);
        };
        if !maker.is_owned_by(&pinocchio_system::ID){
            return  Err(ProgramError::InvalidAccountOwner);
        };
        Ok(Self { mint_a: mint_a, mint_b: mint_b, maker: maker, amount: amount, escrow_bump: None, seed:  None })
    }
}
