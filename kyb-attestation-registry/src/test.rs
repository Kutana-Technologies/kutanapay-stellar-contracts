#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal,
};

fn client<'a>(env: &'a Env, admin: &Address) -> KybAttestationRegistryClient<'a> {
    let id = env.register(KybAttestationRegistry, (admin,));
    KybAttestationRegistryClient::new(env, &id)
}

#[test]
fn constructor_sets_admin() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let c = client(&env, &admin);
    assert_eq!(c.get_admin(), admin);
}

#[test]
fn admin_can_write_and_read_standing() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let subject = Address::generate(&env);
    let c = client(&env, &admin);
    let tier = Symbol::new(&env, "standard");
    c.write_attestation(&admin, &subject, &tier, &1_900_000_000);
    let row = c.get_attestation(&subject).expect("row");
    assert_eq!(row.tier, tier);
    assert_eq!(row.expires_at, 1_900_000_000);
    assert!(!row.revoked);
}

#[test]
fn admin_can_revoke() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let subject = Address::generate(&env);
    let c = client(&env, &admin);
    c.write_attestation(&admin, &subject, &Symbol::new(&env, "standard"), &1_900_000_000);
    c.revoke_attestation(&admin, &subject);
    let row = c.get_attestation(&subject).expect("row");
    assert!(row.revoked);
}

#[test]
#[should_panic(expected = "not admin")]
fn non_admin_cannot_write() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let stranger = Address::generate(&env);
    let subject = Address::generate(&env);
    let c = client(&env, &admin);
    c.write_attestation(
        &stranger,
        &subject,
        &Symbol::new(&env, "standard"),
        &1_900_000_000,
    );
}

#[test]
fn rewrite_clears_revoked_flag() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let subject = Address::generate(&env);
    let c = client(&env, &admin);
    c.write_attestation(&admin, &subject, &Symbol::new(&env, "standard"), &1);
    c.revoke_attestation(&admin, &subject);
    c.write_attestation(&admin, &subject, &Symbol::new(&env, "enhanced"), &2);
    let row = c.get_attestation(&subject).expect("row");
    assert!(!row.revoked);
    assert_eq!(row.tier, Symbol::new(&env, "enhanced"));
}

#[test]
fn get_missing_subject_is_none() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let c = client(&env, &admin);
    assert!(c.get_attestation(&Address::generate(&env)).is_none());
}

#[test]
fn write_requires_admin_auth() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let subject = Address::generate(&env);
    let c = client(&env, &admin);
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &c.address,
            fn_name: "write_attestation",
            args: (
                admin.clone(),
                subject.clone(),
                Symbol::new(&env, "standard"),
                1_900_000_000u64,
            )
                .into_val(&env),
            sub_invokes: &[],
        },
    }]);
    c.write_attestation(&admin, &subject, &Symbol::new(&env, "standard"), &1_900_000_000);
    assert!(c.get_attestation(&subject).is_some());
}
