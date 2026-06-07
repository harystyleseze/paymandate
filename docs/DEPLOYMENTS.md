# Deployments

## Testnet

| | |
|---|---|
| Contract | `CAWFJGZP5SMIHA6YNK2KNVYDWGOPFCGPKLS3KHHNL7BDSAZ2XOQQJEOA` |
| Explorer | https://stellar.expert/explorer/testnet/contract/CAWFJGZP5SMIHA6YNK2KNVYDWGOPFCGPKLS3KHHNL7BDSAZ2XOQQJEOA |
| Deployed | 2026-06-06 |
| wasm | `paymandate_mandate.wasm` (12.5 KB), soroban-sdk 26.0.1 |

### Verified on-chain (2026-06-06, mandate id 0)

- `create` — payer signed **once**; the nested SEP-41 `approve` (allowance = lifetime ceiling, ledger-bounded to `allowance_live_until`) executed under the same auth tree
- `charge` — merchant-signed pull of 2.5 XLM moved funds payer → merchant directly
- period rollover — a second charge in the next 60s period correctly drew on a reset cap
- `revoke` — payer-signed; subsequent charge rejected with `Error #2 (MandateNotActive)`

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
