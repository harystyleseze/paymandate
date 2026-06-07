# Deployments

## Testnet

| | |
|---|---|
| Contract (current, post-security-review) | `CBH6NBXQX5RIK4ZRXO4JXWQ3WCRAIVTBXKC2ZF6SKMMZT6ZNZMDCGETR` |
| Explorer | https://stellar.expert/explorer/testnet/contract/CBH6NBXQX5RIK4ZRXO4JXWQ3WCRAIVTBXKC2ZF6SKMMZT6ZNZMDCGETR |
| Deployed | 2026-06-07 |
| wasm | `paymandate_mandate.wasm`, soroban-sdk 26.0.1 |
| Superseded | `CAWFJGZP...QQJEOA` (2026-06-06, pre-review build — do not use) |

### Verified on-chain

**2026-06-07 (current contract, mandate id 0):** create (sign-once nested approve) → merchant charge → payer revoke → double-revoke correctly rejected with `Error #2 (MandateNotActive)`.

**2026-06-06 (superseded contract):**
- `create` — payer signed **once**; the nested SEP-41 `approve` (allowance = lifetime ceiling, ledger-bounded to `allowance_live_until`) executed under the same auth tree
- `charge` — merchant-signed pull of 2.5 XLM moved funds payer → merchant directly
- period rollover — a second charge in the next 60s period correctly drew on a reset cap
- `revoke` — payer-signed; subsequent charge rejected with `Error #2 (MandateNotActive)`

### Security review (2026-06-07)

Adversarial review found and led to fixes for: a revoke-lockout panic path (stale allowance horizon + manual payer top-up → SAC `approve` panic — now skipped safely), unchecked `lifetime_spent` add, missing explicit lifetime-ceiling guard in `charge`, instance-storage TTL risk for the id counter (now max-TTL extended on every create), `merchant == payer` mandates, and silent double-revoke (now `MandateNotActive`). Remaining accepted limitations are documented in [SECURITY.md](../SECURITY.md). Test count: 21 → 25.

### Design note for integrators (learned the hard way)

`allowance_live_until` must be **caller-supplied**: it is an argument of the
nested, auth-pinned `approve` call. Deriving it from `env.ledger().sequence()`
inside the contract makes the simulated auth tree mismatch at apply time —
transactions trap with `Unauthorized function call for address`. The SDK
computes it client-side (`current ledger + duration/5s + buffer`, clamped to
the network max entry TTL).

> ⚠️ Testnet is reset periodically by SDF; the contract id above will need
> redeployment afterward (`make deploy-testnet`).

## Mainnet

Not deployed. Blocked on: security review/audit, pilot results, allowance-renewal flow (backlog #1).
