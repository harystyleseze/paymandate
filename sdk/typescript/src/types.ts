/** Mirror of the on-chain `Mandate` struct (contracts/mandate/src/lib.rs). */
export interface Mandate {
  payer: string;
  merchant: string;
  token: string;
  /** Max total chargeable per period, in the token's smallest unit (i128 as bigint). */
  amountPerPeriod: bigint;
  /** Period length in seconds. */
  periodSecs: bigint;
  /** Unix timestamp of creation; periods are indexed from here. */
  start: bigint;
  /** Unix timestamp after which charges fail. */
  expiresAt: bigint;
  spentInPeriod: bigint;
  currentPeriod: bigint;
  /** Ledger sequence until which the token allowance stays live. */
  allowanceLiveUntil: number;
  /** Total chargeable over the whole life (cap × periods). */
  lifetimeCeiling: bigint;
  /** Total charged so far across all periods. */
  lifetimeSpent: bigint;
  status: "Active" | "Revoked";
}

export interface CreateMandateParams {
  payer: string;
  merchant: string;
  token: string;
  amountPerPeriod: bigint;
  periodSecs: bigint;
  expiresAt: bigint;
  /**
   * Ledger sequence until which the token allowance stays live. MUST be
   * computed client-side (current ledger + duration/5s + buffer, clamped to
   * the network max entry TTL ≈ 6.3M ledgers): it is an argument of the
   * nested auth-pinned `approve`, so the contract cannot derive it from
   * ledger state without breaking simulation/apply auth matching.
   * If omitted, builders will compute it from the RPC's latest ledger.
   */
  allowanceLiveUntil?: number;
}

/** Common period lengths, in seconds. */
export const PERIODS = {
  WEEKLY: 604_800n,
  MONTHLY_30D: 2_592_000n,
  QUARTERLY_90D: 7_776_000n,
  YEARLY_365D: 31_536_000n,
} as const;
