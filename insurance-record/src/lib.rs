#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contractmeta, contracttype, symbol_short, Address,
    BytesN, Env, Symbol,
};

contractmeta!(key = "name", val = "KutanaPay Insurance Record");
contractmeta!(key = "org", val = "KutanaPay");
contractmeta!(key = "binver", val = "0.3.0");
contractmeta!(
    key = "desc",
    val = "On-chain insurance policy/claim/payout hashes keyed to deals. No PII."
);

/// Ledgers close roughly every five seconds.
const DAY_IN_LEDGERS: u32 = 17_280;
const INSTANCE_EXTEND_TO: u32 = 30 * DAY_IN_LEDGERS;
const INSTANCE_THRESHOLD: u32 = INSTANCE_EXTEND_TO - DAY_IN_LEDGERS;
const RECORD_EXTEND_TO: u32 = 120 * DAY_IN_LEDGERS;
const RECORD_THRESHOLD: u32 = RECORD_EXTEND_TO - 7 * DAY_IN_LEDGERS;

const APPROVED: Symbol = symbol_short!("approved");
const DENIED: Symbol = symbol_short!("denied");
/// Decision stored by 0.2.0 placeholder claims; never accepted as input.
const LEGACY_PAID: Symbol = symbol_short!("paid");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// The `admin` argument is not the stored admin.
    NotAdmin = 1,
    /// `decision` is not `approved` or `denied`.
    InvalidDecision = 2,
    /// No claim decision has been recorded for this claim.
    ClaimNotFound = 3,
    /// `deal_id` differs from the deal the claim is recorded against.
    DealMismatch = 4,
    /// A payout receipt requires an `approved` decision.
    ClaimNotApproved = 5,
    /// The claim is paid; its decision can no longer change.
    DecisionFinal = 6,
    /// A different payout receipt is already recorded.
    ReceiptAlreadyRecorded = 7,
    /// There is no pending admin transfer.
    NoPendingAdmin = 8,
    /// A document hash is all zeros.
    InvalidHash = 9,
}

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
    PendingAdmin,
}

#[contract]
pub struct InsuranceRecord;

#[contractimpl]
impl InsuranceRecord {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        extend_instance(&env);
    }

    /// Create or replace the policy record for a deal.
    pub fn record_policy(
        env: Env,
        admin: Address,
        deal_id: BytesN<32>,
        escrow_id: BytesN<32>,
        policy_hash: BytesN<32>,
        status: Symbol,
    ) -> Result<(), Error> {
        Self::require_admin(&env, &admin)?;
        require_hash(&env, &policy_hash)?;
        let row = PolicyRecord {
            escrow_id,
            policy_hash,
            status,
        };
        write(&env, &policy_key(&env, &deal_id), &row);
        env.events()
            .publish((symbol_short!("policy"), deal_id), row);
        Ok(())
    }

    /// Record or change a claim decision. A paid claim only accepts an identical replay.
    pub fn record_claim_decision(
        env: Env,
        admin: Address,
        deal_id: BytesN<32>,
        claim_id: BytesN<32>,
        decision: Symbol,
        decision_hash: BytesN<32>,
    ) -> Result<(), Error> {
        Self::require_admin(&env, &admin)?;
        if decision != APPROVED && decision != DENIED {
            return Err(Error::InvalidDecision);
        }
        require_hash(&env, &decision_hash)?;
        let key = claim_key(&env, &claim_id);
        let receipt_hash = match read_claim(&env, &key) {
            Some(row) => {
                if row.deal_id != deal_id {
                    return Err(Error::DealMismatch);
                }
                // 0.2.0 stored a placeholder ("paid", zero hash) when a receipt
                // arrived first; a real decision may replace it.
                let placeholder = row.decision == LEGACY_PAID && is_zero(&env, &row.decision_hash);
                if row.receipt_hash.is_some() && !placeholder {
                    if row.decision == decision && row.decision_hash == decision_hash {
                        return Ok(());
                    }
                    return Err(Error::DecisionFinal);
                }
                row.receipt_hash
            }
            None => None,
        };
        write(
            &env,
            &key,
            &ClaimRecord {
                deal_id,
                decision: decision.clone(),
                decision_hash: decision_hash.clone(),
                receipt_hash,
            },
        );
        env.events().publish(
            (symbol_short!("decision"), claim_id),
            (decision, decision_hash),
        );
        Ok(())
    }

    /// Attach a payout receipt to an approved claim. Identical replays succeed.
    pub fn record_payout_receipt(
        env: Env,
        admin: Address,
        deal_id: BytesN<32>,
        claim_id: BytesN<32>,
        receipt_hash: BytesN<32>,
    ) -> Result<(), Error> {
        Self::require_admin(&env, &admin)?;
        require_hash(&env, &receipt_hash)?;
        let key = claim_key(&env, &claim_id);
        let mut row = read_claim(&env, &key).ok_or(Error::ClaimNotFound)?;
        if row.deal_id != deal_id {
            return Err(Error::DealMismatch);
        }
        if row.decision != APPROVED {
            return Err(Error::ClaimNotApproved);
        }
        match &row.receipt_hash {
            Some(existing) if existing == &receipt_hash => return Ok(()),
            Some(_) => return Err(Error::ReceiptAlreadyRecorded),
            None => {}
        }
        row.receipt_hash = Some(receipt_hash.clone());
        write(&env, &key, &row);
        env.events()
            .publish((symbol_short!("payout"), claim_id), receipt_hash);
        Ok(())
    }

    pub fn get_policy(env: Env, deal_id: BytesN<32>) -> Option<PolicyRecord> {
        env.storage().persistent().get(&policy_key(&env, &deal_id))
    }

    pub fn get_claim(env: Env, claim_id: BytesN<32>) -> Option<ClaimRecord> {
        read_claim(&env, &claim_key(&env, &claim_id))
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

    /// Extend a policy record's TTL. Anyone may pay for this.
    pub fn extend_policy_ttl(env: Env, deal_id: BytesN<32>) {
        extend_if_present(&env, &policy_key(&env, &deal_id));
    }

    /// Extend a claim record's TTL. Anyone may pay for this.
    pub fn extend_claim_ttl(env: Env, claim_id: BytesN<32>) {
        extend_if_present(&env, &claim_key(&env, &claim_id));
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

type RecordKey = (Symbol, BytesN<32>);

fn admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

fn policy_key(env: &Env, deal_id: &BytesN<32>) -> RecordKey {
    (Symbol::new(env, "policy"), deal_id.clone())
}

fn claim_key(env: &Env, claim_id: &BytesN<32>) -> RecordKey {
    (Symbol::new(env, "claim"), claim_id.clone())
}

fn read_claim(env: &Env, key: &RecordKey) -> Option<ClaimRecord> {
    env.storage().persistent().get(key)
}

fn write<V: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(env: &Env, key: &RecordKey, value: &V) {
    env.storage().persistent().set(key, value);
    env.storage()
        .persistent()
        .extend_ttl(key, RECORD_THRESHOLD, RECORD_EXTEND_TO);
}

fn extend_if_present(env: &Env, key: &RecordKey) {
    if env.storage().persistent().has(key) {
        env.storage()
            .persistent()
            .extend_ttl(key, RECORD_THRESHOLD, RECORD_EXTEND_TO);
    }
}

fn is_zero(env: &Env, hash: &BytesN<32>) -> bool {
    hash == &BytesN::from_array(env, &[0u8; 32])
}

fn require_hash(env: &Env, hash: &BytesN<32>) -> Result<(), Error> {
    if is_zero(env, hash) {
        return Err(Error::InvalidHash);
    }
    Ok(())
}

fn extend_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_THRESHOLD, INSTANCE_EXTEND_TO);
}

#[cfg(test)]
mod test;
