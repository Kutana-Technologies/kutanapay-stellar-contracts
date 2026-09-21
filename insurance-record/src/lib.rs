#![no_std]
use soroban_sdk::{
    contract, contractimpl, contractmeta, contracttype, Address, BytesN, Env, Symbol,
};

contractmeta!(key = "name", val = "KutanaPay Insurance Record");
contractmeta!(key = "org", val = "KutanaPay");
contractmeta!(key = "binver", val = "0.2.0");
contractmeta!(
    key = "desc",
    val = "On-chain insurance policy/claim/payout hashes keyed to deals. No PII."
);

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyRecord {
    pub escrow_id: BytesN<32>,
    pub policy_hash: BytesN<32>,
    pub status: Symbol,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimRecord {
    pub deal_id: BytesN<32>,
    pub decision: Symbol,
    pub decision_hash: BytesN<32>,
    pub receipt_hash: Option<BytesN<32>>,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
}

#[contract]
pub struct InsuranceRecord;

#[contractimpl]
impl InsuranceRecord {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn record_policy(
        env: Env,
        admin: Address,
        deal_id: BytesN<32>,
        escrow_id: BytesN<32>,
        policy_hash: BytesN<32>,
        status: Symbol,
    ) {
        Self::require_admin(&env, &admin);
        let key = (Symbol::new(&env, "policy"), deal_id);
        env.storage().persistent().set(
            &key,
            &PolicyRecord {
                escrow_id,
                policy_hash,
                status,
            },
        );
    }

    pub fn record_claim_decision(
        env: Env,
        admin: Address,
        deal_id: BytesN<32>,
        claim_id: BytesN<32>,
        decision: Symbol,
        decision_hash: BytesN<32>,
    ) {
        Self::require_admin(&env, &admin);
        let key = (Symbol::new(&env, "claim"), claim_id);
        let existing = env.storage().persistent().get::<_, ClaimRecord>(&key);
        let receipt_hash = existing.and_then(|row| row.receipt_hash);
        env.storage().persistent().set(
            &key,
            &ClaimRecord {
                deal_id,
                decision,
                decision_hash,
                receipt_hash,
            },
        );
    }

    pub fn record_payout_receipt(
        env: Env,
        admin: Address,
        deal_id: BytesN<32>,
        claim_id: BytesN<32>,
        receipt_hash: BytesN<32>,
    ) {
        Self::require_admin(&env, &admin);
        let key = (Symbol::new(&env, "claim"), claim_id);
        let existing = env
            .storage()
            .persistent()
            .get::<_, ClaimRecord>(&key)
            .unwrap_or(ClaimRecord {
                deal_id: deal_id.clone(),
                decision: Symbol::new(&env, "paid"),
                decision_hash: BytesN::from_array(&env, &[0u8; 32]),
                receipt_hash: None,
            });
        env.storage().persistent().set(
            &key,
            &ClaimRecord {
                deal_id,
                decision: existing.decision,
                decision_hash: existing.decision_hash,
                receipt_hash: Some(receipt_hash),
            },
        );
    }

    pub fn get_policy(env: Env, deal_id: BytesN<32>) -> Option<PolicyRecord> {
        let key = (Symbol::new(&env, "policy"), deal_id);
        env.storage().persistent().get(&key)
    }

    pub fn get_claim(env: Env, claim_id: BytesN<32>) -> Option<ClaimRecord> {
        let key = (Symbol::new(&env, "claim"), claim_id);
        env.storage().persistent().get(&key)
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    /// Replace this contract's WASM in place. Same C-id and storage; admin only.
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
