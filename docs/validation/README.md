# Phase 0 — Demand Validation (Kill-Test)

Before locking the roadmap we run **10 structured interviews** with merchants and operators who already accept Stellar payments. This directory holds the kit and (as they land) the anonymized findings.

**The gate (decided in advance, no moving goalposts):** the merchant-subscription thesis passes if —
- ≥6/10 interviewees describe a *manual* recurring-collection workflow costing >2 hrs/month, AND
- ≥3 have built or bought a workaround (script, custodial auto-debit, Stripe running in parallel), AND
- ≥3 commit, unprompted, to piloting on testnet.

**If the gate fails:** we pivot the same contract to scheduled payouts/stipends (B2B, contributor stipends, payroll) — demand that observably exists inside the Stellar ecosystem. The contract layer does not change; SDK examples, checkout priority, and positioning do.

Either outcome is published here. Negative results are results.

## Who to interview (sourcing)

| Pool | Where |
|---|---|
| Merchants on Stellar checkout rails | MugglePay / Paykit / Bexo / Myaza merchant communities |
| SaaS/digital-service founders billing in USDC | Stellar Discord #payments, X, LatAm/Africa builder groups |
| Ecosystem teams paying recurring stipends | GrantFox Telegram, SCF project Discords |

Target the person who *personally* runs collections each month (founder or ops lead), not a spokesperson.

## Files

- [`interview-script.md`](interview-script.md) — the script, verbatim questions, do/don't rules
- [`outreach-templates.md`](outreach-templates.md) — first-contact messages (EN/ES/PT)
- [`tracking.md`](tracking.md) — interview log + criteria scoreboard
- `findings.md` — written after ≥8 interviews; published regardless of outcome
