use swarm_offline_wallet::{CreditBalance, CreditCommitmentToken, OfflineWallet};

#[test]
fn test_wallet_creation() {
    let wallet = OfflineWallet::new(1000);
    assert_eq!(wallet.balance().amount, 1000);
    assert_eq!(wallet.balance().available(), 1000);
}

#[test]
fn test_balance_operations() {
    let mut balance = CreditBalance::new(500);
    assert_eq!(balance.available(), 500);

    balance.try_lock(100).unwrap();
    assert_eq!(balance.available(), 400);
    assert_eq!(balance.locked, 100);

    balance.try_unlock(50).unwrap();
    assert_eq!(balance.available(), 450);
    assert_eq!(balance.locked, 50);
}

#[test]
fn test_balance_underflow_protection() {
    let mut balance = CreditBalance::new(100);
    balance.amount = 50;
    balance.locked = 100;
    // available() should saturate to 0, not underflow
    assert_eq!(balance.available(), 0);
}

#[test]
fn test_try_lock_insufficient() {
    let mut balance = CreditBalance::new(100);
    let result = balance.try_lock(150);
    assert!(result.is_err());
    assert_eq!(balance.locked, 0);
}

#[test]
fn test_try_unlock_exceeds_locked() {
    let mut balance = CreditBalance::new(100);
    balance.locked = 50;
    let result = balance.try_unlock(100);
    assert!(result.is_err());
    assert_eq!(balance.locked, 50);
}

#[test]
fn test_cct_creation() {
    let wallet = OfflineWallet::new(1000);
    let cct = wallet.create_cct(100).unwrap();
    assert_eq!(cct.amount, 100);
    assert!(cct.is_valid());
}

#[test]
fn test_cct_expiry() {
    let cct = CreditCommitmentToken {
        id: uuid::Uuid::new_v4(),
        amount: 100,
        created_at: chrono::Utc::now() - chrono::Duration::minutes(10),
        expires_at: chrono::Utc::now() - chrono::Duration::minutes(5),
        signature: vec![],
    };
    assert!(!cct.is_valid());
}

#[test]
fn test_wallet_insufficient_credits() {
    let wallet = OfflineWallet::new(50);
    let result = wallet.create_cct(100);
    assert!(result.is_err());
}

#[test]
fn test_wallet_redeem_cct() {
    let mut wallet = OfflineWallet::new(1000);
    let cct = wallet.create_cct(200).unwrap();
    let result = wallet.redeem_cct(&cct);
    assert!(result.is_ok());
    assert_eq!(wallet.balance().amount, 1200);
}

#[test]
fn test_cct_replay_prevention() {
    let mut wallet = OfflineWallet::new(1000);
    let cct = wallet.create_cct(100).unwrap();

    // First redemption should succeed
    wallet.redeem_cct(&cct).unwrap();
    assert_eq!(wallet.balance().amount, 1100);

    // Second redemption should fail (replay attack prevented)
    let result = wallet.redeem_cct(&cct);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already redeemed"));
    assert_eq!(wallet.balance().amount, 1100); // Should not increase
}

#[test]
fn test_wallet_is_cct_redeemed() {
    let mut wallet = OfflineWallet::new(1000);
    let cct = wallet.create_cct(100).unwrap();

    assert!(!wallet.is_cct_redeemed(&cct.id));
    wallet.redeem_cct(&cct).unwrap();
    assert!(wallet.is_cct_redeemed(&cct.id));
}

#[test]
fn test_cct_tied_to_specific_node() {
    let wallet_a = OfflineWallet::new(1000);
    let mut wallet_b = OfflineWallet::new(1000);

    let cct = wallet_a.create_cct(100).unwrap();

    // CCT from wallet_a should NOT be redeemable by wallet_b (different secret key)
    let result = wallet_b.redeem_cct(&cct);
    assert!(result.is_err());
}
