//! Each test deploys the exact v0.2.0 WASM running on testnet, writes state
//! through it, upgrades in place to the current release build, and checks that
//! the state survives and the current rules apply.

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger},
    Address, BytesN, Env, Symbol,
};

mod kyb_v2 {
    soroban_sdk::contractimport!(
        file = "fixtures/v0.2.0/kyb_attestation_registry.wasm",
        sha256 = "6c06e0ecafe9616903f0ceef2a8ada208aafb26404ace1deb39ee89720218747"
    );
}
mod kyb_current {
    soroban_sdk::contractimport!(
        file = "../target/wasm32v1-none/release/kyb_attestation_registry.wasm"
    );
}
mod ins_v2 {
    soroban_sdk::contractimport!(
        file = "fixtures/v0.2.0/insurance_record.wasm",
        sha256 = "ffeae58c889f2835cc9716e81724c229703584c26446f18862979a9669563452"
    );
}
mod ins_current {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/insurance_record.wasm");
}

const NOW: u64 = 1_800_000_000;
const LATER: u64 = NOW + 365 * 24 * 60 * 60;

fn env() -> Env {
    let env = Env::default();
    env.ledger().set_timestamp(NOW);
    env.mock_all_auths();
    env
}

/// Imported clients surface contract errors as `soroban_sdk::Error`.
fn contract_error(code: u32) -> soroban_sdk::Error {
    soroban_sdk::Error::from_contract_error(code)
}

fn h32(env: &Env, fill: u8) -> BytesN<32> {
    BytesN::from_array(env, &[fill; 32])
}

#[test]
fn kyb_v02_state_survives_upgrade() {
    let env = env();
    let admin = Address::generate(&env);
    let active = Address::generate(&env);
    let revoked = Address::generate(&env);
    let tier = Symbol::new(&env, "standard");

    let id = env.register(kyb_v2::WASM, (&admin,));
    let v2 = kyb_v2::Client::new(&env, &id);
    v2.write_attestation(&admin, &active, &tier, &LATER);
    v2.write_attestation(&admin, &revoked, &tier, &LATER);
    v2.revoke_attestation(&admin, &revoked);

    let hash = env.deployer().upload_contract_wasm(kyb_current::WASM);
    v2.upgrade(&hash);
    let c = kyb_current::Client::new(&env, &id);

    assert_eq!(c.get_admin(), admin);
    let row = c.get_attestation(&active).unwrap();
    assert_eq!(
        (row.tier, row.expires_at, row.revoked),
        (tier.clone(), LATER, false)
    );
    assert!(c.is_valid(&active));
    assert!(c.get_attestation(&revoked).unwrap().revoked);
    assert!(!c.is_valid(&revoked));

    // Revocations made under v0.2 can no longer be undone by a plain write.
    assert_eq!(
        c.try_write_attestation(&admin, &revoked, &tier, &LATER),
        Err(Ok(contract_error(
            kyb_current::Error::AttestationRevoked as u32
        )))
    );
    c.reinstate_attestation(&admin, &revoked, &tier, &LATER);
    assert!(c.is_valid(&revoked));

    // Admin rotation now works without a further upgrade.
    let next = Address::generate(&env);
    c.propose_admin(&next);
    c.accept_admin();
    assert_eq!(c.get_admin(), next);
    assert_eq!(
        c.try_revoke_attestation(&admin, &active),
        Err(Ok(contract_error(kyb_current::Error::NotAdmin as u32)))
    );

    // The upgraded contract can itself be upgraded again.
    c.upgrade(&hash);
    assert_eq!(c.get_admin(), next);
}

#[test]
fn insurance_v02_state_survives_upgrade() {
    let env = env();
    let admin = Address::generate(&env);
    let (deal, paid_claim, placeholder_claim) = (h32(&env, 1), h32(&env, 9), h32(&env, 10));
    let approved = symbol_short!("approved");

    let id = env.register(ins_v2::WASM, (&admin,));
    let v2 = ins_v2::Client::new(&env, &id);
    v2.record_policy(
        &admin,
        &deal,
        &h32(&env, 2),
        &h32(&env, 3),
        &symbol_short!("issued"),
    );
    v2.record_claim_decision(&admin, &deal, &paid_claim, &approved, &h32(&env, 4));
    v2.record_payout_receipt(&admin, &deal, &paid_claim, &h32(&env, 5));
    // v0.2 creates a placeholder when a receipt arrives before any decision.
    v2.record_payout_receipt(&admin, &deal, &placeholder_claim, &h32(&env, 6));

    let hash = env.deployer().upload_contract_wasm(ins_current::WASM);
    v2.upgrade(&hash);
    let c = ins_current::Client::new(&env, &id);

    assert_eq!(c.get_admin(), admin);
    let policy = c.get_policy(&deal).unwrap();
    assert_eq!(
        (policy.escrow_id, policy.policy_hash),
        (h32(&env, 2), h32(&env, 3))
    );
    let claim = c.get_claim(&paid_claim).unwrap();
    assert_eq!(claim.decision, approved);
    assert_eq!(claim.receipt_hash, Some(h32(&env, 5)));

    // Outbox replays of already-recorded writes succeed; conflicting writes fail.
    c.record_claim_decision(&admin, &deal, &paid_claim, &approved, &h32(&env, 4));
    c.record_payout_receipt(&admin, &deal, &paid_claim, &h32(&env, 5));
    assert_eq!(
        c.try_record_payout_receipt(&admin, &deal, &paid_claim, &h32(&env, 7)),
        Err(Ok(contract_error(
            ins_current::Error::ReceiptAlreadyRecorded as u32
        )))
    );

    // The v0.2 placeholder accepts the real decision and keeps its receipt.
    let placeholder = c.get_claim(&placeholder_claim).unwrap();
    assert_eq!(placeholder.decision_hash, h32(&env, 0));
    c.record_claim_decision(&admin, &deal, &placeholder_claim, &approved, &h32(&env, 8));
    let fixed = c.get_claim(&placeholder_claim).unwrap();
    assert_eq!(
        (fixed.decision_hash, fixed.receipt_hash),
        (h32(&env, 8), Some(h32(&env, 6)))
    );

    // New rule: no receipt without an approved decision.
    assert_eq!(
        c.try_record_payout_receipt(&admin, &deal, &h32(&env, 11), &h32(&env, 6)),
        Err(Ok(contract_error(ins_current::Error::ClaimNotFound as u32)))
    );
}
