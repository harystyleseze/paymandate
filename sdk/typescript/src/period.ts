import type { Mandate } from "./types.js";

/**
 * Client-side mirror of the contract's period accounting, for UX:
 * "what can be charged right now?", "when does the next period start?".
 * The contract remains the source of truth — use this for display and
 * pre-flight checks only.
 */

/** Period index at time `now` (unix seconds) for a mandate. */
export function periodIndexAt(mandate: Pick<Mandate, "start" | "periodSecs">, now: bigint): bigint {
  if (now < mandate.start) return 0n;
  return (now - mandate.start) / mandate.periodSecs;
}

/** Unix timestamp at which the next period begins. */
export function nextPeriodStart(
  mandate: Pick<Mandate, "start" | "periodSecs">,
  now: bigint,
): bigint {
  return mandate.start + (periodIndexAt(mandate, now) + 1n) * mandate.periodSecs;
}

/**
 * Amount still chargeable at `now`, mirroring the contract rules:
 * revoked/expired → 0; rolled-over period → full cap; otherwise cap − spent.
 * Skipped periods do NOT accumulate (no catch-up).
 */
export function chargeableNow(mandate: Mandate, now: bigint): bigint {
  if (mandate.status !== "Active") return 0n;
  if (now >= mandate.expiresAt) return 0n;
  const idx = periodIndexAt(mandate, now);
  const spent = idx > mandate.currentPeriod ? 0n : mandate.spentInPeriod;
  const left = mandate.amountPerPeriod - spent;
  return left > 0n ? left : 0n;
}
