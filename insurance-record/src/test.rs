#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

fn client<'a>(env: &'a Env, admin: &Address) -> InsuranceRecordClient<'a> {
    let id = env.register(InsuranceRecord, (admin,));
    InsuranceRecordClient::new(env, &id)
}

fn h32(env: &Env, fill: u8) -> BytesN<32> {
    BytesN::from_array(env, &[fill; 32])
}

#[test]
fn constructor_sets_admin() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let c = client(&env, &admin);
    assert_eq!(c.get_admin(), admin);
}

#[test]
fn admin_can_record_and_read_policy() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let c = client(&env, &admin);
    let deal = h32(&env, 1);
    let escrow = h32(&env, 2);
    let policy_hash = h32(&env, 3);
    c.record_policy(&admin, &deal, &escrow, &policy_hash, &Symbol::new(&env, "issued"));
    let row = c.get_policy(&deal).expect("policy");
    assert_eq!(row.escrow_id, escrow);
    assert_eq!(row.policy_hash, policy_hash);
    assert_eq!(row.status, Symbol::new(&env, "issued"));
}

#[test]
fn claim_decision_then_payout_receipt() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let c = client(&env, &admin);
    let deal = h32(&env, 1);
    let claim = h32(&env, 9);
    let decision_hash = h32(&env, 4);
    let receipt_hash = h32(&env, 5);
    c.record_claim_decision(
        &admin,
        &deal,
        &claim,
        &Symbol::new(&env, "approved"),
        &decision_hash,
    );
    c.record_payout_receipt(&admin, &deal, &claim, &receipt_hash);
    let row = c.get_claim(&claim).expect("claim");
    assert_eq!(row.deal_id, deal);
    assert_eq!(row.decision, Symbol::new(&env, "approved"));
    assert_eq!(row.decision_hash, decision_hash);
    assert_eq!(row.receipt_hash, Some(receipt_hash));
}

#[test]
#[should_panic(expected = "not admin")]
fn non_admin_cannot_record_policy() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let stranger = Address::generate(&env);
    let c = client(&env, &admin);
    c.record_policy(
        &stranger,
        &h32(&env, 1),
        &h32(&env, 2),
        &h32(&env, 3),
        &Symbol::new(&env, "issued"),
    );
}

#[test]
fn missing_policy_is_none() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let c = client(&env, &admin);
    assert!(c.get_policy(&h32(&env, 1)).is_none());
}
