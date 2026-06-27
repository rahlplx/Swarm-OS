# Phase 1: Gossip Mesh Migration — Task Breakdown

## T1: Membership Crate (libp2p gossip mesh)
- [ ] T1.1: Cargo.toml with libp2p deps
- [ ] T1.2: NodeId type (peer ID wrapper)
- [ ] T1.3: MembershipService trait
- [ ] T1.4: GossipMesh struct (libp2p Gossipsub)
- [ ] T1.5: Join/Leave/Heartbeat
- [ ] T1.6: Integration tests

## T2: Ledger Crate (Merkle-DAG)
- [ ] T2.1: Cargo.toml with sha2, argon2 deps
- [ ] T2.2: Block struct (parent_hash, data, nonce, timestamp)
- [ ] T2.3: MerkleDAG struct (chain validation)
- [ ] T2.4: Argon2id PoW (t=10, m=256MB)
- [ ] T2.5: Append/Validate/GetBlock
- [ ] T2.6: Integration tests

## T3: Offline Wallet Crate (CCTs)
- [ ] T3.1: Cargo.toml with chrono, uuid deps
- [ ] T3.2: CreditBalance struct
- [ ] T3.3: CCT struct (commitment token, 5-min validity)
- [ ] T3.4: OfflineWallet (sign/verify/redeem)
- [ ] T3.5: Integration tests

## Build Order
T1 → T2 → T3 (each crate is independent, can parallelize tests)
