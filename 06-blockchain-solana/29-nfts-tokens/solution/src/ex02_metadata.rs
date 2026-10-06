//! Exercise 2: NFTs and their metadata.
//!
//! An NFT on Solana is an ordinary SPL mint with 0 decimals, a supply of 1
//! and no mint authority (so there can never be a second). What makes it a
//! picture with a name is a *metadata* account next to it: a PDA
//! `["metadata", mint]` of a metadata program, holding the name, symbol, a
//! URI to the JSON/image, royalties and creators, and the collection it
//! belongs to. Metaplex's Token Metadata program is the standard one; this is
//! a small program with the same ideas:
//!
//! - only the **mint authority** can create a mint's metadata (that's what
//!   proves you control the token)
//! - **creators** are listed with royalty shares, and are only `verified`
//!   once they have signed
//! - membership in a **collection** is only `verified` once the collection's
//!   update authority has signed -- otherwise anyone could claim their NFT
//!   is part of a famous collection

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::program_option::COption;
use solana_program::pubkey::Pubkey;

use crate::util::{create_pda_account, mint, require_pda, require_signer};

solana_program::declare_id!("Ew4s344r9Z7ZPpU3hSySNVUPCGHiLGSaDNpYmJUb1rPL");

pub const MAX_NAME_LEN: usize = 32;
pub const MAX_SYMBOL_LEN: usize = 10;
pub const MAX_URI_LEN: usize = 200;
pub const MAX_CREATORS: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[repr(u32)]
pub enum MetadataError {
    #[error("name, symbol or URI too long")]
    TooLong = 0,
    #[error("royalties are at most 10000 basis points")]
    BadRoyalty = 1,
    #[error("at most 5 creators, with shares adding up to 100")]
    BadCreators = 2,
    #[error("the signer isn't the mint authority")]
    NotMintAuthority = 3,
    #[error("the signer isn't the update authority")]
    NotUpdateAuthority = 4,
    #[error("the metadata is immutable")]
    Immutable = 5,
    #[error("the signer isn't one of the creators")]
    NotACreator = 6,
    #[error("that's not this NFT's collection")]
    WrongCollection = 7,
}

impl From<MetadataError> for ProgramError {
    fn from(e: MetadataError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Creator {
    pub address: Pubkey,
    /// Percent of the royalties.
    pub share: u8,
    pub verified: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Collection {
    /// The collection NFT's mint.
    pub key: Pubkey,
    pub verified: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    pub mint: Pubkey,
    pub update_authority: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    /// Royalty on secondary sales, in basis points (500 = 5%).
    pub seller_fee_basis_points: u16,
    pub creators: Vec<Creator>,
    pub collection: Option<Collection>,
    pub is_mutable: bool,
    pub bump: u8,
}

impl Metadata {
    pub const SPACE: usize = 32
        + 32
        + (4 + MAX_NAME_LEN)
        + (4 + MAX_SYMBOL_LEN)
        + (4 + MAX_URI_LEN)
        + 2
        + (4 + MAX_CREATORS * (32 + 1 + 1))
        + (1 + 32 + 1)
        + 1
        + 1;

    pub fn read(data: &[u8]) -> Result<Self, ProgramError> {
        Self::deserialize(&mut &data[..]).map_err(|_| ProgramError::InvalidAccountData)
    }
}

/// What `Create` takes; `verified` is decided by the program, never the caller.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct CreateArgs {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Vec<(Pubkey, u8)>,
    pub collection: Option<Pubkey>,
    pub is_mutable: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum MetadataInstruction {
    /// `[payer (s, w), mint, mint authority (s), metadata (w), system program]`
    /// The update authority is the mint authority.
    Create(CreateArgs),
    /// `[update authority (s), metadata (w)]`
    Update {
        name: Option<String>,
        uri: Option<String>,
        new_update_authority: Option<Pubkey>,
        is_mutable: Option<bool>,
    },
    /// `[creator (s), metadata (w)]`
    VerifyCreator,
    /// `[collection update authority (s), metadata (w), collection metadata]`
    VerifyCollection,
}

// ------------------------------------------------------------------ client

pub fn metadata_address(mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"metadata", mint.as_ref()], &ID).0
}

fn build(accounts: Vec<AccountMeta>, data: MetadataInstruction) -> Instruction {
    Instruction {
        program_id: ID,
        accounts,
        data: borsh::to_vec(&data).expect("serialize"),
    }
}

pub fn create(
    payer: &Pubkey,
    mint: &Pubkey,
    mint_authority: &Pubkey,
    args: CreateArgs,
) -> Instruction {
    build(
        vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(*mint_authority, true),
            AccountMeta::new(metadata_address(mint), false),
            AccountMeta::new_readonly(solana_system_interface::program::ID, false),
        ],
        MetadataInstruction::Create(args),
    )
}

pub fn update(
    update_authority: &Pubkey,
    mint: &Pubkey,
    name: Option<String>,
    uri: Option<String>,
    new_update_authority: Option<Pubkey>,
    is_mutable: Option<bool>,
) -> Instruction {
    build(
        vec![
            AccountMeta::new_readonly(*update_authority, true),
            AccountMeta::new(metadata_address(mint), false),
        ],
        MetadataInstruction::Update {
            name,
            uri,
            new_update_authority,
            is_mutable,
        },
    )
}

pub fn verify_creator(creator: &Pubkey, mint: &Pubkey) -> Instruction {
    build(
        vec![
            AccountMeta::new_readonly(*creator, true),
            AccountMeta::new(metadata_address(mint), false),
        ],
        MetadataInstruction::VerifyCreator,
    )
}

pub fn verify_collection(
    collection_authority: &Pubkey,
    mint: &Pubkey,
    collection_mint: &Pubkey,
) -> Instruction {
    build(
        vec![
            AccountMeta::new_readonly(*collection_authority, true),
            AccountMeta::new(metadata_address(mint), false),
            AccountMeta::new_readonly(metadata_address(collection_mint), false),
        ],
        MetadataInstruction::VerifyCollection,
    )
}

/// How a sale price is split among creators, by share (rounding down; the
/// remainder stays with the seller). Royalty = price * fee_bps / 10000.
pub fn royalty_split(price: u64, metadata: &Metadata) -> Vec<(Pubkey, u64)> {
    let royalty = price as u128 * metadata.seller_fee_basis_points as u128 / 10_000;
    metadata
        .creators
        .iter()
        .map(|c| (c.address, (royalty * c.share as u128 / 100) as u64))
        .collect()
}

// ----------------------------------------------------------------- program

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    match MetadataInstruction::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?
    {
        MetadataInstruction::Create(args) => create_metadata(program_id, accounts, args),
        MetadataInstruction::Update {
            name,
            uri,
            new_update_authority,
            is_mutable,
        } => update_metadata(
            program_id,
            accounts,
            name,
            uri,
            new_update_authority,
            is_mutable,
        ),
        MetadataInstruction::VerifyCreator => verify_a_creator(program_id, accounts),
        MetadataInstruction::VerifyCollection => verify_membership(program_id, accounts),
    }
}

fn check_lengths(name: &str, symbol: &str, uri: &str) -> ProgramResult {
    if name.len() > MAX_NAME_LEN || symbol.len() > MAX_SYMBOL_LEN || uri.len() > MAX_URI_LEN {
        return Err(MetadataError::TooLong.into());
    }
    Ok(())
}

fn save(state: &Metadata, info: &AccountInfo) -> ProgramResult {
    let bytes = borsh::to_vec(state).map_err(|_| ProgramError::InvalidAccountData)?;
    let mut data = info.try_borrow_mut_data()?;
    data[..bytes.len()].copy_from_slice(&bytes);
    data[bytes.len()..].fill(0);
    Ok(())
}

/// Load metadata, checking it's ours and at the PDA of its mint.
pub(crate) fn load(program_id: &Pubkey, info: &AccountInfo) -> Result<Metadata, ProgramError> {
    if info.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let state = Metadata::read(&info.try_borrow_data()?)?;
    let expected = Pubkey::create_program_address(
        &[b"metadata", state.mint.as_ref(), &[state.bump]],
        program_id,
    )?;
    if *info.key != expected {
        return Err(ProgramError::InvalidSeeds);
    }
    Ok(state)
}

fn create_metadata(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: CreateArgs,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let payer = next_account_info(iter)?;
    let mint_info = next_account_info(iter)?;
    let mint_authority = next_account_info(iter)?;
    let metadata = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;

    require_signer(payer)?;
    require_signer(mint_authority)?;
    if mint(mint_info)?.mint_authority != COption::Some(*mint_authority.key) {
        return Err(MetadataError::NotMintAuthority.into());
    }
    check_lengths(&args.name, &args.symbol, &args.uri)?;
    if args.seller_fee_basis_points > 10_000 {
        return Err(MetadataError::BadRoyalty.into());
    }
    let total: u32 = args.creators.iter().map(|(_, s)| *s as u32).sum();
    if args.creators.len() > MAX_CREATORS || (!args.creators.is_empty() && total != 100) {
        return Err(MetadataError::BadCreators.into());
    }
    let bump = require_pda(metadata, &[b"metadata", mint_info.key.as_ref()], program_id)?;
    create_pda_account(
        payer,
        metadata,
        system_program,
        Metadata::SPACE,
        program_id,
        &[b"metadata", mint_info.key.as_ref(), &[bump]],
    )?;
    let state = Metadata {
        mint: *mint_info.key,
        update_authority: *mint_authority.key,
        name: args.name,
        symbol: args.symbol,
        uri: args.uri,
        seller_fee_basis_points: args.seller_fee_basis_points,
        // A creator is verified only if they signed this very instruction.
        creators: args
            .creators
            .into_iter()
            .map(|(address, share)| Creator {
                address,
                share,
                verified: address == *mint_authority.key,
            })
            .collect(),
        collection: args.collection.map(|key| Collection {
            key,
            verified: false,
        }),
        is_mutable: args.is_mutable,
        bump,
    };
    save(&state, metadata)
}

fn update_metadata(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: Option<String>,
    uri: Option<String>,
    new_update_authority: Option<Pubkey>,
    is_mutable: Option<bool>,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let authority = next_account_info(iter)?;
    let metadata = next_account_info(iter)?;
    require_signer(authority)?;
    let mut state = load(program_id, metadata)?;
    if state.update_authority != *authority.key {
        return Err(MetadataError::NotUpdateAuthority.into());
    }
    if !state.is_mutable {
        return Err(MetadataError::Immutable.into());
    }
    if let Some(name) = name {
        state.name = name;
    }
    if let Some(uri) = uri {
        state.uri = uri;
    }
    check_lengths(&state.name, &state.symbol, &state.uri)?;
    if let Some(new) = new_update_authority {
        state.update_authority = new;
    }
    if let Some(false) = is_mutable {
        state.is_mutable = false; // one way only
    }
    save(&state, metadata)
}

fn verify_a_creator(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let creator = next_account_info(iter)?;
    let metadata = next_account_info(iter)?;
    require_signer(creator)?;
    let mut state = load(program_id, metadata)?;
    let entry = state
        .creators
        .iter_mut()
        .find(|c| c.address == *creator.key)
        .ok_or(MetadataError::NotACreator)?;
    entry.verified = true;
    save(&state, metadata)
}

fn verify_membership(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let authority = next_account_info(iter)?;
    let metadata = next_account_info(iter)?;
    let collection_metadata = next_account_info(iter)?;
    require_signer(authority)?;
    let mut state = load(program_id, metadata)?;
    let collection = load(program_id, collection_metadata)?;
    match &mut state.collection {
        Some(c) if c.key == collection.mint => {
            if collection.update_authority != *authority.key {
                return Err(MetadataError::NotUpdateAuthority.into());
            }
            c.verified = true;
        }
        _ => return Err(MetadataError::WrongCollection.into()),
    }
    save(&state, metadata)
}

// -------------------------------------------------------- client: an NFT

/// Mint a 1-of-1 NFT to `owner`: a 0-decimal mint, one token in the owner's
/// ATA, metadata, then the mint authority removed. `creator` pays and signs
/// (and is the verified creator and update authority). Returns the mint.
#[cfg(not(target_os = "solana"))]
pub fn mint_nft(
    sim: &mut solsim::Sim,
    creator: &Pubkey,
    owner: &Pubkey,
    name: &str,
    uri: &str,
    collection: Option<Pubkey>,
) -> Result<Pubkey, solsim::TxError> {
    use solana_program::program_pack::Pack;
    use solsim::ata::{create_associated_token_account, get_associated_token_address};
    use spl_token_interface::instruction as token_ix;

    let mint = Pubkey::new_unique();
    let lamports = sim.minimum_balance(spl_token_interface::state::Mint::LEN);
    let args = CreateArgs {
        name: name.into(),
        symbol: "RUST".into(),
        uri: uri.into(),
        seller_fee_basis_points: 500,
        creators: vec![(*creator, 100)],
        collection,
        is_mutable: true,
    };
    let ata = get_associated_token_address(owner, &mint);
    let ixs = [
        solana_system_interface::instruction::create_account(
            creator,
            &mint,
            lamports,
            spl_token_interface::state::Mint::LEN as u64,
            &spl_token_interface::ID,
        ),
        token_ix::initialize_mint2(&spl_token_interface::ID, &mint, creator, None, 0)
            .expect("valid"),
        create_associated_token_account(creator, owner, &mint),
        token_ix::mint_to(&spl_token_interface::ID, &mint, &ata, creator, &[], 1).expect("valid"),
        create(creator, &mint, creator, args),
        token_ix::set_authority(
            &spl_token_interface::ID,
            &mint,
            None,
            token_ix::AuthorityType::MintTokens,
            creator,
            &[],
        )
        .expect("valid"),
    ];
    sim.process(&ixs, &[*creator, mint])?;
    Ok(mint)
}
