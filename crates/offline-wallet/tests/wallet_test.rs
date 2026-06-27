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

    balance.lock(100);
    assert_eq!(balance.available(), 400);
    assert_eq!(balance.locked, 100);

    balance.unlock(50);
    assert_eq!(balance.available(), 450);
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
