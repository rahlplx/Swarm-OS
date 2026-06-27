use crate::{CreditBalance, CreditCommitmentToken};
use anyhow::{bail, Result};

pub struct OfflineWallet {
    balance: CreditBalance,
    secret_key: [u8; 32],
}

impl OfflineWallet {
    pub fn new(initial_credits: u64) -> Self {
        let mut secret_key = [0u8; 32];
        // In production, this would be derived from a secure source
        getrandom::getrandom(&mut secret_key).expect("Failed to generate random key");

        Self {
            balance: CreditBalance::new(initial_credits),
            secret_key,
        }
    }

    pub fn balance(&self) -> &CreditBalance {
        &self.balance
    }

    pub fn create_cct(&self, amount: u64) -> Result<CreditCommitmentToken> {
        if self.balance.available() < amount {
            bail!(
                "Insufficient credits: available {}, requested {}",
                self.balance.available(),
                amount
            );
        }
        Ok(CreditCommitmentToken::new(amount, &self.secret_key))
    }

    pub fn redeem_cct(&mut self, cct: &CreditCommitmentToken) -> Result<()> {
        if !cct.is_valid() {
            bail!("CCT has expired");
        }
        if !cct.verify(&self.secret_key) {
            bail!("Invalid CCT signature");
        }
        self.balance.credit(cct.amount);
        Ok(())
    }
}
