import type { CreateMandateParams } from "./types.js";

export interface ClientConfig {
  /** Soroban RPC endpoint, e.g. https://soroban-testnet.stellar.org */
  rpcUrl: string;
  /** Deployed mandate contract id (C...). */
  contractId: string;
  /** Network passphrase, e.g. "Test SDF Network ; September 2015". */
  networkPassphrase: string;
}

/**
 * PayMandate client — transaction builders for the mandate contract.
 *
 * ⚠️ Alpha skeleton. The builders below are intentionally unimplemented and
 * tracked as contributor issues (see docs/backlog.md #8–#9). They will
 * assemble, simulate, and return XDR ready for wallet signing using
 * @stellar/stellar-sdk once the testnet deployment is live.
 */
export class PayMandateClient {
  constructor(public readonly config: ClientConfig) {}

  /** Build the create-mandate transaction (payer signs once). Backlog #8. */
  buildCreateMandateTx(_params: CreateMandateParams): Promise<string> {
    throw new Error(
      "Not implemented yet — tracked as backlog issue #8 (good contribution target!). " +
        "See https://github.com/paymandate/paymandate/blob/main/docs/backlog.md",
    );
  }

  /** Build a charge transaction (merchant/keeper signs). Backlog #9. */
  buildChargeTx(_mandateId: bigint, _amount: bigint): Promise<string> {
    throw new Error("Not implemented yet — tracked as backlog issue #9.");
  }

  /** Build a revoke transaction (payer signs). Backlog #9. */
  buildRevokeTx(_mandateId: bigint): Promise<string> {
    throw new Error("Not implemented yet — tracked as backlog issue #9.");
  }
}
