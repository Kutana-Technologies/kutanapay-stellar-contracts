#![no_std]
use soroban_sdk::{
    contract, contractimpl, contractmeta, contracttype, Address, BytesN, Env, Symbol,
};

contractmeta!(key = "name", val = "KutanaPay KYB Attestation Registry");
contractmeta!(key = "org", val = "KutanaPay");
contractmeta!(key = "binver", val = "0.2.0");
contractmeta!(
    key = "desc",
    val = "On-chain KYB standing: tier, expiry, revocation. No PII."
);

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attestation {
    pub tier: Symbol,
    pub expires_at: u64,
    pub revoked: bool,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
}

#[contract]
pub struct KybAttestationRegistry;

#[contractimpl]
impl KybAttestationRegistry {
    /// Bind the sole upgrade/write authority at deploy time.
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn write_attestation(
        env: Env,
        admin: Address,
        subject: Address,
        tier: Symbol,
        expires_at: u64,
    ) {
        Self::require_admin(&env, &admin);
        let key = (Symbol::new(&env, "attest"), subject);
        env.storage().persistent().set(
            &key,
            &Attestation {
                tier,
                expires_at,
                revoked: false,
            },
        );
    }

    pub fn revoke_attestation(env: Env, admin: Address, subject: Address) {
        Self::require_admin(&env, &admin);
        let key = (Symbol::new(&env, "attest"), subject);
        if let Some(mut row) = env.storage().persistent().get::<_, Attestation>(&key) {
            row.revoked = true;
            env.storage().persistent().set(&key, &row);
        }
    }

    pub fn get_attestation(env: Env, subject: Address) -> Option<Attestation> {
        let key = (Symbol::new(&env, "attest"), subject);
        env.storage().persistent().get(&key)
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    /// Replace this contract's WASM in place. Same C-id and storage; admin only.
    /// Upload the new WASM first, then pass its hash here.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    fn require_admin(env: &Env, admin: &Address) {
        let stored: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != &stored {
            panic!("not admin");
        }
        admin.require_auth();
    }
}
