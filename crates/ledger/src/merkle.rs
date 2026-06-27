use crate::{Block, BlockHash};
use anyhow::{bail, Result};
use std::collections::HashMap;

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

    pub fn height(&self) -> usize {
        match self.head {
            None => 0,
            Some(head) => self.height_from(head),
        }
    }

    fn height_from(&self, hash: BlockHash) -> usize {
        match self.blocks.get(&hash) {
            None => 0,
            Some(block) => {
                if block.parent_hash == [0u8; 32] {
                    1
                } else {
                    1 + self.height_from(block.parent_hash)
                }
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

    pub fn validate_chain(&self, head: BlockHash) -> Result<()> {
        let mut current = Some(head);
        while let Some(hash) = current {
            let block = self
                .blocks
                .get(&hash)
                .ok_or_else(|| anyhow::anyhow!("Block not found: {:?}", hash))?;

            let computed =
                Block::compute_hash(block.parent_hash, &block.data, block.nonce, block.timestamp);
            if computed != block.hash {
                bail!("Invalid hash for block");
            }

            if block.parent_hash == [0u8; 32] {
                break;
            }
            current = Some(block.parent_hash);
        }
        Ok(())
    }

    pub fn get_block(&self, hash: &BlockHash) -> Option<&Block> {
        self.blocks.get(hash)
    }
}
