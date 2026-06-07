# Contributing to PayMandate

Thanks for being here. This project is built in the open for the Stellar ecosystem, and most of it does **not** require Rust — TypeScript, UI, docs, and translation contributions are first-class.

## Ground rules (the short version)

1. **Claim before you code.** Comment on an issue (or apply via GrantFox when we're in a campaign) and wait for assignment before starting. One issue at a time for new contributors.
2. **Say how you'll do it.** When applying for an issue, include 2–3 sentences on your approach. "Assign me please" with nothing else will be declined with a pointer back to this guide.
3. **PRs reference their issue** and include a short "what I changed and why" note. Drive-by PRs without an assigned issue are closed politely.
4. **AI-assisted is fine; AI-unreviewed is not.** You must be able to explain every line of your diff. Review questions will test that. Work the author can't explain is returned once, then flagged.
5. **Be kind.** See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Setup (≤10 minutes)

### Contract work (Rust)
```bash
rustup target add wasm32v1-none
cargo install --locked stellar-cli   # or: brew install stellar-cli
make test                            # full suite must pass before you start
```

### SDK / UI work (TypeScript)
```bash
cd sdk/typescript && npm install && npm test
```

## Finding work

- [`good first issue`](https://github.com/paymandate/paymandate/labels/good%20first%20issue) — docs, examples, UI states, translations, SDK errors. No Rust. Target: merged within a week.
- [`intermediate`](https://github.com/paymandate/paymandate/labels/intermediate) — SDK features, keeper logic, wallet adapters.
- [`advanced`](https://github.com/paymandate/paymandate/labels/advanced) — contract changes, fuzzing, gas optimization.
- [`security-sensitive`](https://github.com/paymandate/paymandate/labels/security-sensitive) — contract code paths that move funds. **Gated**: requires prior merged history here, and you'll be paired with a maintainer.

Every issue has acceptance criteria. If an issue is unclear, say so on the issue — improving the issue *is* a contribution.

## Pull request checklist

- [ ] Linked issue, assigned to you
- [ ] `make test` (and `npm test` if SDK touched) green locally
- [ ] `make fmt` + `make lint` clean
- [ ] Contract changes: tests written first (this repo practices TDD; PRs adding contract behavior without tests will be sent back)
- [ ] Short description: what changed, why, anything you're unsure about

## What you can expect from maintainers

- Acknowledgment of applications and PRs within **48 hours**
- Reviews that explain the *why*, especially on your first PR
- Credit in release notes
- Sustained contributors are invited to triage rights

## Growth path

docs/examples → checkout & dashboard UI → SDK internals → keeper service → contract tests → contract code. A TypeScript developer can become a Soroban contributor here — that's the point.
