extern crate std;

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{
        storage::{Instance as _, Persistent as _},
        Address as _, Events, Ledger, MockAuth, MockAuthInvoke,
    },
    vec, Address, BytesN, Env, IntoVal,
};

struct Setup<'a> {
    env: Env,
    admin: Address,
    c: InsuranceRecordClient<'a>,
}

fn setup<'a>() -> Setup<'a> {
    let env = Env::default();
    let admin = Address::generate(&env);
    let id = env.register(InsuranceRecord, (&admin,));
    let c = InsuranceRecordClient::new(&env, &id);
    Setup { env, admin, c }
}

fn h32(env: &Env, fill: u8) -> BytesN<32> {
    BytesN::from_array(env, &[fill; 32])
}

const DEAL: u8 = 1;
const OTHER_DEAL: u8 = 2;
const CLAIM: u8 = 9;

/// Record an approved decision for CLAIM on DEAL.
fn approve(s: &Setup) {
    s.c.record_claim_decision(
        &s.admin,
        &h32(&s.env, DEAL),
        &h32(&s.env, CLAIM),
        &APPROVED,
        &h32(&s.env, 4),
    );
}

fn ttl(s: &Setup, key: &RecordKey) -> u32 {
    s.env
        .as_contract(&s.c.address, || s.env.storage().persistent().get_ttl(key))
}

// ---- deployment and reads ----

#[test]
fn constructor_sets_admin() {
    let s = setup();
    assert_eq!(s.c.get_admin(), s.admin);
    assert_eq!(s.c.get_pending_admin(), None);
}

#[test]
fn missing_records_are_none() {
    let s = setup();
    assert_eq!(s.c.get_policy(&h32(&s.env, DEAL)), None);
    assert_eq!(s.c.get_claim(&h32(&s.env, CLAIM)), None);
}

// ---- policy ----

#[test]
fn admin_can_record_and_read_policy() {
    let s = setup();
    s.env.mock_all_auths();
    let issued = symbol_short!("issued");
    s.c.record_policy(
        &s.admin,
        &h32(&s.env, DEAL),
        &h32(&s.env, 2),
        &h32(&s.env, 3),
        &issued,
    );
    let expected = PolicyRecord {
        escrow_id: h32(&s.env, 2),
        policy_hash: h32(&s.env, 3),
        status: issued,
    };
    assert_eq!(
        s.env.events().all(),
        vec![
            &s.env,
            (
                s.c.address.clone(),
                (symbol_short!("policy"), h32(&s.env, DEAL)).into_val(&s.env),
                expected.clone().into_val(&s.env),
            )
        ]
    );
    assert_eq!(s.c.get_policy(&h32(&s.env, DEAL)), Some(expected));
}

#[test]
fn policy_update_is_recorded_and_emitted() {
    let s = setup();
    s.env.mock_all_auths();
    let deal = h32(&s.env, DEAL);
    s.c.record_policy(
        &s.admin,
        &deal,
        &h32(&s.env, 2),
        &h32(&s.env, 3),
        &symbol_short!("issued"),
    );
    s.c.record_policy(
        &s.admin,
        &deal,
        &h32(&s.env, 2),
        &h32(&s.env, 7),
        &symbol_short!("amended"),
    );
    assert_eq!(s.env.events().all().len(), 1);
    let row = s.c.get_policy(&deal).unwrap();
    assert_eq!(
        (row.policy_hash, row.status),
        (h32(&s.env, 7), symbol_short!("amended"))
    );
}

// ---- claim decision ----

#[test]
fn decision_is_recorded_and_emitted() {
    let s = setup();
    s.env.mock_all_auths();
    approve(&s);
    assert_eq!(
        s.env.events().all(),
        vec![
            &s.env,
            (
                s.c.address.clone(),
                (symbol_short!("decision"), h32(&s.env, CLAIM)).into_val(&s.env),
                (APPROVED, h32(&s.env, 4)).into_val(&s.env),
            )
        ]
    );
    assert_eq!(
        s.c.get_claim(&h32(&s.env, CLAIM)),
        Some(ClaimRecord {
            deal_id: h32(&s.env, DEAL),
            decision: APPROVED,
            decision_hash: h32(&s.env, 4),
            receipt_hash: None,
        })
    );
}

#[test]
fn decision_must_be_approved_or_denied() {
    let s = setup();
    s.env.mock_all_auths();
    for bad in ["paid", "rejected", "APPROVED"] {
        assert_eq!(
            s.c.try_record_claim_decision(
                &s.admin,
                &h32(&s.env, DEAL),
                &h32(&s.env, CLAIM),
                &Symbol::new(&s.env, bad),
                &h32(&s.env, 4),
            ),
            Err(Ok(Error::InvalidDecision))
        );
    }
    assert_eq!(s.c.get_claim(&h32(&s.env, CLAIM)), None);
}

#[test]
fn decision_can_change_before_payout() {
    let s = setup();
    s.env.mock_all_auths();
    let claim = h32(&s.env, CLAIM);
    s.c.record_claim_decision(
        &s.admin,
        &h32(&s.env, DEAL),
        &claim,
        &DENIED,
        &h32(&s.env, 4),
    );
    s.c.record_claim_decision(
        &s.admin,
        &h32(&s.env, DEAL),
        &claim,
        &APPROVED,
        &h32(&s.env, 5),
    );
    let row = s.c.get_claim(&claim).unwrap();
    assert_eq!(
        (row.decision, row.decision_hash),
        (APPROVED, h32(&s.env, 5))
    );
}

#[test]
fn decision_cannot_move_claim_to_another_deal() {
    let s = setup();
    s.env.mock_all_auths();
    approve(&s);
    assert_eq!(
        s.c.try_record_claim_decision(
            &s.admin,
            &h32(&s.env, OTHER_DEAL),
            &h32(&s.env, CLAIM),
            &APPROVED,
            &h32(&s.env, 4),
        ),
        Err(Ok(Error::DealMismatch))
    );
    assert_eq!(
        s.c.get_claim(&h32(&s.env, CLAIM)).unwrap().deal_id,
        h32(&s.env, DEAL)
    );
}

#[test]
fn decision_is_final_after_payout_but_replay_succeeds() {
    let s = setup();
    s.env.mock_all_auths();
    let (deal, claim) = (h32(&s.env, DEAL), h32(&s.env, CLAIM));
    approve(&s);
    s.c.record_payout_receipt(&s.admin, &deal, &claim, &h32(&s.env, 5));

    // An identical replay (outbox retry) is accepted and changes nothing.
    approve(&s);
    assert_eq!(s.env.events().all().len(), 0);

    assert_eq!(
        s.c.try_record_claim_decision(&s.admin, &deal, &claim, &DENIED, &h32(&s.env, 4)),
        Err(Ok(Error::DecisionFinal))
    );
    assert_eq!(
        s.c.try_record_claim_decision(&s.admin, &deal, &claim, &APPROVED, &h32(&s.env, 6)),
        Err(Ok(Error::DecisionFinal))
    );
    assert_eq!(
        s.c.get_claim(&claim).unwrap().receipt_hash,
        Some(h32(&s.env, 5))
    );
}

// ---- payout receipt ----

#[test]
fn payout_on_approved_claim_is_recorded_and_emitted() {
    let s = setup();
    s.env.mock_all_auths();
    approve(&s);
    s.c.record_payout_receipt(
        &s.admin,
        &h32(&s.env, DEAL),
        &h32(&s.env, CLAIM),
        &h32(&s.env, 5),
    );
    assert_eq!(
        s.env.events().all(),
        vec![
            &s.env,
            (
                s.c.address.clone(),
                (symbol_short!("payout"), h32(&s.env, CLAIM)).into_val(&s.env),
                h32(&s.env, 5).into_val(&s.env),
            )
        ]
    );
    assert_eq!(
        s.c.get_claim(&h32(&s.env, CLAIM)),
        Some(ClaimRecord {
            deal_id: h32(&s.env, DEAL),
            decision: APPROVED,
            decision_hash: h32(&s.env, 4),
            receipt_hash: Some(h32(&s.env, 5)),
        })
    );
}

#[test]
fn payout_without_decision_is_rejected() {
    let s = setup();
    s.env.mock_all_auths();
    assert_eq!(
        s.c.try_record_payout_receipt(
            &s.admin,
            &h32(&s.env, DEAL),
            &h32(&s.env, CLAIM),
            &h32(&s.env, 5)
        ),
        Err(Ok(Error::ClaimNotFound))
    );
    assert_eq!(s.c.get_claim(&h32(&s.env, CLAIM)), None);
}

#[test]
fn payout_on_denied_claim_is_rejected() {
    let s = setup();
    s.env.mock_all_auths();
    let (deal, claim) = (h32(&s.env, DEAL), h32(&s.env, CLAIM));
    s.c.record_claim_decision(&s.admin, &deal, &claim, &DENIED, &h32(&s.env, 4));
    assert_eq!(
        s.c.try_record_payout_receipt(&s.admin, &deal, &claim, &h32(&s.env, 5)),
        Err(Ok(Error::ClaimNotApproved))
    );
    assert_eq!(s.c.get_claim(&claim).unwrap().receipt_hash, None);
}

#[test]
fn payout_cannot_move_claim_to_another_deal() {
    let s = setup();
    s.env.mock_all_auths();
    approve(&s);
    assert_eq!(
        s.c.try_record_payout_receipt(
            &s.admin,
            &h32(&s.env, OTHER_DEAL),
            &h32(&s.env, CLAIM),
            &h32(&s.env, 5)
        ),
        Err(Ok(Error::DealMismatch))
    );
}

#[test]
fn payout_receipt_is_immutable_but_replay_succeeds() {
    let s = setup();
    s.env.mock_all_auths();
    let (deal, claim) = (h32(&s.env, DEAL), h32(&s.env, CLAIM));
    approve(&s);
    s.c.record_payout_receipt(&s.admin, &deal, &claim, &h32(&s.env, 5));
    s.c.record_payout_receipt(&s.admin, &deal, &claim, &h32(&s.env, 5));
    assert_eq!(s.env.events().all().len(), 0);
    assert_eq!(
        s.c.try_record_payout_receipt(&s.admin, &deal, &claim, &h32(&s.env, 6)),
        Err(Ok(Error::ReceiptAlreadyRecorded))
    );
    assert_eq!(
        s.c.get_claim(&claim).unwrap().receipt_hash,
        Some(h32(&s.env, 5))
    );
}

#[test]
fn real_decision_can_replace_v02_placeholder_claim() {
    let s = setup();
    s.env.mock_all_auths();
    let (deal, claim) = (h32(&s.env, DEAL), h32(&s.env, CLAIM));
    // Shape of the placeholder v0.2 wrote when a receipt arrived before a decision.
    s.env.as_contract(&s.c.address, || {
        write(
            &s.env,
            &claim_key(&s.env, &claim),
            &ClaimRecord {
                deal_id: deal.clone(),
                decision: symbol_short!("paid"),
                decision_hash: h32(&s.env, 0),
                receipt_hash: Some(h32(&s.env, 5)),
            },
        )
    });
    approve(&s);
    assert_eq!(
        s.c.get_claim(&claim),
        Some(ClaimRecord {
            deal_id: deal,
            decision: APPROVED,
            decision_hash: h32(&s.env, 4),
            receipt_hash: Some(h32(&s.env, 5)),
        })
    );
}

// ---- authorization ----

#[test]
fn non_admin_writes_are_rejected() {
    let s = setup();
    s.env.mock_all_auths();
    let x = Address::generate(&s.env);
    let (deal, claim) = (h32(&s.env, DEAL), h32(&s.env, CLAIM));
    assert_eq!(
        s.c.try_record_policy(
            &x,
            &deal,
            &h32(&s.env, 2),
            &h32(&s.env, 3),
            &symbol_short!("issued")
        ),
        Err(Ok(Error::NotAdmin))
    );
    assert_eq!(
        s.c.try_record_claim_decision(&x, &deal, &claim, &APPROVED, &h32(&s.env, 4)),
        Err(Ok(Error::NotAdmin))
    );
    approve(&s);
    assert_eq!(
        s.c.try_record_payout_receipt(&x, &deal, &claim, &h32(&s.env, 5)),
        Err(Ok(Error::NotAdmin))
    );
    assert_eq!(s.c.get_policy(&deal), None);
    assert_eq!(s.c.get_claim(&claim).unwrap().receipt_hash, None);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn admin_address_without_admin_signature_is_rejected() {
    let s = setup();
    let stranger = Address::generate(&s.env);
    let status = symbol_short!("issued");
    s.env.mock_auths(&[MockAuth {
        address: &stranger,
        invoke: &MockAuthInvoke {
            contract: &s.c.address,
            fn_name: "record_policy",
            args: (
                s.admin.clone(),
                h32(&s.env, 1),
                h32(&s.env, 2),
                h32(&s.env, 3),
                status.clone(),
            )
                .into_val(&s.env),
            sub_invokes: &[],
        },
    }]);
    s.c.record_policy(
        &s.admin,
        &h32(&s.env, 1),
        &h32(&s.env, 2),
        &h32(&s.env, 3),
        &status,
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn stranger_cannot_upgrade() {
    let s = setup();
    let stranger = Address::generate(&s.env);
    let hash = h32(&s.env, 7);
    s.env.mock_auths(&[MockAuth {
        address: &stranger,
        invoke: &MockAuthInvoke {
            contract: &s.c.address,
            fn_name: "upgrade",
            args: (hash.clone(),).into_val(&s.env),
            sub_invokes: &[],
        },
    }]);
    s.c.upgrade(&hash);
}

// ---- storage lifetime ----

#[test]
fn writes_extend_record_and_instance_ttl() {
    let s = setup();
    s.env.mock_all_auths();
    let deal = h32(&s.env, DEAL);
    s.c.record_policy(
        &s.admin,
        &deal,
        &h32(&s.env, 2),
        &h32(&s.env, 3),
        &symbol_short!("issued"),
    );
    approve(&s);
    assert_eq!(ttl(&s, &policy_key(&s.env, &deal)), RECORD_EXTEND_TO);
    assert_eq!(
        ttl(&s, &claim_key(&s.env, &h32(&s.env, CLAIM))),
        RECORD_EXTEND_TO
    );
    let instance = s
        .env
        .as_contract(&s.c.address, || s.env.storage().instance().get_ttl());
    assert_eq!(instance, INSTANCE_EXTEND_TO);
}

#[test]
fn anyone_can_extend_ttls() {
    let s = setup();
    s.env.mock_all_auths();
    let (deal, claim) = (h32(&s.env, DEAL), h32(&s.env, CLAIM));
    s.c.record_policy(
        &s.admin,
        &deal,
        &h32(&s.env, 2),
        &h32(&s.env, 3),
        &symbol_short!("issued"),
    );
    approve(&s);

    let seq = s.env.ledger().sequence();
    s.env
        .ledger()
        .set_sequence_number(seq + 10 * DAY_IN_LEDGERS);
    assert!(ttl(&s, &policy_key(&s.env, &deal)) < RECORD_THRESHOLD);

    s.env.set_auths(&[]);
    s.c.extend_policy_ttl(&deal);
    s.c.extend_claim_ttl(&claim);
    s.c.extend_ttl();
    assert!(s.env.auths().is_empty());
    assert_eq!(ttl(&s, &policy_key(&s.env, &deal)), RECORD_EXTEND_TO);
    assert_eq!(ttl(&s, &claim_key(&s.env, &claim)), RECORD_EXTEND_TO);

    s.c.extend_policy_ttl(&h32(&s.env, 0xEE));
    assert_eq!(s.c.get_policy(&h32(&s.env, 0xEE)), None);
}

// ---- admin transfer ----

#[test]
fn two_step_admin_transfer() {
    let s = setup();
    s.env.mock_all_auths();
    let new_admin = Address::generate(&s.env);
    s.c.propose_admin(&new_admin);
    assert_eq!(s.env.auths()[0].0, s.admin);
    assert_eq!(s.c.get_admin(), s.admin);
    s.c.accept_admin();
    assert_eq!(s.env.auths()[0].0, new_admin);
    assert_eq!(s.c.get_admin(), new_admin);
    assert_eq!(s.c.get_pending_admin(), None);
    assert_eq!(
        s.c.try_record_policy(
            &s.admin,
            &h32(&s.env, 1),
            &h32(&s.env, 2),
            &h32(&s.env, 3),
            &symbol_short!("issued")
        ),
        Err(Ok(Error::NotAdmin))
    );
    s.c.record_policy(
        &new_admin,
        &h32(&s.env, 1),
        &h32(&s.env, 2),
        &h32(&s.env, 3),
        &symbol_short!("issued"),
    );
}

#[test]
fn accept_without_proposal_fails() {
    let s = setup();
    s.env.mock_all_auths();
    assert_eq!(s.c.try_accept_admin(), Err(Ok(Error::NoPendingAdmin)));
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn only_the_nominee_can_accept() {
    let s = setup();
    let nominee = Address::generate(&s.env);
    let stranger = Address::generate(&s.env);
    s.env.mock_auths(&[MockAuth {
        address: &s.admin,
        invoke: &MockAuthInvoke {
            contract: &s.c.address,
            fn_name: "propose_admin",
            args: (nominee.clone(),).into_val(&s.env),
            sub_invokes: &[],
        },
    }]);
    s.c.propose_admin(&nominee);
    s.env.mock_auths(&[MockAuth {
        address: &stranger,
        invoke: &MockAuthInvoke {
            contract: &s.c.address,
            fn_name: "accept_admin",
            args: ().into_val(&s.env),
            sub_invokes: &[],
        },
    }]);
    s.c.accept_admin();
}

// ---- hash validation ----

#[test]
fn zero_hashes_are_rejected() {
    let s = setup();
    s.env.mock_all_auths();
    let (deal, claim, zero) = (h32(&s.env, DEAL), h32(&s.env, CLAIM), h32(&s.env, 0));
    assert_eq!(
        s.c.try_record_policy(
            &s.admin,
            &deal,
            &h32(&s.env, 2),
            &zero,
            &symbol_short!("issued")
        ),
        Err(Ok(Error::InvalidHash))
    );
    assert_eq!(
        s.c.try_record_claim_decision(&s.admin, &deal, &claim, &APPROVED, &zero),
        Err(Ok(Error::InvalidHash))
    );
    approve(&s);
    assert_eq!(
        s.c.try_record_payout_receipt(&s.admin, &deal, &claim, &zero),
        Err(Ok(Error::InvalidHash))
    );
    assert_eq!(s.c.get_policy(&deal), None);
    assert_eq!(s.c.get_claim(&claim).unwrap().receipt_hash, None);
}

#[test]
fn only_legacy_paid_placeholders_are_replaceable() {
    let s = setup();
    s.env.mock_all_auths();
    let (deal, claim) = (h32(&s.env, DEAL), h32(&s.env, CLAIM));
    // A paid claim whose decision hash happens to be zero but whose decision is
    // not the legacy "paid" marker must stay final.
    s.env.as_contract(&s.c.address, || {
        write(
            &s.env,
            &claim_key(&s.env, &claim),
            &ClaimRecord {
                deal_id: deal.clone(),
                decision: APPROVED,
                decision_hash: h32(&s.env, 0),
                receipt_hash: Some(h32(&s.env, 5)),
            },
        )
    });
    assert_eq!(
        s.c.try_record_claim_decision(&s.admin, &deal, &claim, &DENIED, &h32(&s.env, 4)),
        Err(Ok(Error::DecisionFinal))
    );
}

// ---- admin transfer cancellation ----

#[test]
fn admin_can_cancel_pending_transfer() {
    let s = setup();
    s.env.mock_all_auths();
    let nominee = Address::generate(&s.env);
    s.c.propose_admin(&nominee);
    s.c.cancel_admin_transfer();
    assert_eq!(s.env.auths()[0].0, s.admin);
    assert_eq!(
        s.env.events().all(),
        vec![
            &s.env,
            (
                s.c.address.clone(),
                (symbol_short!("adm_cncl"),).into_val(&s.env),
                nominee.clone().into_val(&s.env),
            )
        ]
    );
    assert_eq!(s.c.get_pending_admin(), None);
    assert_eq!(s.c.try_accept_admin(), Err(Ok(Error::NoPendingAdmin)));
    assert_eq!(
        s.c.try_cancel_admin_transfer(),
        Err(Ok(Error::NoPendingAdmin))
    );
    assert_eq!(s.c.get_admin(), s.admin);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn stranger_cannot_cancel_transfer() {
    let s = setup();
    let stranger = Address::generate(&s.env);
    s.env.mock_auths(&[MockAuth {
        address: &stranger,
        invoke: &MockAuthInvoke {
            contract: &s.c.address,
            fn_name: "cancel_admin_transfer",
            args: ().into_val(&s.env),
            sub_invokes: &[],
        },
    }]);
    s.c.cancel_admin_transfer();
}
