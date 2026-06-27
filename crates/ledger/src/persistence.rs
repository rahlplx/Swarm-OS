use crate::{Block, BlockHash, MerkleDAG};
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;

pub struct MerkleDAGStore {
    conn: Connection,
}

impl MerkleDAGStore {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).context("Failed to open SQLite database")?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;",
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS blocks (
                hash BLOB PRIMARY KEY,
                parent_hash BLOB NOT NULL,
                data BLOB NOT NULL,
                nonce INTEGER NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("Failed to open in-memory DB")?;
        conn.execute_batch(
            "CREATE TABLE blocks (
                hash BLOB PRIMARY KEY,
                parent_hash BLOB NOT NULL,
                data BLOB NOT NULL,
                nonce INTEGER NOT NULL,
                timestamp INTEGER NOT NULL
            )",
        )?;
        Ok(Self { conn })
    }

    pub fn save_block(&self, block: &Block) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO blocks (hash, parent_hash, data, nonce, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                block.hash.as_ref(),
                block.parent_hash.as_ref(),
                block.data,
                block.nonce as i64,
                block.timestamp,
            ],
        )?;
        Ok(())
    }

    pub fn load_block(&self, hash: &BlockHash) -> Result<Option<Block>> {
        let mut stmt = self
            .conn
            .prepare("SELECT parent_hash, data, nonce, timestamp FROM blocks WHERE hash = ?1")?;
        let mut rows = stmt.query_map(rusqlite::params![hash.as_ref()], |row| {
            let parent_hash: Vec<u8> = row.get(0)?;
            let data: Vec<u8> = row.get(1)?;
            let nonce: i64 = row.get(2)?;
            let timestamp: i64 = row.get(3)?;
            Ok((parent_hash, data, nonce, timestamp))
        })?;
        match rows.next() {
            Some(Ok((parent_hash, data, nonce, timestamp))) => {
                let mut ph = [0u8; 32];
                ph.copy_from_slice(&parent_hash);
                Ok(Some(Block {
                    parent_hash: ph,
                    data,
                    nonce: nonce as u64,
                    timestamp,
                    hash: *hash,
                }))
            }
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn load_all(&self) -> Result<Vec<Block>> {
        let mut stmt = self
            .conn
            .prepare("SELECT hash, parent_hash, data, nonce, timestamp FROM blocks")?;
        let rows = stmt.query_map([], |row| {
            let hash: Vec<u8> = row.get(0)?;
            let parent_hash: Vec<u8> = row.get(1)?;
            let data: Vec<u8> = row.get(2)?;
            let nonce: i64 = row.get(3)?;
            let timestamp: i64 = row.get(4)?;
            let mut h = [0u8; 32];
            h.copy_from_slice(&hash);
            let mut ph = [0u8; 32];
            ph.copy_from_slice(&parent_hash);
            Ok(Block {
                parent_hash: ph,
                data,
                nonce: nonce as u64,
                timestamp,
                hash: h,
            })
        })?;
        let mut blocks = Vec::new();
        for row in rows {
            blocks.push(row?);
        }
        Ok(blocks)
    }

    pub fn save_dag(&self, dag: &MerkleDAG) -> Result<()> {
        let hashes = dag.chain_hashes();
        for hash in hashes.iter().rev() {
            if let Some(block) = dag.get_block(hash) {
                self.save_block(block)?;
            }
        }
        Ok(())
    }

    pub fn load_dag(&self) -> Result<MerkleDAG> {
        let blocks = self.load_all()?;
        let mut dag = MerkleDAG::new();
        // Find genesis block (parent_hash == [0; 32])
        let genesis = blocks.iter().find(|b| b.parent_hash == [0u8; 32]);
        if let Some(gen) = genesis {
            dag.append_genesis(gen.data.clone());
            // Load remaining blocks in order
            let mut remaining: Vec<&Block> = blocks
                .iter()
                .filter(|b| b.parent_hash != [0u8; 32])
                .collect();
            remaining.sort_by_key(|b| b.timestamp);
            for block in remaining {
                dag.append(block.parent_hash, block.data.clone());
            }
        }
        Ok(dag)
    }

    pub fn block_count(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM blocks", [], |row| row.get(0))?;
        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_save_and_load_block() {
        let store = MerkleDAGStore::in_memory().unwrap();
        let block = Block::new([0u8; 32], b"test".to_vec());
        store.save_block(&block).unwrap();

        let loaded = store.load_block(&block.hash).unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.hash, block.hash);
        assert_eq!(loaded.data, b"test");
    }

    #[test]
    fn test_load_nonexistent_block() {
        let store = MerkleDAGStore::in_memory().unwrap();
        let result = store.load_block(&[99u8; 32]).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_save_and_load_dag() {
        let store = MerkleDAGStore::in_memory().unwrap();
        let mut dag = MerkleDAG::new();
        let h1 = dag.append_genesis(b"block 1".to_vec());
        dag.append(h1, b"block 2".to_vec());

        store.save_dag(&dag).unwrap();
        assert_eq!(store.block_count().unwrap(), 2);

        let loaded_dag = store.load_dag().unwrap();
        assert_eq!(loaded_dag.height(), 2);
    }

    #[test]
    fn test_block_count() {
        let store = MerkleDAGStore::in_memory().unwrap();
        assert_eq!(store.block_count().unwrap(), 0);

        let block = Block::new([0u8; 32], b"test".to_vec());
        store.save_block(&block).unwrap();
        assert_eq!(store.block_count().unwrap(), 1);
    }

    #[test]
    fn test_persistence_to_file() {
        let dir = std::env::temp_dir().join("swarm_test_db");
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("test.db");

        // Save
        {
            let store = MerkleDAGStore::open(&db_path).unwrap();
            let mut dag = MerkleDAG::new();
            let h1 = dag.append_genesis(b"persist me".to_vec());
            dag.append(h1, b"second block".to_vec());
            store.save_dag(&dag).unwrap();
        }

        // Load
        {
            let store = MerkleDAGStore::open(&db_path).unwrap();
            let dag = store.load_dag().unwrap();
            assert_eq!(dag.height(), 2);
        }

        std::fs::remove_dir_all(&dir).ok();
    }
}
