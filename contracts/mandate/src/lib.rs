//! # PayMandate — payment mandate contract
//!
//! A mandate is a customer's standing, capped, revocable authorization that
//! lets a merchant pull up to `amount_per_period` of a token each period until
//! `expires_at`. The customer signs **once** at mandate creation (which also
//! grants the contract a ledger-bounded SEP-41 token allowance); every later
//! `charge` draws on that standing allowance within the per-period cap the
//! contract enforces. The mandate is revocable by the payer at any time.
//!
//! Design notes:
//! - The token allowance (granted to this contract) is the *ceiling*; the
//!   contract's period accounting is the *discipline*. Funds move directly
//!   payer → merchant via `transfer_from`; the contract never holds funds.
//! - No catch-up charging: skipping periods does not accumulate charge room.
//!   At any moment the merchant can pull at most `amount_per_period` minus
//!   what was already pulled in the *current* period.
//! - Allowances are shared per (payer → this contract): `create` *adds* the
//!   new mandate's lifetime ceiling to the existing allowance, and `revoke`
//!   releases exactly the mandate's unspent ceiling — so multiple mandates
//!   from one payer coexist without clobbering each other.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, token, Address, Env,
};

/// Approximate seconds per ledger, used to convert wall-clock durations into
/// ledger sequence numbers for allowance/storage lifetimes.
const LEDGER_SECS_ESTIMATE: u64 = 5;
/// Safety buffer (in ledgers) added on top of estimated expiry: ~1 day.
const LEDGER_BUFFER: u64 = 17_280;
/// Network maximum entry TTL (~1 year of ledgers). Allowance lifetimes are
/// clamped to this; mandates that outlive it need an allowance renewal
/// (`extend_allowance` — see issue backlog) before charges resume.
const MAX_TTL_LEDGERS: u64 = 6_311_000;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    MandateNotFound = 1,
    MandateNotActive = 2,
    MandateExpired = 3,
    ExceedsPeriodCap = 4,
    InvalidParams = 5,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MandateStatus {
    Active,
    Revoked,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mandate {
    /// Who pays. Must authorize creation and revocation.
    pub payer: Address,
    /// Who receives. Must authorize each charge.
    pub merchant: Address,
    /// SEP-41 token the mandate is denominated in (e.g. USDC SAC).
    pub token: Address,
    /// Maximum total amount chargeable per period.
    pub amount_per_period: i128,
    /// Period length in seconds (e.g. 2_592_000 for 30 days).
    pub period_secs: u64,
    /// Unix timestamp of mandate creation; periods are indexed from here.
    pub start: u64,
    /// Unix timestamp after which no further charges are allowed.
    pub expires_at: u64,
    /// Amount already charged in `current_period`.
    pub spent_in_period: i128,
    /// Index of the period the accounting currently refers to.
    pub current_period: u64,
    /// Ledger sequence until which the token allowance stays live.
    /// Caller-supplied at creation (an auth-pinned argument must not be
    /// derived from ledger state inside the contract, or the simulated auth
    /// tree won't match at apply time); reused at revocation.
    pub allowance_live_until: u32,
    /// Total chargeable over the mandate's whole life
    /// (`amount_per_period` × number of periods until expiry).
    pub lifetime_ceiling: i128,
    /// Total charged so far across all periods.
    pub lifetime_spent: i128,
    pub status: MandateStatus,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Mandate(u64),
    NextId,
}

/// Emitted when a mandate is created.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MandateCreated {
    #[topic]
    pub id: u64,
    pub payer: Address,
    pub merchant: Address,
    pub token: Address,
    pub amount_per_period: i128,
    pub period_secs: u64,
    pub expires_at: u64,
}

/// Emitted on every successful charge.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MandateCharged {
    #[topic]
    pub id: u64,
    pub payer: Address,
    pub merchant: Address,
    pub amount: i128,
    pub period: u64,
}

/// Emitted when a mandate is revoked by its payer.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MandateRevoked {
    #[topic]
    pub id: u64,
    pub payer: Address,
    pub merchant: Address,
}

#[contract]
pub struct MandateContract;

#[contractimpl]
impl MandateContract {
    /// Create a mandate. Requires `payer` auth. As part of the same
    /// authorization, grants this contract a token allowance covering the
    /// mandate's full lifetime spend ceiling — this is the "sign once"
    /// moment. The ceiling is *added* to any allowance the payer already
    /// granted this contract for other mandates.
    ///
    /// `allowance_live_until` is the ledger sequence until which the token
    /// allowance stays live. It must be supplied by the caller: it becomes
    /// an argument of the nested, auth-pinned `approve` call, and anything
    /// derived from ledger state inside the contract would differ between
    /// simulation and apply, invalidating the signed auth tree. The SDK
    /// computes it as `current ledger + duration/5s + 1 day buffer`, clamped
    /// to the network max entry TTL (~6.3M ledgers). Mandates outliving the
    /// allowance need a renewal (`extend_allowance`, see backlog) before
    /// charges resume.
    ///
    /// Returns the new mandate id.
    // Contract entrypoints are flat argument lists by design; 8 scalar args
    // beats an opaque struct for wallet display and CLI ergonomics.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        env: Env,
        payer: Address,
        merchant: Address,
        token: Address,
        amount_per_period: i128,
        period_secs: u64,
        expires_at: u64,
        allowance_live_until: u32,
    ) -> Result<u64, Error> {
        payer.require_auth();

        let now = env.ledger().timestamp();
        if amount_per_period <= 0
            || period_secs == 0
            || expires_at <= now
            || allowance_live_until <= env.ledger().sequence()
            || merchant == payer
        {
            return Err(Error::InvalidParams);
        }

        // Number of period slots until expiry, rounded up.
        let duration = expires_at - now;
        let periods = duration.div_ceil(period_secs);
        let lifetime_ceiling = amount_per_period
            .checked_mul(i128::from(periods))
            .ok_or(Error::InvalidParams)?;

        // Grant (add) the allowance on the caller-supplied horizon.
        let token_client = token::TokenClient::new(&env, &token);
        let contract = env.current_contract_address();
        let existing = token_client.allowance(&payer, &contract);
        let new_allowance = existing
            .checked_add(lifetime_ceiling)
            .ok_or(Error::InvalidParams)?;
        token_client.approve(&payer, &contract, &new_allowance, &allowance_live_until);

        // Persist. Keep the instance (and the NextId counter in it) alive on
        // a max-TTL horizon so the counter can never archive and reset while
        // mandates still exist — id reuse would silently overwrite them.
        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextId)
            .unwrap_or(0u64);
        env.storage().instance().set(&DataKey::NextId, &(id + 1));
        let max_ttl = MAX_TTL_LEDGERS as u32;
        env.storage().instance().extend_ttl(max_ttl, max_ttl);

        let mandate = Mandate {
            payer: payer.clone(),
            merchant: merchant.clone(),
            token,
            amount_per_period,
            period_secs,
            start: now,
            expires_at,
            spent_in_period: 0,
            current_period: 0,
            allowance_live_until,
            lifetime_ceiling,
            lifetime_spent: 0,
            status: MandateStatus::Active,
        };
        write_mandate(&env, id, &mandate, duration);

        MandateCreated {
            id,
            payer,
            merchant,
            token: mandate.token.clone(),
            amount_per_period,
            period_secs,
            expires_at,
        }
        .publish(&env);
        Ok(id)
    }

    /// Charge `amount` against the mandate. Requires the mandate's
    /// `merchant` auth. Moves funds payer → merchant via `transfer_from` on
    /// the standing allowance. Enforces status, expiry, and the per-period
    /// cap; skipped periods grant no catch-up room.
    pub fn charge(env: Env, id: u64, amount: i128) -> Result<(), Error> {
        let mut m = read_mandate(&env, id)?;
        m.merchant.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidParams);
        }
        if m.status != MandateStatus::Active {
            return Err(Error::MandateNotActive);
        }
        let now = env.ledger().timestamp();
        if now >= m.expires_at {
            return Err(Error::MandateExpired);
        }

        // Roll period accounting forward; skipped periods don't accumulate.
        let period = (now - m.start) / m.period_secs;
        if period > m.current_period {
            m.current_period = period;
            m.spent_in_period = 0;
        }

        let spent = m
            .spent_in_period
            .checked_add(amount)
            .ok_or(Error::ExceedsPeriodCap)?;
        if spent > m.amount_per_period {
            return Err(Error::ExceedsPeriodCap);
        }
        // Defense in depth: the per-period cap over the mandate's period
        // slots already implies the lifetime ceiling; enforce it explicitly
        // anyway so revoke's allowance math can never be undermined.
        let lifetime_spent = m
            .lifetime_spent
            .checked_add(amount)
            .ok_or(Error::ExceedsPeriodCap)?;
        if lifetime_spent > m.lifetime_ceiling {
            return Err(Error::ExceedsPeriodCap);
        }

        // Pull the funds: payer → merchant, drawing on the allowance.
        token::TokenClient::new(&env, &m.token).transfer_from(
            &env.current_contract_address(),
            &m.payer,
            &m.merchant,
            &amount,
        );

        m.spent_in_period = spent;
        m.lifetime_spent = lifetime_spent;
        write_mandate(&env, id, &m, m.expires_at.saturating_sub(now));

        MandateCharged {
            id,
            payer: m.payer,
            merchant: m.merchant,
            amount,
            period: m.current_period,
        }
        .publish(&env);
        Ok(())
    }

    /// Revoke the mandate. Requires `payer` auth. Releases exactly the
    /// mandate's unspent lifetime ceiling from the token allowance, leaving
    /// allowance granted for the payer's other mandates intact.
    pub fn revoke(env: Env, id: u64) -> Result<(), Error> {
        let mut m = read_mandate(&env, id)?;
        m.payer.require_auth();

        if m.status != MandateStatus::Active {
            return Err(Error::MandateNotActive);
        }

        // Only release allowance while the stored horizon is still live.
        // Past the horizon the contract-granted allowance has expired on the
        // token side (reads 0); calling approve(amount > 0, stale_expiry)
        // would panic in the SAC, and any nonzero allowance remaining is the
        // payer's own manual grant, outside the mandate system — leave it.
        // Note: the approve arguments stay deterministic from contract state
        // (auth-pinned args must never derive from current ledger state, or
        // the simulated auth tree mismatches at apply time and traps).
        if m.allowance_live_until > env.ledger().sequence() {
            let unspent = m.lifetime_ceiling - m.lifetime_spent;
            let token_client = token::TokenClient::new(&env, &m.token);
            let contract = env.current_contract_address();
            let existing = token_client.allowance(&m.payer, &contract);
            let reduced = (existing - unspent).max(0);
            token_client.approve(&m.payer, &contract, &reduced, &m.allowance_live_until);
        }

        m.status = MandateStatus::Revoked;
        write_mandate(&env, id, &m, LEDGER_BUFFER * LEDGER_SECS_ESTIMATE);

        MandateRevoked {
            id,
            payer: m.payer,
            merchant: m.merchant,
        }
        .publish(&env);
        Ok(())
    }

    /// Fetch a mandate by id.
    pub fn get_mandate(env: Env, id: u64) -> Result<Mandate, Error> {
        read_mandate(&env, id)
    }
}

fn read_mandate(env: &Env, id: u64) -> Result<Mandate, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Mandate(id))
        .ok_or(Error::MandateNotFound)
}

/// Store the mandate and keep its storage entry alive for roughly
/// `horizon_secs` of wall-clock time (clamped to the network max TTL).
fn write_mandate(env: &Env, id: u64, m: &Mandate, horizon_secs: u64) {
    let key = DataKey::Mandate(id);
    env.storage().persistent().set(&key, m);
    let ledgers = ttl_ledgers(horizon_secs);
    env.storage()
        .persistent()
        .extend_ttl(&key, ledgers, ledgers);
}

/// Wall-clock seconds → ledger count, with a one-day buffer, clamped to the
/// network maximum entry TTL.
fn ttl_ledgers(secs: u64) -> u32 {
    let ledgers = secs / LEDGER_SECS_ESTIMATE + LEDGER_BUFFER;
    ledgers.min(MAX_TTL_LEDGERS) as u32
}

mod test;
