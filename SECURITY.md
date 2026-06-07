# Security

## Reporting

**Do not open public issues for vulnerabilities that could affect funds.** Contact the maintainers privately (email in README). We acknowledge reports within 48 hours. There is no bug bounty yet; we credit reporters in release notes unless they prefer otherwise.

## Status

⚠️ **Unaudited. Testnet only. Do not use with mainnet funds.**

The contract has a 25-test suite (TDD), an internal AI-assisted adversarial review (2026-06), and live testnet verification of every code path — none of which substitutes for an independent audit. Mainnet is gated on one.

## Design properties

- **Non-custodial**: funds move payer → merchant directly via `transfer_from`; the contract never holds balances.
- **Sign-once, bounded**: the payer signs once at `create`; the granted allowance equals the mandate's lifetime ceiling (cap × periods), is ledger-bounded (`allowance_live_until`), and is reduced automatically by every charge.
- **Per-period discipline**: `charge` enforces the per-period cap with rollover and *no catch-up* from skipped periods, plus an explicit lifetime-ceiling guard (defense in depth).
- **Revocable**: `revoke` (payer-only) releases exactly the mandate's unspent ceiling — never other mandates' allowance — and blocks all future charges.
- **Auth-pinning rule**: arguments of nested, authorized calls (`approve`) are never derived from current ledger state; they're caller-supplied or stored. Deriving them in-contract makes the simulated auth tree mismatch at apply time (found the hard way; see `docs/DEPLOYMENTS.md`).

## Known limitations (documented, accepted for alpha)

1. **Manual-approve desync.** A payer who calls `token.approve(payer, mandate_contract, X)` *directly* changes the shared allowance pool outside the contract's accounting. Consequences: a later `revoke` may leave residual allowance (if they raised it) or other active mandates may fail to charge (if they lowered it). The contract clamps, never panics; integrators should tell payers to manage mandates only through the contract.
2. **Orphaned allowance after storage TTL expiry.** If a mandate's persistent entry archives (TTL lapse), `revoke` through the contract returns `MandateNotFound` while a live token allowance may remain. Charges also fail (same lookup), so no funds can move via the contract — but the payer should zero the allowance directly with `token.approve(..., 0, ...)`. Storage TTLs are extended on every write to make this unlikely; an archival-restore guide is planned (backlog).
3. **Charge timing within a period is the merchant's choice.** A merchant may pull the full period cap at the start of a period, or split it across many charges. The cap is the protection; timing is not constrained. Payers should size caps to what they're truly willing to authorize per period.
4. **Allowance horizon < mandate lifetime.** Network max entry TTL (~1 year of ledgers) bounds `allowance_live_until`. Mandates outliving it need an allowance renewal before charges resume (`extend_allowance`, backlog #1). Charges simply fail in the gap; no funds are at risk.

## Process

- `security-sensitive` label gates contract-touching issues to contributors with merge history, paired with a maintainer.
- CI enforces fmt, clippy `-D warnings`, full test suite, and wasm build on every PR.
- Contract behavior changes require tests written first (TDD) — see CONTRIBUTING.md.
