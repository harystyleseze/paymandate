.PHONY: test build fmt lint clean deploy-testnet

test:
	cargo test

build:
	stellar contract build

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets -- -D warnings

clean:
	cargo clean

# Deploys the mandate contract to Stellar testnet.
# Generates and funds an identity named `paymandate-deployer` if missing.
deploy-testnet: build
	./scripts/deploy-testnet.sh
