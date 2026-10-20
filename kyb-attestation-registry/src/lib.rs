#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contractmeta, contracttype, symbol_short, Address,
    BytesN, Env, Symbol,
};

contractmeta!(key = "name", val = "KutanaPay KYB Attestation Registry");
contractmeta!(key = "org", val = "KutanaPay");
contractmeta!(key = "binver", val = "0.3.0");
contractmeta!(
    key = "desc",
    val = "On-chain KYB standing: tier, expiry, revocation. No PII."
);

/// Ledgers close roughly every five seconds.
const DAY_IN_LEDGERS: u32 = 17_280;
const INSTANCE_EXTEND_TO: u32 = 30 * DAY_IN_LEDGERS;
const INSTANCE_THRESHOLD: u32 = INSTANCE_EXTEND_TO - DAY_IN_LEDGERS;
const RECORD_EXTEND_TO: u32 = 120 * DAY_IN_LEDGERS;
const RECORD_THRESHOLD: u32 = RECORD_EXTEND_TO - 7 * DAY_IN_LEDGERS;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// The `admin` argument is not the stored admin.
    NotAdmin = 1,
    /// `expires_at` is not later than the current ledger time.
    ExpiryNotInFuture = 2,
    /// The subject is revoked; use `reinstate_attestation`.
    AttestationRevoked = 3,
    /// There is no pending admin transfer.
    NoPendingAdmin = 4,
}

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
    PendingAdmin,
}

#[contract]
pub struct KybAttestationRegistry;

#[contractimpl]
impl KybAttestationRegistry {
    /// Bind the sole upgrade/write authority at deploy time.
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        extend_instance(&env);
    }

    /// Create or replace a subject's attestation. Fails if the subject is revoked.
    pub fn write_attestation(
        env: Env,
        admin: Address,
        subject: Address,
        tier: Symbol,
        expires_at: u64,
    ) -> Result<(), Error> {
        Self::require_admin(&env, &admin)?;
        if let Some(row) = read(&env, &subject) {
            if row.revoked {
                return Err(Error::AttestationRevoked);
            }
        }
        Self::store_active(&env, &subject, tier, expires_at, symbol_short!("written"))
    }

    /// Clear a revocation and store a fresh attestation.
    pub fn reinstate_attestation(
        env: Env,
        admin: Address,
        subject: Address,
        tier: Symbol,
        expires_at: u64,
    ) -> Result<(), Error> {
        Self::require_admin(&env, &admin)?;
        Self::store_active(&env, &subject, tier, expires_at, symbol_short!("reinstate"))
    }

    /// Revoke a subject. A subject with no attestation gets a revoked placeholder,
    /// so a write delivered out of order cannot activate it.
    pub fn revoke_attestation(env: Env, admin: Address, subject: Address) -> Result<(), Error> {
        Self::require_admin(&env, &admin)?;
        let row = match read(&env, &subject) {
            Some(mut row) => {
                row.revoked = true;
                row
            }
            None => Attestation {
                tier: symbol_short!("none"),
                expires_at: 0,
                revoked: true,
            },
        };
        write(&env, &subject, &row);
        env.events()
            .publish((symbol_short!("revoked"), subject), ());
        Ok(())
    }

    pub fn get_attestation(env: Env, subject: Address) -> Option<Attestation> {
        read(&env, &subject)
    }

    /// True only if the subject has an attestation that is not revoked and not expired.
    pub fn is_valid(env: Env, subject: Address) -> bool {
        match read(&env, &subject) {
            Some(row) => !row.revoked && row.expires_at > env.ledger().timestamp(),
            None => false,
        }
    }

    pub fn get_admin(env: Env) -> Address {
        admin(&env)
    }

    pub fn get_pending_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::PendingAdmin)
    }

    /// Step one of an admin transfer: the current admin nominates a successor.
    pub fn propose_admin(env: Env, new_admin: Address) {
        admin(&env).require_auth();
        env.storage()
            .instance()
            .set(&DataKey::PendingAdmin, &new_admin);
        extend_instance(&env);
        env.events()
            .publish((symbol_short!("adm_prop"),), new_admin);
    }

    /// Step two of an admin transfer: the nominee accepts.
    pub fn accept_admin(env: Env) -> Result<(), Error> {
        let pending: Address = env
            .storage()
            .instance()
            .get(&DataKey::PendingAdmin)
            .ok_or(Error::NoPendingAdmin)?;
        pending.require_auth();
        let previous = admin(&env);
        env.storage().instance().set(&DataKey::Admin, &pending);
        env.storage().instance().remove(&DataKey::PendingAdmin);
        extend_instance(&env);
        env.events()
            .publish((symbol_short!("adm_set"),), (previous, pending));
        Ok(())
    }

    /// Withdraw a pending admin nomination.
    pub fn cancel_admin_transfer(env: Env) -> Result<(), Error> {
        admin(&env).require_auth();
        let pending: Address = env
            .storage()
            .instance()
            .get(&DataKey::PendingAdmin)
            .ok_or(Error::NoPendingAdmin)?;
        env.storage().instance().remove(&DataKey::PendingAdmin);
        env.events().publish((symbol_short!("adm_cncl"),), pending);
        Ok(())
    }

    /// Replace this contract's WASM in place. Same C-id and storage; admin only.
    /// Upload the new WASM first, then pass its hash here.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        admin(&env).require_auth();
        env.events()
            .publish((symbol_short!("upgraded"),), new_wasm_hash.clone());
        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    /// Extend the contract instance and code TTL. Anyone may pay for this.
    pub fn extend_ttl(env: Env) {
        extend_instance(&env);
    }

    /// Extend one attestation's TTL. Anyone may pay for this.
    pub fn extend_attestation_ttl(env: Env, subject: Address) {
        let key = key(&env, &subject);
        if env.storage().persistent().has(&key) {
            env.storage()
                .persistent()
                .extend_ttl(&key, RECORD_THRESHOLD, RECORD_EXTEND_TO);
        }
    }

    fn store_active(
        env: &Env,
        subject: &Address,
        tier: Symbol,
        expires_at: u64,
        topic: Symbol,
    ) -> Result<(), Error> {
        if expires_at <= env.ledger().timestamp() {
            return Err(Error::ExpiryNotInFuture);
        }
        let row = Attestation {
            tier: tier.clone(),
            expires_at,
            revoked: false,
        };
        write(env, subject, &row);
        env.events()
            .publish((topic, subject.clone()), (tier, expires_at));
        Ok(())
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        if caller != &admin(env) {
            return Err(Error::NotAdmin);
        }
        caller.require_auth();
        extend_instance(env);
        Ok(())
    }
}

fn admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

fn key(env: &Env, subject: &Address) -> (Symbol, Address) {
    (Symbol::new(env, "attest"), subject.clone())
}

fn read(env: &Env, subject: &Address) -> Option<Attestation> {
    env.storage().persistent().get(&key(env, subject))
}

fn write(env: &Env, subject: &Address, row: &Attestation) {
    let key = key(env, subject);
    env.storage().persistent().set(&key, row);
    env.storage()
        .persistent()
        .extend_ttl(&key, RECORD_THRESHOLD, RECORD_EXTEND_TO);
}

fn extend_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_THRESHOLD, INSTANCE_EXTEND_TO);
}

#[cfg(test)]
mod test;
