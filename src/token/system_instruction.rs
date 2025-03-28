use std::str::FromStr;

use ic_solana::types::{AccountMeta, Instruction, Pubkey};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use thiserror::Error;
pub const SYSTEM_PROGRAM_ID: &[u8; 32] = b"11111111111111111111111111111111";
pub const SYSVAR_ID: &str = "SysvarRent111111111111111111111111111111111";

pub trait DecodeError<E> {
    fn decode_custom_error_to_enum(custom: u32) -> Option<E>
    where
        E: FromPrimitive,
    {
        E::from_u32(custom)
    }
    fn type_of() -> &'static str;
}

#[derive(Error, Debug, Serialize, Clone, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum SystemError {
    #[error("an account with the same address already exists")]
    AccountAlreadyInUse,
    #[error("account does not have enough SOL to perform the operation")]
    ResultWithNegativeLamports,
    #[error("cannot assign account to this program id")]
    InvalidProgramId,
    #[error("cannot allocate account data of this length")]
    InvalidAccountDataLength,
    #[error("length of requested seed is too long")]
    MaxSeedLengthExceeded,
    #[error("provided address does not match addressed derived from seed")]
    AddressWithSeedMismatch,
    #[error("advancing stored nonce requires a populated RecentBlockhashes sysvar")]
    NonceNoRecentBlockhashes,
    #[error("stored nonce is still in recent_blockhashes")]
    NonceBlockhashNotExpired,
    #[error("specified nonce does not match stored nonce")]
    NonceUnexpectedBlockhashValue,
}

impl<T> DecodeError<T> for SystemError {
    fn type_of() -> &'static str {
        "SystemError"
    }
}

/// Maximum permitted size of account data (10 MiB).
pub const MAX_PERMITTED_DATA_LENGTH: u64 = 10 * 1024 * 1024;

/// Maximum permitted size of new allocations per transaction, in bytes.
///
/// The value was chosen such that at least one max sized account could be created,
/// plus some additional resize allocations.
pub const MAX_PERMITTED_ACCOUNTS_DATA_ALLOCATIONS_PER_TRANSACTION: i64 =
    MAX_PERMITTED_DATA_LENGTH as i64 * 2;

/// An instruction to the system program.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum SystemInstruction {
    /// Create a new account
    ///
    /// # Account references
    ///   0. `[WRITE, SIGNER]` Funding account
    ///   1. `[WRITE, SIGNER]` New account
    CreateAccount {
        /// Number of lamports to transfer to the new account
        lamports: u64,

        /// Number of bytes of memory to allocate
        space: u64,

        /// Address of program that will own the new account
        owner: Pubkey,
    },

    /// Assign account to a program
    ///
    /// # Account references
    ///   0. `[WRITE, SIGNER]` Assigned account public key
    Assign {
        /// Owner program account
        owner: Pubkey,
    },

    /// Transfer lamports
    ///
    /// # Account references
    ///   0. `[WRITE, SIGNER]` Funding account
    ///   1. `[WRITE]` Recipient account
    Transfer { lamports: u64 },

    /// Create a new account at an address derived from a base pubkey and a seed
    ///
    /// # Account references
    ///   0. `[WRITE, SIGNER]` Funding account
    ///   1. `[WRITE]` Created account
    ///   2. `[SIGNER]` (optional) Base account; the account matching the base Pubkey below must be
    ///                          provided as a signer, but may be the same as the funding account
    ///                          and provided as account 0
    CreateAccountWithSeed {
        /// Base public key
        base: Pubkey,

        /// String of ASCII chars, no longer than `Pubkey::MAX_SEED_LEN`
        seed: String,

        /// Number of lamports to transfer to the new account
        lamports: u64,

        /// Number of bytes of memory to allocate
        space: u64,

        /// Owner program account address
        owner: Pubkey,
    },

    /// Consumes a stored nonce, replacing it with a successor
    ///
    /// # Account references
    ///   0. `[WRITE]` Nonce account
    ///   1. `[]` RecentBlockhashes sysvar
    ///   2. `[SIGNER]` Nonce authority
    AdvanceNonceAccount,

    /// Withdraw funds from a nonce account
    ///
    /// # Account references
    ///   0. `[WRITE]` Nonce account
    ///   1. `[WRITE]` Recipient account
    ///   2. `[]` RecentBlockhashes sysvar
    ///   3. `[]` Rent sysvar
    ///   4. `[SIGNER]` Nonce authority
    ///
    /// The `u64` parameter is the lamports to withdraw, which must leave the
    /// account balance above the rent exempt reserve or at zero.
    WithdrawNonceAccount(u64),

    /// Drive state of Uninitialized nonce account to Initialized, setting the nonce value
    ///
    /// # Account references
    ///   0. `[WRITE]` Nonce account
    ///   1. `[]` RecentBlockhashes sysvar
    ///   2. `[]` Rent sysvar
    ///
    /// The `Pubkey` parameter specifies the entity authorized to execute nonce
    /// instruction on the account
    ///
    /// No signatures are required to execute this instruction, enabling derived
    /// nonce account addresses
    InitializeNonceAccount(Pubkey),

    /// Change the entity authorized to execute nonce instructions on the account
    ///
    /// # Account references
    ///   0. `[WRITE]` Nonce account
    ///   1. `[SIGNER]` Nonce authority
    ///
    /// The `Pubkey` parameter identifies the entity to authorize
    AuthorizeNonceAccount(Pubkey),

    /// Allocate space in a (possibly new) account without funding
    ///
    /// # Account references
    ///   0. `[WRITE, SIGNER]` New account
    Allocate {
        /// Number of bytes of memory to allocate
        space: u64,
    },

    /// Allocate space for and assign an account at an address
    ///    derived from a base public key and a seed
    ///
    /// # Account references
    ///   0. `[WRITE]` Allocated account
    ///   1. `[SIGNER]` Base account
    AllocateWithSeed {
        /// Base public key
        base: Pubkey,

        /// String of ASCII chars, no longer than `pubkey::MAX_SEED_LEN`
        seed: String,

        /// Number of bytes of memory to allocate
        space: u64,

        /// Owner program account
        owner: Pubkey,
    },

    /// Assign account to a program based on a seed
    ///
    /// # Account references
    ///   0. `[WRITE]` Assigned account
    ///   1. `[SIGNER]` Base account
    AssignWithSeed {
        /// Base public key
        base: Pubkey,

        /// String of ASCII chars, no longer than `pubkey::MAX_SEED_LEN`
        seed: String,

        /// Owner program account
        owner: Pubkey,
    },

    /// Transfer lamports from a derived address
    ///
    /// # Account references
    ///   0. `[WRITE]` Funding account
    ///   1. `[SIGNER]` Base for funding account
    ///   2. `[WRITE]` Recipient account
    TransferWithSeed {
        /// Amount to transfer
        lamports: u64,

        /// Seed to use to derive the funding account address
        from_seed: String,

        /// Owner to use to derive the funding account address
        from_owner: Pubkey,
    },

    /// One-time idempotent upgrade of legacy nonce versions in order to bump
    /// them out of chain blockhash domain.
    ///
    /// # Account references
    ///   0. `[WRITE]` Nonce account
    UpgradeNonceAccount,
}

/// Create an account.
///
/// This function produces an [`Instruction`] which must be submitted in a
/// [`Transaction`] or [invoked] to take effect, containing a serialized
/// [`SystemInstruction::CreateAccount`].
///
/// [`Transaction`]: https://docs.rs/solana-sdk/latest/solana_sdk/transaction/struct.Transaction.html
/// [invoked]: crate::program::invoke
///
/// Account creation typically involves three steps: [`allocate`] space,
/// [`transfer`] lamports for rent, [`assign`] to its owning program. The
/// [`create_account`] function does all three at once.
///
/// # Required signers
///
/// The `from_pubkey` and `to_pubkey` signers must sign the transaction.
///
/// # Examples
///
/// These examples use a single invocation of
/// [`SystemInstruction::CreateAccount`] to create a new account, allocate some
/// space, transfer it the minimum lamports for rent exemption, and assign it to
/// the system program,
///
/// ## Example: client-side RPC
///
/// This example submits the instruction from an RPC client.
/// The `payer` and `new_account` are signers.
///
/// ```
/// # use solana_program::example_mocks::{solana_sdk, solana_rpc_client};
/// use solana_rpc_client::rpc_client::RpcClient;
/// use solana_sdk::{
///     pubkey::Pubkey,
///     signature::{Keypair, Signer},
///     system_instruction,
///     system_program,
///     transaction::Transaction,
/// };
/// use anyhow::Result;
///
/// fn create_account(
///     client: &RpcClient,
///     payer: &Keypair,
///     new_account: &Keypair,
///     space: u64,
/// ) -> Result<()> {
///     let rent = client.get_minimum_balance_for_rent_exemption(space.try_into()?)?;
///     let instr = system_instruction::create_account(
///         &payer.pubkey(),
///         &new_account.pubkey(),
///         rent,
///         space,
///         &system_program::ID,
///     );
///
///     let blockhash = client.get_latest_blockhash()?;
///     let tx = Transaction::new_signed_with_payer(
///         &[instr],
///         Some(&payer.pubkey()),
///         &[payer, new_account],
///         blockhash,
///     );
///
///     let _sig = client.send_and_confirm_transaction(&tx)?;
///
///     Ok(())
/// }
/// # let payer = Keypair::new();
/// # let new_account = Keypair::new();
/// # let client = RpcClient::new(String::new());
/// # create_account(&client, &payer, &new_account, 0);
/// #
/// # Ok::<(), anyhow::Error>(())
/// ```
///

pub fn create_account(
    from_pubkey: &Pubkey,
    to_pubkey: &Pubkey,
    lamports: u64,
    space: u64,
    owner: &Pubkey,
) -> Instruction {
    let account_metas = vec![
        AccountMeta::new(*from_pubkey, true),
        AccountMeta::new(*to_pubkey, true),
    ];
    Instruction::new_with_bincode(
        Pubkey::from_str("11111111111111111111111111111111").unwrap(),
        &SystemInstruction::CreateAccount {
            lamports,
            space,
            owner: *owner,
        },
        account_metas,
    )
}

// we accept `to` as a parameter so that callers do their own error handling when
//   calling create_with_seed()
pub fn create_account_with_seed(
    from_pubkey: &Pubkey,
    to_pubkey: &Pubkey, // must match create_with_seed(base, seed, owner)
    base: &Pubkey,
    seed: &str,
    lamports: u64,
    space: u64,
    owner: &Pubkey,
) -> Instruction {
    let account_metas = vec![
        AccountMeta::new(*from_pubkey, true),
        AccountMeta::new(*to_pubkey, false),
        AccountMeta::new_readonly(*base, true),
    ];

    Instruction::new_with_bincode(
        Pubkey::from_str("11111111111111111111111111111111").unwrap(),
        &SystemInstruction::CreateAccountWithSeed {
            base: *base,
            seed: seed.to_string(),
            lamports,
            space,
            owner: *owner,
        },
        account_metas,
    )
}

/// Assign ownership of an account from the system program.
///
/// This function produces an [`Instruction`] which must be submitted in a
/// [`Transaction`] or [invoked] to take effect, containing a serialized
/// [`SystemInstruction::Assign`].
///
/// [`Transaction`]: https://docs.rs/solana-sdk/latest/solana_sdk/transaction/struct.Transaction.html
/// [invoked]: crate::program::invoke
///
/// # Required signers
///
/// The `pubkey` signer must sign the transaction.
///
/// # Examples
///
/// These examples allocate space for an account, transfer it the minimum
/// balance for rent exemption, and assign the account to a program.
///
/// ## Example: client-side RPC
///
/// This example submits the instructions from an RPC client.
/// It assigns the account to a provided program account.
/// The `payer` and `new_account` are signers.
///
/// ```
/// # use solana_program::example_mocks::{solana_sdk, solana_rpc_client};
/// use solana_rpc_client::rpc_client::RpcClient;
/// use solana_sdk::{
///     pubkey::Pubkey,
///     signature::{Keypair, Signer},
///     system_instruction,
///     transaction::Transaction,
/// };
/// use anyhow::Result;
///
/// fn create_account(
///     client: &RpcClient,
///     payer: &Keypair,
///     new_account: &Keypair,
///     owning_program: &Pubkey,
///     space: u64,
/// ) -> Result<()> {
///     let rent = client.get_minimum_balance_for_rent_exemption(space.try_into()?)?;
///
///     let transfer_instr = system_instruction::transfer(
///         &payer.pubkey(),
///         &new_account.pubkey(),
///         rent,
///     );
///
///     let allocate_instr = system_instruction::allocate(
///         &new_account.pubkey(),
///         space,
///     );
///
///     let assign_instr = system_instruction::assign(
///         &new_account.pubkey(),
///         owning_program,
///     );
///
///     let blockhash = client.get_latest_blockhash()?;
///     let tx = Transaction::new_signed_with_payer(
///         &[transfer_instr, allocate_instr, assign_instr],
///         Some(&payer.pubkey()),
///         &[payer, new_account],
///         blockhash,
///     );
///
///     let _sig = client.send_and_confirm_transaction(&tx)?;
///
///     Ok(())
/// }
/// # let client = RpcClient::new(String::new());
/// # let payer = Keypair::new();
/// # let new_account = Keypair::new();
/// # let owning_program = Pubkey::new_unique();
/// # create_account(&client, &payer, &new_account, &owning_program, 1);
/// #
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// ## Example: on-chain program
///
/// This example submits the instructions from an on-chain Solana program. The
/// created account is a [program derived address][pda], funded by `payer`, and
/// assigned to the running program. The `payer` and `new_account_pda` are
/// signers, with `new_account_pda` being signed for virtually by the program
/// itself via [`invoke_signed`], `payer` being signed for by the client that
/// submitted the transaction.
///
/// [pda]: Pubkey::find_program_address
/// [`invoke_signed`]: crate::program::invoke_signed
///
/// ```
/// # use borsh::{BorshDeserialize, BorshSerialize};
/// use solana_program::{
///     account_info::{next_account_info, AccountInfo},
///     entrypoint,
///     entrypoint::ProgramResult,
///     msg,
///     program::invoke_signed,
///     pubkey::Pubkey,
///     system_instruction,
///     system_program,
///     sysvar::rent::Rent,
///     sysvar::Sysvar,
/// };
///
/// #[derive(BorshSerialize, BorshDeserialize, Debug)]
/// # #[borsh(crate = "borsh")]
/// pub struct CreateAccountInstruction {
///     /// The PDA seed used to distinguish the new account from other PDAs
///     pub new_account_seed: [u8; 16],
///     /// The PDA bump seed
///     pub new_account_bump_seed: u8,
///     /// The amount of space to allocate for `new_account_pda`
///     pub space: u64,
/// }
///
/// entrypoint!(process_instruction);
///
/// fn process_instruction(
///     program_id: &Pubkey,
///     accounts: &[AccountInfo],
///     instruction_data: &[u8],
/// ) -> ProgramResult {
///     let instr = CreateAccountInstruction::deserialize(&mut &instruction_data[..])?;
///
///     let account_info_iter = &mut accounts.iter();
///
///     let payer = next_account_info(account_info_iter)?;
///     let new_account_pda = next_account_info(account_info_iter)?;
///     let system_account = next_account_info(account_info_iter)?;
///
///     assert!(payer.is_signer);
///     assert!(payer.is_writable);
///     // Note that `new_account_pda` is not a signer yet.
///     // This program will sign for it via `invoke_signed`.
///     assert!(!new_account_pda.is_signer);
///     assert!(new_account_pda.is_writable);
///     assert!(system_program::check_id(system_account.key));
///
///     let new_account_seed = &instr.new_account_seed;
///     let new_account_bump_seed = instr.new_account_bump_seed;
///
///     let rent = Rent::get()?
///         .minimum_balance(instr.space.try_into().expect("overflow"));
///
///     invoke_signed(
///         &system_instruction::transfer(
///             payer.key,
///             new_account_pda.key,
///             rent,
///         ),
///         &[payer.clone(), new_account_pda.clone()],
///         &[&[payer.key.as_ref(), new_account_seed, &[new_account_bump_seed]]],
///     )?;
///
///     invoke_signed(
///         &system_instruction::allocate(
///             new_account_pda.key,
///             instr.space,
///         ),
///         &[new_account_pda.clone()],
///         &[&[payer.key.as_ref(), new_account_seed, &[new_account_bump_seed]]],
///     )?;
///
///     invoke_signed(
///         &system_instruction::assign(
///             new_account_pda.key,
///             &program_id,
///         ),
///         &[new_account_pda.clone()],
///         &[&[payer.key.as_ref(), new_account_seed, &[new_account_bump_seed]]],
///     )?;
///
///     Ok(())
/// }
///
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn assign(pubkey: &Pubkey, owner: &Pubkey) -> Instruction {
    let account_metas = vec![AccountMeta::new(*pubkey, true)];
    Instruction::new_with_bincode(
        Pubkey::from_str("11111111111111111111111111111111").unwrap(),
        &SystemInstruction::Assign { owner: *owner },
        account_metas,
    )
}

pub fn assign_with_seed(
    address: &Pubkey, // must match create_with_seed(base, seed, owner)
    base: &Pubkey,
    seed: &str,
    owner: &Pubkey,
) -> Instruction {
    let account_metas = vec![
        AccountMeta::new(*address, false),
        AccountMeta::new_readonly(*base, true),
    ];
    Instruction::new_with_bincode(
        Pubkey::from_str("11111111111111111111111111111111").unwrap(),
        &SystemInstruction::AssignWithSeed {
            base: *base,
            seed: seed.to_string(),
            owner: *owner,
        },
        account_metas,
    )
}

/// Transfer lamports from an account owned by the system program.
///
/// This function produces an [`Instruction`] which must be submitted in a
/// [`Transaction`] or [invoked] to take effect, containing a serialized
/// [`SystemInstruction::Transfer`].
///
/// [`Transaction`]: https://docs.rs/solana-sdk/latest/solana_sdk/transaction/struct.Transaction.html
/// [invoked]: crate::program::invoke
///
/// # Required signers
///
/// The `from_pubkey` signer must sign the transaction.
///
/// # Examples
///
/// These examples allocate space for an account, transfer it the minimum
/// balance for rent exemption, and assign the account to a program.
///
/// # Example: client-side RPC
///
/// This example submits the instructions from an RPC client.
/// It assigns the account to a provided program account.
/// The `payer` and `new_account` are signers.
///
/// ```
/// # use solana_program::example_mocks::{solana_sdk, solana_rpc_client};
/// use solana_rpc_client::rpc_client::RpcClient;
/// use solana_sdk::{
///     pubkey::Pubkey,
///     signature::{Keypair, Signer},
///     system_instruction,
///     transaction::Transaction,
/// };
/// use anyhow::Result;
///
/// fn create_account(
///     client: &RpcClient,
///     payer: &Keypair,
///     new_account: &Keypair,
///     owning_program: &Pubkey,
///     space: u64,
/// ) -> Result<()> {
///     let rent = client.get_minimum_balance_for_rent_exemption(space.try_into()?)?;
///
///     let transfer_instr = system_instruction::transfer(
///         &payer.pubkey(),
///         &new_account.pubkey(),
///         rent,
///     );
///
///     let allocate_instr = system_instruction::allocate(
///         &new_account.pubkey(),
///         space,
///     );
///
///     let assign_instr = system_instruction::assign(
///         &new_account.pubkey(),
///         owning_program,
///     );
///
///     let blockhash = client.get_latest_blockhash()?;
///     let tx = Transaction::new_signed_with_payer(
///         &[transfer_instr, allocate_instr, assign_instr],
///         Some(&payer.pubkey()),
///         &[payer, new_account],
///         blockhash,
///     );
///
///     let _sig = client.send_and_confirm_transaction(&tx)?;
///
///     Ok(())
/// }
/// # let client = RpcClient::new(String::new());
/// # let payer = Keypair::new();
/// # let new_account = Keypair::new();
/// # let owning_program = Pubkey::new_unique();
/// # create_account(&client, &payer, &new_account, &owning_program, 1);
/// #
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// ## Example: on-chain program
///
/// This example submits the instructions from an on-chain Solana program. The
/// created account is a [program derived address][pda], funded by `payer`, and
/// assigned to the running program. The `payer` and `new_account_pda` are
/// signers, with `new_account_pda` being signed for virtually by the program
/// itself via [`invoke_signed`], `payer` being signed for by the client that
/// submitted the transaction.
///
/// [pda]: Pubkey::find_program_address
/// [`invoke_signed`]: crate::program::invoke_signed
///
/// ```
/// # use borsh::{BorshDeserialize, BorshSerialize};
/// use solana_program::{
///     account_info::{next_account_info, AccountInfo},
///     entrypoint,
///     entrypoint::ProgramResult,
///     msg,
///     program::invoke_signed,
///     pubkey::Pubkey,
///     system_instruction,
///     system_program,
///     sysvar::rent::Rent,
///     sysvar::Sysvar,
/// };
///
/// #[derive(BorshSerialize, BorshDeserialize, Debug)]
/// # #[borsh(crate = "borsh")]
/// pub struct CreateAccountInstruction {
///     /// The PDA seed used to distinguish the new account from other PDAs
///     pub new_account_seed: [u8; 16],
///     /// The PDA bump seed
///     pub new_account_bump_seed: u8,
///     /// The amount of space to allocate for `new_account_pda`
///     pub space: u64,
/// }
///
/// entrypoint!(process_instruction);
///
/// fn process_instruction(
///     program_id: &Pubkey,
///     accounts: &[AccountInfo],
///     instruction_data: &[u8],
/// ) -> ProgramResult {
///     let instr = CreateAccountInstruction::deserialize(&mut &instruction_data[..])?;
///
///     let account_info_iter = &mut accounts.iter();
///
///     let payer = next_account_info(account_info_iter)?;
///     let new_account_pda = next_account_info(account_info_iter)?;
///     let system_account = next_account_info(account_info_iter)?;
///
///     assert!(payer.is_signer);
///     assert!(payer.is_writable);
///     // Note that `new_account_pda` is not a signer yet.
///     // This program will sign for it via `invoke_signed`.
///     assert!(!new_account_pda.is_signer);
///     assert!(new_account_pda.is_writable);
///     assert!(system_program::check_id(system_account.key));
///
///     let new_account_seed = &instr.new_account_seed;
///     let new_account_bump_seed = instr.new_account_bump_seed;
///
///     let rent = Rent::get()?
///         .minimum_balance(instr.space.try_into().expect("overflow"));
///
///     invoke_signed(
///         &system_instruction::transfer(
///             payer.key,
///             new_account_pda.key,
///             rent,
///         ),
///         &[payer.clone(), new_account_pda.clone()],
///         &[&[payer.key.as_ref(), new_account_seed, &[new_account_bump_seed]]],
///     )?;
///
///     invoke_signed(
///         &system_instruction::allocate(
///             new_account_pda.key,
///             instr.space,
///         ),
///         &[new_account_pda.clone()],
///         &[&[payer.key.as_ref(), new_account_seed, &[new_account_bump_seed]]],
///     )?;
///
///     invoke_signed(
///         &system_instruction::assign(
///             new_account_pda.key,
///             &program_id,
///         ),
///         &[new_account_pda.clone()],
///         &[&[payer.key.as_ref(), new_account_seed, &[new_account_bump_seed]]],
///     )?;
///
///     Ok(())
/// }
///
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn transfer(from_pubkey: &Pubkey, to_pubkey: &Pubkey, lamports: u64) -> Instruction {
    let account_metas = vec![
        AccountMeta::new(*from_pubkey, true),
        AccountMeta::new(*to_pubkey, false),
    ];
    Instruction::new_with_bincode(
        // Pubkey::new(*SYSTEM_PROGRAM_ID),
        Pubkey::from_str("11111111111111111111111111111111").unwrap(),
        &SystemInstruction::Transfer { lamports },
        account_metas,
    )
}

pub fn transfer_with_seed(
    from_pubkey: &Pubkey, // must match create_with_seed(base, seed, owner)
    from_base: &Pubkey,
    from_seed: String,
    from_owner: &Pubkey,
    to_pubkey: &Pubkey,
    lamports: u64,
) -> Instruction {
    let account_metas = vec![
        AccountMeta::new(*from_pubkey, false),
        AccountMeta::new_readonly(*from_base, true),
        AccountMeta::new(*to_pubkey, false),
    ];
    Instruction::new_with_bincode(
        Pubkey::from_str("11111111111111111111111111111111").unwrap(),
        &SystemInstruction::TransferWithSeed {
            lamports,
            from_seed,
            from_owner: *from_owner,
        },
        account_metas,
    )
}

/// Allocate space for an account.
///
/// This function produces an [`Instruction`] which must be submitted in a
/// [`Transaction`] or [invoked] to take effect, containing a serialized
/// [`SystemInstruction::Allocate`].
///
/// [`Transaction`]: https://docs.rs/solana-sdk/latest/solana_sdk/transaction/struct.Transaction.html
/// [invoked]: crate::program::invoke
///
/// The transaction will fail if the account already has size greater than 0,
/// or if the requested size is greater than [`MAX_PERMITTED_DATA_LENGTH`].
///
/// # Required signers
///
/// The `pubkey` signer must sign the transaction.
///
/// # Examples
///
/// These examples allocate space for an account, transfer it the minimum
/// balance for rent exemption, and assign the account to a program.
///
/// # Example: client-side RPC
///
/// This example submits the instructions from an RPC client.
/// It assigns the account to a provided program account.
/// The `payer` and `new_account` are signers.
///
/// ```
/// # use solana_program::example_mocks::{solana_sdk, solana_rpc_client};
/// use solana_rpc_client::rpc_client::RpcClient;
/// use solana_sdk::{
///     pubkey::Pubkey,
///     signature::{Keypair, Signer},
///     system_instruction,
///     transaction::Transaction,
/// };
/// use anyhow::Result;
///
/// fn create_account(
///     client: &RpcClient,
///     payer: &Keypair,
///     new_account: &Keypair,
///     owning_program: &Pubkey,
///     space: u64,
/// ) -> Result<()> {
///     let rent = client.get_minimum_balance_for_rent_exemption(space.try_into()?)?;
///
///     let transfer_instr = system_instruction::transfer(
///         &payer.pubkey(),
///         &new_account.pubkey(),
///         rent,
///     );
///
///     let allocate_instr = system_instruction::allocate(
///         &new_account.pubkey(),
///         space,
///     );
///
///     let assign_instr = system_instruction::assign(
///         &new_account.pubkey(),
///         owning_program,
///     );
///
///     let blockhash = client.get_latest_blockhash()?;
///     let tx = Transaction::new_signed_with_payer(
///         &[transfer_instr, allocate_instr, assign_instr],
///         Some(&payer.pubkey()),
///         &[payer, new_account],
///         blockhash,
///     );
///
///     let _sig = client.send_and_confirm_transaction(&tx)?;
///
///     Ok(())
/// }
/// # let client = RpcClient::new(String::new());
/// # let payer = Keypair::new();
/// # let new_account = Keypair::new();
/// # let owning_program = Pubkey::new_unique();
/// # create_account(&client, &payer, &new_account, &owning_program, 1);
/// #
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// ## Example: on-chain program
///
/// This example submits the instructions from an on-chain Solana program. The
/// created account is a [program derived address][pda], funded by `payer`, and
/// assigned to the running program. The `payer` and `new_account_pda` are
/// signers, with `new_account_pda` being signed for virtually by the program
/// itself via [`invoke_signed`], `payer` being signed for by the client that
/// submitted the transaction.
///
/// [pda]: Pubkey::find_program_address
/// [`invoke_signed`]: crate::program::invoke_signed
///
/// ```
/// # use borsh::{BorshDeserialize, BorshSerialize};
/// use solana_program::{
///     account_info::{next_account_info, AccountInfo},
///     entrypoint,
///     entrypoint::ProgramResult,
///     msg,
///     program::invoke_signed,
///     pubkey::Pubkey,
///     system_instruction,
///     system_program,
///     sysvar::rent::Rent,
///     sysvar::Sysvar,
/// };
///
/// #[derive(BorshSerialize, BorshDeserialize, Debug)]
/// # #[borsh(crate = "borsh")]
/// pub struct CreateAccountInstruction {
///     /// The PDA seed used to distinguish the new account from other PDAs
///     pub new_account_seed: [u8; 16],
///     /// The PDA bump seed
///     pub new_account_bump_seed: u8,
///     /// The amount of space to allocate for `new_account_pda`
///     pub space: u64,
/// }
///
/// entrypoint!(process_instruction);
///
/// fn process_instruction(
///     program_id: &Pubkey,
///     accounts: &[AccountInfo],
///     instruction_data: &[u8],
/// ) -> ProgramResult {
///     let instr = CreateAccountInstruction::deserialize(&mut &instruction_data[..])?;
///
///     let account_info_iter = &mut accounts.iter();
///
///     let payer = next_account_info(account_info_iter)?;
///     let new_account_pda = next_account_info(account_info_iter)?;
///     let system_account = next_account_info(account_info_iter)?;
///
///     assert!(payer.is_signer);
///     assert!(payer.is_writable);
///     // Note that `new_account_pda` is not a signer yet.
///     // This program will sign for it via `invoke_signed`.
///     assert!(!new_account_pda.is_signer);
///     assert!(new_account_pda.is_writable);
///     assert!(system_program::check_id(system_account.key));
///
///     let new_account_seed = &instr.new_account_seed;
///     let new_account_bump_seed = instr.new_account_bump_seed;
///
///     let rent = Rent::get()?
///         .minimum_balance(instr.space.try_into().expect("overflow"));
///
///     invoke_signed(
///         &system_instruction::transfer(
///             payer.key,
///             new_account_pda.key,
///             rent,
///         ),
///         &[payer.clone(), new_account_pda.clone()],
///         &[&[payer.key.as_ref(), new_account_seed, &[new_account_bump_seed]]],
///     )?;
///
///     invoke_signed(
///         &system_instruction::allocate(
///             new_account_pda.key,
///             instr.space,
///         ),
///         &[new_account_pda.clone()],
///         &[&[payer.key.as_ref(), new_account_seed, &[new_account_bump_seed]]],
///     )?;
///
///     invoke_signed(
///         &system_instruction::assign(
///             new_account_pda.key,
///             &program_id,
///         ),
///         &[new_account_pda.clone()],
///         &[&[payer.key.as_ref(), new_account_seed, &[new_account_bump_seed]]],
///     )?;
///
///     Ok(())
/// }
///
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn allocate(pubkey: &Pubkey, space: u64) -> Instruction {
    let account_metas = vec![AccountMeta::new(*pubkey, true)];
    Instruction::new_with_bincode(
        Pubkey::from_str("11111111111111111111111111111111").unwrap(),
        &SystemInstruction::Allocate { space },
        account_metas,
    )
}

pub fn allocate_with_seed(
    address: &Pubkey, // must match create_with_seed(base, seed, owner)
    base: &Pubkey,
    seed: &str,
    space: u64,
    owner: &Pubkey,
) -> Instruction {
    let account_metas = vec![
        AccountMeta::new(*address, false),
        AccountMeta::new_readonly(*base, true),
    ];
    Instruction::new_with_bincode(
        Pubkey::from_str("11111111111111111111111111111111").unwrap(),
        &SystemInstruction::AllocateWithSeed {
            base: *base,
            seed: seed.to_string(),
            space,
            owner: *owner,
        },
        account_metas,
    )
}

/// Transfer lamports from an account owned by the system program to multiple accounts.
///
/// This function produces a vector of [`Instruction`]s which must be submitted
/// in a [`Transaction`] or [invoked] to take effect, containing serialized
/// [`SystemInstruction::Transfer`]s.
///
/// [`Transaction`]: https://docs.rs/solana-sdk/latest/solana_sdk/transaction/struct.Transaction.html
/// [invoked]: crate::program::invoke
///
/// # Required signers
///
/// The `from_pubkey` signer must sign the transaction.
///
/// # Examples
///
/// ## Example: client-side RPC
///
/// This example performs multiple transfers in a single transaction.
///
/// ```
/// # use solana_program::example_mocks::{solana_sdk, solana_rpc_client};
/// use solana_rpc_client::rpc_client::RpcClient;
/// use solana_sdk::{
///     pubkey::Pubkey,
///     signature::{Keypair, Signer},
///     system_instruction,
///     transaction::Transaction,
/// };
/// use anyhow::Result;
///
/// fn transfer_lamports_to_many(
///     client: &RpcClient,
///     from: &Keypair,
///     to_and_amount: &[(Pubkey, u64)],
/// ) -> Result<()> {
///     let instrs = system_instruction::transfer_many(&from.pubkey(), to_and_amount);
///
///     let blockhash = client.get_latest_blockhash()?;
///     let tx = Transaction::new_signed_with_payer(
///         &instrs,
///         Some(&from.pubkey()),
///         &[from],
///         blockhash,
///     );
///
///     let _sig = client.send_and_confirm_transaction(&tx)?;
///
///     Ok(())
/// }
/// # let from = Keypair::new();
/// # let to_and_amount = vec![
/// #     (Pubkey::new_unique(), 1_000),
/// #     (Pubkey::new_unique(), 2_000),
/// #     (Pubkey::new_unique(), 3_000),
/// # ];
/// # let client = RpcClient::new(String::new());
/// # transfer_lamports_to_many(&client, &from, &to_and_amount);
/// #
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// ## Example: on-chain program
///
/// This example makes multiple transfers out of a "bank" account,
/// a [program derived address][pda] owned by the calling program.
/// This example submits the instructions from an on-chain Solana program. The
/// created account is a [program derived address][pda], and it is assigned to
/// the running program. The `payer` and `new_account_pda` are signers, with
/// `new_account_pda` being signed for virtually by the program itself via
/// [`invoke_signed`], `payer` being signed for by the client that submitted the
/// transaction.
///
/// [pda]: Pubkey::find_program_address
/// [`invoke_signed`]: crate::program::invoke_signed
///
/// ```
/// # use borsh::{BorshDeserialize, BorshSerialize};
/// use solana_program::{
///     account_info::{next_account_info, next_account_infos, AccountInfo},
///     entrypoint,
///     entrypoint::ProgramResult,
///     msg,
///     program::invoke_signed,
///     pubkey::Pubkey,
///     system_instruction,
///     system_program,
/// };
///
/// /// # Accounts
/// ///
/// /// - 0: bank_pda - writable
/// /// - 1: system_program - executable
/// /// - *: to - writable
/// #[derive(BorshSerialize, BorshDeserialize, Debug)]
/// # #[borsh(crate = "borsh")]
/// pub struct TransferLamportsToManyInstruction {
///     pub bank_pda_bump_seed: u8,
///     pub amount_list: Vec<u64>,
/// }
///
/// entrypoint!(process_instruction);
///
/// fn process_instruction(
///     program_id: &Pubkey,
///     accounts: &[AccountInfo],
///     instruction_data: &[u8],
/// ) -> ProgramResult {
///     let instr = TransferLamportsToManyInstruction::deserialize(&mut &instruction_data[..])?;
///
///     let account_info_iter = &mut accounts.iter();
///
///     let bank_pda = next_account_info(account_info_iter)?;
///     let bank_pda_bump_seed = instr.bank_pda_bump_seed;
///     let system_account = next_account_info(account_info_iter)?;
///
///     assert!(system_program::check_id(system_account.key));
///
///     let to_accounts = next_account_infos(account_info_iter, account_info_iter.len())?;
///
///     for to_account in to_accounts {
///          assert!(to_account.is_writable);
///          // ... do other verification ...
///     }
///
///     let to_and_amount = to_accounts
///         .iter()
///         .zip(instr.amount_list.iter())
///         .map(|(to, amount)| (*to.key, *amount))
///         .collect::<Vec<(Pubkey, u64)>>();
///
///     let instrs = system_instruction::transfer_many(bank_pda.key, to_and_amount.as_ref());
///
///     for instr in instrs {
///         invoke_signed(&instr, accounts, &[&[b"bank", &[bank_pda_bump_seed]]])?;
///     }
///
///     Ok(())
/// }
///
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn transfer_many(from_pubkey: &Pubkey, to_lamports: &[(Pubkey, u64)]) -> Vec<Instruction> {
    to_lamports
        .iter()
        .map(|(to_pubkey, lamports)| transfer(from_pubkey, to_pubkey, *lamports))
        .collect()
}
