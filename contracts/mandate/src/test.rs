#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{Address, Env};

/// 30 days in seconds.
const PERIOD: u64 = 2_592_000;
/// One whole token unit at 7 decimals (1 USDC).
const ONE: i128 = 10_000_000;
/// 100 USDC per period.
const CAP: i128 = 100 * ONE;
/// Test epoch.
const T0: u64 = 1_700_000_000;
/// Test ledger sequence at T0.
const SEQ0: u32 = 100;
/// Allowance expiry ledger passed by the client (auth-pinned, must be
/// caller-supplied — deriving it from env.ledger().sequence() inside the
/// contract breaks sim/apply auth matching on-chain).
const LIVE_UNTIL: u32 = SEQ0 + 1_600_000;

struct Setup<'a> {
    env: Env,
    client: MandateContractClient<'a>,
    payer: Address,
    merchant: Address,
    token: TokenClient<'a>,
    token_address: Address,
}

fn setup() -> Setup<'static> {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| {
        li.timestamp = T0;
        li.sequence_number = SEQ0;
    });

    let contract_id = env.register(MandateContract, ());
    let client = MandateContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin);
    let token = TokenClient::new(&env, &sac.address());
    let token_admin = StellarAssetClient::new(&env, &sac.address());

    let payer = Address::generate(&env);
    let merchant = Address::generate(&env);
    // Fund the payer with 10,000 USDC.
    token_admin.mint(&payer, &(10_000 * ONE));

    Setup {
        token_address: sac.address(),
        env,
        client,
        payer,
        merchant,
        token,
    }
}

/// Create a 3-period mandate expiring exactly at T0 + 3 * PERIOD.
fn create_default(s: &Setup) -> u64 {
    s.client.create(
        &s.payer,
        &s.merchant,
        &s.token_address,
        &CAP,
        &PERIOD,
        &(T0 + 3 * PERIOD),
        &LIVE_UNTIL,
    )
}

// ---------------------------------------------------------------- create ---

#[test]
fn create_stores_mandate_and_returns_sequential_ids() {
    let s = setup();
    let id0 = create_default(&s);
    let id1 = create_default(&s);
    assert_eq!(id0, 0);
    assert_eq!(id1, 1);

    let m = s.client.get_mandate(&id0);
    assert_eq!(m.payer, s.payer);
    assert_eq!(m.merchant, s.merchant);
    assert_eq!(m.token, s.token_address);
    assert_eq!(m.amount_per_period, CAP);
    assert_eq!(m.period_secs, PERIOD);
    assert_eq!(m.start, T0);
    assert_eq!(m.expires_at, T0 + 3 * PERIOD);
    assert_eq!(m.spent_in_period, 0);
    assert_eq!(m.current_period, 0);
    assert_eq!(m.status, MandateStatus::Active);
}

#[test]
fn create_grants_allowance_covering_full_lifetime() {
    let s = setup();
    let _ = create_default(&s);
    // 3 periods × CAP — the "sign once" ceiling.
    assert_eq!(s.token.allowance(&s.payer, &s.client.address), 3 * CAP);
}

#[test]
fn create_second_mandate_accumulates_allowance_instead_of_overwriting() {
    // SEP-41 approve() SETS the allowance. Two mandates from the same payer
    // must not clobber each other's ceilings — create() must add to any
    // existing allowance this contract holds.
    let s = setup();
    let _ = create_default(&s);
    let _ = create_default(&s);
    assert_eq!(s.token.allowance(&s.payer, &s.client.address), 6 * CAP);
}

#[test]
fn revoke_after_charge_releases_exactly_the_unspent_ceiling() {
    // transfer_from already consumed 30 from the allowance; revoke must
    // release the mandate's remaining lifetime ceiling (3*CAP − 30), which
    // for a single mandate leaves exactly 0 — not a blind zeroing that would
    // break other mandates sharing the payer's allowance.
    let s = setup();
    let id = create_default(&s);
    s.client.charge(&id, &(30 * ONE));
    assert_eq!(
        s.token.allowance(&s.payer, &s.client.address),
        3 * CAP - 30 * ONE
    );

    s.client.revoke(&id);

    assert_eq!(s.token.allowance(&s.payer, &s.client.address), 0);
}

#[test]
fn create_records_payer_auth() {
    let s = setup();
    let _ = create_default(&s);
    let auths = s.env.auths();
    assert!(!auths.is_empty());
    assert_eq!(auths[0].0, s.payer);
}

#[test]
fn create_rejects_zero_amount() {
    let s = setup();
    let r = s.client.try_create(
        &s.payer,
        &s.merchant,
        &s.token_address,
        &0,
        &PERIOD,
        &(T0 + PERIOD),
        &LIVE_UNTIL,
    );
    assert_eq!(r, Err(Ok(Error::InvalidParams)));
}

#[test]
fn create_rejects_zero_period() {
    let s = setup();
    let r = s.client.try_create(
        &s.payer,
        &s.merchant,
        &s.token_address,
        &CAP,
        &0,
        &(T0 + PERIOD),
        &LIVE_UNTIL,
    );
    assert_eq!(r, Err(Ok(Error::InvalidParams)));
}

#[test]
fn create_rejects_expiry_not_in_future() {
    let s = setup();
    let r = s.client.try_create(
        &s.payer,
        &s.merchant,
        &s.token_address,
        &CAP,
        &PERIOD,
        &T0,
        &LIVE_UNTIL,
    );
    assert_eq!(r, Err(Ok(Error::InvalidParams)));
}

#[test]
fn create_rejects_allowance_expiry_at_or_before_current_ledger() {
    let s = setup();
    let r = s.client.try_create(
        &s.payer,
        &s.merchant,
        &s.token_address,
        &CAP,
        &PERIOD,
        &(T0 + PERIOD),
        &SEQ0, // not strictly in the future
    );
    assert_eq!(r, Err(Ok(Error::InvalidParams)));
}

// ---------------------------------------------------------------- charge ---

#[test]
fn charge_within_cap_moves_funds_payer_to_merchant() {
    let s = setup();
    let id = create_default(&s);

    s.client.charge(&id, &(30 * ONE));

    assert_eq!(s.token.balance(&s.payer), 10_000 * ONE - 30 * ONE);
    assert_eq!(s.token.balance(&s.merchant), 30 * ONE);
    assert_eq!(s.client.get_mandate(&id).spent_in_period, 30 * ONE);
}

#[test]
fn charge_records_merchant_auth() {
    let s = setup();
    let id = create_default(&s);
    s.client.charge(&id, &ONE);
    let auths = s.env.auths();
    assert!(!auths.is_empty());
    assert_eq!(auths[0].0, s.merchant);
}

#[test]
fn charge_accumulates_and_rejects_beyond_cap_in_same_period() {
    let s = setup();
    let id = create_default(&s);

    s.client.charge(&id, &(60 * ONE));
    s.client.charge(&id, &(40 * ONE)); // exactly reaches CAP

    let r = s.client.try_charge(&id, &1);
    assert_eq!(r, Err(Ok(Error::ExceedsPeriodCap)));
}

#[test]
fn charge_cap_resets_at_period_rollover() {
    let s = setup();
    let id = create_default(&s);

    s.client.charge(&id, &CAP); // fill period 0

    // Move into period 1.
    s.env.ledger().with_mut(|li| li.timestamp = T0 + PERIOD);
    s.client.charge(&id, &CAP); // full cap available again

    assert_eq!(s.token.balance(&s.merchant), 2 * CAP);
    let m = s.client.get_mandate(&id);
    assert_eq!(m.current_period, 1);
    assert_eq!(m.spent_in_period, CAP);
}

#[test]
fn charge_skipped_periods_do_not_accumulate() {
    let s = setup();
    let id = create_default(&s);

    // Jump straight into period 2 without charging periods 0 and 1.
    s.env.ledger().with_mut(|li| li.timestamp = T0 + 2 * PERIOD);

    s.client.charge(&id, &CAP); // current period's cap: fine
    let r = s.client.try_charge(&id, &1); // no catch-up room from skipped periods
    assert_eq!(r, Err(Ok(Error::ExceedsPeriodCap)));
}

#[test]
fn charge_fails_after_expiry() {
    let s = setup();
    let id = create_default(&s);

    s.env.ledger().with_mut(|li| li.timestamp = T0 + 3 * PERIOD); // == expires_at
    let r = s.client.try_charge(&id, &ONE);
    assert_eq!(r, Err(Ok(Error::MandateExpired)));
}

#[test]
fn charge_rejects_non_positive_amount() {
    let s = setup();
    let id = create_default(&s);
    assert_eq!(s.client.try_charge(&id, &0), Err(Ok(Error::InvalidParams)));
    assert_eq!(s.client.try_charge(&id, &-5), Err(Ok(Error::InvalidParams)));
}

#[test]
fn charge_fails_for_unknown_mandate() {
    let s = setup();
    let r = s.client.try_charge(&999, &1);
    assert_eq!(r, Err(Ok(Error::MandateNotFound)));
}

// ---------------------------------------------------------------- revoke ---

#[test]
fn revoke_blocks_future_charges() {
    let s = setup();
    let id = create_default(&s);

    s.client.revoke(&id);

    assert_eq!(s.client.get_mandate(&id).status, MandateStatus::Revoked);
    let r = s.client.try_charge(&id, &ONE);
    assert_eq!(r, Err(Ok(Error::MandateNotActive)));
}

#[test]
fn revoke_zeroes_remaining_allowance() {
    let s = setup();
    let id = create_default(&s);
    assert!(s.token.allowance(&s.payer, &s.client.address) > 0);

    s.client.revoke(&id);

    assert_eq!(s.token.allowance(&s.payer, &s.client.address), 0);
}

#[test]
fn revoke_fails_for_unknown_mandate() {
    let s = setup();
    let r = s.client.try_revoke(&999);
    assert_eq!(r, Err(Ok(Error::MandateNotFound)));
}

// ----------------------------------------------------------------- query ---

#[test]
fn get_mandate_fails_for_unknown_id() {
    let s = setup();
    let r = s.client.try_get_mandate(&42);
    assert_eq!(r, Err(Ok(Error::MandateNotFound)));
}
