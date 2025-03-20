use borsh::BorshSerialize;
use ic_solana::types::{AccountMeta, Instruction, Pubkey};
pub fn new_with_borsh<T: BorshSerialize>(
    program_id: Pubkey,
    data: &T,
    accounts: Vec<AccountMeta>,
) -> Instruction {
    let data = borsh::to_vec(data).unwrap();
    Instruction {
        program_id,
        accounts,
        data,
    }
}
