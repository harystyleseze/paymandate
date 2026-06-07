#!/usr/bin/env bash
# Deploy the mandate contract to Stellar testnet.
# Requires: stellar-cli >= 25 (https://developers.stellar.org/docs/tools/cli)
set -euo pipefail

IDENTITY="${PAYMANDATE_IDENTITY:-paymandate-deployer}"
NETWORK="testnet"
WASM="target/wasm32v1-none/release/paymandate_mandate.wasm"

if [[ ! -f "$WASM" ]]; then
  echo "wasm not found at $WASM — run 'make build' first" >&2
  exit 1
fi

# Create + fund the deployer identity if it doesn't exist yet.
if ! stellar keys address "$IDENTITY" >/dev/null 2>&1; then
  echo "Creating identity '$IDENTITY' and funding via friendbot..."
  stellar keys generate "$IDENTITY" --network "$NETWORK" --fund
fi

echo "Deployer: $(stellar keys address "$IDENTITY")"

CONTRACT_ID=$(stellar contract deploy \
  --wasm "$WASM" \
  --source "$IDENTITY" \
  --network "$NETWORK")

echo
echo "✅ Deployed mandate contract to testnet:"
echo "   $CONTRACT_ID"
echo
echo "Explorer: https://stellar.expert/explorer/testnet/contract/$CONTRACT_ID"
echo "Save this id for the SDK: export PAYMANDATE_CONTRACT_ID=$CONTRACT_ID"
