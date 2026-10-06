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
    hashv(&[
        b"leaf",
        owner.as_ref(),
        name.as_bytes(),
        &[0],
        uri.as_bytes(),
    ])
    .to_bytes()
}

pub fn hash_pair(left: &Hash, right: &Hash) -> Hash {
    hashv(&[b"node", left, right]).to_bytes()
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
        MerkleTree {
            depth,
            leaves: vec![EMPTY; 1 << depth],
        }
    }

    pub fn set(&mut self, index: usize, leaf: Hash) {
        self.leaves[index] = leaf;
    }

    /// The levels from the leaves (level 0) up to the root (level `depth`).
    fn levels(&self) -> Vec<Vec<Hash>> {
        let mut levels = vec![self.leaves.clone()];
        for _ in 0..self.depth {
            let below = levels.last().expect("a level");
            let above = below
                .chunks(2)
                .map(|pair| hash_pair(&pair[0], &pair[1]))
                .collect();
            levels.push(above);
        }
        levels
    }

    pub fn root(&self) -> Hash {
        self.levels().last().expect("a level")[0]
    }

    /// The sibling hashes from the leaf up: `depth` of them.
    pub fn proof(&self, index: usize) -> Vec<Hash> {
        let levels = self.levels();
        let mut i = index;
        let mut proof = Vec::with_capacity(self.depth as usize);
        for level in &levels[..self.depth as usize] {
            proof.push(level[i ^ 1]);
            i /= 2;
        }
        proof
    }
}

/// The root a leaf at `index` with this `proof` leads to.
pub fn root_from_proof(leaf: &Hash, index: usize, proof: &[Hash]) -> Hash {
    let mut node = *leaf;
    let mut i = index;
    for sibling in proof {
        node = if i.is_multiple_of(2) {
            hash_pair(&node, sibling)
        } else {
            hash_pair(sibling, &node)
        };
        i /= 2;
    }
    node
}

pub fn verify(root: &Hash, leaf: &Hash, index: usize, proof: &[Hash]) -> bool {
    root_from_proof(leaf, index, proof) == *root
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
    verify(root, old_leaf, index, proof).then(|| root_from_proof(new_leaf, index, proof))
}
