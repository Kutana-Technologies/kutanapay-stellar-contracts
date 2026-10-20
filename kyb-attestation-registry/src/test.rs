extern crate std;

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{
        storage::{Instance as _, Persistent as _},
        Address as _, AuthorizedFunction, AuthorizedInvocation, Events, Ledger, MockAuth,
        MockAuthInvoke,
    },
    vec, Address, Env, IntoVal,
};

const NOW: u64 = 1_800_000_000;
const LATER: u64 = NOW + 365 * 24 * 60 * 60;

struct Setup<'a> {
    env: Env,
    admin: Address,
    c: KybAttestationRegistryClient<'a>,
}

fn setup<'a>() -> Setup<'a> {
    let env = Env::default();
    env.ledger().set_timestamp(NOW);
    let admin = Address::generate(&env);
    let id = env.register(KybAttestationRegistry, (&admin,));
    let c = KybAttestationRegistryClient::new(&env, &id);
    Setup { env, admin, c }
}

fn standard(env: &Env) -> Symbol {
    Symbol::new(env, "standard")
}

fn record_ttl(s: &Setup, subject: &Address) -> u32 {
    s.env.as_contract(&s.c.address, || {
        s.env.storage().persistent().get_ttl(&key(&s.env, subject))
    })
}

fn instance_ttl(s: &Setup) -> u32 {
    s.env
        .as_contract(&s.c.address, || s.env.storage().instance().get_ttl())
}

// ---- deployment and reads ----

#[test]
fn constructor_sets_admin() {
    let s = setup();
    assert_eq!(s.c.get_admin(), s.admin);
    assert_eq!(s.c.get_pending_admin(), None);
}

#[test]
fn unknown_subject_reads_as_none_and_invalid() {
    let s = setup();
    let subject = Address::generate(&s.env);
    assert_eq!(s.c.get_attestation(&subject), None);
    assert!(!s.c.is_valid(&subject));
}

// ---- write ----

#[test]
fn admin_can_write_and_read_standing() {
    let s = setup();
    s.env.mock_all_auths();
    let subject = Address::generate(&s.env);
    s.c.write_attestation(&s.admin, &subject, &standard(&s.env), &LATER);
    assert_eq!(
        s.c.get_attestation(&subject),
        Some(Attestation {
            tier: standard(&s.env),
            expires_at: LATER,
            revoked: false
        })
    );
    assert!(s.c.is_valid(&subject));
}

#[test]
fn write_emits_event() {
    let s = setup();
    s.env.mock_all_auths();
    let subject = Address::generate(&s.env);
    s.c.write_attestation(&s.admin, &subject, &standard(&s.env), &LATER);
    assert_eq!(
        s.env.events().all(),
        vec![
            &s.env,
            (
                s.c.address.clone(),
                (symbol_short!("written"), subject.clone()).into_val(&s.env),
                (standard(&s.env), LATER).into_val(&s.env),
            )
        ]
    );
}

#[test]
fn write_can_update_an_active_attestation() {
    let s = setup();
    s.env.mock_all_auths();
    let subject = Address::generate(&s.env);
    s.c.write_attestation(&s.admin, &subject, &standard(&s.env), &LATER);
    let enhanced = Symbol::new(&s.env, "enhanced");
    s.c.write_attestation(&s.admin, &subject, &enhanced, &(LATER + 1));
    let row = s.c.get_attestation(&subject).unwrap();
    assert_eq!((row.tier, row.expires_at), (enhanced, LATER + 1));
}

#[test]
fn identical_write_replay_succeeds() {
    let s = setup();
    s.env.mock_all_auths();
    let subject = Address::generate(&s.env);
    s.c.write_attestation(&s.admin, &subject, &standard(&s.env), &LATER);
    s.c.write_attestation(&s.admin, &subject, &standard(&s.env), &LATER);
    assert!(s.c.is_valid(&subject));
}

#[test]
fn write_rejects_expiry_not_in_future() {
    let s = setup();
    s.env.mock_all_auths();
    let subject = Address::generate(&s.env);
    for expires_at in [0, NOW - 1, NOW] {
        assert_eq!(
            s.c.try_write_attestation(&s.admin, &subject, &standard(&s.env), &expires_at),
            Err(Ok(Error::ExpiryNotInFuture))
        );
    }
    assert_eq!(s.c.get_attestation(&subject), None);
}

#[test]
fn write_requires_exactly_the_admin_signature() {
    let s = setup();
    let subject = Address::generate(&s.env);
    let args = (s.admin.clone(), subject.clone(), standard(&s.env), LATER);
    s.env.mock_auths(&[MockAuth {
        address: &s.admin,
        invoke: &MockAuthInvoke {
            contract: &s.c.address,
            fn_name: "write_attestation",
            args: args.clone().into_val(&s.env),
            sub_invokes: &[],
        },
    }]);
    s.c.write_attestation(&s.admin, &subject, &standard(&s.env), &LATER);
    assert_eq!(
        s.env.auths(),
        std::vec![(
            s.admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    s.c.address.clone(),
                    Symbol::new(&s.env, "write_attestation"),
                    args.into_val(&s.env),
                )),
                sub_invocations: std::vec![],
            }
        )]
    );
}

// ---- authorization ----

#[test]
fn non_admin_writes_are_rejected() {
    let s = setup();
    s.env.mock_all_auths();
    let stranger = Address::generate(&s.env);
    let subject = Address::generate(&s.env);
    let tier = standard(&s.env);
    assert_eq!(
        s.c.try_write_attestation(&stranger, &subject, &tier, &LATER),
        Err(Ok(Error::NotAdmin))
    );
    assert_eq!(
        s.c.try_reinstate_attestation(&stranger, &subject, &tier, &LATER),
        Err(Ok(Error::NotAdmin))
    );
    assert_eq!(
        s.c.try_revoke_attestation(&stranger, &subject),
        Err(Ok(Error::NotAdmin))
    );
    assert_eq!(s.c.get_attestation(&subject), None);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn admin_address_without_admin_signature_is_rejected() {
    let s = setup();
    let subject = Address::generate(&s.env);
    s.c.write_attestation(&s.admin, &subject, &standard(&s.env), &LATER);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn revoke_without_admin_signature_is_rejected() {
    let s = setup();
    let subject = Address::generate(&s.env);
    s.c.revoke_attestation(&s.admin, &subject);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn stranger_cannot_upgrade() {
    let s = setup();
    let stranger = Address::generate(&s.env);
    let hash = BytesN::from_array(&s.env, &[7u8; 32]);
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

// ---- revoke and reinstate ----

#[test]
fn admin_can_revoke_and_event_is_emitted() {
    let s = setup();
    s.env.mock_all_auths();
    let subject = Address::generate(&s.env);
    s.c.write_attestation(&s.admin, &subject, &standard(&s.env), &LATER);
    s.c.revoke_attestation(&s.admin, &subject);
    assert_eq!(
        s.env.events().all(),
        vec![
            &s.env,
            (
                s.c.address.clone(),
                (symbol_short!("revoked"), subject.clone()).into_val(&s.env),
                ().into_val(&s.env),
            )
        ]
    );
    let row = s.c.get_attestation(&subject).unwrap();
    assert!(row.revoked);
    assert_eq!(row.expires_at, LATER);
    assert!(!s.c.is_valid(&subject));
}

#[test]
fn write_after_revoke_is_rejected() {
    let s = setup();
    s.env.mock_all_auths();
    let subject = Address::generate(&s.env);
    s.c.write_attestation(&s.admin, &subject, &standard(&s.env), &LATER);
    s.c.revoke_attestation(&s.admin, &subject);
    // A delayed replay of the original write must not reactivate the subject.
    assert_eq!(
        s.c.try_write_attestation(&s.admin, &subject, &standard(&s.env), &LATER),
        Err(Ok(Error::AttestationRevoked))
    );
    assert!(!s.c.is_valid(&subject));
}

#[test]
fn revoke_before_first_write_blocks_out_of_order_write() {
    let s = setup();
    s.env.mock_all_auths();
    let subject = Address::generate(&s.env);
    s.c.revoke_attestation(&s.admin, &subject);
    assert_eq!(
        s.c.get_attestation(&subject),
        Some(Attestation {
            tier: symbol_short!("none"),
            expires_at: 0,
            revoked: true
        })
    );
    assert_eq!(
        s.c.try_write_attestation(&s.admin, &subject, &standard(&s.env), &LATER),
        Err(Ok(Error::AttestationRevoked))
    );
}

#[test]
fn revoke_is_idempotent() {
    let s = setup();
    s.env.mock_all_auths();
    let subject = Address::generate(&s.env);
    s.c.write_attestation(&s.admin, &subject, &standard(&s.env), &LATER);
    s.c.revoke_attestation(&s.admin, &subject);
    s.c.revoke_attestation(&s.admin, &subject);
    assert!(s.c.get_attestation(&subject).unwrap().revoked);
}

#[test]
fn reinstate_clears_revocation() {
    let s = setup();
    s.env.mock_all_auths();
    let subject = Address::generate(&s.env);
    let enhanced = Symbol::new(&s.env, "enhanced");
    s.c.write_attestation(&s.admin, &subject, &standard(&s.env), &LATER);
    s.c.revoke_attestation(&s.admin, &subject);
    s.c.reinstate_attestation(&s.admin, &subject, &enhanced, &(LATER + 5));
    assert_eq!(
        s.env.events().all(),
        vec![
            &s.env,
            (
                s.c.address.clone(),
                (symbol_short!("reinstate"), subject.clone()).into_val(&s.env),
                (enhanced.clone(), LATER + 5).into_val(&s.env),
            )
        ]
    );
    assert_eq!(
        s.c.get_attestation(&subject),
        Some(Attestation {
            tier: enhanced,
            expires_at: LATER + 5,
            revoked: false
        })
    );
    assert!(s.c.is_valid(&subject));
}

#[test]
fn reinstate_rejects_expiry_not_in_future() {
    let s = setup();
    s.env.mock_all_auths();
    let subject = Address::generate(&s.env);
    s.c.revoke_attestation(&s.admin, &subject);
    assert_eq!(
        s.c.try_reinstate_attestation(&s.admin, &subject, &standard(&s.env), &NOW),
        Err(Ok(Error::ExpiryNotInFuture))
    );
    assert!(s.c.get_attestation(&subject).unwrap().revoked);
}

// ---- validity ----

#[test]
fn is_valid_turns_false_at_expiry() {
    let s = setup();
    s.env.mock_all_auths();
    let subject = Address::generate(&s.env);
    s.c.write_attestation(&s.admin, &subject, &standard(&s.env), &(NOW + 100));
    assert!(s.c.is_valid(&subject));
    s.env.ledger().set_timestamp(NOW + 99);
    assert!(s.c.is_valid(&subject));
    s.env.ledger().set_timestamp(NOW + 100);
    assert!(!s.c.is_valid(&subject));
    // The stored row is still returned unchanged.
    assert!(!s.c.get_attestation(&subject).unwrap().revoked);
}

// ---- storage lifetime ----

#[test]
fn writes_extend_record_and_instance_ttl() {
    let s = setup();
    s.env.mock_all_auths();
    let subject = Address::generate(&s.env);
    s.c.write_attestation(&s.admin, &subject, &standard(&s.env), &LATER);
    assert_eq!(record_ttl(&s, &subject), RECORD_EXTEND_TO);
    assert_eq!(instance_ttl(&s), INSTANCE_EXTEND_TO);
}

#[test]
fn anyone_can_extend_ttls() {
    let s = setup();
    s.env.mock_all_auths();
    let subject = Address::generate(&s.env);
    s.c.write_attestation(&s.admin, &subject, &standard(&s.env), &LATER);

    let seq = s.env.ledger().sequence();
    s.env
        .ledger()
        .set_sequence_number(seq + 10 * DAY_IN_LEDGERS);
    assert!(record_ttl(&s, &subject) < RECORD_THRESHOLD);

    // No authorization is mocked for these calls.
    s.env.set_auths(&[]);
    s.c.extend_attestation_ttl(&subject);
    s.c.extend_ttl();
    assert_eq!(record_ttl(&s, &subject), RECORD_EXTEND_TO);
    assert_eq!(instance_ttl(&s), INSTANCE_EXTEND_TO);
    assert!(s.env.auths().is_empty());

    // Unknown subjects are ignored rather than created.
    let unknown = Address::generate(&s.env);
    s.c.extend_attestation_ttl(&unknown);
    assert_eq!(s.c.get_attestation(&unknown), None);
}

// ---- admin transfer ----

#[test]
fn two_step_admin_transfer() {
    let s = setup();
    s.env.mock_all_auths();
    let new_admin = Address::generate(&s.env);
    let subject = Address::generate(&s.env);

    s.c.propose_admin(&new_admin);
    assert_eq!(
        s.env.auths()[0].0,
        s.admin,
        "proposal must be authorized by the current admin"
    );
    assert_eq!(s.c.get_pending_admin(), Some(new_admin.clone()));
    // The admin does not change until the nominee accepts.
    assert_eq!(s.c.get_admin(), s.admin);

    s.c.accept_admin();
    assert_eq!(
        s.env.auths()[0].0,
        new_admin,
        "acceptance must be authorized by the nominee"
    );
    assert_eq!(
        s.env.events().all(),
        vec![
            &s.env,
            (
                s.c.address.clone(),
                (symbol_short!("adm_set"),).into_val(&s.env),
                (s.admin.clone(), new_admin.clone()).into_val(&s.env),
            )
        ]
    );
    assert_eq!(s.c.get_admin(), new_admin);
    assert_eq!(s.c.get_pending_admin(), None);

    assert_eq!(
        s.c.try_write_attestation(&s.admin, &subject, &standard(&s.env), &LATER),
        Err(Ok(Error::NotAdmin))
    );
    s.c.write_attestation(&new_admin, &subject, &standard(&s.env), &LATER);
    assert!(s.c.is_valid(&subject));
}

#[test]
fn accept_without_proposal_fails() {
    let s = setup();
    s.env.mock_all_auths();
    assert_eq!(s.c.try_accept_admin(), Err(Ok(Error::NoPendingAdmin)));
}

#[test]
fn later_proposal_replaces_earlier_nominee() {
    let s = setup();
    s.env.mock_all_auths();
    let first = Address::generate(&s.env);
    let second = Address::generate(&s.env);
    s.c.propose_admin(&first);
    s.c.propose_admin(&second);
    assert_eq!(s.c.get_pending_admin(), Some(second));
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn stranger_cannot_propose_admin() {
    let s = setup();
    let stranger = Address::generate(&s.env);
    s.env.mock_auths(&[MockAuth {
        address: &stranger,
        invoke: &MockAuthInvoke {
            contract: &s.c.address,
            fn_name: "propose_admin",
            args: (stranger.clone(),).into_val(&s.env),
            sub_invokes: &[],
        },
    }]);
    s.c.propose_admin(&stranger);
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
