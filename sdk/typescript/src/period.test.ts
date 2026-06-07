import { describe, expect, it } from "vitest";
import { errorFromCode } from "./errors.js";
import { chargeableNow, nextPeriodStart, periodIndexAt } from "./period.js";
import type { Mandate } from "./types.js";

const PERIOD = 2_592_000n; // 30 days
const CAP = 100_0000000n; // 100 USDC (7 decimals)
const T0 = 1_700_000_000n;

const base: Mandate = {
  payer: "GPAYER",
  merchant: "GMERCHANT",
  token: "CTOKEN",
  amountPerPeriod: CAP,
  periodSecs: PERIOD,
  start: T0,
  expiresAt: T0 + 3n * PERIOD,
  spentInPeriod: 0n,
  currentPeriod: 0n,
  allowanceLiveUntil: 1_700_000,
  lifetimeCeiling: 3n * CAP,
  lifetimeSpent: 0n,
  status: "Active",
};

describe("periodIndexAt", () => {
  it("is 0 during the first period", () => {
    expect(periodIndexAt(base, T0)).toBe(0n);
    expect(periodIndexAt(base, T0 + PERIOD - 1n)).toBe(0n);
  });
  it("increments at exact period boundaries", () => {
    expect(periodIndexAt(base, T0 + PERIOD)).toBe(1n);
    expect(periodIndexAt(base, T0 + 2n * PERIOD)).toBe(2n);
  });
});

describe("nextPeriodStart", () => {
  it("points at the next boundary", () => {
    expect(nextPeriodStart(base, T0)).toBe(T0 + PERIOD);
    expect(nextPeriodStart(base, T0 + PERIOD)).toBe(T0 + 2n * PERIOD);
  });
});

describe("chargeableNow (mirrors contract rules)", () => {
  it("full cap on a fresh mandate", () => {
    expect(chargeableNow(base, T0)).toBe(CAP);
  });
  it("cap minus spent within the same period", () => {
    expect(chargeableNow({ ...base, spentInPeriod: 30_0000000n }, T0)).toBe(CAP - 30_0000000n);
  });
  it("resets to full cap after rollover, with no catch-up accumulation", () => {
    const spent = { ...base, spentInPeriod: CAP };
    expect(chargeableNow(spent, T0)).toBe(0n);
    expect(chargeableNow(spent, T0 + PERIOD)).toBe(CAP); // not 2*CAP
    expect(chargeableNow(spent, T0 + 2n * PERIOD)).toBe(CAP); // skipped ≠ accumulated
  });
  it("zero when revoked or expired", () => {
    expect(chargeableNow({ ...base, status: "Revoked" }, T0)).toBe(0n);
    expect(chargeableNow(base, base.expiresAt)).toBe(0n);
  });
});

describe("errorFromCode", () => {
  it("maps known contract codes to named errors", () => {
    expect(errorFromCode(4).name_).toBe("ExceedsPeriodCap");
    expect(errorFromCode(2).message).toMatch(/revoked/i);
  });
  it("handles unknown codes gracefully", () => {
    expect(errorFromCode(99).name_).toBe("Unknown");
  });
});
