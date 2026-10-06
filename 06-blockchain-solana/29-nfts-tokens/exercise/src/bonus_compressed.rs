//! Bonus: compressed NFTs -- ownership as a Merkle tree.
//!
//! A million NFTs as accounts would cost a million rent deposits. Compressed
//! NFTs (Bubblegum) store only a Merkle *root* on-chain; each NFT is a leaf
//! (a hash of its owner and metadata) kept off-chain by indexers. To transfer
//! one, the owner submits the leaf's proof -- the sibling hashes on the path
//! to the root -- and the program checks it against the stored root, then
//! stores the new root with the leaf replaced. Everything here is plain
//! hashing; the on-chain part is `replace_leaf`.

use solana_program::hash::hashv;
use solana_program::pubkey::Pubkey;

pub type Hash = [u8; 32];

/// The empty leaf.
pub const EMPTY: Hash = [0; 32];

/// A leaf: who owns it, and what it is. The prefixes keep a leaf from ever
/// being mistaken for an inner node.
pub fn leaf_hash(owner: &Pubkey, name: &str, uri: &str) -> Hash {
    todo!("Bonus")
}

pub fn hash_pair(left: &Hash, right: &Hash) -> Hash {
    todo!("Bonus")
}

/// The full tree, as an indexer keeps it off-chain.
#[derive(Debug, Clone)]
pub struct MerkleTree {
    depth: u32,
    leaves: Vec<Hash>,
}

impl MerkleTree {
    /// `2^depth` empty leaves.
    pub fn new(depth: u32) -> Self {
        todo!("Bonus")
    }

    pub fn set(&mut self, index: usize, leaf: Hash) {
        todo!("Bonus")
    }

    /// The levels from the leaves (level 0) up to the root (level `depth`).
    fn levels(&self) -> Vec<Vec<Hash>> {
        todo!("Bonus")
    }

    pub fn root(&self) -> Hash {
        todo!("Bonus")
    }

    /// The sibling hashes from the leaf up: `depth` of them.
    pub fn proof(&self, index: usize) -> Vec<Hash> {
        todo!("Bonus")
    }
}

/// The root a leaf at `index` with this `proof` leads to.
pub fn root_from_proof(leaf: &Hash, index: usize, proof: &[Hash]) -> Hash {
    todo!("Bonus")
}

pub fn verify(root: &Hash, leaf: &Hash, index: usize, proof: &[Hash]) -> bool {
    todo!("Bonus")
}

/// What the on-chain program does for a transfer: if `old_leaf` is really at
/// `index` under `root`, return the root with `new_leaf` there instead.
pub fn replace_leaf(
    root: &Hash,
    old_leaf: &Hash,
    new_leaf: &Hash,
    index: usize,
    proof: &[Hash],
) -> Option<Hash> {
    todo!("Bonus")
}
