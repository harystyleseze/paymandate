# @paymandate/sdk (alpha)

TypeScript SDK for [PayMandate](https://github.com/paymandate/paymandate) — recurring payment mandates on Stellar.

## Honest status

| Module | Status |
|---|---|
| `types` — Mandate types, period constants | ✅ usable |
| `errors` — typed contract error mapping | ✅ usable, tested |
| `period` — client-side period math (chargeable-now, rollover) | ✅ usable, tested (mirrors contract tests) |
| `client` — transaction builders (create/charge/revoke) | 🚧 **unimplemented** — [backlog #8–#9](../../docs/backlog.md), good contribution targets |

```bash
npm install && npm test
```

## Planned API

```ts
const client = new PayMandateClient({
  rpcUrl: "https://soroban-testnet.stellar.org",
  contractId: process.env.PAYMANDATE_CONTRACT_ID!,
  networkPassphrase: "Test SDF Network ; September 2015",
});

const xdr = await client.buildCreateMandateTx({
  payer, merchant, token: USDC,
  amountPerPeriod: 100_0000000n,   // 100 USDC / period
  periodSecs: PERIODS.MONTHLY_30D,
  expiresAt: BigInt(Math.floor(Date.now() / 1000)) + 3n * PERIODS.MONTHLY_30D,
});
// → pass to Freighter/xBull/LOBSTR for the payer's ONE signature
```
