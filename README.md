# PayMandate

**Recurring payments for Stellar.** A customer signs once; a merchant can then charge them automatically — up to a capped amount per period, in USDC (or any SEP-41 token), revocable by the customer at any moment.

> Stellar solved the first payment for merchants. PayMandate exists to solve every payment after it.

## Why

There is no way to charge a customer *repeatedly* on Stellar without asking them to sign every transaction. Every SaaS, membership, telco top-up, or contributor-stipend use case dies at "come back next month and click pay." Recurring revenue — the mechanic Stripe Billing built a business on — has no rail here.

PayMandate is that rail, built as a public good:

- **Payment mandate** (Soroban contract): a standing, capped, time-boxed, revocable pull-authorization. The customer signs once at subscribe time. The contract enforces per-period caps; funds move directly payer → merchant — the protocol never holds money.
- **No unlimited approvals**: the token allowance is bounded to the mandate's lifetime ceiling and ledger-bounded by SEP-41 expiry; revoke zeroes it.
- **Why Stellar**: ~$0.0007 fees make $3/mo subscriptions economical; native USDC/EURC means merchants earn money, not volatile tokens; SEP-24 anchors give emerging-market merchants a real exit to local fiat.

## Status

🚧 **Pre-alpha, testnet only.** The core mandate contract is implemented and tested; SDK, checkout component, keeper service, and dashboard are in progress — see [the backlog](docs/backlog.md). **Do not use with mainnet funds.** The contract is unaudited.

We are also running structured merchant interviews to validate demand before locking the roadmap — findings will be published in [`docs/validation/`](docs/validation/) as they land.

## How it works

```
 subscribe (signs ONCE)                     every period
┌────────┐  create(payer, merchant,        ┌──────────┐ charge(id, amt)
│ Payer  │─ token, cap/period, expiry) ───▶│ Mandate  │◀──────────────  Merchant/keeper
│ wallet │   └ grants ledger-bounded       │ contract │  enforces: active,
└────────┘     SEP-41 allowance            └────┬─────┘  not expired,
     ▲ revoke(id) anytime — zeroes allowance    │        spent+amt ≤ cap
     └──────────────────────────────────────────┤
                                   transfer_from: payer ──▶ merchant
```

- `create(payer, merchant, token, amount_per_period, period_secs, expires_at) → id`
- `charge(id, amount)` — merchant-authorized; per-period cap enforced; no catch-up room from skipped periods
- `revoke(id)` — payer-authorized, instant, zeroes remaining allowance
- `get_mandate(id)`

## Quick start (development)

Prerequisites: Rust (with `wasm32v1-none` target), [stellar-cli](https://developers.stellar.org/docs/tools/cli) ≥ 25.

```bash
git clone https://github.com/paymandate/paymandate
cd paymandate
make test          # run the contract test suite
make build         # compile the contract to wasm
make deploy-testnet  # deploy to Stellar testnet (generates + funds a key)
```

## Repository layout

```
contracts/mandate/   Core Soroban contract (Rust) + test suite
sdk/typescript/      TypeScript SDK (alpha)
examples/            SaaS billing demo · contributor-stipend demo
docs/                Protocol spec, integration guides, validation research
scripts/             Deploy & development scripts
```

## Contributing

We want contributors — most issues need **no Rust**: SDK, checkout UI, docs, examples, translations (es/pt). Start with [CONTRIBUTING.md](CONTRIBUTING.md) and the [`good first issue` label](https://github.com/paymandate/paymandate/labels/good%20first%20issue). Maintainers respond to applications and PRs within 48h.

## License

[Apache-2.0](LICENSE)
