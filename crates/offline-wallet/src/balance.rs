use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditBalance {
    pub amount: u64,
    pub locked: u64,
}

impl CreditBalance {
    pub fn new(initial: u64) -> Self {
        Self {
            amount: initial,
            locked: 0,
        }
    }

    pub fn available(&self) -> u64 {
        self.amount - self.locked
    }

    pub fn lock(&mut self, amount: u64) {
        self.locked += amount;
    }

    pub fn unlock(&mut self, amount: u64) {
        self.locked -= amount;
    }

    pub fn credit(&mut self, amount: u64) {
        self.amount += amount;
    }

    pub fn debit(&mut self, amount: u64) -> anyhow::Result<()> {
        if self.available() < amount {
            anyhow::bail!("Insufficient credits");
        }
        self.amount -= amount;
        Ok(())
    }
}
