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

    /// Available credits. Uses saturating subtraction to prevent underflow.
    pub fn available(&self) -> u64 {
        self.amount.saturating_sub(self.locked)
    }

    /// Lock credits. Returns error if insufficient available credits.
    pub fn try_lock(&mut self, amount: u64) -> anyhow::Result<()> {
        if self.available() < amount {
            anyhow::bail!(
                "Insufficient available credits: have {}, need {}",
                self.available(),
                amount
            );
        }
        self.locked = self.locked.saturating_add(amount);
        Ok(())
    }

    /// Lock credits (panics on underflow - use try_lock for safe version).
    #[deprecated(note = "Use try_lock() for safe arithmetic")]
    pub fn lock(&mut self, amount: u64) {
        self.locked = self.locked.saturating_add(amount);
    }

    /// Unlock credits. Returns error if trying to unlock more than locked.
    pub fn try_unlock(&mut self, amount: u64) -> anyhow::Result<()> {
        if amount > self.locked {
            anyhow::bail!(
                "Cannot unlock more than locked: locked={}, requested={}",
                self.locked,
                amount
            );
        }
        self.locked -= amount;
        Ok(())
    }

    /// Unlock credits (panics on underflow - use try_unlock for safe version).
    #[deprecated(note = "Use try_unlock() for safe arithmetic")]
    pub fn unlock(&mut self, amount: u64) {
        self.locked = self.locked.saturating_sub(amount);
    }

    /// Credit (add) credits.
    pub fn credit(&mut self, amount: u64) {
        self.amount = self.amount.saturating_add(amount);
    }

    /// Debit (subtract) credits. Returns error if insufficient available.
    pub fn debit(&mut self, amount: u64) -> anyhow::Result<()> {
        if self.available() < amount {
            anyhow::bail!(
                "Insufficient credits: available={}, requested={}",
                self.available(),
                amount
            );
        }
        self.amount -= amount;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_available_saturating() {
        let mut balance = CreditBalance::new(100);
        balance.amount = 50;
        balance.locked = 100;
        // Should saturate to 0, not underflow
        assert_eq!(balance.available(), 0);
    }

    #[test]
    fn test_try_lock_insufficient() {
        let mut balance = CreditBalance::new(100);
        let result = balance.try_lock(150);
        assert!(result.is_err());
        assert_eq!(balance.locked, 0); // Should not change
    }

    #[test]
    fn test_try_unlock_exceeds_locked() {
        let mut balance = CreditBalance::new(100);
        balance.locked = 50;
        let result = balance.try_unlock(100);
        assert!(result.is_err());
        assert_eq!(balance.locked, 50); // Should not change
    }

    #[test]
    fn test_credit_saturating() {
        let mut balance = CreditBalance::new(u64::MAX - 10);
        balance.credit(20);
        // Should saturate at u64::MAX, not wrap
        assert_eq!(balance.amount, u64::MAX);
    }

    #[test]
    fn test_debit_insufficient() {
        let mut balance = CreditBalance::new(100);
        let result = balance.debit(150);
        assert!(result.is_err());
        assert_eq!(balance.amount, 100); // Should not change
    }
}
