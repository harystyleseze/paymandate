# Seed Issue Backlog

> 28 issues ready to post to GitHub once the org exists. Format matches `.github/ISSUE_TEMPLATE/task.yml` (context → acceptance criteria → difficulty). GFI = `good first issue` (no Rust). Post in waves: ~10 at launch, rest as the earlier ones merge, so the issue list always looks alive, not dumped.

## Contract (`contracts/mandate`)

1. **[advanced] Add `extend_allowance` for long-lived mandates** — allowances are ledger-bounded (~1y max TTL); mandates outliving it need payer-authorized renewal. AC: new fn requires payer auth; extends SEP-41 allowance to new ceiling; tests incl. expiry-edge.
2. **[advanced] Emit typed events for created/charged/revoked** with full payload (ids, amounts, period index). AC: events asserted in tests; documented in protocol spec.
3. **[advanced] Property/fuzz tests for period accounting** — invariant: total charged in any window ≤ ⌈window/period⌉ × cap. AC: proptest or soroban fuzz harness in CI.
4. **[advanced] Gas/footprint benchmark for `charge`** — measure CPU/mem/entry-bytes; document costs per call. AC: numbers in docs; regression check in CI.
5. **[security-sensitive] Pause/guardian design RFC** — should the protocol have an emergency stop? Trade-offs vs. credible neutrality. AC: written RFC with recommendation, discussed before any code.
6. **[advanced] Mandate metadata field** — optional `name/ref` (e.g. plan id) for indexers. AC: bounded bytes; storage cost documented; tests.
7. **[intermediate] `get_mandates_by_payer` pagination design** — on-chain vs indexer trade-off analysis. AC: decision doc; if indexer, event schema confirmed sufficient.

## TypeScript SDK (`sdk/typescript`)

8. **[intermediate] Implement `createMandate()` transaction builder** — assembles invocation, simulates, returns XDR for wallet signing. AC: works against testnet deployment; integration test.
9. **[intermediate] Implement `charge()` + `revoke()` builders.** AC: same bar as #8.
10. **[intermediate] Event subscription helper** — poll RPC `getEvents` for mandate events, typed callbacks. AC: example script tails charges live on testnet.
11. **[GFI] Typed error mapping** — map contract error codes (1–5) to named JS errors with helpful messages. AC: unit tests; errors documented in README.
12. **[GFI] SDK README quick-start** — subscribe→charge→revoke happy path, copy-pasteable. AC: a newcomer can run it against testnet in <15 min.
13. **[intermediate] Freighter wallet adapter** — sign the create flow in-browser. AC: demo page subscribes with Freighter on testnet.
14. **[intermediate] xBull + LOBSTR adapters** behind a common `WalletAdapter` interface. AC: adapter contract typed; at least one tested end-to-end.

## Keeper service (`services/keeper` — new)

15. **[intermediate] Keeper MVP** — scan a mandate list, submit due charges, exponential retry ladder (T+0/T+1/T+3). AC: runs as a single binary/script against testnet; dry-run mode.
16. **[intermediate] Webhook emitter** — `charge.succeeded/charge.failed/mandate.revoked` with HMAC signatures. AC: receiver example + signature verification doc.
17. **[GFI] Charge-day notification template pack** — email/WhatsApp/push copy in EN/ES/PT reminding payers to fund wallets (the empty-wallet mitigation). AC: reviewed copy in `docs/`.
18. **[advanced] Keeper metrics** — success-rate counters by failure cause (insufficient balance vs revoked vs expired); Prometheus endpoint. AC: this is the data we promised to publish — dashboard-ready.

## Checkout & dashboard (`ui/` — new)

19. **[intermediate] `<SubscribeButton>` web component MVP** — props: merchant, token, cap, period; opens wallet, calls create. AC: framework-agnostic; used in example app.
20. **[GFI] Checkout states design** — loading/success/declined/revoked/expired UI states (Figma or HTML). AC: all states specced incl. empty-wallet failure messaging.
21. **[intermediate] Merchant dashboard skeleton** — list mandates, status, next charge date, revoke-rate. AC: reads from chain/indexer; no backend required for v0.
22. **[GFI] Dashboard i18n scaffolding (es/pt)** — extract strings, add locale switcher. AC: two locales render fully.

## Docs & examples

23. **[GFI] Protocol spec page** — mandate lifecycle, period math (no catch-up rule), allowance-ceiling vs period-discipline distinction. AC: reviewed; linked from README.
24. **[GFI] Example: SaaS billing demo app** — fake SaaS charging 3 test customers monthly on testnet. AC: `npm run demo` works; README walkthrough.
25. **[GFI] Example: contributor-stipend demo** — org pays N contributors a fixed monthly stipend via mandates (the pivot path, alive from day 1). AC: script + README.
26. **[GFI] Anchor top-up recipe** — doc: how a payer keeps a wallet funded via SEP-24 deposit before charge day. AC: step-by-step with one real testnet anchor.
27. **[GFI] Translate README + CONTRIBUTING to es/pt.** AC: native-quality review by a second speaker.
28. **[intermediate] "Why not just streaming?" comparison doc** — mandates vs Superfluid/Sablier-style streams vs Paystreme; when each fits. AC: honest, cited, linked from spec.

## Posting rules

- Every issue gets acceptance criteria + difficulty label + pointers before posting (use the template).
- `security-sensitive` issues note the gating rule explicitly.
- Keep ≥8 GFI open at all times during GrantFox campaigns.
