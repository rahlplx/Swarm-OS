use crate::{Block, BlockHash};
use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};

impl Default for MerkleDAG {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MerkleDAG {
    blocks: HashMap<BlockHash, Block>,
    head: Option<BlockHash>,
}

impl MerkleDAG {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            head: None,
        }
    }

    /// Iterative height calculation with cycle detection.
    /// O(N) time, O(N) space for visited set.
    pub fn height(&self) -> usize {
        match self.head {
            None => 0,
            Some(head) => {
                let mut count = 0;
                let mut current = Some(head);
                let mut visited = HashSet::new();

                while let Some(hash) = current {
                    if !visited.insert(hash) {
                        // Cycle detected - break to prevent infinite loop
                        tracing::error!("Cycle detected in block chain at {:?}", hash);
                        break;
                    }

                    match self.blocks.get(&hash) {
                        None => break,
                        Some(block) => {
                            count += 1;
                            if block.parent_hash == [0u8; 32] {
                                break; // Genesis block reached
                            }
                            current = Some(block.parent_hash);
                        }
                    }
                }
                count
            }
        }
    }

    pub fn genesis_hash(&self) -> Option<&BlockHash> {
        self.head.as_ref()
    }

    pub fn append_genesis(&mut self, data: Vec<u8>) -> BlockHash {
        let block = Block::new([0u8; 32], data);
        let hash = block.hash;
        self.blocks.insert(hash, block);
        self.head = Some(hash);
        hash
    }

    pub fn append(&mut self, parent_hash: BlockHash, data: Vec<u8>) -> BlockHash {
        let block = Block::new(parent_hash, data);
        let hash = block.hash;
        self.blocks.insert(hash, block);
        self.head = Some(hash);
        hash
    }

    /// Validate chain with cycle detection.
    /// Returns Err if: block not found, hash mismatch, or cycle detected.
    pub fn validate_chain(&self, head: BlockHash) -> Result<()> {
        let mut current = Some(head);
        let mut visited = HashSet::new();

        while let Some(hash) = current {
            if !visited.insert(hash) {
                bail!("Cycle detected in chain at block {:?}", hash);
            }

            let block = self
                .blocks
                .get(&hash)
                .ok_or_else(|| anyhow::anyhow!("Block not found: {:?}", hash))?;

            let computed =
                Block::compute_hash(block.parent_hash, &block.data, block.nonce, block.timestamp);
            if computed != block.hash {
                bail!("Invalid hash for block {:?}", hash);
            }

            if block.parent_hash == [0u8; 32] {
                break; // Genesis block reached
            }
            current = Some(block.parent_hash);
        }
        Ok(())
    }

    pub fn get_block(&self, hash: &BlockHash) -> Option<&Block> {
        self.blocks.get(hash)
    }

    /// Get all block hashes in chain order (head to genesis).
    pub fn chain_hashes(&self) -> Vec<BlockHash> {
        let mut hashes = Vec::new();
        let mut current = self.head;
        let mut visited = HashSet::new();

        while let Some(hash) = current {
            if !visited.insert(hash) {
                break; // Cycle
            }
            hashes.push(hash);
            if let Some(block) = self.blocks.get(&hash) {
                if block.parent_hash == [0u8; 32] {
                    break;
                }
                current = Some(block.parent_hash);
            } else {
                break;
            }
        }
        hashes
    }
}
