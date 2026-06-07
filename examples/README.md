# Examples

Two reference integrations, kept deliberately small. Both are planned against the alpha SDK and tracked in [docs/backlog.md](../docs/backlog.md):

- **`saas-billing/`** (backlog #24) — a fake SaaS charging 3 test customers monthly on testnet: subscribe (one signature) → keeper charges on schedule → dashboard shows status.
- **`contributor-stipends/`** (backlog #25) — an org pays N contributors a fixed monthly USDC stipend via mandates. This is the protocol's second use case, alive from day one: any Stellar project paying recurring stipends (including GrantFox projects) can run this.

Until the SDK transaction builders land (#8–#9), the contract can be exercised directly with `stellar contract invoke` — see the walkthrough in [docs/](../docs/) or the test suite in `contracts/mandate/src/test.rs`, which documents every behavior.
